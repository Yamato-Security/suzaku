use crate::core::color::SuzakuColor::{Green, Orange};
use crate::core::errorlog::{log_error, log_warn};
use crate::core::log_source::{LogSource, is_match_service};
use crate::core::summary::DetectionSummary;
use crate::core::timeline_writer::{OutputContext, event_timestamp, write_record};
use crate::core::util::p;
use crate::option::cli::{FileDateOption, TimeOption, TimelineOptions};
use crate::option::timefiler::{filter_by_time, filter_file_by_date_path};
use bytesize::ByteSize;
use chrono::NaiveDateTime;
use colored::Colorize;
use console::style;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use num_format::{Locale, ToFormattedString};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use serde_json::Value;
use sigma_rust::{CorrelationEngine, Event, Rule, TimestampedEvent, event_from_json};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

/// Scan one log file. An `Err` means the file could not be read as a log at all, so it must not
/// be counted as scanned.
#[allow(clippy::too_many_arguments)]
pub fn scan_file<'a>(
    f: &PathBuf,
    context: &mut OutputContext<'a>,
    summary: &mut DetectionSummary,
    options: &TimelineOptions,
    rules: &Vec<&Rule>,
    matched_correlation: &mut Vec<TimestampedEvent<'a>>,
    correlation_engine: &'a CorrelationEngine,
    log: &LogSource,
) -> Result<(), Box<dyn Error>> {
    let path_str = f.display().to_string();
    let events = if path_str.ends_with(".csv") {
        load_csv_events(&get_content(f), log, &path_str)?
    } else if path_str.ends_with(".parquet") {
        load_parquet_events(f)?
    } else {
        load_json_from_file(&get_content(f), log)?
    };
    let events = normalize_events(events, log);
    detect_events(
        &events,
        context,
        summary,
        options,
        rules,
        matched_correlation,
        correlation_engine,
    );
    Ok(())
}

/// Scan every log file under `d`, returning how many files were read (recorded in the DuckDB
/// output's `suzaku_meta.scanned_files`, so a report can state the coverage of the run).
#[allow(clippy::too_many_arguments)]
pub fn scan_directory<'a>(
    d: &PathBuf,
    context: &mut OutputContext<'a>,
    summary: &mut DetectionSummary,
    options: &TimelineOptions,
    rules: &Vec<&Rule>,
    matched_correlation: &mut Vec<TimestampedEvent<'a>>,
    correlation_engine: &'a CorrelationEngine,
    log: &LogSource,
) -> usize {
    let no_color = context.config.no_color;
    let process_events = |events: &[Value]| {
        detect_events(
            events,
            context,
            summary,
            options,
            rules,
            matched_correlation,
            correlation_engine,
        );
    };
    match process_events_from_dir(
        process_events,
        d,
        options.output_opt.output.is_some(),
        no_color,
        log,
        &options.input_opt.file_date_opt,
    ) {
        Ok(files) => files,
        Err(e) => {
            log_error(&format!("Failed to scan directory {}: {e}", d.display()));
            0
        }
    }
}

/// Returns the number of log files that were handed to `process_events`.
pub fn process_events_from_dir<F>(
    mut process_events: F,
    directory: &PathBuf,
    show_progress: bool,
    no_color: bool,
    log: &LogSource,
    file_date_opt: &FileDateOption,
) -> Result<usize, Box<dyn Error>>
where
    F: FnMut(&[Value]),
{
    if file_date_opt.file_date_from.is_some() || file_date_opt.file_date_to.is_some() {
        let from_str = file_date_opt
            .file_date_from
            .as_deref()
            .map(format_date_display)
            .unwrap_or_else(|| "*".to_string());
        let to_str = file_date_opt
            .file_date_to
            .as_deref()
            .map(format_date_display)
            .unwrap_or_else(|| "*".to_string());
        p(
            Orange.rdg(no_color),
            &format!(
                "Filtering files by filename date prefix ({} - {}). Please wait. This may take a few minutes.",
                from_str, to_str
            ),
            true,
        );
        println!();
    }
    let (count, file_paths, total_size) = count_files_recursive(directory, file_date_opt)?;
    let size = ByteSize::b(total_size).display().to_string();

    p(Green.rdg(no_color), "Total log files: ", false);
    p(None, &count.to_formatted_string(&Locale::en), true);
    p(Green.rdg(no_color), "Total file size: ", false);
    p(None, size.to_string().as_str(), true);
    println!();

    p(Orange.rdg(no_color), "Scanning now. Please wait.", true);
    println!();

    let template = if no_color {
        "[{elapsed_precise}] {human_pos} / {human_len} {spinner} [{bar:40}] {percent}%\n\n{msg}"
            .to_string()
    } else {
        format!(
            "[{{elapsed_precise}}] {{human_pos}} / {{human_len}} {} [{}] {{percent}}%\n\n{{msg}}",
            "{spinner}".truecolor(0, 255, 0),
            "{bar:40}".truecolor(0, 255, 0)
        )
    };
    let pb_style = ProgressStyle::with_template(&template)
        .unwrap()
        .progress_chars("=> ");
    let pb =
        ProgressBar::with_draw_target(Some(count as u64), ProgressDrawTarget::stdout_with_hz(10))
            .with_tab_width(55);
    pb.set_style(pb_style);
    if show_progress {
        pb.enable_steady_tick(Duration::from_millis(300));
    }

    let mut scanned_files = 0usize;
    // Files that were found but could not be read as logs. Each reason is in the error log;
    // the total is shown once the scan ends so the reduced coverage is visible on the terminal.
    let mut skipped_files = 0usize;
    for path in file_paths {
        // `path` is the real `PathBuf`, so files with non-UTF-8 names still resolve and are read.
        // Render lossily only for the extension checks (extensions are ASCII) and the progress
        // display.
        let path_str = path.to_string_lossy();
        if show_progress {
            // The file may have been removed mid-scan; fall back to 0 rather than panicking.
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let size = ByteSize::b(size).display().to_string();
            let pb_msg = format!("{path_str} ({size})");
            pb.set_message(pb_msg);
        }
        if path_str.ends_with("parquet") {
            // Warn rather than silently skipping, matching the json/gz branches below,
            // so an unreadable or over-cap Parquet file does not overstate coverage.
            match load_parquet_events(&path) {
                Ok(events) => {
                    let events = normalize_events(events, log);
                    process_events(&events);
                    scanned_files += 1;
                }
                Err(e) => {
                    log_warn(&format!("Skipping {path_str}: {e}"));
                    skipped_files += 1;
                }
            }
            if show_progress {
                pb.inc(1);
            }
            continue;
        }
        let log_contents = if path_str.ends_with("json")
            || path_str.ends_with("jsonl")
            || path_str.ends_with("csv")
        {
            match fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(e) => {
                    // The file was counted but could not be read (permissions,
                    // non-UTF-8 content, removed mid-scan). Warn instead of
                    // silently skipping, so the run's coverage is not overstated.
                    // The message goes to the error log, not the terminal, so it
                    // cannot corrupt the progress bar redraw.
                    log_warn(&format!("Skipping {path_str}: {e}"));
                    skipped_files += 1;
                    if show_progress {
                        pb.inc(1);
                    }
                    continue;
                }
            }
        } else if path_str.ends_with("gz") {
            match read_gz_file(&path) {
                Ok(contents) => contents,
                Err(e) => {
                    log_warn(&format!("Skipping {path_str}: {e}"));
                    skipped_files += 1;
                    if show_progress {
                        pb.inc(1);
                    }
                    continue;
                }
            }
        } else {
            if show_progress {
                pb.inc(1);
            }
            continue;
        };

        let events = if path_str.ends_with("csv") {
            match load_csv_events(&log_contents, log, &path_str) {
                Ok(events) => events,
                Err(e) => {
                    log_warn(&format!("Skipping {path_str}: {e}"));
                    skipped_files += 1;
                    if show_progress {
                        pb.inc(1);
                    }
                    continue;
                }
            }
        } else {
            log_contents_to_events(&log_contents, log)
        };
        let events = normalize_events(events, log);
        process_events(&events);
        scanned_files += 1;

        if show_progress {
            pb.inc(1);
        }
    }
    if show_progress {
        if no_color {
            pb.finish_with_message("Scanning finished.\n");
        } else {
            pb.finish_with_message(style("Scanning finished.\n").color256(214).to_string());
        }
    }
    if skipped_files > 0 {
        p(Orange.rdg(no_color), "Skipped files: ", false);
        p(
            None,
            &format!(
                "{} (could not be read as logs; see the error log for the reasons)",
                skipped_files.to_formatted_string(&Locale::en)
            ),
            true,
        );
        println!();
    }
    Ok(scanned_files)
}

