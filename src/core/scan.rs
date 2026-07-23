use crate::core::color::SuzakuColor::{Green, Orange, Red};
use crate::core::log_source::{LogSource, is_match_service};
use crate::core::summary::DetectionSummary;
use crate::core::timeline_writer::{OutputContext, write_record};
use crate::core::util::p;
use crate::option::cli::{FileDateOption, TimeOption, TimelineOptions};
use crate::option::timefiler::{filter_by_time, filter_file_by_date_path};
use bytesize::ByteSize;
use chrono::{DateTime, Utc};
use colored::Colorize;
use console::style;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use num_format::{Locale, ToFormattedString};
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
) {
    let log_contents = get_content(f);
    let events = if f.display().to_string().ends_with(".csv") {
        parse_csv_events(&log_contents)
    } else {
        match load_json_from_file(&log_contents, log) {
            Ok(value) => value,
            Err(_e) => return,
        }
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
}

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
) {
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
    if let Err(e) = process_events_from_dir(
        process_events,
        d,
        options.output_opt.output.is_some(),
        no_color,
        log,
        &options.input_opt.file_date_opt,
    ) {
        p(
            Red.rdg(no_color),
            &format!("Failed to scan directory {}: {e}", d.display()),
            true,
        );
    }
}

pub fn process_events_from_dir<F>(
    mut process_events: F,
    directory: &PathBuf,
    show_progress: bool,
    no_color: bool,
    log: &LogSource,
    file_date_opt: &FileDateOption,
) -> Result<(), Box<dyn Error>>
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
                    eprintln!("[WARNING] Skipping {path_str}: {e}");
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
                    eprintln!("[WARNING] Skipping {path_str}: {e}");
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
            parse_csv_events(&log_contents)
        } else {
            log_contents_to_events(&log_contents, log)
        };
        let events = normalize_events(events, log);
        process_events(&events);

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
    Ok(())
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

/// Apply `normalize_azure_event` to every event when scanning Azure logs.
fn normalize_events(events: Vec<Value>, log: &LogSource) -> Vec<Value> {
    match log {
        LogSource::Azure => events.into_iter().map(normalize_azure_event).collect(),
        _ => events,
    }
}

/// Parse a `Search-UnifiedAuditLog` CSV export into one JSON object per row.
/// The real audit record is carried in the `AuditData` column and is unwrapped
/// afterwards by `normalize_azure_event`.
fn parse_csv_events(contents: &str) -> Vec<Value> {
    let mut events = Vec::new();
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(contents.as_bytes());
    let headers = match reader.headers() {
        Ok(h) => h.clone(),
        Err(_) => return events,
    };
    for record in reader.records().flatten() {
        let mut map = serde_json::Map::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            map.insert(header.to_string(), Value::String(field.to_string()));
        }
        events.push(Value::Object(map));
    }
    events
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

        // perform post-processing
        // calculate some statistics values
        summary.event_with_hits += results
            .iter()
            .filter(|(_, _, matched_rules)| !matched_rules.is_empty())
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

        // process correlation base rules
        let base_rule_matched: Vec<TimestampedEvent> =
            process_correlation_base_rule(engine, json_events, context);
        matched_correlation.extend(base_rule_matched);
    }
}

fn process_correlation_base_rule<'a>(
    engine: &'a CorrelationEngine,
    json_events: Vec<(&Value, Event)>,
    context: &mut OutputContext,
) -> Vec<TimestampedEvent<'a>> {
    json_events
        .par_iter()
        .flat_map(|(_, event)| {
            engine
                .base_rules
                .values()
                .filter_map(|rule| {
                    if rule.is_match(event)
                        && let Some(timestamp_field) = event.get(context.prof_ts_key)
                    {
                        let ts = timestamp_field.value_to_string();
                        if let Ok(parsed_time) = DateTime::parse_from_rfc3339(&ts) {
                            let utc_time = parsed_time.with_timezone(&Utc);
                            return Some(TimestampedEvent {
                                event: event.clone(),
                                timestamp: utc_time,
                                rule,
                            });
                        }
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
    if let Some(event_time) = event.get(context.prof_ts_key) {
        let event_time_str = event_time.value_to_string();
        if let Ok(event_time) = event_time_str.parse::<DateTime<Utc>>() {
            let unix_time = event_time.timestamp();
            summary.timestamps.push(unix_time);
            if summary.first_event_time.is_none() || event_time < summary.first_event_time.unwrap()
            {
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
                && (ext == "json" || ext == "jsonl" || ext == "gz" || ext == "csv")
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
        eprintln!("[WARNING] Skipping {path}: {e}");
        String::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let rows = parse_csv_events(csv);
        assert_eq!(rows.len(), 1);
        // Row carries AuditData as a string; normalization unwraps it to the record.
        let ev = normalize_azure_event(rows.into_iter().next().unwrap());
        assert_eq!(
            ev.get("Operation").unwrap().as_str().unwrap(),
            "UserLoggedIn"
        );
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