/// Normalize one raw Azure/M365 record before rule matching.
///
/// M365 Unified Audit Log records exported via `Search-UnifiedAuditLog` are
/// wrapped in a row that carries the real record in an `AuditData` field — a
/// JSON string (CSV export) or a nested object (JSON export). Unwrap it so rules
/// match the actual record. Then fold the UAL Name/Value property bags
/// (`ExtendedProperties`, `DeviceProperties`, `Parameters`, `ModifiedProperties`)
/// into plain objects so rules can reach nested values like
/// `ExtendedProperties.UserAgent`. Non-UAL Azure events (Azure Monitor
/// diagnostic logs) are returned unchanged.
fn normalize_azure_event(mut v: Value) -> Value {
    // Unwrap the Search-UnifiedAuditLog `AuditData` wrapper.
    if let Value::Object(map) = &v
        && let Some(audit_data) = map.get("AuditData")
    {
        let inner = match audit_data {
            Value::String(s) => serde_json::from_str::<Value>(s).ok(),
            Value::Object(_) => Some(audit_data.clone()),
            _ => None,
        };
        if let Some(inner) = inner {
            v = inner;
        }
    }
    // Only UAL records carry these Name/Value property bags; leave diagnostic logs untouched.
    let is_ual = matches!(&v, Value::Object(m) if m.contains_key("Workload") || m.contains_key("RecordType"));
    if is_ual && let Value::Object(map) = &mut v {
        for key in [
            "ExtendedProperties",
            "DeviceProperties",
            "Parameters",
            "ModifiedProperties",
        ] {
            if let Some(Value::Array(arr)) = map.get(key) {
                let mut folded = serde_json::Map::new();
                for item in arr {
                    if let Value::Object(pair) = item
                        && let Some(Value::String(name)) = pair.get("Name")
                    {
                        let val = pair
                            .get("Value")
                            .or_else(|| pair.get("NewValue"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        folded.insert(name.clone(), val);
                    }
                }
                if !folded.is_empty() {
                    map.insert(key.to_string(), Value::Object(folded));
                }
            }
        }
    }
    // Synthesize a stable, human-readable `_Details` summary of the
    // security-relevant change (the Exchange cmdlet `Parameters`, or the
    // directory-change `ModifiedProperties`) for the output timeline. The folded
    // objects above stay for rule matching; this string renders in a
    // deterministic key order (serde_json object iteration), unlike rendering
    // sigma_rust's HashMap-backed event value directly.
    if is_ual && let Value::Object(map) = &v {
        let src = ["Parameters", "ModifiedProperties"]
            .into_iter()
            .find_map(|k| match map.get(k) {
                Some(Value::Object(o)) if !o.is_empty() => Some(o),
                _ => None,
            });
        let details = src.map(|o| {
            o.iter()
                .map(|(k, val)| match val {
                    Value::String(s) => format!("{k}: {s}"),
                    other => format!("{k}: {other}"),
                })
                .collect::<Vec<_>>()
                .join(", ")
        });
        if let Some(details) = details
            && let Value::Object(map) = &mut v
        {
            map.insert("_Details".to_string(), Value::String(details));
        }
    }
    v
}

/// True when `map` is a raw Google Workspace Reports API activity object.
fn is_gws_activity_map(map: &serde_json::Map<String, Value>) -> bool {
    map.get("kind").and_then(Value::as_str) == Some("admin#reports#activity")
}

/// Extract the individual Google Workspace activities from one parsed JSON document. Handles the
/// shapes the Reports API produces: a single activity object, a bare array of activities, and an
/// `activities.list` response page (`{ "kind": "admin#reports#activities", "items": [...] }`).
fn gws_records(value: Value) -> Vec<Value> {
    match value {
        Value::Array(records) => records.into_iter().flat_map(gws_records).collect(),
        Value::Object(mut map) => {
            // Only an envelope carries `items`; an activity is returned as it stands.
            if !is_gws_activity_map(&map) && map.contains_key("items") {
                // An `activities.list` response page. A page that matched nothing carries
                // `"items": null` (or an empty array), which must yield no records at all rather
                // than one opaque envelope object that matches no rule but is counted as a
                // scanned event by every statistic in the run.
                return match map.remove("items") {
                    Some(Value::Array(items)) => items.into_iter().flat_map(gws_records).collect(),
                    _ => vec![],
                };
            }
            vec![Value::Object(map)]
        }
        _ => vec![],
    }
}

/// The value of one `events[].parameters[]` entry, or `None` when the parameter carries no value.
///
/// The Reports API puts the value in exactly one of several differently typed fields.
/// `intValue`/`multiIntValue` arrive as *strings*; they are parsed so numeric Sigma comparisons
/// work, and left as strings when they do not parse.
///
/// A parameter with none of those fields is NOT a set flag. The API elides protobuf default
/// values, so a bare `{"name": "is_suspicious"}` means *false* and a bare
/// `{"name": "recurrence_rule"}` means the empty string — `is_suspicious` is emitted on every
/// `login_success`, valueless roughly 13x more often than as `boolValue: true`. Folding those to
/// `true` would make a rule on `is_suspicious: true` fire on every successful login in the
/// tenant. Such a parameter is therefore dropped: `is_suspicious: true` then matches only the
/// explicit trues, and `is_suspicious: null` matches the elided default.
fn gws_parameter_value(param: &serde_json::Map<String, Value>) -> Option<Value> {
    if let Some(value) = param.get("value") {
        return Some(value.clone());
    }
    if let Some(value) = param.get("intValue") {
        return Some(gws_int_value(value));
    }
    if let Some(value) = param.get("boolValue") {
        return Some(value.clone());
    }
    if let Some(Value::Array(items)) = param.get("multiValue") {
        return Some(Value::Array(items.clone()));
    }
    if let Some(Value::Array(items)) = param.get("multiIntValue") {
        return Some(Value::Array(items.iter().map(gws_int_value).collect()));
    }
    if let Some(value) = param.get("messageValue") {
        return Some(gws_message_value(value));
    }
    if let Some(Value::Array(items)) = param.get("multiMessageValue") {
        return Some(Value::Array(items.iter().map(gws_message_value).collect()));
    }
    None
}

/// Parse a Reports API integer, which is transported as a string (`"63900000000"`). A value that
/// does not parse is kept verbatim rather than dropped.
fn gws_int_value(value: &Value) -> Value {
    match value {
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or_else(|_| value.clone()),
        other => other.clone(),
    }
}

/// Fold a `messageValue` into a plain object. Its payload is itself a `parameter` array (the same
/// shape as `events[].parameters`), so rules can reach `SETTING_METADATA.rule_type` once folded.
fn gws_message_value(value: &Value) -> Value {
    let Value::Object(map) = value else {
        return value.clone();
    };
    let Some(Value::Array(params)) = map.get("parameter") else {
        return value.clone();
    };
    Value::Object(fold_gws_parameters(params))
}

/// Fold an `events[].parameters` array into a flat object keyed by parameter name.
fn fold_gws_parameters(params: &[Value]) -> serde_json::Map<String, Value> {
    let mut folded = serde_json::Map::new();
    for param in params {
        if let Value::Object(param) = param
            && let Some(Value::String(name)) = param.get("name")
            && let Some(value) = gws_parameter_value(param)
        {
            folded.insert(name.clone(), value);
        }
    }
    folded
}

/// True for an `UPPER_CASE` parameter name, i.e. `^[A-Z0-9_]+$`.
fn is_gws_upper_case(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// The most `events[]` entries one activity is split into.
///
/// Each sub-event carries a copy of every top-level activity field, so an activity with N
/// sub-events costs N copies of it. Real activities stay far below this (the largest seen is a
/// Calendar invite with one `add_event_guest` per guest, in the hundreds), but the array is
/// attacker-influenced and the input is untrusted JSON: one crafted record with a few million
/// sub-events would otherwise allocate until the host runs out of memory. Past the cap the
/// remaining sub-events are dropped with a warning, while `eventCount` keeps reporting the
/// activity's real length so the truncation is visible in the output.
const MAX_GWS_SUB_EVENTS: usize = 10_000;

/// Normalize one raw Google Workspace Reports API activity into one record per `events[]` entry.
///
/// Unlike the AWS/Azure normalizers this returns MANY records: an activity is a container, and
/// its sub-events are what a rule actually describes. `CHANGE_PASSWORD` +
/// `CHANGE_PASSWORD_ON_NEXT_LOGIN` arrive as one activity, and a Calendar `create_event` carries
/// one `add_event_guest` sub-event per guest. Sigma cannot see inside arrays, so an unsplit
/// activity would match on at most its first sub-event and hide the rest.
///
/// Each record keeps every top-level activity field (`kind`, `id`, `actor`, `ipAddress`,
/// `networkInfo`, `resourceDetails`, `ownerDomain`, ...) and adds the sub-event's `eventName` /
/// `eventType` / `resourceIds`, its position (`eventIndex` of `eventCount`), the synthesized
/// `eventService`, and the sub-event's parameters folded into top-level keys. The folded
/// parameters are written first and everything else over them, so the activity's own fields and
/// Suzaku's synthesized keys always win a name collision with a parameter.
fn normalize_gws_event(v: Value) -> Vec<Value> {
    let mut activity = match v {
        Value::Object(map) => map,
        other => return vec![other],
    };
    // Emitted only when the activity names an application: a bare ".googleapis.com" would be a
    // value no rule means and would hide the fact that `id.applicationName` is missing.
    let event_service = activity
        .get("id")
        .and_then(|id| id.get("applicationName"))
        .and_then(Value::as_str)
        .map(|app| format!("{app}.googleapis.com"));
    let events = match activity.remove("events") {
        Some(Value::Array(events)) if !events.is_empty() => events,
        // Nothing to split on -- `events` missing, not an array, or an empty array. Emit the
        // activity as it stands rather than dropping it, so a malformed or already-flattened
        // record still reaches the rules.
        other => {
            if let Some(other) = other {
                activity.insert("events".to_string(), other);
            }
            if let Some(event_service) = event_service {
                activity.insert("eventService".to_string(), Value::String(event_service));
            }
            return vec![Value::Object(activity)];
        }
    };

    let event_count = events.len();
    let events = if event_count > MAX_GWS_SUB_EVENTS {
        log_warn(&format!(
            "Google Workspace activity with {event_count} sub-events: only the first {MAX_GWS_SUB_EVENTS} were scanned"
        ));
        &events[..MAX_GWS_SUB_EVENTS]
    } else {
        events.as_slice()
    };

    let last = events.len() - 1;
    let mut records = Vec::with_capacity(events.len());
    for (index, event) in events.iter().enumerate() {
        // The folded parameters go in FIRST. A parameter name is free-form data from the logged
        // event, so one can be called `eventName`, `id` or `kind`; letting it land on top would
        // give a record whose `id` resolves no timestamp and whose `kind` matches no service.
        // Everything Suzaku synthesizes, and every field of the activity itself, is written after
        // them and therefore wins the collision.
        let mut record = serde_json::Map::new();
        if let Value::Object(event) = event
            && let Some(Value::Array(params)) = event.get("parameters")
        {
            let folded = fold_gws_parameters(params);
            // Lowercase aliases go in only after every parameter is in place, so a real
            // lowercase parameter is never overwritten by an alias of its UPPER_CASE twin.
            let aliases: Vec<(String, Value)> = folded
                .iter()
                .filter(|(name, _)| is_gws_upper_case(name))
                .map(|(name, value)| (name.to_ascii_lowercase(), value.clone()))
                .filter(|(alias, _)| !folded.contains_key(alias))
                .collect();
            for (name, value) in folded {
                record.insert(name, value);
            }
            // SigmaHQ's gworkspace admin rules inherited lowercase parameter names
            // (`new_value`, `setting_name`) from the Elastic Filebeat module, while the
            // Reports API emits `NEW_VALUE`. Emitting both lets those rules load unmodified.
            for (alias, value) in aliases {
                record.entry(alias).or_insert(value);
            }
        }
        // The last record takes the activity instead of copying it one final time.
        let fields = if index == last {
            std::mem::take(&mut activity)
        } else {
            activity.clone()
        };
        for (name, value) in fields {
            record.insert(name, value);
        }
        if let Some(event_service) = &event_service {
            record.insert(
                "eventService".to_string(),
                Value::String(event_service.clone()),
            );
        }
        record.insert("eventIndex".to_string(), Value::from(index as u64));
        record.insert("eventCount".to_string(), Value::from(event_count as u64));
        if let Value::Object(event) = event {
            for (from, to) in [("name", "eventName"), ("type", "eventType")] {
                if let Some(value) = event.get(from) {
                    record.insert(to.to_string(), value.clone());
                }
            }
            if let Some(resource_ids) = event.get("resourceIds") {
                record.insert("resourceIds".to_string(), resource_ids.clone());
            }
        }
        records.push(Value::Object(record));
    }
    records
}

/// Apply the per-source normalizer to every event. Google Workspace is the one source where a
/// single input record can produce several output records (one per `events[]` entry).
fn normalize_events(events: Vec<Value>, log: &LogSource) -> Vec<Value> {
    match log {
        LogSource::Azure => events.into_iter().map(normalize_azure_event).collect(),
        LogSource::Gws => events.into_iter().flat_map(normalize_gws_event).collect(),
        _ => events,
    }
}

/// How many CSV row warnings are written out in full per file. The rest are only counted, so a
/// badly broken file cannot flood the error log, but the total still says how much was lost.
const MAX_DETAILED_CSV_WARNINGS: usize = 10;

/// The columns that identify an AWS Management Console CloudTrail "Event history" download.
const AWS_CONSOLE_CSV_REQUIRED_COLUMNS: &[&str] =
    &["Event ID", "Event time", "Event name", "Event source"];

/// Console CSV column -> CloudTrail record field (dot path) for the columns copied as strings.
/// `Resources` and `Read-only` need converting and are handled separately.
const AWS_CONSOLE_CSV_COLUMNS: &[(&str, &str)] = &[
    ("Event time", "eventTime"),
    ("Event source", "eventSource"),
    ("Event name", "eventName"),
    ("AWS region", "awsRegion"),
    ("Source IP address", "sourceIPAddress"),
    ("User agent", "userAgent"),
    ("Error code", "errorCode"),
    ("User name", "userIdentity.userName"),
    ("AWS access key", "userIdentity.accessKeyId"),
    ("Request ID", "requestID"),
    ("Event ID", "eventID"),
    ("Event type", "eventType"),
    ("Recipient Account Id", "recipientAccountId"),
    ("Event category", "eventCategory"),
];

/// A problem with one CSV row that did not stop the rest of the file from being read.
#[derive(Debug, Clone, PartialEq)]
pub enum CsvWarningKind {
    /// The row has a different number of fields than the header; the row is skipped. With the
    /// reader's `flexible(true)` this is how a stray or unterminated quote shows up, as the
    /// `csv` crate itself does not report malformed quoting.
    FieldCountMismatch { expected: usize, found: usize },
    /// A field of an AWS console CSV contains a line break; the row is kept. The console never
    /// writes one, so this is almost always an unterminated quote in the last column that has
    /// swallowed the following rows into this field.
    EmbeddedNewline,
    /// An AWS console CSV row has no `Event time`; the row is skipped.
    MissingEventTime,
    /// `Resources` is not a JSON array of objects; the field is dropped and the row kept.
    InvalidResources,
    /// `Read-only` is neither `true` nor `false`; the field is dropped and the row kept.
    InvalidReadOnly(String),
    /// The `csv` crate could not read the row; the row is skipped.
    RecordError(String),
}

impl std::fmt::Display for CsvWarningKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvWarningKind::FieldCountMismatch { expected, found } => write!(
                f,
                "expected {expected} fields but found {found}, row skipped (possibly broken quoting)"
            ),
            CsvWarningKind::EmbeddedNewline => write!(
                f,
                "a field contains a line break, possibly an unterminated quote that swallowed the following rows"
            ),
            CsvWarningKind::MissingEventTime => write!(f, "no Event time, row skipped"),
            CsvWarningKind::InvalidResources => {
                write!(f, "Resources is not a JSON array of objects, field dropped")
            }
            CsvWarningKind::InvalidReadOnly(v) => {
                write!(f, "Read-only is {v:?}, not true/false, field dropped")
            }
            CsvWarningKind::RecordError(e) => write!(f, "cannot read the row: {e}, row skipped"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CsvWarning {
    /// 1-based line number where the row starts.
    pub line: u64,
    pub kind: CsvWarningKind,
}

/// The rows parsed from a CSV file, plus the row-level problems met on the way. The parser does
/// not log anything itself: the caller knows the file name and records the warnings with it.
#[derive(Debug, Default)]
pub struct CsvParseOutcome {
    pub events: Vec<Value>,
    pub warnings: Vec<CsvWarning>,
}

/// A CSV file that could not be read at all.
#[derive(Debug)]
pub enum CsvParseError {
    Header(csv::Error),
    /// Read as AWS input, but the header is not that of a console CloudTrail CSV.
    UnsupportedAwsFormat,
}

impl std::fmt::Display for CsvParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvParseError::Header(e) => write!(f, "cannot read the CSV header: {e}"),
            CsvParseError::UnsupportedAwsFormat => write!(
                f,
                "not an AWS Management Console CloudTrail event history CSV (expected the columns {})",
                AWS_CONSOLE_CSV_REQUIRED_COLUMNS.join(", ")
            ),
        }
    }
}

impl Error for CsvParseError {}

fn is_aws_console_csv(headers: &csv::StringRecord) -> bool {
    AWS_CONSOLE_CSV_REQUIRED_COLUMNS
        .iter()
        .all(|c| headers.iter().any(|h| h == *c))
}

/// Parse a CSV file into one JSON object per row.
///
/// For AWS input the file must be a Management Console CloudTrail "Event history" download, and
/// each row is rebuilt as a CloudTrail record (see `aws_console_csv_row_to_event`) so rules and
/// the summary/metrics commands see the same field names as with JSON input. Any other CSV read as
/// AWS input is rejected, rather than turned into rows no rule can match.
///
/// For every other log source each row stays a flat `{header: value}` map. That is what a
/// `Search-UnifiedAuditLog` export needs: the real audit record is carried in the `AuditData`
/// column and is unwrapped afterwards by `normalize_azure_event`. `LogSource::All` also keeps the
/// flat map, as no command scans logs with it (it is only used to load rules).
fn parse_csv_events(contents: &str, log: &LogSource) -> Result<CsvParseOutcome, CsvParseError> {
    let mut outcome = CsvParseOutcome::default();
    // The `csv` crate strips a leading UTF-8 BOM itself.
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(contents.as_bytes());
    let headers = reader.headers().map_err(CsvParseError::Header)?.clone();
    let aws = matches!(log, LogSource::Aws);
    if aws && !is_aws_console_csv(&headers) {
        return Err(CsvParseError::UnsupportedAwsFormat);
    }
    // For CRLF files (console downloads are CRLF) a record's `Position` points at the `\n` that
    // ends the previous line, so `Position::line()` is one short. Line numbers are counted from
    // the byte offset instead, after stepping past any line-break bytes there (a record never
    // starts with one, as blank lines are skipped), incrementally to stay linear.
    let bytes = contents.as_bytes();
    let (mut counted_to, mut line) = (0usize, 1u64);
    let mut line_of = |byte: u64| {
        let mut byte = (byte as usize).min(bytes.len());
        while byte < bytes.len() && matches!(bytes[byte], b'\r' | b'\n') {
            byte += 1;
        }
        if byte > counted_to {
            line += bytes[counted_to..byte]
                .iter()
                .filter(|&&b| b == b'\n')
                .count() as u64;
            counted_to = byte;
        }
        line
    };
    for record in reader.records() {
        // Reading from an in-memory `&str` with `flexible(true)` does not fail in practice (the
        // contents are already valid UTF-8); if a row ever does, report it and move on.
        let record = match record {
            Ok(record) => record,
            Err(e) => {
                let line = e.position().map(|p| line_of(p.byte())).unwrap_or_default();
                outcome.warnings.push(CsvWarning {
                    line,
                    kind: CsvWarningKind::RecordError(e.to_string()),
                });
                continue;
            }
        };
        let row_line = record
            .position()
            .map(|p| line_of(p.byte()))
            .unwrap_or_default();
        if record.len() != headers.len() {
            outcome.warnings.push(CsvWarning {
                line: row_line,
                kind: CsvWarningKind::FieldCountMismatch {
                    expected: headers.len(),
                    found: record.len(),
                },
            });
            continue;
        }
        if aws {
            if let Some(event) =
                aws_console_csv_row_to_event(&headers, &record, row_line, &mut outcome.warnings)
            {
                outcome.events.push(event);
            }
        } else {
            let mut map = serde_json::Map::new();
            for (header, field) in headers.iter().zip(record.iter()) {
                map.insert(header.to_string(), Value::String(field.to_string()));
            }
            outcome.events.push(Value::Object(map));
        }
    }
    Ok(outcome)
}

/// Insert `value` at a dot-separated `path` (e.g. `userIdentity.userName`), creating the
/// intermediate objects.
fn insert_path(map: &mut serde_json::Map<String, Value>, path: &str, value: Value) {
    match path.split_once('.') {
        None => {
            map.insert(path.to_string(), value);
        }
        Some((head, rest)) => {
            let child = map
                .entry(head.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(child) = child {
                insert_path(child, rest, value);
            }
        }
    }
}

/// Convert the console's `Resources` column (`[{"resourceType": ..., "resourceName": ...}]`) to
/// CloudTrail's `resources` (`[{"type": ..., "ARN": ...}]`). Other keys are kept as they are.
/// `None` unless the value is a JSON array of objects; the array is not partially salvaged.
fn convert_console_resources(raw: &str) -> Option<Value> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(raw) else {
        return None;
    };
    let mut converted = Vec::with_capacity(items.len());
    for item in items {
        let Value::Object(obj) = item else {
            return None;
        };
        let renamed = obj
            .into_iter()
            .map(|(k, v)| match k.as_str() {
                "resourceName" => ("ARN".to_string(), v),
                "resourceType" => ("type".to_string(), v),
                _ => (k, v),
            })
            .collect();
        converted.push(Value::Object(renamed));
    }
    Some(Value::Array(converted))
}

/// Rebuild one row of a console CloudTrail CSV as a CloudTrail record.
///
/// The console CSV carries only a summary of each event (no `userIdentity.arn`, `errorMessage`,
/// `requestParameters`, `responseElements`, ...), so rules on those fields cannot match.
///
/// An empty cell becomes an absent field, never `""`: the console writes an empty `Error code` for
/// every successful call, and most CloudTrail rules test `errorCode|exists`, which an empty string
/// would satisfy -- every row would then match them.
fn aws_console_csv_row_to_event(
    headers: &csv::StringRecord,
    record: &csv::StringRecord,
    line: u64,
    warnings: &mut Vec<CsvWarning>,
) -> Option<Value> {
    let mut map = serde_json::Map::new();
    let mut warn = |kind| warnings.push(CsvWarning { line, kind });
    let mut newline_seen = false;
    for (header, value) in headers.iter().zip(record.iter()) {
        if value.is_empty() {
            continue;
        }
        if !newline_seen && value.contains('\n') {
            newline_seen = true;
            warn(CsvWarningKind::EmbeddedNewline);
        }
        match header {
            "Resources" => match convert_console_resources(value) {
                // CloudTrail omits `resources` when an event has none.
                Some(Value::Array(items)) if items.is_empty() => {}
                Some(resources) => {
                    map.insert("resources".to_string(), resources);
                }
                None => warn(CsvWarningKind::InvalidResources),
            },
            "Read-only" => match value {
                "true" => {
                    map.insert("readOnly".to_string(), Value::Bool(true));
                }
                "false" => {
                    map.insert("readOnly".to_string(), Value::Bool(false));
                }
                other => warn(CsvWarningKind::InvalidReadOnly(other.to_string())),
            },
            _ => match AWS_CONSOLE_CSV_COLUMNS.iter().find(|(c, _)| *c == header) {
                Some((_, path)) => insert_path(&mut map, path, Value::String(value.to_string())),
                // A column this version does not know: keep it under its own name rather than
                // silently dropping data.
                None => {
                    map.insert(header.to_string(), Value::String(value.to_string()));
                }
            },
        }
    }
    if !map.contains_key("eventTime") {
        warn(CsvWarningKind::MissingEventTime);
        return None;
    }
    // The console shows the root user as `root`; in CloudTrail that is `userIdentity.type: Root`
    // (which carries no `userName`). Restore the type so rules on root activity still match.
    if let Some(Value::Object(identity)) = map.get_mut("userIdentity")
        && identity.get("userName").and_then(Value::as_str) == Some("root")
    {
        identity.insert("type".to_string(), Value::String("Root".to_string()));
    }
    Some(Value::Object(map))
}

/// Record a CSV file's row warnings in the error log: the first `MAX_DETAILED_CSV_WARNINGS` in
/// full, then one line with the total.
fn log_csv_warnings(path: &str, warnings: &[CsvWarning]) {
    for w in warnings.iter().take(MAX_DETAILED_CSV_WARNINGS) {
        log_warn(&format!("{path}:{}: {}", w.line, w.kind));
    }
    if !warnings.is_empty() {
        let more = warnings.len().saturating_sub(MAX_DETAILED_CSV_WARNINGS);
        let suffix = if more > 0 {
            format!(" ({more} not shown individually)")
        } else {
            String::new()
        };
        log_warn(&format!(
            "{path}: {} CSV row problem(s){suffix}",
            warnings.len()
        ));
    }
}

/// Parse a CSV file's contents, recording its row warnings under `path`.
fn load_csv_events(
    contents: &str,
    log: &LogSource,
    path: &str,
) -> Result<Vec<Value>, CsvParseError> {
    let outcome = parse_csv_events(contents, log)?;
    log_csv_warnings(path, &outcome.warnings);
    Ok(outcome.events)
}

fn log_contents_to_events(log_contents: &str, log: &LogSource) -> Vec<Value> {
    match log {
        LogSource::Aws => {
            // Try parsing the whole file as a single JSON document first.
            if let Ok(json_value) = serde_json::from_str::<Value>(log_contents) {
                return aws_records(json_value);
            }
            // Fall back to JSONL: one JSON document per line, each of which may be a single
            // CloudTrail event or a `{ "Records": [...] }` batch.
            log_contents
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .flat_map(aws_records)
                .collect()
        }
        LogSource::Azure => {
            // Try parsing the whole file as a single JSON document first.
            if let Ok(json_value) = serde_json::from_str::<Value>(log_contents) {
                return azure_records(json_value);
            }
            // Fall back to JSONL: one JSON document per line, each of which may
            // itself be a `{ "records": [...] }` batch (Event Hub capture).
            log_contents
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .flat_map(azure_records)
                .collect()
        }
        LogSource::Gws => {
            // Try parsing the whole file as a single JSON document first.
            if let Ok(json_value) = serde_json::from_str::<Value>(log_contents) {
                return gws_records(json_value);
            }
            // Fall back to JSONL: one JSON document per line, each of which may itself be an
            // `activities.list` response page or an array.
            log_contents
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .flat_map(gws_records)
                .collect()
        }
        _ => vec![],
    }
}

/// Extract the individual CloudTrail events from one parsed JSON document. Handles the shapes
/// seen across CloudTrail exports: the standard `{ "Records": [...] }` delivery batch, a bare
/// array of events, or a single event object (e.g. one JSONL line).
fn aws_records(value: Value) -> Vec<Value> {
    match value {
        Value::Array(records) => records,
        Value::Object(mut map) => {
            if let Some(Value::Array(records)) = map.remove("Records") {
                records
            } else {
                vec![Value::Object(map)]
            }
        }
        _ => vec![],
    }
}

/// Extract the individual Azure records from one parsed JSON document. Handles the
/// shapes seen across Azure exports: a bare array of records, the Azure Monitor
/// diagnostic-settings / Event Hub batch envelope `{ "records": [...] }`, the REST
/// `{ "value": [...] }` shape, or a single record object.
fn azure_records(value: Value) -> Vec<Value> {
    match value {
        Value::Array(records) => records,
        Value::Object(mut map) => {
            if let Some(Value::Array(records)) = map.remove("records") {
                records
            } else if let Some(Value::Array(records)) = map.remove("value") {
                records
            } else {
                vec![Value::Object(map)]
            }
        }
        _ => vec![],
    }
}

fn detect_events<'a>(
    events: &[Value],
    context: &mut OutputContext<'a>,
    summary: &mut DetectionSummary,
    options: &TimelineOptions,
    rules: &Vec<&Rule>,
    matched_correlation: &mut Vec<TimestampedEvent<'a>>,
    engine: &'a CorrelationEngine,
) {
    // If all the events are loaded at once, it can consume too much memory.
    // To avoid the problem, we split the events into chunks.
    const CHUNK_SIZE: usize = 1000;
    let ts_key = context
        .prof_ts_key
        .strip_prefix(".")
        .unwrap_or(context.prof_ts_key);
    for event_chunks in events.chunks(CHUNK_SIZE) {
        // Convert loaded events into JSON
        // I call the collect() function at the end of this block due to a lifetime issue of json_event.
        // The ownership of json_event's reference is going to be moved in the next code block, so I ensure that the lifetime of json_event is longer than the next code block.
        let repeated_time_opt: rayon::iter::RepeatN<&TimeOption> =
            rayon::iter::repeat(&options.input_opt.time_opt).take(events.len());
        let json_events: Vec<(&Value, Event)> = event_chunks
            .par_iter()
            .zip(repeated_time_opt.into_par_iter())
            .filter_map(|(event, time_opt)| {
                if filter_by_time(time_opt, event, ts_key) {
                    Some(event)
                } else {
                    None
                }
            })
            .filter_map(|event| match event_from_json(event.to_string().as_str()) {
                Ok(json_event) => Some((event, json_event)),
                Err(_) => None,
            })
            .collect();
        // conduct rule's matches and return pairs of json_event and matched_rules
        let results: Vec<(&Value, &Event, Vec<&Rule>)> = json_events
            .par_iter()
            .map(|(event, json_event)| {
                let matched_rules: Vec<&Rule> = rules
                    .par_iter()
                    .filter(move |rule| {
                        rule.is_match(json_event)
                            && is_match_service(&rule.logsource.service, json_event)
                    })
                    .map(|rule| *rule)
                    .collect();
                (*event, json_event, matched_rules)
            })
            .collect();

        // process correlation base rules
        // Kept aligned with `json_events` (one entry per event) so the hit statistics below can
        // tell whether an event matched a base rule; flattened into `matched_correlation` after.
        let base_rule_matched: Vec<Vec<TimestampedEvent>> =
            process_correlation_base_rule(engine, &json_events, context);

        // perform post-processing
        // calculate some statistics values
        // An event counts once, here in the scan pass, which is the only place every event is
        // visited exactly once. It is "with hits" if any detection rule or any correlation base
        // rule matched it. The correlation pass used to add to this counter as well, once per
        // event per firing correlation result, so an event in several results (or already
        // counted here) was counted several times and `event_with_hits` could exceed
        // `total_events`, which pinned the reported data reduction at 0 events (0.00%).
        summary.event_with_hits += results
            .iter()
            .zip(base_rule_matched.iter())
            .filter(|((_, _, matched_rules), base_matched)| {
                !matched_rules.is_empty() || !base_matched.is_empty()
            })
            .count();
        summary.total_events += json_events.len();

        // The post-processing contains codes that shouldn't be executed in parallel, like setting values to variable summary, so please don't use rayon here.
        for (event, json_event, matched_rules) in results {
            for rule in matched_rules {
                // write to console
                write_record(json_event, event, Some(rule), context);
                append_summary_data(summary, json_event, rule, true, context);
            }
        }

        matched_correlation.extend(base_rule_matched.into_iter().flatten());
    }
}

/// Match every event against the correlation base rules, returning one `Vec` per event (in the
/// order of `json_events`) holding the events it produced — empty when no base rule matched.
/// The per-event grouping is what lets the caller count an event as "with hits" exactly once.
fn process_correlation_base_rule<'a>(
    engine: &'a CorrelationEngine,
    json_events: &[(&Value, Event)],
    context: &mut OutputContext,
) -> Vec<Vec<TimestampedEvent<'a>>> {
    json_events
        .par_iter()
        .map(|(_, event)| {
            engine
                .base_rules
                .values()
                .filter_map(|rule| {
                    // The timestamp must resolve through the profile's field spec: it is
                    // `.eventTime` for AWS and a `|`-separated list for Azure/M365, and neither
                    // form is a bare key `Event::get` would accept. An event whose time cannot be
                    // read cannot be placed in a correlation window, so it is dropped here.
                    if rule.is_match(event)
                        && let Some(timestamp) = event_timestamp(context.prof_ts_key, event)
                    {
                        return Some(TimestampedEvent {
                            event: event.clone(),
                            timestamp,
                            rule,
                        });
                    }
                    None
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn append_summary_data(
    summary: &mut DetectionSummary,
    event: &Event,
    rule: &Rule,
    generate: bool,
    context: &mut OutputContext,
) {
    // add information to summary
    if generate {
        if let Some(author) = &rule.author {
            summary
                .author_titles
                .entry(author.clone())
                .or_default()
                .insert(rule.title.clone());
        }

        if let Some(level) = &rule.level {
            let level = format!("{level:?}").to_lowercase();
            summary
                .level_with_hits
                .entry(level)
                .or_default()
                .entry(rule.title.clone())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }
    if let Some(event_time) = event_timestamp(context.prof_ts_key, event) {
        let unix_time = event_time.timestamp();
        summary.timestamps.push(unix_time);
        if summary.first_event_time.is_none() || event_time < summary.first_event_time.unwrap() {
            summary.first_event_time = Some(event_time);
        }
        if summary.last_event_time.is_none() || event_time > summary.last_event_time.unwrap() {
            summary.last_event_time = Some(event_time);
        }
        if let Some(level) = &rule.level
            && generate
        {
            let level = format!("{level:?}").to_lowercase();
            let date = event_time.date_naive().format("%Y-%m-%d").to_string();
            summary
                .dates_with_hits
                .entry(level)
                .or_default()
                .entry(date)
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }
}

/// Convert YYYYMMDD string to YYYY-MM-DD for display.
fn format_date_display(s: &str) -> String {
    if s.len() == 8 {
        format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8])
    } else {
        s.to_string()
    }
}

fn count_files_recursive(
    directory: &PathBuf,
    file_date_opt: &FileDateOption,
) -> Result<(usize, Vec<PathBuf>, u64), Box<dyn Error>> {
    let mut count = 0;
    let mut paths = Vec::new();
    let mut total_size = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str())
                && (ext == "json"
                    || ext == "jsonl"
                    || ext == "gz"
                    || ext == "csv"
                    || ext == "parquet")
            {
                // The date filter matches on the path string; filenames need not be valid UTF-8
                // (e.g. on Linux), so render lossily *only for the filter*. The real `PathBuf` is
                // stored so a non-UTF-8 name still resolves when the file is read later.
                if !filter_file_by_date_path(file_date_opt, &path.to_string_lossy()) {
                    continue;
                }
                count += 1;
                total_size += fs::metadata(&path)?.len();
                paths.push(path);
            }
        } else if path.is_dir() {
            let (sub_count, sub_paths, sub_size) = count_files_recursive(&path, file_date_opt)?;
            count += sub_count;
            total_size += sub_size;
            paths.extend(sub_paths);
        }
    }
    Ok((count, paths, total_size))
}

/// Upper bound on the decompressed size of a single `.gz` input. DEFLATE can inflate at
/// roughly 1032:1, so a few-MB archive can otherwise expand to many GB and OOM-kill the
/// whole scan. Generous enough for real logs, finite enough to stop a decompression bomb.
const MAX_DECOMPRESSED_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 GiB

pub fn read_gz_file(file_path: &PathBuf) -> io::Result<String> {
    read_gz_file_capped(file_path, MAX_DECOMPRESSED_BYTES)
}

/// Decompresses a gzip file, refusing to buffer more than `max_bytes` of decompressed data.
///
/// `Read::take` alone is not enough: it truncates silently and returns `Ok`, which would feed
/// a partial/corrupted log to the parser. Instead we read one byte past the ceiling and treat
/// hitting it as an error, so the caller's per-file handling (`Err(_) => continue` /
/// `unwrap_or_default()`) skips just that file and the scan continues.
fn read_gz_file_capped(file_path: &PathBuf, max_bytes: u64) -> io::Result<String> {
    let file = File::open(file_path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let mut buf = Vec::new();
    decoder.take(max_bytes + 1).read_to_end(&mut buf)?;
    if buf.len() as u64 > max_bytes {
        // The descriptive message is carried in the error so the caller's
        // single "[WARNING] Skipping <file>: <err>" line reports the reason.
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "decompressed size exceeds the {} GiB limit (possible gzip bomb)",
                max_bytes / (1024 * 1024 * 1024)
            ),
        ));
    }
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
/// CloudTrail fields that are objects in the original JSON schema. Parquet pipelines
/// (Athena CTAS, Glue, Firehose) often store these variable-schema fields as serialized
/// JSON strings because their keys differ per event; parse them back into objects so
/// rules can match nested values like `requestParameters.bucketName`. Only these known
/// envelope fields are parsed — free-text fields such as `errorMessage` are left alone
/// even if they happen to contain JSON-looking text.
const JSON_STRING_FIELDS: [&str; 8] = [
    "userIdentity",
    "requestParameters",
    "responseElements",
    "additionalEventData",
    "serviceEventDetails",
    "resources",
    "tlsDetails",
    "insightDetails",
];

/// Normalize one Parquet row into the shape the JSON pipeline produces: revive
/// JSON-string envelope fields and mark naive `eventTime` timestamps as UTC
/// (Parquet TIMESTAMP columns without a timezone render as "2026-05-03T16:05:07",
/// which the RFC 3339 consumers downstream — time filter, summary — reject;
/// CloudTrail timestamps are always UTC).
fn normalize_parquet_event(mut v: Value) -> Value {
    if let Value::Object(map) = &mut v {
        for key in JSON_STRING_FIELDS {
            if let Some(Value::String(s)) = map.get(key)
                && let Ok(parsed) = serde_json::from_str::<Value>(s)
                && matches!(parsed, Value::Object(_) | Value::Array(_))
            {
                map.insert(key.to_string(), parsed);
            }
        }
        if let Some(Value::String(ts)) = map.get_mut("eventTime")
            && NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
        {
            ts.push('Z');
        }
    }
    v
}

/// Upper bound on the JSON-decoded size of a single `.parquet` input. Parquet is a
/// compressed columnar format (this build enables snappy/zstd/gzip/lz4), so a small
/// crafted file can inflate to many GB and OOM-kill the whole scan — the same
/// decompression-bomb class capped for `.gz` above, and `.parquet` files are
/// auto-discovered during a `-d` walk over an attacker-influenceable log tree.
const MAX_PARQUET_DECODED_BYTES: u64 = 3 * 1024 * 1024 * 1024; // 3 GiB

/// Read a Parquet file and convert each row into one JSON event object.
/// Nested columns (structs/lists) become nested JSON values; see
/// `normalize_parquet_event` for how string-encoded envelope fields and
/// timezone-less timestamps are handled.
pub fn load_parquet_events(file_path: &PathBuf) -> Result<Vec<Value>, Box<dyn Error>> {
    load_parquet_events_capped(file_path, MAX_PARQUET_DECODED_BYTES)
}

/// Decodes a Parquet file into events, refusing to buffer more than `max_bytes` of
/// JSON-decoded data. Batches are serialized and accumulated one at a time so a bomb
/// is caught after roughly one batch past the ceiling rather than after the whole file
/// is expanded in memory; over the limit returns an error so the caller's per-file
/// handling (`Err(_) => return`/`if let Ok`) skips just that file and the scan continues.
fn load_parquet_events_capped(
    file_path: &PathBuf,
    max_bytes: u64,
) -> Result<Vec<Value>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;
    let mut events = Vec::new();
    let mut decoded_bytes: u64 = 0;
    for batch in reader {
        let mut buf = Vec::new();
        let mut writer = arrow_json::ArrayWriter::new(&mut buf);
        writer.write(&batch?)?;
        writer.finish()?;
        decoded_bytes += buf.len() as u64;
        if decoded_bytes > max_bytes {
            // The descriptive message is carried in the error so the caller's
            // single "Skipping <file>: <err>" line reports the reason.
            return Err(format!(
                "decoded size exceeds the {} GiB limit (possible parquet bomb)",
                max_bytes / (1024 * 1024 * 1024)
            )
            .into());
        }
        if buf.is_empty() {
            continue;
        }
        if let Value::Array(rows) = serde_json::from_slice::<Value>(&buf)? {
            events.extend(rows.into_iter().map(normalize_parquet_event));
        }
    }
    Ok(events)
}

/// Load AWS events from one file of any supported format (json/jsonl/gz/parquet/csv).
pub fn load_aws_events_from_file(f: &PathBuf) -> Result<Vec<Value>, Box<dyn Error>> {
    let path = f.display().to_string();
    if path.ends_with(".parquet") {
        load_parquet_events(f)
    } else if path.ends_with(".csv") {
        Ok(load_csv_events(&get_content(f), &LogSource::Aws, &path)?)
    } else {
        load_json_from_file(&get_content(f), &LogSource::Aws)
    }
}

pub fn load_json_from_file(
    log_contents: &str,
    log: &LogSource,
) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut events = Vec::new();
    match log {
        LogSource::Aws => {
            let log_contents_trimmed = log_contents
                .strip_prefix('\u{FEFF}')
                .unwrap_or(log_contents);
            match serde_json::from_str::<Value>(log_contents_trimmed) {
                // Array, `{ "Records": [...] }` batch, or a single event object.
                Ok(json_value) => events.extend(aws_records(json_value)),
                Err(_) => {
                    // Fall back to JSONL (each line may itself be a `Records` batch).
                    log_contents.lines().for_each(|line| {
                        if let Ok(json_value) = serde_json::from_str::<Value>(line) {
                            events.extend(aws_records(json_value));
                        }
                    });
                }
            }
        }
        LogSource::Azure => {
            let log_contents_trimmed = log_contents
                .strip_prefix('\u{FEFF}')
                .unwrap_or(log_contents);
            let json_value: Result<Value, _> = serde_json::from_str(log_contents_trimmed);
            match json_value {
                // Array, `{ records|value: [...] }` batch envelope, or a single record.
                Ok(json_value) => events.extend(azure_records(json_value)),
                Err(_) => {
                    // Fall back to JSONL (each line may itself be a `records` batch).
                    log_contents.lines().for_each(|line| {
                        if let Ok(json_value) = serde_json::from_str::<Value>(line) {
                            events.extend(azure_records(json_value));
                        }
                    });
                }
            }
        }
        LogSource::Gws => {
            let log_contents_trimmed = log_contents
                .strip_prefix('\u{FEFF}')
                .unwrap_or(log_contents);
            match serde_json::from_str::<Value>(log_contents_trimmed) {
                // Array, `activities.list` response page, or one activity.
                Ok(json_value) => events.extend(gws_records(json_value)),
                Err(_) => {
                    // Fall back to JSONL (each line may itself be any of those shapes).
                    log_contents.lines().for_each(|line| {
                        if let Ok(json_value) = serde_json::from_str::<Value>(line) {
                            events.extend(gws_records(json_value));
                        }
                    });
                }
            }
        }

        _ => {}
    }
    Ok(events)
}

pub fn get_content(f: &PathBuf) -> String {
    let path = f.display().to_string();
    let result = if path.ends_with(".json") || path.ends_with(".jsonl") || path.ends_with(".csv") {
        fs::read_to_string(f)
    } else if path.ends_with(".gz") {
        read_gz_file(f)
    } else {
        return "".to_string();
    };
    // Warn instead of silently returning empty content on a read failure.
    result.unwrap_or_else(|e| {
        log_warn(&format!("Skipping {path}: {e}"));
        String::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // The summary's date breakdown and its first/last event times are derived from the event
    // field the output profile NAMES -- `.eventTime` for AWS, `.time|.eventTimestamp|
    // .CreationTime` for Azure/M365. That spec is dot-prefixed and may be a fallback list, so
    // handing it to `Event::get` verbatim resolved nothing for any event: a run with hundreds of
    // detections still printed "n/a" for every level under "Dates with most total detections".
    // Verified by mutation: restoring `event.get(context.prof_ts_key)` fails this test for both
    // log sources.
    #[test]
    fn append_summary_data_records_event_times_from_the_shipped_profiles() {
        use crate::core::log_source::LogSource;
        use crate::core::timeline_writer::{OutputConfig, Writers};
        use crate::core::util::load_profile;
        use chrono::{TimeZone, Utc};
        use sigma_rust::{event_from_json, rule_from_yaml};

        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\nlevel: medium\n",
        )
        .unwrap();
        let expected = Utc.with_ymd_and_hms(2023, 7, 10, 12, 27, 45).unwrap();

        for (name, log, event_json) in [
            (
                "aws",
                LogSource::Aws,
                r#"{"eventTime": "2023-07-10T12:27:45Z", "eventName": "E"}"#,
            ),
            (
                "azure",
                LogSource::Azure,
                r#"{"time": "2023-07-10T12:27:45Z", "eventName": "E"}"#,
            ),
            // Google Workspace names a NESTED Timestamp spec (`.id.time`), which is exactly the
            // shape `Event::get` does not read verbatim.
            (
                "gws",
                LogSource::Gws,
                r#"{"kind": "admin#reports#activity", "id": {"applicationName": "login", "time": "2023-07-10T12:27:45Z"}, "eventName": "E"}"#,
            ),
        ] {
            let mut geo = None;
            let profile = load_profile(&log, &geo, true);
            let config = OutputConfig::new(true, false, false, false);
            let mut context = OutputContext::new(&profile, &mut geo, &config, Writers::new(), &[]);
            let mut summary = DetectionSummary::default();
            let event = event_from_json(event_json).unwrap();

            append_summary_data(&mut summary, &event, &rule, true, &mut context);

            assert_eq!(
                summary.first_event_time,
                Some(expected),
                "{name}: the event time must be read through the profile's Timestamp spec"
            );
            assert_eq!(summary.last_event_time, Some(expected), "{name}");
            assert_eq!(summary.timestamps, vec![expected.timestamp()], "{name}");
            assert_eq!(
                summary
                    .dates_with_hits
                    .get("medium")
                    .and_then(|d| d.get("2023-07-10")),
                Some(&1),
                "{name}: the date breakdown must not be empty"
            );
        }
    }

    #[test]
    fn test_load_parquet_events_athena_style() {
        // Envelope fields stored as JSON strings and eventTime as a naive TIMESTAMP,
        // the shape Athena CTAS / Glue pipelines produce.
        let result = load_parquet_events(&PathBuf::from("test_files/parquet/test.parquet"));
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 29);
        assert!(events[0]["userIdentity"].is_object());
        let ts = events[0]["eventTime"].as_str().unwrap();
        assert!(ts.ends_with('Z'), "eventTime not marked as UTC: {ts}");
    }

    #[test]
    fn test_load_parquet_events_nested_structs() {
        let result = load_parquet_events(&PathBuf::from("test_files/parquet/DeleteTrail.parquet"));
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["eventName"], "DeleteTrail");
        assert!(events[0]["userIdentity"].is_object());
    }

    #[test]
    fn test_load_parquet_events_bomb_cap() {
        let path = PathBuf::from("test_files/parquet/test.parquet");
        // A ceiling below the fixture's decoded size is rejected (skipped), not truncated.
        assert!(load_parquet_events_capped(&path, 1).is_err());
        // A generous ceiling loads every row.
        let events = load_parquet_events_capped(&path, MAX_PARQUET_DECODED_BYTES).unwrap();
        assert_eq!(events.len(), 29);
    }

    #[test]
    fn test_load_event_from_file() {
        let test_file = "test_files/json/DeleteTrail.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents, &LogSource::Aws);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 1);
    }

    #[test]
    fn test_load_event_from_file_record() {
        let test_file = "test_files/json/test.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents, &LogSource::Aws);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 29);
    }

    // --- Google Workspace ---

    /// The Reports API delivers a password reset and the forced change on next login as ONE
    /// activity with two `events[]` entries. Sigma cannot see inside an array, so an unsplit
    /// activity would match on at most its first sub-event and the second would be invisible.
    #[test]
    fn normalize_gws_event_splits_a_multi_event_activity() {
        let activity: Value = serde_json::from_str(
            r#"{
                "kind": "admin#reports#activity",
                "id": {"applicationName": "admin", "time": "2024-01-02T03:04:05.678Z",
                       "customerId": "C0example", "uniqueQualifier": "1234567890123456789"},
                "actor": {"callerType": "USER", "email": "admin@example.test"},
                "ipAddress": "203.0.113.86",
                "events": [
                    {"name": "CHANGE_PASSWORD", "type": "USER_SETTINGS",
                     "parameters": [{"name": "USER_EMAIL", "value": "victim@example.test"}]},
                    {"name": "CHANGE_PASSWORD_ON_NEXT_LOGIN", "type": "USER_SETTINGS",
                     "parameters": [
                        {"name": "USER_EMAIL", "value": "victim@example.test"},
                        {"name": "OLD_VALUE", "value": "false"},
                        {"name": "NEW_VALUE", "value": "true"}]}
                ]
            }"#,
        )
        .unwrap();

        let records = normalize_gws_event(activity);
        assert_eq!(records.len(), 2);
        for (i, record) in records.iter().enumerate() {
            assert_eq!(record["eventIndex"], i as u64, "eventIndex");
            assert_eq!(record["eventCount"], 2, "eventCount");
            assert_eq!(record["eventService"], "admin.googleapis.com");
            assert_eq!(record["eventType"], "USER_SETTINGS");
            // Every top-level activity field is carried onto each record, and `events` is gone.
            assert_eq!(record["kind"], "admin#reports#activity");
            assert_eq!(record["id"]["time"], "2024-01-02T03:04:05.678Z");
            assert_eq!(record["actor"]["email"], "admin@example.test");
            assert_eq!(record["ipAddress"], "203.0.113.86");
            assert!(record.get("events").is_none());
            assert_eq!(record["USER_EMAIL"], "victim@example.test");
            assert_eq!(record["user_email"], "victim@example.test");
        }
        assert_eq!(records[0]["eventName"], "CHANGE_PASSWORD");
        assert_eq!(records[1]["eventName"], "CHANGE_PASSWORD_ON_NEXT_LOGIN");
        // The UPPER_CASE parameter and its lowercase alias hold the same value, so an upstream
        // SigmaHQ gworkspace rule written against `new_value` loads unmodified.
        assert_eq!(records[1]["NEW_VALUE"], "true");
        assert_eq!(records[1]["new_value"], "true");
        assert_eq!(records[1]["OLD_VALUE"], "false");
        assert_eq!(records[1]["old_value"], "false");
        // The first sub-event carried no NEW_VALUE, so neither spelling may appear on it.
        assert!(records[0].get("NEW_VALUE").is_none());
        assert!(records[0].get("new_value").is_none());
        // No Cloud-Logging-shaped `protoPayload` is synthesized: a field Suzaku invents would end
        // up in every output format, including `--raw-output`, for the benefit of two upstream
        // rules. Those rules are expected to converge on the flat Reports API schema instead.
        assert!(records.iter().all(|r| r.get("protoPayload").is_none()));
    }

    /// Every `events[].parameters[]` value shape the Reports API emits folds into a top-level key.
    #[test]
    fn normalize_gws_event_folds_every_parameter_shape() {
        let activity: Value = serde_json::from_str(
            r#"{
                "kind": "admin#reports#activity",
                "id": {"applicationName": "login", "time": "2024-01-02T04:05:06.789Z"},
                "events": [{"name": "login_success", "type": "login",
                    "resourceIds": ["fixture00000001"],
                    "parameters": [
                        {"name": "login_type", "value": "reauth"},
                        {"name": "login_challenge_method", "multiValue": ["none", "idv_preregistered_phone"]},
                        {"name": "is_suspicious", "boolValue": true},
                        {"name": "start_time", "intValue": "63900000000"},
                        {"name": "not_a_number", "intValue": "12x"},
                        {"name": "counts", "multiIntValue": ["1", "2"]},
                        {"name": "recurrence_rule"},
                        {"name": "SETTING_METADATA", "messageValue": {"parameter": [
                            {"name": "rule_type", "value": "ALIAS_TABLE"},
                            {"name": "hits", "intValue": "3"}]}},
                        {"name": "SETTINGS", "multiMessageValue": [
                            {"parameter": [{"name": "k", "value": "v"}]}]}
                    ]}]
            }"#,
        )
        .unwrap();

        let records = normalize_gws_event(activity);
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert_eq!(r["eventService"], "login.googleapis.com");
        assert_eq!(r["eventCount"], 1);
        assert_eq!(r["resourceIds"], json!(["fixture00000001"]));
        assert_eq!(r["login_type"], "reauth");
        // A multiValue stays a list rather than being flattened into a string.
        assert_eq!(
            r["login_challenge_method"],
            json!(["none", "idv_preregistered_phone"])
        );
        assert_eq!(r["is_suspicious"], json!(true));
        // intValue arrives as a string and is parsed so numeric comparisons work; an unparseable
        // one is kept verbatim instead of being dropped.
        assert_eq!(r["start_time"], json!(63900000000i64));
        assert_eq!(r["not_a_number"], "12x");
        assert_eq!(r["counts"], json!([1, 2]));
        // A parameter with no value field at all is an elided protobuf default, NOT a set flag.
        // It is dropped, so `recurrence_rule: null` (and not `: true`) describes it in a rule.
        assert!(r.get("recurrence_rule").is_none());
        // A messageValue's own `parameter` array folds recursively into an object.
        assert_eq!(
            r["SETTING_METADATA"],
            json!({"rule_type": "ALIAS_TABLE", "hits": 3})
        );
        assert_eq!(r["setting_metadata"]["rule_type"], "ALIAS_TABLE");
        assert_eq!(r["SETTINGS"], json!([{"k": "v"}]));
    }

    /// An activity with no `events[]` array must still reach the rules, not be dropped.
    #[test]
    fn normalize_gws_event_keeps_an_activity_without_sub_events() {
        let activity: Value = serde_json::from_str(
            r#"{"kind": "admin#reports#activity",
                "id": {"applicationName": "drive", "time": "2024-01-02T05:06:07.890Z"}}"#,
        )
        .unwrap();
        let records = normalize_gws_event(activity);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["eventService"], "drive.googleapis.com");
        assert_eq!(records[0]["id"]["applicationName"], "drive");
    }

    /// An EMPTY `events[]` array is the same situation as a missing one: there is nothing to
    /// split on, so the activity must survive as one record instead of vanishing from the scan.
    #[test]
    fn normalize_gws_event_keeps_an_activity_with_an_empty_sub_event_array() {
        let activity: Value = serde_json::from_str(
            r#"{"kind": "admin#reports#activity",
                "id": {"applicationName": "drive", "time": "2024-01-02T05:06:07.890Z"},
                "events": []}"#,
        )
        .unwrap();
        let records = normalize_gws_event(activity);
        assert_eq!(
            records.len(),
            1,
            "an empty events[] must not drop the activity"
        );
        assert_eq!(records[0]["eventService"], "drive.googleapis.com");
        assert_eq!(records[0]["id"]["applicationName"], "drive");
        // The empty array is put back untouched, exactly as a non-array `events` value is.
        assert_eq!(records[0]["events"], json!([]));
    }

    /// `id.applicationName` is what `eventService` is built from. When it is absent there is no
    /// service to name, and emitting a bare ".googleapis.com" would hand rules and output columns
    /// a value that looks like a real application but is not one.
    #[test]
    fn normalize_gws_event_omits_event_service_without_an_application_name() {
        for activity in [
            r#"{"kind": "admin#reports#activity", "id": {"time": "2024-01-02T05:06:07.890Z"},
                "events": [{"name": "E"}]}"#,
            // The same must hold on the no-sub-events path.
            r#"{"kind": "admin#reports#activity", "id": {"time": "2024-01-02T05:06:07.890Z"}}"#,
        ] {
            let records = normalize_gws_event(serde_json::from_str(activity).unwrap());
            assert_eq!(records.len(), 1, "{activity}");
            assert!(
                records[0].get("eventService").is_none(),
                "eventService must be absent, not \".googleapis.com\": {}",
                records[0]
            );
        }
    }

    /// A parameter name is free-form data from the logged event. One called `eventName` must not
    /// displace the sub-event's real name, and the same holds for every other key the normalizer
    /// owns -- a record whose `id` is a parameter string resolves no timestamp at all.
    #[test]
    fn normalize_gws_event_parameters_never_clobber_synthesized_keys() {
        let activity: Value = serde_json::from_str(
            r#"{
                "kind": "admin#reports#activity",
                "id": {"applicationName": "drive", "time": "2024-01-02T03:04:05.678Z"},
                "actor": {"email": "owner@example.test"},
                "events": [{"name": "edit", "type": "access", "resourceIds": ["fixture00000001"],
                    "parameters": [
                        {"name": "eventName", "value": "SPOOFED"},
                        {"name": "eventType", "value": "SPOOFED"},
                        {"name": "eventService", "value": "spoofed.googleapis.com"},
                        {"name": "eventIndex", "intValue": "99"},
                        {"name": "eventCount", "intValue": "99"},
                        {"name": "resourceIds", "multiValue": ["spoofed"]},
                        {"name": "id", "value": "spoofed"},
                        {"name": "actor", "value": "spoofed"},
                        {"name": "kind", "value": "spoofed"},
                        {"name": "doc_title", "value": "quarterly plan"}
                    ]}]
            }"#,
        )
        .unwrap();

        let records = normalize_gws_event(activity);
        let r = &records[0];
        assert_eq!(r["eventName"], "edit");
        assert_eq!(r["eventType"], "access");
        assert_eq!(r["eventService"], "drive.googleapis.com");
        assert_eq!(r["eventIndex"], 0);
        assert_eq!(r["eventCount"], 1);
        assert_eq!(r["resourceIds"], json!(["fixture00000001"]));
        assert_eq!(r["id"]["time"], "2024-01-02T03:04:05.678Z");
        assert_eq!(r["actor"]["email"], "owner@example.test");
        assert_eq!(r["kind"], "admin#reports#activity");
        // A parameter that collides with nothing is still folded as usual.
        assert_eq!(r["doc_title"], "quarterly plan");
    }

    /// Every sub-event copies the whole activity, so the split is bounded: a crafted record with
    /// an enormous `events[]` must not be allowed to allocate without limit. The records past the
    /// cap are dropped, but `eventCount` still reports the activity's real length.
    #[test]
    fn normalize_gws_event_caps_the_number_of_sub_events() {
        let over = MAX_GWS_SUB_EVENTS + 5;
        let events: Vec<Value> = (0..over)
            .map(|i| json!({"name": format!("sub_event_{i}"), "type": "test"}))
            .collect();
        let activity = json!({
            "kind": "admin#reports#activity",
            "id": {"applicationName": "drive", "time": "2024-01-02T03:04:05.678Z"},
            "events": events,
        });

        let records = normalize_gws_event(activity);
        assert_eq!(records.len(), MAX_GWS_SUB_EVENTS);
        assert_eq!(records[0]["eventName"], "sub_event_0");
        assert_eq!(
            records[MAX_GWS_SUB_EVENTS - 1]["eventName"],
            format!("sub_event_{}", MAX_GWS_SUB_EVENTS - 1)
        );
        // The count is the activity's, not the truncated slice's, so the loss is visible.
        assert!(
            records
                .iter()
                .all(|r| r["eventCount"] == json!(over as u64))
        );
        assert_eq!(records[0]["id"]["applicationName"], "drive");
    }

    /// A native lowercase parameter is never clobbered by the alias of an UPPER_CASE twin.
    /// Drive emits `new_value` natively (as a list) while Admin emits `NEW_VALUE` (as a string),
    /// so both spellings must be able to hold different values in one record.
    #[test]
    fn normalize_gws_event_alias_never_overwrites_an_existing_key() {
        let activity: Value = serde_json::from_str(
            r#"{"kind": "admin#reports#activity", "id": {"applicationName": "admin"},
                "events": [{"name": "E", "parameters": [
                    {"name": "new_value", "multiValue": ["shared_externally"]},
                    {"name": "NEW_VALUE", "value": "true"}]}]}"#,
        )
        .unwrap();
        let records = normalize_gws_event(activity);
        assert_eq!(records[0]["new_value"], json!(["shared_externally"]));
        // The Reports API transports every `value` as a string, including "true"/"false".
        // Coercing it to a JSON bool would silently change what a Sigma rule has to write.
        assert_eq!(records[0]["NEW_VALUE"], json!("true"));
    }

    /// Nested activity objects stay nested, which is what the profile specs (`.id.time`,
    /// `.actor.email`, `.networkInfo.regionCode`) and a correlation `group-by: actor.email`
    /// resolve through. Flattening them would break every one of those paths.
    #[test]
    fn normalize_gws_event_keeps_activity_objects_nested() {
        use sigma_rust::event_from_json;

        let activity: Value = serde_json::from_str(
            r#"{"kind": "admin#reports#activity",
                "id": {"applicationName": "calendar", "time": "2024-01-02T03:04:05.678Z"},
                "actor": {"callerType": "USER", "email": "organizer@example.test"},
                "networkInfo": {"ipAsn": [64512], "regionCode": "ZZ"},
                "events": [{"name": "add_event_guest", "parameters": [
                    {"name": "event_id", "value": "fixtureevent00000000000001"}]}]}"#,
        )
        .unwrap();
        let records = normalize_gws_event(activity);
        let record = &records[0];
        assert_eq!(record["actor"]["email"], "organizer@example.test");
        assert_eq!(record["id"]["time"], "2024-01-02T03:04:05.678Z");
        assert_eq!(record["networkInfo"]["regionCode"], "ZZ");

        // The paths a correlation rule's `group-by` uses go through `Event::get`, which walks
        // nested maps but reads nothing from a flattened `"actor.email"` string key.
        let event = event_from_json(&record.to_string()).unwrap();
        assert_eq!(
            event.get("actor.email").map(|v| v.value_to_string()),
            Some("organizer@example.test".to_string())
        );
        assert_eq!(
            event.get("event_id").map(|v| v.value_to_string()),
            Some("fixtureevent00000000000001".to_string())
        );
        assert_eq!(
            event.get("id.time").map(|v| v.value_to_string()),
            Some("2024-01-02T03:04:05.678Z".to_string())
        );
    }

    /// The `activities.list` response page is unwrapped down to its `items`.
    #[test]
    fn gws_records_unwraps_a_reports_api_page() {
        let page = r#"{"kind": "admin#reports#activities", "etag": "x", "items": [
            {"kind": "admin#reports#activity", "id": {"applicationName": "login"},
             "events": [{"name": "login_success"}]},
            {"kind": "admin#reports#activity", "id": {"applicationName": "login"},
             "events": [{"name": "logout"}]}]}"#;
        let events = load_json_from_file(page, &LogSource::Gws).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["kind"], "admin#reports#activity");
        let normalized = normalize_events(events, &LogSource::Gws);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0]["eventName"], "login_success");
        assert_eq!(normalized[1]["eventName"], "logout");
    }

    /// An `activities.list` page that matched nothing carries `"items": null` (or an empty
    /// array). It must produce NO records: emitting the envelope itself would put an object no
    /// rule can match into the scan, inflating `total_events` and the data-reduction figure
    /// derived from it.
    #[test]
    fn gws_records_yields_nothing_for_an_empty_activities_page() {
        for page in [
            r#"{"kind": "admin#reports#activities", "etag": "x", "items": null}"#,
            r#"{"kind": "admin#reports#activities", "etag": "x", "items": []}"#,
        ] {
            assert!(
                load_json_from_file(page, &LogSource::Gws)
                    .unwrap()
                    .is_empty(),
                "{page}"
            );
            // The directory-scan path must agree with the file path.
            assert!(
                log_contents_to_events(page, &LogSource::Gws).is_empty(),
                "{page}"
            );
        }
    }

    /// JSONL where the lines are a mix of every accepted shape.
    #[test]
    fn gws_jsonl_accepts_mixed_shapes_per_line() {
        let jsonl = concat!(
            r#"{"kind":"admin#reports#activity","id":{"applicationName":"login"},"events":[{"name":"login_success"}]}"#,
            "\n",
            r#"[{"kind":"admin#reports#activity","id":{"applicationName":"login"},"events":[{"name":"logout"}]}]"#,
            "\n",
            r#"{"kind":"admin#reports#activities","items":[{"kind":"admin#reports#activity","id":{"applicationName":"admin"},"events":[{"name":"ASSIGN_ROLE"}]}]}"#,
            "\n",
            r#"{"kind":"admin#reports#activity","id":{"applicationName":"admin"},"events":[{"name":"CHANGE_PASSWORD"},{"name":"CHANGE_PASSWORD_ON_NEXT_LOGIN"}]}"#,
            "\n",
            "not json at all\n",
        );
        let events = load_json_from_file(jsonl, &LogSource::Gws).unwrap();
        assert_eq!(events.len(), 4, "one activity per parseable line");
        let normalized = normalize_events(events, &LogSource::Gws);
        // The last line's activity carries two sub-events, so it yields two records.
        let names: Vec<&str> = normalized
            .iter()
            .map(|r| r["eventName"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            vec![
                "login_success",
                "logout",
                "ASSIGN_ROLE",
                "CHANGE_PASSWORD",
                "CHANGE_PASSWORD_ON_NEXT_LOGIN"
            ]
        );
        // `log_contents_to_events` (the directory-scan path) must agree with the file path.
        assert_eq!(log_contents_to_events(jsonl, &LogSource::Gws).len(), 4);
    }

    /// The shipped fixtures load, split and normalize end to end.
    ///
    /// Asserts shape and invariants rather than exact counts: the fixtures are regenerated from
    /// freshly redacted data, and a test that pins their row count turns every refresh into a
    /// failure that says nothing about the code.
    #[test]
    fn gws_test_files_load_and_normalize() {
        let jsonl = fs::read_to_string("test_files/json/gws/login_admin_sample.jsonl").unwrap();
        let activities = load_json_from_file(&jsonl, &LogSource::Gws).unwrap();
        assert!(activities.len() > 100, "activities: {}", activities.len());
        let activity_count = activities.len();
        let records = normalize_events(activities, &LogSource::Gws);
        // Splitting multi-event activities yields strictly more records than activities.
        assert!(
            records.len() > activity_count,
            "{} records from {activity_count} activities",
            records.len()
        );
        assert!(records.iter().all(|r| r.get("eventName").is_some()));
        assert!(records.iter().all(|r| r.get("eventService").is_some()));
        // The CHANGE_PASSWORD pair survives the split.
        assert!(
            records
                .iter()
                .any(|r| r["eventName"] == "CHANGE_PASSWORD_ON_NEXT_LOGIN" && r["eventIndex"] == 1)
        );
        // A suspicious login is present and its boolValue folded to a real JSON bool.
        assert!(
            records
                .iter()
                .any(|r| r["eventName"] == "login_success" && r["is_suspicious"] == json!(true))
        );

        let page = fs::read_to_string("test_files/json/gws/activities_page.json").unwrap();
        let activities = load_json_from_file(&page, &LogSource::Gws).unwrap();
        assert!(activities.len() >= 10, "activities: {}", activities.len());
        let activity_count = activities.len();
        let records = normalize_events(activities, &LogSource::Gws);
        assert!(
            records.len() > activity_count,
            "{} records from {activity_count} activities",
            records.len()
        );
        // The multi-guest create_event is one activity with several `add_event_guest` siblings.
        assert!(
            records
                .iter()
                .any(|r| r["eventCount"].as_u64().unwrap() > 5)
        );
        assert!(
            records
                .iter()
                .all(|r| r["eventService"] == "calendar.googleapis.com")
        );
    }

    /// The `Timestamp` spec the shipped Google Workspace profile names must resolve: it points at
    /// the NESTED `id.time`, which neither `Event::get` nor `Value::get` reads verbatim.
    #[test]
    fn gws_profile_timestamp_resolves_nested_id_time() {
        use crate::core::util::load_profile;
        use chrono::{TimeZone, Utc};
        use sigma_rust::event_from_json;

        let profile = load_profile(&LogSource::Gws, &None, true);
        let (_, spec) = profile.iter().find(|(k, _)| k == "Timestamp").unwrap();
        let event = event_from_json(
            r#"{"kind":"admin#reports#activity","id":{"applicationName":"admin","time":"2024-01-02T03:04:05.678Z"}}"#,
        )
        .unwrap();
        assert_eq!(
            event_timestamp(spec, &event),
            Some(
                Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap()
                    + chrono::Duration::milliseconds(678)
            )
        );
    }

    #[test]
    fn test_load_azure_value_format() {
        let test_file = "test_files/json/azure_value_format.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents, &LogSource::Azure);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        // Verify that the event has expected fields
        assert!(events[0].get("caller").is_some());
        assert_eq!(
            events[0].get("caller").unwrap().as_str().unwrap(),
            "admin@contoso.com"
        );
    }

    #[test]
    fn test_load_azure_graph_api_format() {
        let test_file = "test_files/json/azure_graph_api_format.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents, &LogSource::Azure);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 3);

        // Verify first event has expected fields
        assert!(events[0].get("eventTimestamp").is_some());
        assert_eq!(
            events[0].get("eventTimestamp").unwrap().as_str().unwrap(),
            "2025-11-30T01:45:06.4650448Z"
        );
        assert!(events[0].get("caller").is_some());
        assert_eq!(
            events[0].get("caller").unwrap().as_str().unwrap(),
            "rob@contoso.com"
        );
    }

    #[test]
    fn test_normalize_unwraps_auditdata_string() {
        // CSV export shape: AuditData is a JSON string carrying the real record.
        let row = serde_json::json!({
            "RecordType": "AzureActiveDirectory",
            "AuditData": "{\"Operation\":\"UserLoggedIn\",\"Workload\":\"AzureActiveDirectory\"}"
        });
        let ev = normalize_azure_event(row);
        assert_eq!(
            ev.get("Operation").unwrap().as_str().unwrap(),
            "UserLoggedIn"
        );
        assert_eq!(
            ev.get("Workload").unwrap().as_str().unwrap(),
            "AzureActiveDirectory"
        );
    }

    #[test]
    fn test_normalize_unwraps_auditdata_object() {
        // JSON export shape: AuditData is a nested object.
        let row = serde_json::json!({
            "Operations": "New-InboxRule",
            "AuditData": {"Operation": "New-InboxRule", "Workload": "Exchange"}
        });
        let ev = normalize_azure_event(row);
        assert_eq!(
            ev.get("Operation").unwrap().as_str().unwrap(),
            "New-InboxRule"
        );
    }

    #[test]
    fn test_normalize_folds_name_value_property_bag() {
        // ExtendedProperties (array of {Name,Value}) is folded into an object so
        // nested keys like ExtendedProperties.UserAgent become matchable.
        let rec = serde_json::json!({
            "Operation": "UserLoggedIn",
            "Workload": "AzureActiveDirectory",
            "ExtendedProperties": [
                {"Name": "UserAgent", "Value": "azurehound/v2.0.4"},
                {"Name": "RequestType", "Value": "OAuth2"}
            ]
        });
        let ev = normalize_azure_event(rec);
        assert_eq!(
            ev.pointer("/ExtendedProperties/UserAgent")
                .unwrap()
                .as_str()
                .unwrap(),
            "azurehound/v2.0.4"
        );
    }

    #[test]
    fn test_normalize_leaves_non_ual_event_untouched() {
        // Azure Monitor diagnostic log (no Workload/RecordType) is unchanged.
        let rec = serde_json::json!({"category": "Administrative", "operationName": "x"});
        let ev = normalize_azure_event(rec.clone());
        assert_eq!(ev, rec);
    }

    #[test]
    fn test_parse_csv_events_unified_audit_log() {
        let csv = "\"RecordType\",\"Operations\",\"AuditData\"\r\n\
            \"AzureActiveDirectory\",\"UserLoggedIn\",\"{\"\"Operation\"\":\"\"UserLoggedIn\"\",\"\"Workload\"\":\"\"AzureActiveDirectory\"\"}\"\r\n";
        let rows = parse_csv_events(csv, &LogSource::Azure).unwrap().events;
        assert_eq!(rows.len(), 1);
        // Row carries AuditData as a string; normalization unwraps it to the record.
        let ev = normalize_azure_event(rows.into_iter().next().unwrap());
        assert_eq!(
            ev.get("Operation").unwrap().as_str().unwrap(),
            "UserLoggedIn"
        );
    }

    // -----------------------------------------------------------------------
    // AWS Management Console CloudTrail CSV
    // -----------------------------------------------------------------------

    const CONSOLE_CSV_FIXTURE: &str = "test_files/csv/aws_console_event_history.csv";
    const CONSOLE_CSV_HEADER: &str = "User name,AWS access key,Event time,Event source,Event name,AWS region,Source IP address,User agent,Error code,Resources,Request ID,Event ID,Read-only,Event type,Recipient Account Id,Event category";

    fn console_csv(rows: &[&str]) -> String {
        let mut s = CONSOLE_CSV_HEADER.to_string();
        for r in rows {
            s.push('\n');
            s.push_str(r);
        }
        s.push('\n');
        s
    }

    fn parse_aws(contents: &str) -> CsvParseOutcome {
        parse_csv_events(contents, &LogSource::Aws).unwrap()
    }

    #[test]
    fn console_csv_rows_become_cloudtrail_records() {
        let events = load_aws_events_from_file(&PathBuf::from(CONSOLE_CSV_FIXTURE)).unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(
            events[0],
            json!({
                "userIdentity": {"userName": "example-session", "accessKeyId": "ASIAIOSFODNN7EXAMPLE"},
                "eventTime": "2024-01-01T00:00:00Z",
                "eventSource": "s3.amazonaws.com",
                "eventName": "GetBucketAcl",
                "awsRegion": "us-east-1",
                "sourceIPAddress": "192.0.2.10",
                "userAgent": "aws-cli/2.15.0",
                "resources": [{"type": "AWS::S3::Bucket", "ARN": "example-bucket"}],
                "requestID": "11111111-1111-4111-8111-111111111111",
                "eventID": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                "readOnly": true,
                "eventType": "AwsApiCall",
                "recipientAccountId": "123456789012",
                "eventCategory": "Management",
            })
        );
        assert_eq!(events[1]["errorCode"], "AccessDenied");
        // `[]` means no resources, which CloudTrail expresses by omitting the field.
        assert!(events[1].get("resources").is_none());
        assert_eq!(events[3]["readOnly"], false);
        // An AWS service event carries no identity at all.
        assert!(events[4].get("userIdentity").is_none());
        assert_eq!(events[4]["eventType"], "AwsServiceEvent");
    }

    #[test]
    fn console_csv_empty_error_code_is_absent_so_exists_rules_do_not_match() {
        // Most CloudTrail rules test `errorCode|exists`. The console writes an empty `Error code`
        // for every successful call; turning that into `""` would make those rules match every
        // row. Evaluated through Sigma, not just by inspecting the JSON.
        let rule = sigma_rust::rule_from_yaml(
            r#"
title: t
logsource: {product: aws, service: cloudtrail}
detection:
    selection:
        eventName: GetBucketAcl
    error_code:
        errorCode|exists: true
    condition: selection and error_code
"#,
        )
        .unwrap();
        let events = load_aws_events_from_file(&PathBuf::from(CONSOLE_CSV_FIXTURE)).unwrap();
        assert!(events[0].get("errorCode").is_none());
        let matched: Vec<bool> = events[..2]
            .iter()
            .map(|e| rule.is_match(&event_from_json(&e.to_string()).unwrap()))
            .collect();
        assert_eq!(matched, [false, true]);
    }

    #[test]
    fn console_csv_root_user_gets_root_identity_type() {
        let events = load_aws_events_from_file(&PathBuf::from(CONSOLE_CSV_FIXTURE)).unwrap();
        assert_eq!(events[2]["userIdentity"]["userName"], "root");
        assert_eq!(events[2]["userIdentity"]["type"], "Root");
        // Nobody else is given a type: the CSV does not say what it is.
        assert!(events[0]["userIdentity"].get("type").is_none());
    }

    #[test]
    fn console_csv_with_bom_and_reordered_and_unknown_columns() {
        let csv = "\u{FEFF}Event name,Event source,Event time,Event ID,Some new column\n\
                   ListUsers,iam.amazonaws.com,2024-01-01T00:00:00Z,id-1,extra\n";
        let out = parse_aws(csv);
        assert!(out.warnings.is_empty());
        assert_eq!(
            out.events,
            [json!({
                "eventName": "ListUsers",
                "eventSource": "iam.amazonaws.com",
                "eventTime": "2024-01-01T00:00:00Z",
                "eventID": "id-1",
                // An unknown column is kept under its own name rather than dropped.
                "Some new column": "extra",
            })]
        );
    }

    #[test]
    fn console_csv_header_only_is_an_empty_success() {
        let out = parse_aws(&console_csv(&[]));
        assert!(out.events.is_empty());
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn aws_rejects_a_csv_that_is_not_a_console_download() {
        let ual = "\"RecordType\",\"Operations\",\"AuditData\"\r\n\"x\",\"y\",\"{}\"\r\n";
        assert!(matches!(
            parse_csv_events(ual, &LogSource::Aws),
            Err(CsvParseError::UnsupportedAwsFormat)
        ));
        assert!(matches!(
            parse_csv_events("", &LogSource::Aws),
            Err(CsvParseError::UnsupportedAwsFormat)
        ));
    }

    #[test]
    fn non_aws_log_sources_keep_console_csv_rows_flat() {
        // Only the AWS commands convert. `LogSource::All` is not used to scan logs, and keeps
        // the flat map like Azure.
        let csv = console_csv(&[
            "u,,2024-01-01T00:00:00Z,s3.amazonaws.com,GetBucketAcl,us-east-1,192.0.2.1,ua,,[],r,e,true,AwsApiCall,123456789012,Management",
        ]);
        for log in [LogSource::Azure, LogSource::All] {
            let out = parse_csv_events(&csv, &log).unwrap();
            assert_eq!(out.events[0]["Event name"], "GetBucketAcl");
            assert!(out.events[0].get("eventName").is_none());
        }
    }

    #[test]
    fn console_csv_row_with_wrong_field_count_is_skipped_and_reported() {
        // An unterminated quote in a middle column swallows the following rows into one field,
        // leaving a short row. The `csv` crate reports nothing; the field count gives it away.
        let csv = console_csv(&[
            "u,,2024-01-01T00:00:00Z,s3.amazonaws.com,A,us-east-1,192.0.2.1,\"ua,,[],r,e1,true,AwsApiCall,123456789012,Management",
            "u,,2024-01-01T00:00:01Z,s3.amazonaws.com,B,us-east-1,192.0.2.1,ua,,[],r,e2,true,AwsApiCall,123456789012,Management",
        ]);
        let out = parse_aws(&csv);
        assert!(out.events.is_empty());
        assert_eq!(
            out.warnings,
            [CsvWarning {
                line: 2,
                kind: CsvWarningKind::FieldCountMismatch {
                    expected: 16,
                    found: 8
                }
            }]
        );
    }

    #[test]
    fn console_csv_unterminated_quote_in_last_column_is_reported() {
        // In the last column the swallowed rows leave the field count intact, so the line break
        // inside the field is what gives it away. The row itself is kept.
        let csv = console_csv(&[
            "u,,2024-01-01T00:00:00Z,s3.amazonaws.com,A,us-east-1,192.0.2.1,ua,,[],r,e1,true,AwsApiCall,123456789012,\"Management",
            "u,,2024-01-01T00:00:01Z,s3.amazonaws.com,B,us-east-1,192.0.2.1,ua,,[],r,e2,true,AwsApiCall,123456789012,Management",
        ]);
        let out = parse_aws(&csv);
        assert_eq!(out.events.len(), 1);
        assert_eq!(
            out.warnings,
            [CsvWarning {
                line: 2,
                kind: CsvWarningKind::EmbeddedNewline
            }]
        );
    }

    #[test]
    fn console_csv_invalid_values_drop_the_field_or_the_row() {
        let csv = console_csv(&[
            // Read-only is neither true nor false: field dropped, row kept.
            "u,,2024-01-01T00:00:00Z,s3.amazonaws.com,A,us-east-1,192.0.2.1,ua,,[],r,e1,yes,AwsApiCall,123456789012,Management",
            // Resources is not JSON: field dropped, row kept.
            "u,,2024-01-01T00:00:01Z,s3.amazonaws.com,B,us-east-1,192.0.2.1,ua,,not-json,r,e2,true,AwsApiCall,123456789012,Management",
            // Resources is JSON but not an array of objects: same.
            "u,,2024-01-01T00:00:02Z,s3.amazonaws.com,C,us-east-1,192.0.2.1,ua,,\"[\"\"x\"\"]\",r,e3,true,AwsApiCall,123456789012,Management",
            // No Event time: row skipped.
            "u,,,s3.amazonaws.com,D,us-east-1,192.0.2.1,ua,,[],r,e4,true,AwsApiCall,123456789012,Management",
        ]);
        let out = parse_aws(&csv);
        let names: Vec<&str> = out
            .events
            .iter()
            .map(|e| e["eventName"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["A", "B", "C"]);
        assert!(out.events[0].get("readOnly").is_none());
        assert!(out.events[1].get("resources").is_none());
        assert!(out.events[2].get("resources").is_none());
        assert_eq!(
            out.warnings,
            [
                CsvWarning {
                    line: 2,
                    kind: CsvWarningKind::InvalidReadOnly("yes".to_string())
                },
                CsvWarning {
                    line: 3,
                    kind: CsvWarningKind::InvalidResources
                },
                CsvWarning {
                    line: 4,
                    kind: CsvWarningKind::InvalidResources
                },
                CsvWarning {
                    line: 5,
                    kind: CsvWarningKind::MissingEventTime
                },
            ]
        );
    }

    #[test]
    fn console_csv_line_numbers_are_right_for_crlf_and_quoted_line_breaks() {
        // `csv::Position::line()` is off by one for CRLF input, which is what the console
        // downloads, so the line is derived from the byte offset.
        let bad = "u,,,s3.amazonaws.com,X,us-east-1,192.0.2.1,ua,,[],r,e,true,AwsApiCall,123456789012,Management";
        let ok = "u,,2024-01-01T00:00:00Z,s3.amazonaws.com,A,us-east-1,192.0.2.1,ua,,[],r,e,true,AwsApiCall,123456789012,Management";
        let crlf = console_csv(&[ok, bad]).replace('\n', "\r\n");
        assert_eq!(parse_aws(&crlf).warnings[0].line, 3);
        // A quoted line break makes a row span two lines; the next row starts two lines later.
        let multiline = console_csv(&[&ok.replace(",ua,", ",\"u\na\","), bad]);
        assert_eq!(parse_aws(&multiline).warnings.last().unwrap().line, 4);
    }

    #[test]
    fn console_csv_warnings_beyond_the_detail_cap_are_still_counted() {
        let bad = "u,,,s3.amazonaws.com,X,us-east-1,192.0.2.1,ua,,[],r,e,true,AwsApiCall,123456789012,Management";
        let rows = vec![bad; MAX_DETAILED_CSV_WARNINGS + 5];
        let out = parse_aws(&console_csv(&rows));
        assert_eq!(out.warnings.len(), MAX_DETAILED_CSV_WARNINGS + 5);
        assert!(out.events.is_empty());
    }

    #[test]
    fn unrecognized_csv_is_not_counted_as_scanned() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("ual.csv"), "RecordType,AuditData\nx,{}\n").unwrap();
        fs::copy(CONSOLE_CSV_FIXTURE, dir.path().join("console.csv")).unwrap();
        let mut processed = 0usize;
        let scanned = process_events_from_dir(
            |events: &[Value]| processed += events.len(),
            &dir.path().to_path_buf(),
            false,
            true,
            &LogSource::Aws,
            &FileDateOption::default(),
        )
        .unwrap();
        assert_eq!(scanned, 1, "only the console CSV is a scanned file");
        assert_eq!(processed, 5);
        // The single-file path reports the same file as a failure instead of an empty success.
        assert!(load_aws_events_from_file(&dir.path().join("ual.csv")).is_err());
    }

    #[test]
    fn scan_file_fails_on_unrecognized_csv_and_succeeds_on_console_csv() {
        // `aws-ct-timeline -f` counts the file as scanned only when `scan_file` succeeds.
        use crate::core::timeline_writer::{OutputConfig, Writers};
        use crate::core::util::load_profile;

        let dir = tempfile::tempdir().unwrap();
        let ual = dir.path().join("ual.csv");
        fs::write(&ual, "RecordType,AuditData\nx,{}\n").unwrap();
        let mut geo = None;
        let profile = load_profile(&LogSource::Aws, &geo, true);
        let config = OutputConfig::new(true, false, false, false);
        let mut context = OutputContext::new(&profile, &mut geo, &config, Writers::new(), &[]);
        let mut summary = DetectionSummary::default();
        let options = TimelineOptions::default();
        let engine = CorrelationEngine::new();
        let mut matched_correlation = Vec::new();
        for (path, ok) in [(ual, false), (PathBuf::from(CONSOLE_CSV_FIXTURE), true)] {
            let result = scan_file(
                &path,
                &mut context,
                &mut summary,
                &options,
                &vec![],
                &mut matched_correlation,
                &engine,
                &LogSource::Aws,
            );
            assert_eq!(result.is_ok(), ok, "{}", path.display());
        }
    }

    #[test]
    fn test_normalize_synthesizes_deterministic_details_summary() {
        // The `_Details` field summarizes the change (Exchange cmdlet Parameters)
        // for the output timeline, in a stable key order.
        let rec = serde_json::json!({
            "Operation": "Set-Mailbox",
            "Workload": "Exchange",
            "Parameters": [
                {"Name": "ForwardingSmtpAddress", "Value": "attacker@evil.com"},
                {"Name": "DeliverToMailboxAndForward", "Value": "True"}
            ]
        });
        let ev = normalize_azure_event(rec);
        let details = ev.get("_Details").unwrap().as_str().unwrap();
        assert!(details.contains("ForwardingSmtpAddress: attacker@evil.com"));
        // serde_json object iteration is deterministic, so the summary is stable.
        assert_eq!(
            details,
            "DeliverToMailboxAndForward: True, ForwardingSmtpAddress: attacker@evil.com"
        );
    }

    #[test]
    fn test_azure_records_unwraps_batch_envelope() {
        // Azure Monitor diagnostic-settings / Event Hub blobs wrap events as
        // `{ "records": [...] }`; each record must become its own event.
        let contents = r#"{"records":[{"category":"SignInLogs","properties":{"a":1}},{"category":"SignInLogs","properties":{"a":2}}]}"#;
        let events = log_contents_to_events(contents, &LogSource::Azure);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].get("category").unwrap().as_str().unwrap(),
            "SignInLogs"
        );
    }

    #[test]
    fn test_azure_records_unwraps_per_line_batches() {
        // Event Hub capture can write one `{ "records": [...] }` batch per line.
        let contents = concat!(
            r#"{"records":[{"category":"AuditLogs","properties":{"a":1}}]}"#,
            "\n",
            r#"{"records":[{"category":"AuditLogs","properties":{"a":2}},{"category":"AuditLogs","properties":{"a":3}}]}"#,
        );
        let events = log_contents_to_events(contents, &LogSource::Azure);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_azure_records_helper_shapes() {
        // bare array
        assert_eq!(azure_records(serde_json::json!([{"x":1},{"x":2}])).len(), 2);
        // { value: [...] } REST shape
        assert_eq!(
            azure_records(serde_json::json!({"value":[{"x":1}]})).len(),
            1
        );
        // single record object
        assert_eq!(
            azure_records(serde_json::json!({"category":"SignInLogs"})).len(),
            1
        );
    }

    #[test]
    fn test_azure_single_object_json_is_one_event() {
        // A bare (or pretty-printed) single JSON object must parse to one event.
        let contents = "{\n  \"Operation\": \"Set-Mailbox\",\n  \"Workload\": \"Exchange\"\n}";
        let events = log_contents_to_events(contents, &LogSource::Azure);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].get("Operation").unwrap().as_str().unwrap(),
            "Set-Mailbox"
        );
    }

    #[test]
    fn test_aws_records_helper_shapes() {
        // standard CloudTrail `{ "Records": [...] }` batch
        assert_eq!(
            aws_records(serde_json::json!({"Records":[{"eventName":"A"},{"eventName":"B"}]})).len(),
            2
        );
        // bare array of events
        assert_eq!(aws_records(serde_json::json!([{"eventName":"A"}])).len(), 1);
        // single event object (one JSONL line)
        assert_eq!(
            aws_records(serde_json::json!({"eventName":"A","eventSource":"iam.amazonaws.com"}))
                .len(),
            1
        );
    }

    #[test]
    fn test_aws_jsonl_content_is_parsed() {
        // CloudTrail exported as JSONL: one event object per line.
        let contents = concat!(
            r#"{"eventName":"ConsoleLogin","eventSource":"signin.amazonaws.com"}"#,
            "\n",
            r#"{"eventName":"RunInstances","eventSource":"ec2.amazonaws.com"}"#,
            "\n",
            r#"{"eventName":"PutObject","eventSource":"s3.amazonaws.com"}"#,
        );
        let events = log_contents_to_events(contents, &LogSource::Aws);
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0].get("eventName").unwrap().as_str().unwrap(),
            "ConsoleLogin"
        );
    }

    #[test]
    fn test_aws_jsonl_per_line_batches_are_parsed() {
        // Each JSONL line may itself be a `{ "Records": [...] }` batch.
        let contents = concat!(
            r#"{"Records":[{"eventName":"A"}]}"#,
            "\n",
            r#"{"Records":[{"eventName":"B"},{"eventName":"C"}]}"#,
        );
        assert_eq!(log_contents_to_events(contents, &LogSource::Aws).len(), 3);
    }

    #[test]
    fn test_aws_batch_and_array_whole_file_still_parse() {
        // Regression: the standard whole-file shapes must keep working.
        let batch = r#"{"Records":[{"eventName":"A"},{"eventName":"B"}]}"#;
        assert_eq!(log_contents_to_events(batch, &LogSource::Aws).len(), 2);
        let array = r#"[{"eventName":"A"},{"eventName":"B"},{"eventName":"C"}]"#;
        assert_eq!(log_contents_to_events(array, &LogSource::Aws).len(), 3);
    }

    #[test]
    fn test_load_json_from_file_aws_handles_jsonl() {
        // The path used by aws-ct-metrics/search/summary must also read JSONL.
        let contents = concat!(
            r#"{"eventName":"A"}"#,
            "\n",
            r#"{"Records":[{"eventName":"B"},{"eventName":"C"}]}"#,
        );
        let events = load_json_from_file(contents, &LogSource::Aws).unwrap();
        assert_eq!(events.len(), 3);
    }

    fn write_gz(path: &PathBuf, decompressed: &[u8]) {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write as _;
        let mut enc = GzEncoder::new(File::create(path).unwrap(), Compression::default());
        enc.write_all(decompressed).unwrap();
        enc.finish().unwrap();
    }

    #[test]
    fn read_gz_file_capped_rejects_oversized_decompression() {
        // 200 decompressed bytes with a 16-byte cap must error (not return partial data),
        // so the caller skips the file instead of the process OOM-ing on a bomb.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bomb.json.gz");
        write_gz(&path, &[b'A'; 200]);
        assert!(read_gz_file_capped(&path, 16).is_err());
    }

    #[test]
    fn read_gz_file_capped_accepts_within_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ok.json.gz");
        write_gz(&path, b"[]");
        assert_eq!(read_gz_file_capped(&path, 1024).unwrap(), "[]");
    }

    #[test]
    fn read_gz_file_capped_accepts_exactly_at_limit() {
        // Exactly `max_bytes` decompressed is allowed; only strictly-over is rejected.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("edge.json.gz");
        write_gz(&path, &[b'x'; 32]);
        assert_eq!(read_gz_file_capped(&path, 32).unwrap().len(), 32);
    }

    // A file whose extension is valid UTF-8 (.json) but whose stem is not must not panic the
    // count walk that runs before any file is processed (issue #149, case 1).
    #[cfg(unix)]
    #[test]
    fn count_files_recursive_handles_non_utf8_filename() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(OsStr::from_bytes(b"bad-\xff.json"));
        // Some filesystems reject non-UTF-8 names at creation; only assert when it was created.
        if fs::write(&path, b"[]").is_err() {
            return;
        }
        let (count, paths, _size) =
            count_files_recursive(&dir.path().to_path_buf(), &FileDateOption::default())
                .expect("non-UTF-8 filename must not panic the count walk");
        assert_eq!(count, 1);
        assert_eq!(paths.len(), 1);
    }

    // Regression for the `-o`/progress path: `show_progress = true` previously reached
    // `fs::metadata(&path).unwrap()` on the lossy (non-resolving) path and panicked.
    #[cfg(unix)]
    #[test]
    fn process_events_from_dir_with_progress_survives_non_utf8_filename() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(OsStr::from_bytes(b"bad-\xff.json"));
        if fs::write(&path, b"[]").is_err() {
            return;
        }
        let result = process_events_from_dir(
            |_events: &[Value]| {},
            &dir.path().to_path_buf(),
            true, // show_progress (as when -o/--output is set)
            true, // no_color
            &LogSource::Aws,
            &FileDateOption::default(),
        );
        assert!(
            result.is_ok(),
            "scanning a directory containing a non-UTF-8 filename must not panic"
        );
    }

    // A non-UTF-8 filename must be actually READ and its events processed, not merely counted
    // and skipped (the real PathBuf is kept through the pipeline instead of a lossy string).
    #[cfg(unix)]
    #[test]
    fn process_events_from_dir_reads_non_utf8_filename() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(OsStr::from_bytes(b"bad-\xff.json"));
        // Non-UTF-8 name, but valid JSON content with one event.
        if fs::write(
            &path,
            br#"[{"eventName":"X","eventTime":"2024-01-01T00:00:00Z"}]"#,
        )
        .is_err()
        {
            return;
        }
        let mut processed = 0usize;
        let result = process_events_from_dir(
            |events: &[Value]| processed += events.len(),
            &dir.path().to_path_buf(),
            false,
            true,
            &LogSource::Aws,
            &FileDateOption::default(),
        );
        assert!(result.is_ok());
        assert_eq!(
            processed, 1,
            "the event in the non-UTF-8-named file must be processed, not skipped"
        );
    }
}
