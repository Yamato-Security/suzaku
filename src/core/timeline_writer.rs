use crate::core::color::SuzakuColor;
use crate::core::color::SuzakuColor::{Green, Orange, Red, White, Yellow};
use crate::core::duckdb_out::{
    self, GEO_COLUMN_COMMENT, GEO_COLUMNS, LEVEL_TYPE, MULTI_VALUE_SEPARATOR, SuzakuMeta,
    list_expr, nullable, quote_ident, timestamp_expr,
};
use crate::core::errorlog::log_error;
use crate::core::util::{get_json_writer, get_writer, sanitize_csv_field};
use crate::option::cli::OutputFormat;
use crate::option::geoip::{GeoIPSearch, parse_ip};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use csv::Writer;
use duckdb::{Connection, ToSql};
use itertools::Itertools;
use serde_json::Value;
use sigma_rust::{Event, Rule, SigmaCorrelationRule, TimestampedEvent};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use termcolor::{BufferWriter, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
pub struct OutputConfig {
    pub no_color: bool,
    pub raw_output: bool,
    pub localtime: bool,
}

/// Formats an event timestamp for output.
///
/// By default the value is shown in UTC (`T`/`Z` stripped, e.g. `2023-07-10 12:27:45`). When
/// `localtime` is set, the timestamp is parsed (RFC 3339, or a naive datetime assumed to be UTC)
/// and rendered in the local timezone with an explicit offset, e.g. `2023-07-10 21:27:45+09:00`.
/// Unparseable values fall back to the UTC rendering so nothing is dropped.
fn format_timestamp(value: &str, localtime: bool) -> String {
    if !localtime {
        return value.replace("T", " ").replace("Z", "");
    }
    match parse_event_time(value) {
        Some(u) => u
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S%:z")
            .to_string(),
        None => value.replace("T", " ").replace("Z", ""),
    }
}

/// Parses an event timestamp into a UTC instant.
///
/// Accepts RFC 3339 (`2023-07-10T12:27:45Z`, `2023-07-10T21:27:45+09:00`) and a naive datetime,
/// which is assumed to be UTC. This is the same ladder [`format_timestamp`] renders with, so the
/// instant a timestamp is *displayed* as and the instant it is *analysed* as can never disagree.
/// Anything else yields `None`; a caller decides whether to skip the value or place it last.
fn parse_event_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
                .ok()
                .map(|ndt| Utc.from_utc_datetime(&ndt))
        })
}

/// Resolves an output-profile field spec against an event, returning the bare key that matched
/// and its value.
///
/// A spec is dot-prefixed and may be a `|`-separated fallback list: `.eventTime` for AWS,
/// `.time|.eventTimestamp|.CreationTime` for Azure/M365. [`Event::get`] takes a *bare* field name
/// and reads `.` as a nesting separator, so handing it a spec verbatim resolves nothing at all —
/// `event.get(".eventTime")` looks for an empty-named parent map and returns `None` for every
/// event on earth. Every caller that needs a profile-named field must go through here.
///
/// The first candidate PRESENT wins, which is exactly how the column itself is rendered, so a
/// caller always sees the same field the output shows. The matched key is returned because the
/// caller may need it to decide what kind of value it has (see [`get_value_from_event_common`]).
fn resolve_field_spec<'a>(spec: &'a str, event: &Event) -> Option<(&'a str, String)> {
    spec.split('|')
        .map(|k| k.trim_matches('.').trim())
        .filter(|k| !k.is_empty())
        .find_map(|k| event.get(k).map(|v| (k, v.value_to_string())))
}

/// The event's timestamp as a UTC instant, resolved through the profile's `Timestamp` field spec.
///
/// `None` when the field is absent or unparseable — the caller decides what that means, and no
/// caller may assume it cannot happen: cloud logs carry malformed timestamps.
pub fn event_timestamp(spec: &str, event: &Event) -> Option<DateTime<Utc>> {
    resolve_field_spec(spec, event).and_then(|(_, value)| parse_event_time(&value))
}

pub struct Writers {
    csv: Option<Writer<Box<dyn Write>>>,
    json: Option<BufWriter<Box<dyn Write>>>,
    jsonl: Option<BufWriter<Box<dyn Write>>>,
    duckdb: Option<DuckDbSink>,
    std: Option<BufferWriter>,
}

/// How many rows are buffered before being handed to a DuckDB `Appender` in one go. Large enough
/// that the per-batch appender setup disappears into the noise, small enough that peak memory
/// stays bounded on huge scans.
const DUCKDB_BATCH_ROWS: usize = 10_000;

/// Name of the staging table rows are appended to before the final typed rewrite.
///
/// It is a `TEMP` table on purpose: temp data lives outside the database file, so the finished
/// `.duckdb` holds only the typed `timeline` table and does not carry the raw copy's blocks
/// around forever (DuckDB reuses freed blocks but never shrinks the file).
const DUCKDB_STAGING_TABLE: &str = "suzaku_timeline_staging";

/// The grain of the `timeline` table, recorded as a table comment so a reader can discover it
/// from the file instead of guessing. `EventID` is deliberately *not* unique: one event matching
/// several rules legitimately produces one row per match.
const TIMELINE_GRAIN: &str = "One row per (event x rule match): an event matching several rules \
     produces one row per match, so EventID is not unique. Exact-duplicate rows are removed on \
     write; see suzaku_meta.duplicate_rows_removed.";

/// How a rule tag is rendered once abbreviated (see [`abbreviate_tag`]), so the packed `Tags`
/// string can be split into typed lists instead of leaving consumers to guess with a
/// "starts with T and a digit" heuristic.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TagKind {
    /// An ATT&CK tactic abbreviation from `config/mitre_tactics.txt`, e.g. `PrivEsc`.
    Tactic,
    /// An ATT&CK technique ID, e.g. `T1078.004`.
    Technique,
    /// Anything else Suzaku passes through: group IDs (`G0035`), `cve.*`, `car.*`.
    Other,
}

/// Classify one already-abbreviated tag. Done here rather than in SQL because this is where the
/// tag vocabulary is known — `mitre_tactics()` is the same table that produced the abbreviation.
fn classify_tag(tag: &str) -> TagKind {
    if mitre_tactics().values().any(|abbrev| abbrev == tag) {
        return TagKind::Tactic;
    }
    let mut chars = tag.chars();
    if matches!(chars.next(), Some('T')) && matches!(chars.next(), Some(c) if c.is_ascii_digit()) {
        return TagKind::Technique;
    }
    TagKind::Other
}

/// Split a packed `Tags` value into the `(tactics, techniques, other)` triple written to the
/// three list columns, each still [`MULTI_VALUE_SEPARATOR`]-joined for the staging table.
fn split_tags(tags: &str) -> [String; 3] {
    let mut buckets = [Vec::new(), Vec::new(), Vec::new()];
    for tag in tags.split(MULTI_VALUE_SEPARATOR) {
        let tag = tag.trim();
        if tag.is_empty() || tag == "-" {
            continue;
        }
        let bucket = match classify_tag(tag) {
            TagKind::Tactic => 0,
            TagKind::Technique => 1,
            TagKind::Other => 2,
        };
        buckets[bucket].push(tag);
    }
    buckets.map(|b| b.join(MULTI_VALUE_SEPARATOR))
}

/// The DuckDB columns the `Tags` profile column expands into, in output order.
const TAG_COLUMNS: [&str; 3] = ["Tactics", "TechniqueIDs", "OtherTags"];

/// Column name used in the DuckDB output for an output-profile key.
///
/// The only rewrite today is `AWS-Region` -> `AwsRegion`. A hyphen makes the identifier illegal
/// unquoted, so every consumer — and every piece of ad-hoc or generated SQL — has to remember to
/// double-quote it forever, which is a cost paid on every query to save one character here.
///
/// Shared with `aws-ct-metrics`, which names the timeline column a metric aggregates so the two
/// commands cannot end up spelling the same fact differently.
pub fn duckdb_column_name(profile_key: &str) -> String {
    match profile_key {
        "AWS-Region" => "AwsRegion".to_string(),
        other => other.replace('-', ""),
    }
}

/// DuckDB output sink: a `.duckdb` database file holding a typed `timeline` table plus the
/// `suzaku_meta` provenance table. Unlike the CSV/JSON sinks this is not a byte stream, so it
/// lives outside the `dyn Write` writers.
///
/// Rows arrive as strings (that is what the output profile produces) and are appended to a
/// `TEMP` staging table through DuckDB's `Appender` in batches. A per-row
/// `INSERT INTO ... VALUES` re-parses and re-plans the statement and commits its own transaction
/// every time, which made `-t duckdb` roughly 200x slower per row than the appender path.
///
/// [`Self::finalize`] then rewrites the staging table into the real one in a single statement:
/// that is where placeholders become NULL, text becomes `TIMESTAMP`/`ENUM`/`VARCHAR[]`, exact
/// duplicates are dropped and rows are sorted by time. Doing it in one pass at the end — rather
/// than typing each value in Rust on the way in — keeps the hot append path untouched, lets one
/// unparseable value degrade to a `NULL` instead of failing the run, and is the only way to
/// produce `LIST` columns, which the appender API cannot bind.
struct DuckDbSink {
    conn: Connection,
    /// DuckDB column names, in record order, with `Tags` already expanded to [`TAG_COLUMNS`].
    columns: Vec<String>,
    /// Index of the `Tags` value inside an incoming record, when the profile has one.
    tags_index: Option<usize>,
    /// Where to splice the empty [`GEO_COLUMNS`] cells into a record, when the profile has a
    /// `SrcIP` but no geo columns because `-G` was not given. An index into the final column list,
    /// so it is applied after the `Tags` expansion has already shifted the row.
    geo_fill_at: Option<usize>,
    /// Rows accumulated since the last flush, drained into an `Appender` by [`Self::flush`].
    pending: Vec<Vec<String>>,
    /// Provenance written to `suzaku_meta` by [`Self::finalize`].
    meta: SuzakuMeta,
}

impl DuckDbSink {
    fn new(path: &Path, profile_keys: &[String], meta: SuzakuMeta) -> Result<Self, String> {
        let tags_index = profile_keys.iter().position(|k| k == "Tags");
        let mut columns: Vec<String> = Vec::with_capacity(profile_keys.len() + 2);
        for key in profile_keys {
            if key == "Tags" {
                columns.extend(TAG_COLUMNS.iter().map(|c| c.to_string()));
            } else {
                columns.push(duckdb_column_name(key));
            }
        }

        // Without `-G` the profile carries no geo keys, but the table still gets the columns —
        // all-NULL — so that one query works against every file this command writes. They are
        // enrichment *of* `SrcIP`, so a profile without one has nothing to describe and gets
        // nothing; `suzaku_meta.geoip_enabled` records which case a NULL came from.
        let geo_fill_at = match columns.iter().position(|c| c == "SrcIP") {
            Some(i) if !columns.iter().any(|c| c == GEO_COLUMNS[0]) => {
                let at = i + 1;
                columns.splice(at..at, GEO_COLUMNS.iter().map(|c| c.to_string()));
                Some(at)
            }
            _ => None,
        };

        let conn = Connection::open(path)
            .map_err(|e| format!("Cannot write to output file {}: {e}", path.display()))?;
        let cols_ddl = columns
            .iter()
            .map(|c| format!("{} VARCHAR", quote_ident(c)))
            .collect::<Vec<_>>()
            .join(", ");
        conn.execute_batch(&format!(
            "CREATE OR REPLACE TEMP TABLE {DUCKDB_STAGING_TABLE} ({cols_ddl});"
        ))
        .map_err(|e| format!("Cannot create DuckDB table in {}: {e}", path.display()))?;
        Ok(Self {
            conn,
            columns,
            tags_index,
            geo_fill_at,
            pending: Vec::with_capacity(DUCKDB_BATCH_ROWS),
            meta,
        })
    }

    fn append_row(&mut self, record: &[String]) {
        let mut row = match self.tags_index {
            // Expand the packed tag string in place, keeping every other column where it was.
            Some(i) if i < record.len() => {
                let mut row = Vec::with_capacity(self.columns.len());
                row.extend_from_slice(&record[..i]);
                row.extend(split_tags(&record[i]));
                row.extend_from_slice(&record[i + 1..]);
                row
            }
            _ => record.to_vec(),
        };
        // After the tag expansion, so the recorded column-space index lines up whether `Tags`
        // comes before or after `SrcIP` in the profile.
        if let Some(at) = self.geo_fill_at
            && at <= row.len()
        {
            row.splice(at..at, GEO_COLUMNS.iter().map(|_| String::new()));
        }
        self.pending.push(row);
        if self.pending.len() >= DUCKDB_BATCH_ROWS {
            self.flush();
        }
    }

    /// Write the buffered rows and clear the buffer. The appender is created per batch rather than
    /// held in the struct because `Connection::appender` borrows the connection, which a
    /// self-referential field cannot express; recreating it costs ~1% of the batch write.
    ///
    /// A failure does not abort the scan — the other output formats are already written and the
    /// staged rows are still worth typing — but the batch is lost, so it is logged with the number
    /// of rows at stake instead of leaving the user with a quietly short database. Dropping the
    /// batch rather than holding it for a retry is what keeps memory bounded when a failure
    /// persists.
    fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        if let Err(e) = self.append_pending() {
            log_error(&format!(
                "{e} Up to {} row(s) are missing from the DuckDB output.",
                self.pending.len()
            ));
        }
        self.pending.clear();
    }

    /// Append the buffered rows to the staging table, stopping at the first failure. Rows already
    /// handed to the appender may still land, which is why the caller reports its count as an
    /// upper bound on what was lost.
    fn append_pending(&self) -> Result<(), String> {
        let mut appender = self
            .conn
            .appender_to_catalog_and_db(DUCKDB_STAGING_TABLE, "temp", "main")
            .map_err(|e| format!("Cannot open the DuckDB staging appender: {e}."))?;
        for row in &self.pending {
            let params: Vec<&dyn ToSql> = row.iter().map(|v| v as &dyn ToSql).collect();
            appender
                .append_row(params.as_slice())
                .map_err(|e| format!("Cannot stage a row for the DuckDB timeline table: {e}."))?;
        }
        appender
            .flush()
            .map_err(|e| format!("Cannot stage rows for the DuckDB timeline table: {e}."))
    }

    /// The `SELECT` expression producing one final column from its staging counterpart.
    fn column_expr(&self, column: &str) -> String {
        let quoted = quote_ident(column);
        let expr = match column {
            "Timestamp" => timestamp_expr(&quoted),
            // `lower` because the terminal writer abbreviates severities in place; the file
            // writers never see that, but an unexpected spelling should not become a silent NULL.
            "Level" => format!("TRY_CAST(lower({}) AS {LEVEL_TYPE})", nullable(&quoted)),
            c if TAG_COLUMNS.contains(&c) => list_expr(&quoted),
            _ => nullable(&quoted),
        };
        format!("{expr} AS {quoted}")
    }

    /// Rewrite the staged strings into the typed `timeline` table, write `suzaku_meta`, and leave
    /// the file checkpointed and ready to be opened read-only.
    ///
    /// `SELECT DISTINCT` is the resolution of the duplicate-row problem: byte-identical rows carry
    /// no information, but they inflate every count, Top-N and trend a dashboard derives from the
    /// file (37% of the rows in the reference corpus were exact duplicates, inflating counts ~1.6x).
    /// They come from re-delivered log records rather than from a bug here, so how many were
    /// dropped is reported in `suzaku_meta.duplicate_rows_removed` rather than silently discarded.
    fn finalize(&mut self) -> Result<(), String> {
        self.flush();
        if self.columns.iter().any(|c| c == "Level") {
            duckdb_out::create_level_type(&self.conn)?;
        }
        let exprs = self
            .columns
            .iter()
            .map(|c| self.column_expr(c))
            .collect::<Vec<_>>()
            .join(",\n       ");
        // Sorting by time also compresses better: a typed, time-sorted rewrite of the 1.9 M-row
        // reference corpus measured 15% smaller than the untyped, insertion-ordered table.
        let order_by = if self.columns.iter().any(|c| c == "Timestamp") {
            "\nORDER BY \"Timestamp\""
        } else {
            ""
        };
        self.conn
            .execute_batch(&format!(
                "CREATE OR REPLACE TABLE timeline AS\nSELECT DISTINCT {exprs}\nFROM {DUCKDB_STAGING_TABLE}{order_by};"
            ))
            .map_err(|e| format!("Cannot write the DuckDB timeline table: {e}"))?;

        let staged = duckdb_out::count_rows(&self.conn, DUCKDB_STAGING_TABLE)?;
        let written = duckdb_out::count_rows(&self.conn, "timeline")?;
        self.conn
            .execute_batch(&format!("DROP TABLE {DUCKDB_STAGING_TABLE};"))
            .map_err(|e| format!("Cannot drop the DuckDB staging table: {e}"))?;

        self.meta.output_rows = Some(written);
        self.meta.duplicate_rows_removed = Some(staged - written);
        duckdb_out::write_meta(&self.conn, &self.meta)?;
        duckdb_out::comment_on_table(&self.conn, "timeline", TIMELINE_GRAIN)?;
        for (column, comment) in [
            (
                "Timestamp",
                "Event time. The timezone is stated in suzaku_meta.timestamp_tz.",
            ),
            (
                "Level",
                "Sigma rule severity. Ordered, so ORDER BY / max() rank by severity; compare \
                 against a literal with an explicit cast, e.g. Level >= 'high'::suzaku_level.",
            ),
            ("Tactics", "ATT&CK tactic abbreviations from the rule tags."),
            ("TechniqueIDs", "ATT&CK technique IDs from the rule tags."),
            (
                "OtherTags",
                "Rule tags that are neither a tactic nor a technique (e.g. ATT&CK groups, CVEs).",
            ),
            (GEO_COLUMNS[0], GEO_COLUMN_COMMENT),
            (GEO_COLUMNS[1], GEO_COLUMN_COMMENT),
            (GEO_COLUMNS[2], GEO_COLUMN_COMMENT),
        ] {
            if self.columns.iter().any(|c| c == column) {
                duckdb_out::comment_on_column(&self.conn, "timeline", column, comment)?;
            }
        }
        duckdb_out::checkpoint(&self.conn)
    }
}

impl Drop for DuckDbSink {
    fn drop(&mut self) {
        // Safety net so buffered rows are never lost if the sink is dropped without an explicit
        // flush (e.g. an early return on error).
        self.flush();
    }
}

pub struct OutputContext<'a> {
    pub profile: &'a [(String, String)],
    pub prof_ts_key: &'a str,
    pub geo: &'a mut Option<GeoIPSearch>,
    pub config: &'a OutputConfig,
    pub writers: Writers,
    pub has_written: bool,
    pub output_paths: Vec<PathBuf>,
}

pub fn write_record(event: &Event, json: &Value, rule: Option<&Rule>, context: &mut OutputContext) {
    let localtime = context.config.localtime;
    let src_ip = src_ip_spec(context.profile).to_string();
    let mut record: Vec<String> = context
        .profile
        .iter()
        .map(|(_k, v)| get_value_from_event(v, event, rule, context.geo, localtime, &src_ip))
        .collect();
    write_to_stdout(&mut record, context, json, Some(event), rule);
    write_to_csv(&record, context);
    write_to_duckdb(&record, context);
    write_to_json(&record, json, Some(event), rule, context);
    write_to_jsonl(&record, json, Some(event), rule, context);
    context.has_written = true;
}

pub fn write_correlation_record(
    events: &Vec<&TimestampedEvent>,
    rule: &SigmaCorrelationRule,
    context: &mut OutputContext,
) {
    let mut record: Vec<String> = build_correlation_record(events, rule, context);
    write_to_stdout(&mut record, context, &Value::Null, None, None);
    write_to_csv(&record, context);
    write_to_duckdb(&record, context);
    write_to_json(&record, &Value::Null, None, None, context);
    write_to_jsonl(&record, &Value::Null, None, None, context);
}

/// Adds the profile-derived fields that raw output promises alongside the original event JSON.
///
/// Sigma fields are declared directly by profile values beginning with sigma. GeoIP fields are
/// injected by load_profile only when SrcIP is present and GeoIP is enabled, where their value is
/// the same as their column name. Keeping the selection here shared prevents JSON, JSONL, and
/// terminal raw output from drifting apart.
fn enrich_raw_record(
    json_record: &mut Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
    profile: &[(String, String)],
    geo: &mut Option<GeoIPSearch>,
    localtime: bool,
) {
    let Some(event) = event else {
        return;
    };

    for (key, value) in profile.iter().filter(|(key, value)| {
        value.starts_with("sigma.") || (GEO_COLUMNS.contains(&key.as_str()) && value == key)
    }) {
        let value = get_value_from_event(value, event, rule, geo, localtime, src_ip_spec(profile));
        json_record[key] = Value::String(value);
    }
}

fn write_to_stdout(
    record: &mut [String],
    context: &mut OutputContext,
    json: &Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
) {
    if let Some(writer) = &mut context.writers.std {
        let level_index = context.profile.iter().position(|(k, _)| k == "Level");
        let level = if let Some(index) = level_index {
            let org = record[index].to_lowercase();
            let abb = abbreviate_level(&org);
            record[index] = abb.to_string();
            abb.to_string()
        } else {
            "info".to_string()
        };

        let color = get_level_color(&level);
        let mut buf = writer.buffer();

        if context.config.raw_output {
            buf.set_color(ColorSpec::new().set_fg(color.rdg(context.config.no_color)))
                .ok();
            let profile = context.profile;
            let localtime = context.config.localtime;
            let geo = &mut context.geo;
            let mut json_record = json.clone();
            enrich_raw_record(&mut json_record, event, rule, profile, geo, localtime);

            let json_string = serde_json::to_string_pretty(&json_record);
            if let Ok(json_string) = json_string {
                write!(buf, "{}\n\n", json_string).ok();
                writer.print(&buf).ok();
            }
        } else {
            for (i, col) in record.iter().enumerate() {
                buf.set_color(ColorSpec::new().set_fg(color.rdg(context.config.no_color)))
                    .ok();
                write!(buf, "{col}").ok();
                if i != record.len() - 1 {
                    if context.config.no_color {
                        buf.set_color(ColorSpec::new().set_fg(None)).ok();
                    } else {
                        buf.set_color(ColorSpec::new().set_fg(Orange.rdg(context.config.no_color)))
                            .ok();
                    }
                    write!(buf, " · ").ok();
                }
            }
            write!(buf, "\n\n").ok();
            writer.print(&buf).ok();
        }
    }
}

fn write_to_csv(record: &[String], context: &mut OutputContext) {
    if let Some(writer) = &mut context.writers.csv {
        let sanitized: Vec<String> = record.iter().map(|f| sanitize_csv_field(f)).collect();
        writer.write_record(&sanitized).unwrap();
    }
}

fn write_to_duckdb(record: &[String], context: &mut OutputContext) {
    if let Some(sink) = &mut context.writers.duckdb {
        sink.append_row(record);
    }
}

fn write_to_json_format(
    record: &[String],
    json: &Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
    context: &mut OutputContext,
    pretty: bool,
) {
    let raw_output = context.config.raw_output;

    if raw_output {
        let profile = context.profile;
        let localtime = context.config.localtime;
        let geo = &mut context.geo;

        let writer = if pretty {
            &mut context.writers.json
        } else {
            &mut context.writers.jsonl
        };

        if let Some(writer) = writer {
            let mut json_record = json.clone();
            enrich_raw_record(&mut json_record, event, rule, profile, geo, localtime);

            let json_string = if pretty {
                serde_json::to_string_pretty(&json_record)
            } else {
                serde_json::to_string(&json_record)
            };

            if let Ok(json_string) = json_string {
                writer.write_all(json_string.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            }
        }
    } else {
        let writer = if pretty {
            &mut context.writers.json
        } else {
            &mut context.writers.jsonl
        };

        if let Some(writer) = writer {
            let mut json_record: BTreeMap<String, String> = BTreeMap::new();
            for ((k, _), value) in context.profile.iter().zip(record.iter()) {
                json_record.insert(k.clone(), value.clone());
            }

            let json_string = if pretty {
                serde_json::to_string_pretty(&json_record)
            } else {
                serde_json::to_string(&json_record)
            };

            if let Ok(json_string) = json_string {
                writer.write_all(json_string.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            }
        }
    }
}

fn write_to_json(
    record: &[String],
    json: &Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
    context: &mut OutputContext,
) {
    write_to_json_format(record, json, event, rule, context, true);
}

fn write_to_jsonl(
    record: &[String],
    json: &Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
    context: &mut OutputContext,
) {
    write_to_json_format(record, json, event, rule, context, false);
}

fn get_level_color(level: &str) -> SuzakuColor {
    match level {
        "crit" => Red,
        "high" => Orange,
        "med" => Yellow,
        "low" => Green,
        _ => White,
    }
}

fn abbreviate_level(level: &str) -> &str {
    match level {
        "critical" => "crit",
        "medium" => "med",
        "informational" => "info",
        _ => level,
    }
}

/// Path (relative to the working directory, like the output profiles) of the ATT&CK tactic
/// abbreviation table. This is the same file Hayabusa ships, minus its `html_tag_output_str`
/// column: each line is `<full tag>,<abbreviation>` (e.g. `attack.credential-access,CredAccess`).
const MITRE_TACTICS_PATH: &str = "config/mitre_tactics.txt";

/// Parses the `config/mitre_tactics.txt` table into a `full-tag -> abbreviation` map. Keys are
/// lowercased with `_` folded to `-` so lookups are case- and separator-insensitive. The header
/// row and any non-`attack.` line are skipped. A missing/unreadable file yields an empty map, in
/// which case tactic tags simply pass through un-abbreviated (techniques/groups are unaffected).
fn load_mitre_tactics(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(contents) = std::fs::read_to_string(path) {
        for line in contents.lines() {
            let mut fields = line.split(',');
            let (Some(full), Some(abbrev)) = (fields.next(), fields.next()) else {
                continue;
            };
            let key = full.trim().to_lowercase().replace('_', "-");
            if !key.starts_with("attack.") {
                continue; // header row / comments / blanks
            }
            map.insert(key, abbrev.trim().to_string());
        }
    }
    map
}

/// Process-wide cache of the ATT&CK tactic table, loaded once on first use.
fn mitre_tactics() -> &'static HashMap<String, String> {
    static MAP: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    MAP.get_or_init(|| load_mitre_tactics(MITRE_TACTICS_PATH))
}

/// Abbreviates a single Sigma `tags` entry following the conventions requested in
/// <https://github.com/Yamato-Security/suzaku/issues/62> (matching Hayabusa's tag output):
/// ATT&CK tactics are looked up in `config/mitre_tactics.txt`, techniques (`attack.t1562.001`)
/// become `T1562.001`, and groups (`attack.g0035`) become `G0035`. Separators are normalized so
/// both the hyphen (`attack.credential-access`) and underscore (`attack.credential_access`)
/// spellings map to the same abbreviation. Unrecognized tags (e.g. `cve.*`) are returned unchanged.
fn abbreviate_tag(tag: &str) -> String {
    let lower = tag.to_lowercase();
    // Tactics: look up in the config-driven table, folding `_` to `-` to match its keys.
    if let Some(abbrev) = mitre_tactics().get(&lower.replace('_', "-")) {
        return abbrev.clone();
    }
    // Techniques: attack.t1562.001 -> T1562.001
    if let Some(rest) = lower.strip_prefix("attack.t") {
        return format!("T{}", rest.to_uppercase());
    }
    // Groups: attack.g0035 -> G0035
    if let Some(rest) = lower.strip_prefix("attack.g") {
        return format!("G{}", rest.to_uppercase());
    }
    // Unknown namespace: leave the tag untouched.
    tag.to_string()
}

/// Joins a rule's `tags` list into a single ` ¦ `-separated string of abbreviations
/// (like Hayabusa), so the list can be rendered in one flat CSV/JSON column.
fn format_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|tag| abbreviate_tag(tag))
        .collect::<Vec<_>>()
        .join(" ¦ ")
}

fn build_correlation_record(
    events: &Vec<&TimestampedEvent>,
    rule: &SigmaCorrelationRule,
    context: &mut OutputContext,
) -> Vec<String> {
    let events: Vec<Event> = events.iter().map(|e| e.event.clone()).collect();
    let profile = &context.profile;
    let localtime = context.config.localtime;
    let mut correlation_map: HashMap<String, String> = HashMap::new();
    for (_, profile_value) in profile.iter() {
        let mut values = HashSet::new();
        for (i, event) in events.iter().enumerate() {
            if profile_value == ".eventTime" && i < events.len() - 1 {
                continue;
            }
            let value = get_value_from_correlation_event(
                profile_value,
                event,
                rule,
                context.geo,
                localtime,
                src_ip_spec(profile),
            );
            values.insert(value);
        }
        let values: Vec<String> = values.into_iter().sorted().collect();
        let concatenated = values.join(" ¦ ");
        correlation_map.insert(profile_value.clone(), concatenated);
    }
    profile
        .iter()
        .map(|(_, profile_value)| {
            correlation_map
                .get(profile_value)
                .cloned()
                .unwrap_or_else(|| "-".to_string())
        })
        .collect()
}

/// The `SrcIP` field spec declared by the active output profile — e.g.
/// `.sourceIPAddress` for AWS, or `.claims.ipaddr|.callerIpAddress|.ClientIP|.ActorIpAddress`
/// for Azure/M365. Empty when the profile has no `SrcIP` column. Used to resolve
/// the source IP for GeoIP enrichment without hardcoding an AWS-only field name.
fn src_ip_spec(profile: &[(String, String)]) -> &str {
    profile
        .iter()
        .find(|(k, _)| k == "SrcIP")
        .map(|(_, v)| v.as_str())
        .unwrap_or("")
}

/// Picks the source IP out of a `|`-separated `SrcIP` field spec.
///
/// Returns the chosen field's raw value together with its parsed form. The first candidate that
/// PARSES as an IP wins; if none does, the first candidate merely present is returned with no
/// parsed form, so the `SrcIP` column still shows what the log recorded (an AWS-service event
/// writes `cloudtrail.amazonaws.com` there, and that is worth displaying even though it cannot be
/// geolocated).
///
/// One selector for both the `SrcIP` column and the three geo columns, so they always describe
/// the same field. Choosing per-column would let `SrcIP` display one address while
/// `SrcCountry` described another.
fn select_source_ip(spec: &str, event: &Event) -> Option<(String, Option<IpAddr>)> {
    let mut first_present: Option<String> = None;
    for key in spec
        .split('|')
        .map(|k| k.trim_matches('.').trim())
        .filter(|k| !k.is_empty())
    {
        let Some(value) = event.get(key) else {
            continue;
        };
        let raw = value.value_to_string();
        if let Some(ip) = parse_ip(&raw) {
            return Some((raw, Some(ip)));
        }
        if first_present.is_none() {
            first_present = Some(raw);
        }
    }
    first_present.map(|raw| (raw, None))
}

fn get_value_from_event_common(
    key: &str,
    event: &Event,
    rule_info: RuleInfo,
    geo_ip: &mut Option<GeoIPSearch>,
    localtime: bool,
    src_ip: &str,
) -> String {
    // GeoIP処理部分（共通）: only the three geo columns are enriched. The source IP
    // is resolved from the profile's SrcIP field spec (`.sourceIPAddress` for AWS,
    // `.claims.ipaddr|.callerIpAddress|.ClientIP|...` for Azure/M365) — NOT a hardcoded
    // field name — so Azure/M365 enrich just like AWS. A missing GeoIP DB, no usable
    // source IP, or a non-IP value (e.g. a service principal like
    // "cloudtrail.amazonaws.com") yields the "-" placeholder for those columns only —
    // it must never overwrite an unrelated column's value.
    //
    // The spec is a fallback list, so the search takes the first candidate that PARSES
    // as an IP, not merely the first that exists. Stopping at the first present key
    // meant an earlier candidate holding "", "-", a host:port pair or any other non-IP
    // ended the lookup, and a valid address in a later field was never consulted — the
    // geo columns then rendered "-" for a record that plainly carried a routable IP
    // (issue #183). Azure records commonly have `claims.ipaddr` absent-or-empty while
    // `callerIpAddress` is populated, so this was reachable on ordinary input.
    //
    // The `SrcIP` column resolves through the same selector, so the address displayed
    // and the address geolocated are always the same field.
    if matches!(key, "SrcASN" | "SrcCity" | "SrcCountry") {
        if let Some(geo) = geo_ip {
            let resolved = select_source_ip(src_ip, event).and_then(|(_, ip)| ip);
            if let Some(ip) = resolved {
                return match key {
                    "SrcASN" => geo.get_asn(ip),
                    "SrcCity" => geo.get_city(ip),
                    _ => geo.get_country(ip),
                };
            }
        }
        return "-".to_string();
    }
    // The SrcIP column resolves through the same selector as the geo columns above, so the
    // address displayed is always the one that was (or would have been) geolocated. Without this
    // the generic resolver below would show the first field merely PRESENT, letting SrcIP report
    // one address while SrcCountry described another (#183).
    if !src_ip.is_empty() && key == src_ip {
        return select_source_ip(src_ip, event)
            .map(|(raw, _)| raw)
            .unwrap_or_else(|| "-".to_string());
    }
    // イベントフィールド処理（共通）
    if key.starts_with(".") {
        match resolve_field_spec(key, event) {
            Some((matched, value)) => {
                if matched.contains("eventTime")
                    || matched.contains("time")
                    || matched.contains("eventTimestamp")
                    || matched.contains("CreationTime")
                {
                    format_timestamp(&value, localtime)
                } else {
                    value
                }
            }
            None => "-".to_string(),
        }
    } else if key.starts_with("sigma.") {
        let key = key.replace("sigma.", "");
        match key.as_str() {
            "title" => rule_info.title(),
            "id" => rule_info.id().unwrap_or_else(|| "-".to_string()),
            "status" => rule_info.status().unwrap_or_else(|| "-".to_string()),
            "author" => rule_info.author().unwrap_or_else(|| "-".to_string()),
            "description" => rule_info.description().unwrap_or_else(|| "-".to_string()),
            "references" => rule_info.references().unwrap_or_else(|| "-".to_string()),
            "date" => rule_info.date().unwrap_or_else(|| "-".to_string()),
            "modified" => rule_info.modified().unwrap_or_else(|| "-".to_string()),
            "tags" => rule_info.tags().unwrap_or_else(|| "-".to_string()),
            "falsepositives" => rule_info
                .falsepositives()
                .unwrap_or_else(|| "-".to_string()),
            "level" => rule_info.level().unwrap_or_else(|| "-".to_string()),
            _ => "-".to_string(),
        }
    } else {
        "-".to_string()
    }
}

enum RuleInfo<'a> {
    Rule(&'a Rule),
    CorrelationRule(&'a SigmaCorrelationRule),
}
impl<'a> RuleInfo<'a> {
    fn title(&self) -> String {
        match self {
            RuleInfo::Rule(rule) => rule.title.to_string(),
            RuleInfo::CorrelationRule(rule) => rule.title.to_string(),
        }
    }

    fn id(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.id.as_ref().map(|id| id.to_string()),
            RuleInfo::CorrelationRule(rule) => rule.id.as_ref().map(|id| id.to_string()),
        }
    }

    fn status(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.status.as_ref().map(|status| format!("{status:?}")),
            RuleInfo::CorrelationRule(rule) => {
                rule.status.as_ref().map(|status| status.to_string())
            }
        }
    }

    fn author(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.author.as_ref().map(|author| author.to_string()),
            RuleInfo::CorrelationRule(rule) => {
                rule.author.as_ref().map(|author| author.to_string())
            }
        }
    }

    fn description(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.description.as_ref().map(|desc| desc.to_string()),
            RuleInfo::CorrelationRule(rule) => {
                rule.description.as_ref().map(|desc| desc.to_string())
            }
        }
    }

    fn references(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.references.as_ref().map(|refs| refs.join(", ")),
            RuleInfo::CorrelationRule(rule) => rule.references.as_ref().map(|refs| refs.join(", ")),
        }
    }

    fn date(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.date.as_ref().map(|date| date.to_string()),
            RuleInfo::CorrelationRule(rule) => rule.date.as_ref().map(|date| date.to_string()),
        }
    }

    fn modified(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.modified.as_ref().map(|date| date.to_string()),
            RuleInfo::CorrelationRule(_) => None,
        }
    }

    fn tags(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.tags.as_ref().map(|tags| format_tags(tags)),
            RuleInfo::CorrelationRule(rule) => rule.tags.as_ref().map(|tags| format_tags(tags)),
        }
    }

    fn falsepositives(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule.falsepositives.as_ref().map(|fp| fp.join(", ")),
            RuleInfo::CorrelationRule(rule) => rule.falsepositives.as_ref().map(|fp| fp.join(", ")),
        }
    }

    fn level(&self) -> Option<String> {
        match self {
            RuleInfo::Rule(rule) => rule
                .level
                .as_ref()
                .map(|level| format!("{level:?}").to_lowercase()),
            RuleInfo::CorrelationRule(rule) => rule.level.as_ref().map(|level| level.to_string()),
        }
    }
}

fn get_value_from_correlation_event(
    key: &str,
    event: &Event,
    rule: &SigmaCorrelationRule,
    geo_ip: &mut Option<GeoIPSearch>,
    localtime: bool,
    src_ip: &str,
) -> String {
    get_value_from_event_common(
        key,
        event,
        RuleInfo::CorrelationRule(rule),
        geo_ip,
        localtime,
        src_ip,
    )
}

fn get_value_from_event(
    key: &str,
    event: &Event,
    rule: Option<&Rule>,
    geo_ip: &mut Option<GeoIPSearch>,
    localtime: bool,
    src_ip: &str,
) -> String {
    if let Some(rule) = rule {
        get_value_from_event_common(key, event, RuleInfo::Rule(rule), geo_ip, localtime, src_ip)
    } else {
        "".to_string()
    }
}

// 使用例
impl OutputConfig {
    pub fn new(no_color: bool, raw_output: bool, localtime: bool) -> Self {
        Self {
            no_color,
            raw_output,
            localtime,
        }
    }
}

impl Writers {
    pub fn new() -> Self {
        Self {
            csv: None,
            json: None,
            jsonl: None,
            duckdb: None,
            std: None,
        }
    }

    pub fn with_csv(mut self, writer: Writer<Box<dyn Write>>) -> Self {
        self.csv = Some(writer);
        self
    }

    fn with_duckdb(mut self, sink: DuckDbSink) -> Self {
        self.duckdb = Some(sink);
        self
    }

    pub fn with_json(mut self, writer: BufWriter<Box<dyn Write>>) -> Self {
        self.json = Some(writer);
        self
    }

    pub fn with_jsonl(mut self, writer: BufWriter<Box<dyn Write>>) -> Self {
        self.jsonl = Some(writer);
        self
    }

    pub fn with_stdout(mut self, writer: BufferWriter) -> Self {
        self.std = Some(writer);
        self
    }
}

impl<'a> OutputContext<'a> {
    pub fn new(
        profile: &'a [(String, String)],
        geo: &'a mut Option<GeoIPSearch>,
        config: &'a OutputConfig,
        writers: Writers,
        output_paths: &[PathBuf],
    ) -> Self {
        let prof_ts_key = profile
            .iter()
            .find(|(k, _)| k == "Timestamp")
            .map(|(_k, v)| v.as_str())
            .unwrap_or(".eventTime|.time|.eventTimestamp");
        Self {
            profile,
            prof_ts_key,
            geo,
            config,
            writers,
            has_written: false,
            output_paths: output_paths.to_vec(),
        }
    }

    /// Record how much input the run covered, for `suzaku_meta`. Call before [`Self::flush_all`];
    /// a run whose scan stats are unknown simply leaves the columns NULL.
    pub fn set_scan_stats(&mut self, scanned_files: Option<i64>, scanned_events: Option<i64>) {
        if let Some(ref mut sink) = self.writers.duckdb {
            sink.meta.scanned_files = scanned_files;
            sink.meta.scanned_events = scanned_events;
        }
    }

    pub fn flush_all(&mut self) {
        if let Some(ref mut writer) = self.writers.csv {
            writer.flush().unwrap();
        }
        if let Some(ref mut writer) = self.writers.json {
            writer.flush().unwrap();
        }
        if let Some(ref mut writer) = self.writers.jsonl {
            writer.flush().unwrap();
        }
        if let Some(ref mut sink) = self.writers.duckdb {
            if self.has_written {
                // Turn the staged strings into the typed, deduplicated table. A failure here
                // costs the DuckDB output only, so it is logged rather than ending the run that
                // just spent minutes scanning — the other formats are already written.
                if let Err(e) = sink.finalize() {
                    log_error(&e);
                }
            } else {
                sink.flush();
            }
        }
        if !self.has_written {
            self.writers.csv = None;
            self.writers.json = None;
            self.writers.jsonl = None;
            // Drop the DuckDB sink too: its open connection holds a lock on the `.duckdb` file on
            // Windows, so `remove_file` below would fail silently and leave an empty database behind.
            self.writers.duckdb = None;

            for path in &self.output_paths {
                if path.exists() {
                    std::fs::remove_file(path).ok();
                }
            }
        }
    }

    pub fn write_header(&mut self) {
        let csv_header: Vec<&str> = self.profile.iter().map(|(k, _v)| k.as_str()).collect();
        if let Some(ref mut std_out) = self.writers.std {
            let mut buf = std_out.buffer();
            writeln!(buf, "{}", csv_header.join(" · ")).ok();
        }

        if let Some(ref mut writer) = self.writers.csv {
            writer.write_record(&csv_header).unwrap();
        }
    }
}

/// File extension written for each output format.
fn output_format_ext(fmt: OutputFormat) -> &'static str {
    match fmt {
        OutputFormat::Csv => "csv",
        OutputFormat::Json => "json",
        OutputFormat::Jsonl => "jsonl",
        OutputFormat::Duckdb => "duckdb",
    }
}

/// Resolve the concrete `(format, path)` targets for a base `-o` path: each requested format maps
/// to `<base>.<ext>` (the base's extension normalized per format), with duplicate formats removed.
/// Single source of truth for opening the writers, the `--clobber` preflight, and the summary
/// command, which writes its own files but must resolve `-o`/`-t` identically.
pub fn resolve_output_targets(
    output_path: &Path,
    output_types: &[OutputFormat],
) -> Vec<(OutputFormat, PathBuf)> {
    let mut seen = HashSet::new();
    output_types
        .iter()
        .copied()
        .filter(|f| seen.insert(*f))
        .map(|fmt| {
            let ext = output_format_ext(fmt);
            let mut path = output_path.to_path_buf();
            if path.extension().and_then(|e| e.to_str()) != Some(ext) {
                path.set_extension(ext);
            }
            (fmt, path)
        })
        .collect()
}

/// The concrete output file paths a run would write for `output_types` under a base `-o` path,
/// e.g. `<base>.csv` / `<base>.duckdb`. Used to preflight `--clobber` against every file that
/// would actually be created, not just the literal `-o` value.
pub fn resolve_output_paths(output_path: &Path, output_types: &[OutputFormat]) -> Vec<PathBuf> {
    resolve_output_targets(output_path, output_types)
        .into_iter()
        .map(|(_, path)| path)
        .collect()
}

/// Build the output writers for the requested formats. Each format writes to `<output>.<ext>`
/// (the base path's extension is normalized per format), so a single `-o` base path can fan out
/// to several files at once. `profile` supplies the column names for the DuckDB table, and `meta`
/// the provenance its `suzaku_meta` table records. With no `output_path`, results go to the
/// stdout table and `output_types` is ignored.
pub fn init_writers(
    output_path: Option<&PathBuf>,
    output_types: &[OutputFormat],
    profile: &[(String, String)],
    meta: SuzakuMeta,
) -> Result<(Writers, Vec<PathBuf>), String> {
    let mut output_pathes = vec![];
    let mut writers = Writers::new();

    if let Some(output_path) = output_path {
        for (fmt, path) in resolve_output_targets(output_path, output_types) {
            output_pathes.push(path.clone());
            writers = match fmt {
                OutputFormat::Csv => writers.with_csv(get_writer(&Some(path))?),
                OutputFormat::Json => writers.with_json(get_json_writer(&Some(path))?),
                OutputFormat::Jsonl => writers.with_jsonl(get_json_writer(&Some(path))?),
                OutputFormat::Duckdb => {
                    let columns: Vec<String> = profile.iter().map(|(k, _)| k.clone()).collect();
                    writers.with_duckdb(DuckDbSink::new(&path, &columns, meta.clone())?)
                }
            };
        }
    } else {
        let disp_wtr = BufferWriter::stdout(ColorChoice::Always);
        let mut disp_wtr_buf = disp_wtr.buffer();
        disp_wtr_buf.set_color(ColorSpec::new().set_fg(None)).ok();
        writers = writers.with_stdout(disp_wtr);
    }

    Ok((writers, output_pathes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_utc_default_strips_t_and_z() {
        assert_eq!(
            format_timestamp("2023-07-10T12:27:45Z", false),
            "2023-07-10 12:27:45"
        );
    }

    #[test]
    fn format_timestamp_localtime_preserves_rfc3339_instant() {
        // The localtime rendering carries an explicit offset, so re-parsing it must
        // recover the same UTC instant regardless of the machine's local timezone.
        let out = format_timestamp("2023-07-10T12:27:45Z", true);
        let parsed = DateTime::parse_from_str(&out, "%Y-%m-%d %H:%M:%S%:z")
            .expect("localtime output should be parseable with an offset");
        assert_eq!(
            parsed.with_timezone(&Utc),
            Utc.with_ymd_and_hms(2023, 7, 10, 12, 27, 45).unwrap()
        );
    }

    #[test]
    fn format_timestamp_localtime_assumes_naive_is_utc() {
        // A naive timestamp (no offset, e.g. M365 CreationTime) is treated as UTC.
        let out = format_timestamp("2023-07-10T12:27:45", true);
        let parsed = DateTime::parse_from_str(&out, "%Y-%m-%d %H:%M:%S%:z")
            .expect("localtime output should be parseable with an offset");
        assert_eq!(
            parsed.with_timezone(&Utc),
            Utc.with_ymd_and_hms(2023, 7, 10, 12, 27, 45).unwrap()
        );
    }

    #[test]
    fn format_timestamp_localtime_falls_back_on_unparseable() {
        // Non-timestamp values must not be dropped; fall back to the UTC rendering.
        assert_eq!(format_timestamp("not-a-timestamp", true), "not-a-timestamp");
    }

    // `Event::get` takes a bare field name and reads `.` as a nesting separator, so every
    // profile spec -- all of which are dot-prefixed, and half of which are `|`-separated
    // fallback lists -- resolves to `None` when handed to it verbatim. That is not a
    // hypothetical: `append_summary_data`, `process_correlation_base_rule` and the correlation
    // date histogram all did exactly that, which left the summary's date breakdown empty and
    // stopped every correlation rule from ever firing. Verified by mutation: replacing the
    // body of `resolve_field_spec` with `event.get(spec)` fails both tests below.
    #[test]
    fn resolve_field_spec_reads_dot_prefixed_and_fallback_specs() {
        use sigma_rust::event_from_json;

        let aws = event_from_json(r#"{"eventTime": "2023-07-10T12:27:45Z"}"#).unwrap();
        assert_eq!(
            resolve_field_spec(".eventTime", &aws),
            Some(("eventTime", "2023-07-10T12:27:45Z".to_string())),
            "the AWS profile's own Timestamp spec must resolve"
        );

        // Azure/M365 name the field three different ways; the first one PRESENT wins, which is
        // how the Timestamp column itself is rendered.
        let azure = event_from_json(r#"{"CreationTime": "2023-07-10T12:27:45"}"#).unwrap();
        assert_eq!(
            resolve_field_spec(".time|.eventTimestamp|.CreationTime", &azure),
            Some(("CreationTime", "2023-07-10T12:27:45".to_string()))
        );
        let graph = event_from_json(r#"{"time": "a", "CreationTime": "b"}"#).unwrap();
        assert_eq!(
            resolve_field_spec(".time|.eventTimestamp|.CreationTime", &graph),
            Some(("time", "a".to_string())),
            "an earlier candidate that is present must win"
        );

        // Nested keys keep their inner dots; only the spec's own delimiters are stripped.
        let nested = event_from_json(r#"{"userIdentity": {"userName": "root"}}"#).unwrap();
        assert_eq!(
            resolve_field_spec(".userIdentity.userName", &nested),
            Some(("userIdentity.userName", "root".to_string()))
        );

        assert_eq!(resolve_field_spec(".missing", &aws), None);
        assert_eq!(resolve_field_spec("", &aws), None);
    }

    #[test]
    fn event_timestamp_resolves_profile_specs_to_utc_instants() {
        use sigma_rust::event_from_json;

        let expected = Utc.with_ymd_and_hms(2023, 7, 10, 12, 27, 45).unwrap();

        let aws = event_from_json(r#"{"eventTime": "2023-07-10T12:27:45Z"}"#).unwrap();
        assert_eq!(event_timestamp(".eventTime", &aws), Some(expected));

        // An explicit offset is an instant, not a wall clock.
        let offset = event_from_json(r#"{"time": "2023-07-10T21:27:45+09:00"}"#).unwrap();
        assert_eq!(
            event_timestamp(".time|.eventTimestamp|.CreationTime", &offset),
            Some(expected)
        );

        // Naive values (M365 CreationTime) are UTC, matching how they are displayed.
        let naive = event_from_json(r#"{"CreationTime": "2023-07-10T12:27:45"}"#).unwrap();
        assert_eq!(
            event_timestamp(".time|.eventTimestamp|.CreationTime", &naive),
            Some(expected)
        );

        // Garbage and absence are both `None`, never a panic and never a bogus instant.
        let garbage = event_from_json(r#"{"eventTime": "yesterday"}"#).unwrap();
        assert_eq!(event_timestamp(".eventTime", &garbage), None);
        assert_eq!(event_timestamp(".eventTime", &naive), None);
    }

    #[test]
    fn abbreviate_tag_maps_all_tactics() {
        // Mappings come from config/mitre_tactics.txt (the same table Hayabusa ships), so this
        // exercises the file loader end to end. Note defense-evasion maps to `Stealth`, matching
        // Hayabusa (not the `Evas` originally listed in issue #62).
        let cases = [
            ("attack.reconnaissance", "Recon"),
            ("attack.resource-development", "ResDev"),
            ("attack.initial-access", "InitAccess"),
            ("attack.execution", "Exec"),
            ("attack.persistence", "Persis"),
            ("attack.privilege-escalation", "PrivEsc"),
            ("attack.stealth", "Stealth"),
            ("attack.defense-evasion", "Stealth"),
            ("attack.defense-impairment", "DefImpair"),
            ("attack.credential-access", "CredAccess"),
            ("attack.discovery", "Disc"),
            ("attack.lateral-movement", "LatMov"),
            ("attack.collection", "Collect"),
            ("attack.command-and-control", "C2"),
            ("attack.exfiltration", "Exfil"),
            ("attack.impact", "Impact"),
        ];
        for (input, expected) in cases {
            assert_eq!(abbreviate_tag(input), expected, "tactic {input}");
        }
    }

    #[test]
    fn load_mitre_tactics_parses_and_normalizes() {
        use std::io::Write;
        // Include the header row and a stray 3-column (Hayabusa-style) line to prove the loader
        // skips the header and tolerates/ignores extra columns.
        let dir = std::env::temp_dir();
        let path = dir.join("suzaku_test_mitre_tactics.txt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "tag_full_str,tag_output_str").unwrap();
            writeln!(f, "attack.credential-access,CredAccess").unwrap();
            writeln!(f, "attack.command-and-control,C2,13. C2").unwrap();
            writeln!(f).unwrap();
        }
        let map = load_mitre_tactics(path.to_str().unwrap());
        assert_eq!(
            map.get("attack.credential-access").map(String::as_str),
            Some("CredAccess")
        );
        // Third column is ignored.
        assert_eq!(
            map.get("attack.command-and-control").map(String::as_str),
            Some("C2")
        );
        // Header row is not inserted.
        assert!(!map.contains_key("tag_full_str"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_mitre_tactics_missing_file_is_empty() {
        // A missing table degrades gracefully to an empty map (tactics pass through).
        assert!(load_mitre_tactics("config/does_not_exist_mitre_tactics.txt").is_empty());
    }

    #[test]
    fn abbreviate_tag_normalizes_hyphen_and_underscore() {
        // Both spellings appear in the real rule corpus and must collapse to one abbreviation.
        assert_eq!(abbreviate_tag("attack.credential_access"), "CredAccess");
        assert_eq!(abbreviate_tag("attack.credential-access"), "CredAccess");
        assert_eq!(abbreviate_tag("attack.initial_access"), "InitAccess");
        assert_eq!(abbreviate_tag("attack.command_and_control"), "C2");
    }

    #[test]
    fn abbreviate_tag_handles_techniques_and_groups() {
        assert_eq!(abbreviate_tag("attack.t1562.001"), "T1562.001");
        assert_eq!(abbreviate_tag("attack.t1110"), "T1110");
        assert_eq!(abbreviate_tag("attack.g0035"), "G0035");
        // Mixed-case input is normalized before matching.
        assert_eq!(abbreviate_tag("attack.T1087"), "T1087");
    }

    #[test]
    fn abbreviate_tag_leaves_unknown_namespaces_unchanged() {
        assert_eq!(abbreviate_tag("cve.2021.1234"), "cve.2021.1234");
        assert_eq!(abbreviate_tag("car.2013-05-004"), "car.2013-05-004");
    }

    #[test]
    fn format_tags_matches_issue_example() {
        // Verbatim example from issue #62.
        let tags = vec![
            "attack.g0035".to_string(),
            "attack.credential_access".to_string(),
            "attack.discovery".to_string(),
            "attack.t1110".to_string(),
            "attack.t1087".to_string(),
        ];
        assert_eq!(
            format_tags(&tags),
            "G0035 ¦ CredAccess ¦ Disc ¦ T1110 ¦ T1087"
        );
    }

    // Regression for #145: with --geo-ip enabled, a `sourceIPAddress` that is not a parseable IP
    // (routine for AWS-service events like `cloudtrail.amazonaws.com`) must NOT overwrite every
    // column with the raw string. Only the three GeoIP columns are affected (they show `-` when
    // the address can't be enriched); all other columns fall through to normal field processing.
    #[test]
    fn geoip_non_ip_source_only_affects_geo_columns() {
        use crate::option::geoip::GeoIPSearch;
        use sigma_rust::{event_from_json, rule_from_yaml};
        use std::path::Path;

        // Small GeoLite2 test databases shipped under test_files/mmdb/.
        let geo = GeoIPSearch::new(Path::new("test_files/mmdb"))
            .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/");
        let mut geo_ip = Some(geo);

        let event = event_from_json(
            r#"{"sourceIPAddress": "cloudtrail.amazonaws.com", "eventName": "ListBuckets"}"#,
        )
        .unwrap();
        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: ListBuckets\n    condition: selection\n",
        )
        .unwrap();

        // A normal column keeps its own value — it is NOT clobbered by the non-IP source address.
        assert_eq!(
            get_value_from_event(
                ".eventName",
                &event,
                Some(&rule),
                &mut geo_ip,
                false,
                ".sourceIPAddress"
            ),
            "ListBuckets"
        );
        // The GeoIP columns can't be enriched from a non-IP value, so they show the placeholder.
        assert_eq!(
            get_value_from_event(
                "SrcCountry",
                &event,
                Some(&rule),
                &mut geo_ip,
                false,
                ".sourceIPAddress"
            ),
            "-"
        );
        assert_eq!(
            get_value_from_event(
                "SrcASN",
                &event,
                Some(&rule),
                &mut geo_ip,
                false,
                ".sourceIPAddress"
            ),
            "-"
        );
    }

    // #183: the SrcIP spec is a fallback list, so an earlier candidate that exists but holds
    // nothing usable must not end the search. Before the fix `find_map(event.get)` committed to
    // the first PRESENT key and parsing happened afterwards, so each case below rendered "-"
    // even though a routable address sat in the next field.
    #[test]
    fn geoip_skips_unusable_candidates_and_takes_the_first_parseable_one() {
        use crate::option::geoip::GeoIPSearch;
        use sigma_rust::{event_from_json, rule_from_yaml};
        use std::path::Path;

        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        let mut geo = Some(
            GeoIPSearch::new(Path::new("test_files/mmdb"))
                .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/"),
        );
        let spec = ".claims.ipaddr|.callerIpAddress|.ClientIP|.ActorIpAddress";

        // Every shape an earlier candidate can take while a later one holds a real address.
        // "89.160.20.112:443" is the M365 UAL `ClientIP` host:port form, the likeliest trigger.
        for first in [
            "\"\"",
            "\"-\"",
            "\"not-an-ip\"",
            "\"89.160.20.112:443\"",
            "null",
        ] {
            let event = event_from_json(&format!(
                r#"{{"claims": {{"ipaddr": {first}}}, "callerIpAddress": "89.160.20.112", "eventName": "E"}}"#
            ))
            .unwrap();
            assert_eq!(
                get_value_from_event("SrcCountry", &event, Some(&rule), &mut geo, false, spec),
                "Sweden",
                "an unusable first candidate ({first}) must not end the search"
            );
        }

        // A usable earlier candidate still wins -- the search takes the FIRST parseable value,
        // it does not simply prefer whichever field resolves in the database.
        let both = event_from_json(
            r#"{"claims": {"ipaddr": "81.2.69.144"}, "callerIpAddress": "89.160.20.112", "eventName": "E"}"#,
        )
        .unwrap();
        assert_eq!(
            get_value_from_event("SrcCity", &both, Some(&rule), &mut geo, false, spec),
            "London",
            "the first parseable candidate must win, not a later one"
        );

        // Nothing usable anywhere still yields the placeholder rather than enriching from an
        // unrelated field.
        let none = event_from_json(
            r#"{"claims": {"ipaddr": "not-an-ip"}, "callerIpAddress": "also-not-an-ip", "eventName": "E"}"#,
        )
        .unwrap();
        assert_eq!(
            get_value_from_event("SrcCountry", &none, Some(&rule), &mut geo, false, spec),
            "-"
        );
    }

    // The SrcIP column and the geo columns must always describe the SAME field. They are
    // resolved by one selector for exactly this reason; resolving them separately let SrcIP show
    // an unusable earlier candidate while the geo columns described a later, valid one.
    #[test]
    fn src_ip_column_and_geo_columns_describe_the_same_field() {
        use crate::option::geoip::GeoIPSearch;
        use sigma_rust::{event_from_json, rule_from_yaml};
        use std::path::Path;

        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        let mut geo = Some(
            GeoIPSearch::new(Path::new("test_files/mmdb"))
                .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/"),
        );
        let spec = ".claims.ipaddr|.callerIpAddress|.ClientIP|.ActorIpAddress";
        let value = |key: &str, event: &sigma_rust::Event, geo: &mut _| {
            get_value_from_event(key, event, Some(&rule), geo, false, spec)
        };

        // An unusable first candidate: SrcIP must show the address that was geolocated, not the
        // junk that could not be.
        for first in [
            "\"\"",
            "\"-\"",
            "\"not-an-ip\"",
            "\"89.160.20.112:443\"",
            "null",
        ] {
            let event = event_from_json(&format!(
                r#"{{"claims": {{"ipaddr": {first}}}, "callerIpAddress": "89.160.20.112", "eventName": "E"}}"#
            ))
            .unwrap();
            assert_eq!(
                value(spec, &event, &mut geo),
                "89.160.20.112",
                "SrcIP must show the field the geo columns resolved (first candidate {first})"
            );
            assert_eq!(value("SrcCountry", &event, &mut geo), "Sweden");
        }

        // When nothing parses, SrcIP still reports what the log recorded and the geo columns say
        // so -- they agree that the displayed value could not be geolocated. This is the routine
        // AWS-service case.
        let service = event_from_json(
            r#"{"claims": {"ipaddr": "cloudtrail.amazonaws.com"}, "eventName": "E"}"#,
        )
        .unwrap();
        assert_eq!(value(spec, &service, &mut geo), "cloudtrail.amazonaws.com");
        assert_eq!(value("SrcCountry", &service, &mut geo), "-");

        // No candidate at all: the placeholder, not an empty cell.
        let empty = event_from_json(r#"{"eventName": "E"}"#).unwrap();
        assert_eq!(value(spec, &empty, &mut geo), "-");
        assert_eq!(value("SrcCountry", &empty, &mut geo), "-");

        // The column behaves the same without -G, so enabling GeoIP never changes which address
        // SrcIP displays.
        let mut no_geo = None;
        let event = event_from_json(
            r#"{"claims": {"ipaddr": ""}, "callerIpAddress": "89.160.20.112", "eventName": "E"}"#,
        )
        .unwrap();
        assert_eq!(value(spec, &event, &mut no_geo), "89.160.20.112");
    }

    #[test]
    fn src_ip_spec_reads_profile_srcip_field() {
        let aws = vec![
            ("EventName".to_string(), ".eventName".to_string()),
            ("SrcIP".to_string(), ".sourceIPAddress".to_string()),
        ];
        assert_eq!(src_ip_spec(&aws), ".sourceIPAddress");
        let azure = vec![(
            "SrcIP".to_string(),
            ".claims.ipaddr|.callerIpAddress|.ClientIP|.ActorIpAddress".to_string(),
        )];
        assert_eq!(
            src_ip_spec(&azure),
            ".claims.ipaddr|.callerIpAddress|.ClientIP|.ActorIpAddress"
        );
        let no_srcip: Vec<(String, String)> = vec![("X".to_string(), ".y".to_string())];
        assert_eq!(src_ip_spec(&no_srcip), "");
    }

    // #159: GeoIP enrichment must resolve the source IP from the profile's SrcIP
    // spec, so an Azure `callerIpAddress` enriches identically to an AWS
    // `sourceIPAddress`. Before the fix the Azure field was ignored (the lookup
    // hardcoded `sourceIPAddress`) and the geo columns were always "-".
    //
    // The IP must be one the checked-in GeoLite2 *test* databases actually contain --
    // they are MaxMind's small fixtures, not the real feeds, and resolve only a handful
    // of ranges. 89.160.20.112 is in all three (ASN "Bredband2 AB", city Linköping,
    // country Sweden). An IP outside them, such as a public resolver address, makes
    // every branch return "-" and the assertions stop distinguishing the fix from the
    // bug: verified by mutation that with the hardcoded `event.get("sourceIPAddress")`
    // restored, this test fails on the Azure lookups below and passed before this change.
    #[test]
    fn geoip_resolves_source_ip_via_profile_spec() {
        use crate::option::geoip::GeoIPSearch;
        use sigma_rust::{event_from_json, rule_from_yaml};
        use std::path::Path;

        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        // Present in test_files/mmdb for all three databases.
        let ip = "89.160.20.112";
        let aws_event = event_from_json(&format!(
            r#"{{"sourceIPAddress": "{ip}", "eventName": "E"}}"#
        ))
        .unwrap();
        let azure_event = event_from_json(&format!(
            r#"{{"callerIpAddress": "{ip}", "eventName": "E"}}"#
        ))
        .unwrap();

        let mut geo = Some(
            GeoIPSearch::new(Path::new("test_files/mmdb"))
                .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/"),
        );
        let lookup = |key: &str, event: &sigma_rust::Event, spec: &str, geo: &mut _| {
            get_value_from_event(key, event, Some(&rule), geo, false, spec)
        };

        // The AWS field enriches -- this is the control. If these ever go back to "-" the
        // test databases have changed and the assertions below would stop meaning anything.
        assert_eq!(
            lookup("SrcCountry", &aws_event, ".sourceIPAddress", &mut geo),
            "Sweden",
            "the control lookup must enrich; is 89.160.20.112 still in test_files/mmdb?"
        );

        // The Azure field must enrich identically, resolved through the profile's spec
        // rather than a hardcoded AWS field name. Each of the three geo columns is checked
        // because they are three separate lookups against three separate databases.
        assert_eq!(
            lookup("SrcCountry", &azure_event, ".callerIpAddress", &mut geo),
            "Sweden"
        );
        assert_eq!(
            lookup("SrcASN", &azure_event, ".callerIpAddress", &mut geo),
            "Bredband2 AB"
        );
        assert_eq!(
            lookup("SrcCity", &azure_event, ".callerIpAddress", &mut geo),
            "Linköping"
        );

        // The real Azure/M365 profile declares a `|`-separated fallback list; the field
        // that is actually present must be found wherever it sits in that list.
        let azure_spec = ".claims.ipaddr|.callerIpAddress|.ClientIP|.ActorIpAddress";
        assert_eq!(
            lookup("SrcCountry", &azure_event, azure_spec, &mut geo),
            "Sweden",
            "a later candidate in the SrcIP spec must still be found"
        );

        // A profile with no SrcIP column, or an event carrying none of the candidates,
        // yields the placeholder rather than enriching from some unrelated field.
        assert_eq!(lookup("SrcCountry", &azure_event, "", &mut geo), "-");
        assert_eq!(
            lookup("SrcCountry", &aws_event, ".callerIpAddress", &mut geo),
            "-"
        );
    }

    /// Drives the real `write_record` path -- the shipped profile from `load_profile`, the
    /// profile-derived `SrcIP` spec, the CSV writer -- and returns the emitted row as a map.
    ///
    /// The point is to bind the profile to the enrichment. `geoip_resolves_source_ip_via_profile_spec`
    /// passes literal specs straight to `get_value_from_event`, so hardcoding a field name back
    /// into `write_record` (which is where #159 actually lived) would leave every assertion there
    /// green. Verified by mutation: replacing `src_ip_spec(context.profile)` at the top of
    /// `write_record` with a literal `".sourceIPAddress"` fails this test and nothing else in the
    /// suite.
    fn write_record_columns(
        log: &crate::core::log_source::LogSource,
        event_json: &str,
        geo: &mut Option<crate::option::geoip::GeoIPSearch>,
    ) -> std::collections::HashMap<String, String> {
        use crate::core::util::load_profile;
        use sigma_rust::{event_from_json, rule_from_yaml};

        let profile = load_profile(log, geo, true);
        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        let event = event_from_json(event_json).unwrap();
        let json: Value = serde_json::from_str(event_json).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("out.csv");
        let config = OutputConfig::new(true, false, false);
        {
            let file = std::fs::File::create(&csv_path).unwrap();
            let writers = Writers::new()
                .with_csv(csv::WriterBuilder::new().from_writer(Box::new(file) as Box<dyn Write>));
            let mut context = OutputContext::new(&profile, geo, &config, writers, &[]);
            write_record(&event, &json, Some(&rule), &mut context);
            context.flush_all();
        }

        let text = std::fs::read_to_string(&csv_path).unwrap();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(text.as_bytes());
        let row: Vec<String> = reader
            .records()
            .next()
            .expect("write_record emitted no CSV row")
            .unwrap()
            .iter()
            .map(|f| f.to_string())
            .collect();
        profile.iter().map(|(k, _)| k.clone()).zip(row).collect()
    }

    // #159 must stay fixed at the level it actually broke: the profile. The test above pins the
    // resolution given a spec; this one pins that the shipped Azure/M365 profile is what supplies
    // that spec, end to end through `write_record`.
    #[test]
    fn write_record_enriches_geoip_from_the_shipped_profile() {
        use crate::core::log_source::LogSource;
        use crate::option::geoip::GeoIPSearch;
        use std::path::Path;

        let mut geo = Some(
            GeoIPSearch::new(Path::new("test_files/mmdb"))
                .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/"),
        );

        // Azure: the caller IP lives in `callerIpAddress`, a field name that does not exist in
        // CloudTrail. This is the exact record shape #159 reported as always rendering "-".
        let azure = write_record_columns(
            &LogSource::Azure,
            r#"{"callerIpAddress": "89.160.20.112", "eventName": "E", "operationName": "op"}"#,
            &mut geo,
        );
        assert_eq!(
            azure.get("SrcIP").map(String::as_str),
            Some("89.160.20.112")
        );
        assert_eq!(
            azure.get("SrcASN").map(String::as_str),
            Some("Bredband2 AB")
        );
        assert_eq!(azure.get("SrcCity").map(String::as_str), Some("Linköping"));
        assert_eq!(azure.get("SrcCountry").map(String::as_str), Some("Sweden"));

        // AWS through the same path, so a regression that breaks one log source and not the other
        // is still caught.
        let aws = write_record_columns(
            &LogSource::Aws,
            r#"{"sourceIPAddress": "89.160.20.112", "eventName": "E"}"#,
            &mut geo,
        );
        assert_eq!(aws.get("SrcASN").map(String::as_str), Some("Bredband2 AB"));
        assert_eq!(aws.get("SrcCountry").map(String::as_str), Some("Sweden"));
    }

    /// Drives the complete raw-output path with the shipped profile and returns the JSON object
    /// written to either JSON or JSONL output.
    fn write_raw_record(
        jsonl: bool,
        log: &crate::core::log_source::LogSource,
        event_json: &str,
        geo: &mut Option<crate::option::geoip::GeoIPSearch>,
    ) -> Value {
        use crate::core::util::load_profile;
        use sigma_rust::{event_from_json, rule_from_yaml};

        let profile = load_profile(log, geo, false);
        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        let event = event_from_json(event_json).unwrap();
        let json: Value = serde_json::from_str(event_json).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let output_path = dir
            .path()
            .join(if jsonl { "out.jsonl" } else { "out.json" });
        let config = OutputConfig::new(true, true, false);
        {
            let file = std::fs::File::create(&output_path).unwrap();
            let writer = BufWriter::new(Box::new(file) as Box<dyn Write>);
            let writers = if jsonl {
                Writers::new().with_jsonl(writer)
            } else {
                Writers::new().with_json(writer)
            };
            let mut context = OutputContext::new(&profile, geo, &config, writers, &[]);
            write_record(&event, &json, Some(&rule), &mut context);
            context.flush_all();
        }

        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap()).unwrap()
    }

    // Raw output retains its original log fields and every relevant profile-derived field.
    #[test]
    fn raw_output_preserves_geoip_enrichment() {
        use crate::core::log_source::LogSource;
        use crate::option::geoip::GeoIPSearch;
        use std::path::Path;

        let mut geo = Some(
            GeoIPSearch::new(Path::new("test_files/mmdb"))
                .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/"),
        );
        let event_json =
            r#"{"callerIpAddress": "89.160.20.112", "eventName": "E", "operationName": "op"}"#;

        for jsonl in [false, true] {
            let record = write_raw_record(jsonl, &LogSource::Azure, event_json, &mut geo);
            let output_name = if jsonl { "JSONL" } else { "JSON" };

            assert_eq!(record["callerIpAddress"], "89.160.20.112", "{output_name}");
            assert_eq!(record["RuleTitle"], "t", "{output_name}");
            assert_eq!(record["SrcASN"], "Bredband2 AB", "{output_name}");
            assert_eq!(record["SrcCity"], "Link\u{00f6}ping", "{output_name}");
            assert_eq!(record["SrcCountry"], "Sweden", "{output_name}");
        }
    }

    /// The correlation sibling of `write_record_columns`: drives `write_correlation_record`,
    /// which builds its row through `get_value_from_correlation_event` and computes the spec at a
    /// *different* call site (`build_correlation_record`). A hardcode there would be invisible to
    /// the non-correlation tests.
    fn write_correlation_columns(
        log: &crate::core::log_source::LogSource,
        event_json: &str,
        geo: &mut Option<crate::option::geoip::GeoIPSearch>,
    ) -> std::collections::HashMap<String, String> {
        use crate::core::util::load_profile;
        use sigma_rust::{SigmaCorrelationRule, TimestampedEvent, event_from_json, rule_from_yaml};

        let profile = load_profile(log, geo, true);
        let base_rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        let correlation_rule = SigmaCorrelationRule {
            title: "c".to_string(),
            ..Default::default()
        };
        let timestamped = TimestampedEvent {
            event: event_from_json(event_json).unwrap(),
            timestamp: "2024-01-01T00:00:00Z".parse().unwrap(),
            rule: &base_rule,
        };

        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("out.csv");
        let config = OutputConfig::new(true, false, false);
        {
            let file = std::fs::File::create(&csv_path).unwrap();
            let writers = Writers::new()
                .with_csv(csv::WriterBuilder::new().from_writer(Box::new(file) as Box<dyn Write>));
            let mut context = OutputContext::new(&profile, geo, &config, writers, &[]);
            write_correlation_record(&vec![&timestamped], &correlation_rule, &mut context);
            context.flush_all();
        }

        let text = std::fs::read_to_string(&csv_path).unwrap();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(text.as_bytes());
        let row: Vec<String> = reader
            .records()
            .next()
            .expect("write_correlation_record emitted no CSV row")
            .unwrap()
            .iter()
            .map(|f| f.to_string())
            .collect();
        profile.iter().map(|(k, _)| k.clone()).zip(row).collect()
    }

    // Correlation rows go through their own record builder and their own `src_ip_spec` call, so
    // #159 could regress there alone. Verified by mutation: hardcoding the spec inside
    // `build_correlation_record` fails this test and nothing else.
    #[test]
    fn write_correlation_record_enriches_geoip_from_the_shipped_profile() {
        use crate::option::geoip::GeoIPSearch;
        use std::path::Path;

        let mut geo = Some(
            GeoIPSearch::new(Path::new("test_files/mmdb"))
                .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/"),
        );
        let azure = write_correlation_columns(
            &crate::core::log_source::LogSource::Azure,
            r#"{"callerIpAddress": "89.160.20.112", "eventName": "E", "operationName": "op"}"#,
            &mut geo,
        );
        assert_eq!(
            azure.get("SrcASN").map(String::as_str),
            Some("Bredband2 AB")
        );
        assert_eq!(azure.get("SrcCountry").map(String::as_str), Some("Sweden"));
    }

    // Without -G the profile has no geo columns at all, so the enrichment above cannot be an
    // artifact of columns that are always present.
    #[test]
    fn write_record_omits_geo_columns_without_geoip() {
        use crate::core::log_source::LogSource;
        let mut geo = None;
        let azure = write_record_columns(
            &LogSource::Azure,
            r#"{"callerIpAddress": "89.160.20.112", "eventName": "E", "operationName": "op"}"#,
            &mut geo,
        );
        assert_eq!(
            azure.get("SrcIP").map(String::as_str),
            Some("89.160.20.112")
        );
        for key in ["SrcASN", "SrcCity", "SrcCountry"] {
            assert!(
                !azure.contains_key(key),
                "{key} must not be emitted without -G"
            );
        }
    }

    /// The full AWS timeline profile, so the schema assertions below run against the real
    /// column set rather than a hand-picked subset.
    fn aws_profile_keys() -> Vec<String> {
        [
            "Timestamp",
            "RuleTitle",
            "Level",
            "EventName",
            "ErrorCode",
            "AWS-Region",
            "UserName",
            "EventID",
            "Tags",
            "RuleID",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn aws_row(
        ts: &str,
        level: &str,
        error: &str,
        user: &str,
        id: &str,
        tags: &str,
    ) -> Vec<String> {
        vec![
            ts.to_string(),
            "Rule A".to_string(),
            level.to_string(),
            "GetPolicy".to_string(),
            error.to_string(),
            "us-east-1".to_string(),
            user.to_string(),
            id.to_string(),
            tags.to_string(),
            "rule-1".to_string(),
        ]
    }

    fn finalized_sink(path: &Path, keys: &[String], rows: &[Vec<String>]) -> Connection {
        let mut sink = DuckDbSink::new(path, keys, SuzakuMeta::new("aws-ct-timeline")).unwrap();
        for row in rows {
            sink.append_row(row);
        }
        sink.finalize().unwrap();
        // Drop the writer connection before re-opening so the assertions read the file, not the
        // in-flight transaction.
        drop(sink);
        Connection::open(path).unwrap()
    }

    #[test]
    fn duckdb_sink_creates_named_table_and_appends_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let conn = finalized_sink(
            &path,
            &keys,
            &[
                aws_row("2024-01-01 00:00:00", "high", "-", "alice", "e1", "-"),
                aws_row("2024-01-02 00:00:00", "low", "-", "bob", "e2", "-"),
            ],
        );

        let rows: i64 = conn
            .query_row("SELECT count(*) FROM timeline", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2);
        // The hyphenated profile key becomes a plain identifier: no quoting needed, ever.
        let title: String = conn
            .query_row(
                "SELECT RuleTitle FROM timeline WHERE AwsRegion = 'us-east-1' AND UserName = 'bob'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title, "Rule A");
    }

    /// P2: `-` and `''` are CSV presentation conventions. In DuckDB the absence of a value must be
    /// NULL, or `ErrorCode IS NOT NULL` silently answers the wrong question.
    #[test]
    fn duckdb_sink_writes_null_not_placeholders() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let conn = finalized_sink(
            &path,
            &keys,
            &[
                aws_row("2024-01-01 00:00:00", "high", "-", "", "e1", "-"),
                aws_row(
                    "2024-01-02 00:00:00",
                    "high",
                    "AccessDenied",
                    "bob",
                    "e2",
                    "-",
                ),
            ],
        );

        let (nulls, dashes, failures): (i64, i64, i64) = conn
            .query_row(
                "SELECT count(*) FILTER (WHERE ErrorCode IS NULL),
                        count(*) FILTER (WHERE ErrorCode = '-'),
                        count(*) FILTER (WHERE ErrorCode IS NOT NULL)
                 FROM timeline",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(nulls, 1);
        assert_eq!(dashes, 0, "'-' must not survive into the database");
        assert_eq!(failures, 1, "IS NOT NULL must mean 'the call failed'");
        // The empty-string placeholder is the same absence and must map to the same NULL.
        let empty_users: i64 = conn
            .query_row(
                "SELECT count(*) FROM timeline WHERE UserName IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(empty_users, 1);
    }

    /// P3: temporal and severity columns carry real types, so range filters and severity ordering
    /// do not depend on the rendering happening to sort lexicographically.
    #[test]
    fn duckdb_sink_types_timestamp_and_level() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let conn = finalized_sink(
            &path,
            &keys,
            &[
                aws_row("2024-01-01 00:00:00", "low", "-", "a", "e1", "-"),
                aws_row("2024-03-01 12:30:00", "critical", "-", "b", "e2", "-"),
                aws_row("not-a-timestamp", "informational", "-", "c", "e3", "-"),
            ],
        );

        let types: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT column_name, data_type FROM duckdb_columns()
                     WHERE table_name = 'timeline' AND column_name IN ('Timestamp', 'Level')",
                )
                .unwrap();
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        let timestamp_type = &types.iter().find(|(c, _)| c == "Timestamp").unwrap().1;
        assert_eq!(timestamp_type, "TIMESTAMP");
        let level_type = &types.iter().find(|(c, _)| c == "Level").unwrap().1;
        assert!(level_type.starts_with("ENUM"), "got: {level_type}");

        // A range filter is a real temporal comparison, not string prefix luck.
        let in_range: i64 = conn
            .query_row(
                "SELECT count(*) FROM timeline
                 WHERE Timestamp BETWEEN TIMESTAMP '2024-02-01' AND TIMESTAMP '2024-04-01'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(in_range, 1);
        // Severity order lives in the type: 'critical' outranks 'low' even though it sorts before
        // it alphabetically.
        let worst: String = conn
            .query_row("SELECT max(Level)::VARCHAR FROM timeline", [], |r| r.get(0))
            .unwrap();
        assert_eq!(worst, "critical");
        // The type is named, which is what makes a severity threshold expressible: an ENUM
        // compared against a bare literal falls back to text comparison, where 'informational'
        // would count as ">= high".
        let at_least_high: i64 = conn
            .query_row(
                "SELECT count(*) FROM timeline WHERE Level >= 'high'::suzaku_level",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(at_least_high, 1, "only the 'critical' row is >= high");
        // One unparseable timestamp degrades to NULL instead of failing the whole write.
        let unparseable: i64 = conn
            .query_row(
                "SELECT count(*) FROM timeline WHERE Timestamp IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unparseable, 1);
    }

    /// P4: byte-identical rows carry no information but inflate every count derived from the file.
    /// They are dropped, and how many were dropped is reported rather than hidden.
    #[test]
    fn duckdb_sink_drops_exact_duplicate_rows_and_reports_the_count() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let dup = aws_row("2024-01-01 00:00:00", "high", "-", "alice", "e1", "-");
        let conn = finalized_sink(
            &path,
            &keys,
            &[
                dup.clone(),
                dup.clone(),
                dup,
                aws_row("2024-01-02 00:00:00", "high", "-", "bob", "e2", "-"),
            ],
        );

        assert_eq!(
            conn.query_row("SELECT count(*) FROM timeline", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        let (rows, removed): (i64, i64) = conn
            .query_row(
                "SELECT output_rows, duplicate_rows_removed FROM suzaku_meta",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 2);
        assert_eq!(removed, 2);
    }

    /// P5: tactics and technique IDs share one delimiter-packed column today, so a consumer has to
    /// split on a non-ASCII separator and then guess which entries are techniques.
    #[test]
    fn duckdb_sink_splits_tags_into_typed_lists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let conn = finalized_sink(
            &path,
            &keys,
            &[
                aws_row(
                    "2024-01-01 00:00:00",
                    "high",
                    "-",
                    "alice",
                    "e1",
                    "PrivEsc ¦ InitAccess ¦ T1078.004 ¦ G0035",
                ),
                aws_row("2024-01-02 00:00:00", "high", "-", "bob", "e2", "-"),
            ],
        );

        // `Tags` is gone; the three typed lists replace it.
        let tags_column: i64 = conn
            .query_row(
                "SELECT count(*) FROM duckdb_columns()
                 WHERE table_name = 'timeline' AND column_name = 'Tags'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tags_column, 0);

        let (tactics, techniques, other): (String, String, String) = conn
            .query_row(
                "SELECT list_aggregate(Tactics, 'string_agg', ','),
                        list_aggregate(TechniqueIDs, 'string_agg', ','),
                        list_aggregate(OtherTags, 'string_agg', ',')
                 FROM timeline WHERE UserName = 'alice'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(tactics, "PrivEsc,InitAccess");
        assert_eq!(techniques, "T1078.004");
        assert_eq!(other, "G0035", "group tags are kept, just not as tactics");

        // Technique coverage is now one unnest, with no "starts with T" heuristic.
        let hits: i64 = conn
            .query_row(
                "SELECT count(*) FROM timeline WHERE list_contains(TechniqueIDs, 'T1078.004')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1);
        // An untagged rule yields empty lists, never a one-element list holding the placeholder.
        let empty: i64 = conn
            .query_row(
                "SELECT len(Tactics) + len(TechniqueIDs) + len(OtherTags)
                 FROM timeline WHERE UserName = 'bob'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(empty, 0);
    }

    /// The profile a run produces: `SrcIP` plus, under `-G` only, the three geo keys after it.
    fn profile_keys_with_src_ip(geo: bool) -> Vec<String> {
        let mut keys = vec![
            "Timestamp".to_string(),
            "SrcIP".to_string(),
            "UserName".to_string(),
            "Tags".to_string(),
        ];
        if geo {
            keys.splice(2..2, GEO_COLUMNS.iter().map(|c| c.to_string()));
        }
        keys
    }

    fn timeline_columns(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT column_name FROM duckdb_columns()
                 WHERE table_name = 'timeline' ORDER BY column_index",
            )
            .unwrap();
        stmt.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    /// P8: without `-G` the geo columns are still part of the table, all NULL. A column that comes
    /// and goes with a run-time flag makes one query a binder error on half the files.
    #[test]
    fn duckdb_sink_adds_null_geo_columns_without_geoip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = profile_keys_with_src_ip(false);
        let conn = finalized_sink(
            &path,
            &keys,
            &[vec![
                "2024-01-01 00:00:00".to_string(),
                "81.2.69.142".to_string(),
                "alice".to_string(),
                "PrivEsc".to_string(),
            ]],
        );

        // The geo columns sit where enrichment would have put them, right after SrcIP, and the
        // Tags expansion still lands on the columns it belongs to.
        assert_eq!(
            timeline_columns(&conn),
            [
                "Timestamp",
                "SrcIP",
                "SrcASN",
                "SrcCity",
                "SrcCountry",
                "UserName",
                "Tactics",
                "TechniqueIDs",
                "OtherTags",
            ]
        );
        let (nulls, ip, tactics): (i64, String, String) = conn
            .query_row(
                "SELECT count(*) FILTER (WHERE SrcASN IS NULL AND SrcCity IS NULL
                                            AND SrcCountry IS NULL),
                        any_value(SrcIP),
                        any_value(list_aggregate(Tactics, 'string_agg', ','))
                 FROM timeline",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(nulls, 1);
        assert_eq!(ip, "81.2.69.142");
        assert_eq!(tactics, "PrivEsc");
    }

    /// With `-G` the profile already carries the geo keys, so the filler must not add a second set
    /// and the enriched values must stay in their own columns.
    #[test]
    fn duckdb_sink_keeps_enriched_geo_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = profile_keys_with_src_ip(true);
        let conn = finalized_sink(
            &path,
            &keys,
            &[vec![
                "2024-01-01 00:00:00".to_string(),
                "81.2.69.142".to_string(),
                "AS1234".to_string(),
                "London".to_string(),
                "United Kingdom".to_string(),
                "alice".to_string(),
                "-".to_string(),
            ]],
        );

        assert_eq!(
            timeline_columns(&conn),
            [
                "Timestamp",
                "SrcIP",
                "SrcASN",
                "SrcCity",
                "SrcCountry",
                "UserName",
                "Tactics",
                "TechniqueIDs",
                "OtherTags",
            ]
        );
        let (city, country, user): (String, String, String) = conn
            .query_row(
                "SELECT SrcCity, SrcCountry, UserName FROM timeline",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(city, "London");
        assert_eq!(country, "United Kingdom");
        assert_eq!(user, "alice");
    }

    /// The geo columns describe `SrcIP`, so a profile without one gets nothing to describe.
    #[test]
    fn duckdb_sink_omits_geo_columns_when_the_profile_has_no_src_ip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let conn = finalized_sink(
            &path,
            &keys,
            &[aws_row("2024-01-01 00:00:00", "high", "-", "a", "e1", "-")],
        );
        assert!(!timeline_columns(&conn).iter().any(|c| c == "SrcCountry"));
    }

    /// P1: the file says what produced it, so a consumer looks the command up instead of
    /// inferring it from which tables happen to exist.
    #[test]
    fn duckdb_sink_writes_self_describing_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let mut sink = DuckDbSink::new(
            &path,
            &keys,
            SuzakuMeta::new("aws-ct-timeline").with_localtime(false),
        )
        .unwrap();
        sink.meta.scanned_files = Some(7);
        sink.meta.scanned_events = Some(1234);
        sink.append_row(&aws_row("2024-01-01 00:00:00", "high", "-", "a", "e1", "-"));
        sink.finalize().unwrap();
        drop(sink);

        let conn = Connection::open(&path).unwrap();
        let (schema_version, command, tz, files, events): (i32, String, String, i64, i64) = conn
            .query_row(
                "SELECT schema_version, command, timestamp_tz, scanned_files, scanned_events
                 FROM suzaku_meta",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert_eq!(schema_version, duckdb_out::SCHEMA_VERSION);
        assert_eq!(command, "aws-ct-timeline");
        assert_eq!(tz, "UTC");
        assert_eq!(files, 7);
        assert_eq!(events, 1234);

        // P9: the row grain is recorded in the file, not only in the docs.
        let grain: String = conn
            .query_row(
                "SELECT comment FROM duckdb_tables() WHERE table_name = 'timeline'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(grain.contains("rule match"), "got: {grain}");
    }

    /// P10: a database left with an unreplayed WAL cannot be opened from a read-only mount, which
    /// is how dashboards attach the evidence file.
    #[test]
    fn duckdb_sink_leaves_no_wal_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let keys = aws_profile_keys();
        let conn = finalized_sink(
            &path,
            &keys,
            &[aws_row("2024-01-01 00:00:00", "high", "-", "a", "e1", "-")],
        );
        drop(conn);

        let wal = dir.path().join("t.duckdb.wal");
        assert!(
            !wal.exists() || std::fs::metadata(&wal).unwrap().len() == 0,
            "the checkpoint on exit must leave no work in the WAL"
        );
        // And the file really is readable read-only, with no writer around.
        let cfg = duckdb::Config::default()
            .access_mode(duckdb::AccessMode::ReadOnly)
            .unwrap();
        let ro = Connection::open_with_flags(&path, cfg).unwrap();
        assert_eq!(
            ro.query_row("SELECT count(*) FROM timeline", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn classify_tag_separates_tactics_techniques_and_the_rest() {
        assert_eq!(classify_tag("PrivEsc"), TagKind::Tactic);
        assert_eq!(classify_tag("CredAccess"), TagKind::Tactic);
        assert_eq!(classify_tag("T1078.004"), TagKind::Technique);
        assert_eq!(classify_tag("T1110"), TagKind::Technique);
        assert_eq!(classify_tag("G0035"), TagKind::Other);
        assert_eq!(classify_tag("cve.2021.1234"), TagKind::Other);
        // A tactic abbreviation starting with T must not be mistaken for a technique.
        assert_eq!(classify_tag("Trans"), TagKind::Other);
    }

    #[test]
    fn split_tags_buckets_a_packed_value() {
        let [tactics, techniques, other] = split_tags("PrivEsc ¦ T1078.004 ¦ G0035");
        assert_eq!(tactics, "PrivEsc");
        assert_eq!(techniques, "T1078.004");
        assert_eq!(other, "G0035");
        // The placeholder is an absence, not a tag.
        assert_eq!(split_tags("-"), ["", "", ""]);
    }

    #[test]
    fn duckdb_column_name_removes_the_hyphen() {
        assert_eq!(duckdb_column_name("AWS-Region"), "AwsRegion");
        assert_eq!(duckdb_column_name("EventName"), "EventName");
    }

    /// Rows are buffered and written per batch, so every row must survive both an automatic
    /// mid-scan flush at the batch boundary and the final flush of the partial batch.
    #[test]
    fn duckdb_sink_writes_every_row_across_batch_boundaries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let cols = vec!["Timestamp".to_string(), "RuleTitle".to_string()];
        let rows = DUCKDB_BATCH_ROWS + 7;

        let mut sink = DuckDbSink::new(&path, &cols, SuzakuMeta::new("aws-ct-timeline")).unwrap();
        for i in 0..rows {
            // Distinct timestamps as well as titles, so the dedup pass cannot mask a lost row.
            sink.append_row(&[format!("2024-01-01 00:00:00.{i:06}"), format!("Rule {i}")]);
        }
        sink.finalize().unwrap();
        drop(sink);

        let conn = Connection::open(&path).unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM timeline", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count as usize, rows);
        // Spot-check a row from the first (auto-flushed) batch and the last (partial) one.
        for i in [0, DUCKDB_BATCH_ROWS - 1, rows - 1] {
            let found: i64 = conn
                .query_row(
                    "SELECT count(*) FROM timeline WHERE RuleTitle = ?",
                    [format!("Rule {i}")],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(found, 1, "row {i} must be present exactly once");
        }
    }

    #[test]
    fn resolve_output_paths_expands_base_per_format_and_dedups() {
        // Each format maps the base to <base>.<ext>, preserving order.
        assert_eq!(
            resolve_output_paths(
                Path::new("result"),
                &[OutputFormat::Csv, OutputFormat::Duckdb, OutputFormat::Jsonl]
            ),
            vec![
                PathBuf::from("result.csv"),
                PathBuf::from("result.duckdb"),
                PathBuf::from("result.jsonl"),
            ]
        );
        // A base that already carries some extension is normalized to the format's extension.
        assert_eq!(
            resolve_output_paths(Path::new("out.csv"), &[OutputFormat::Duckdb]),
            vec![PathBuf::from("out.duckdb")]
        );
        // Repeated formats collapse to a single path.
        assert_eq!(
            resolve_output_paths(Path::new("result"), &[OutputFormat::Csv, OutputFormat::Csv]),
            vec![PathBuf::from("result.csv")]
        );
    }

    #[test]
    fn flush_all_drops_duckdb_sink_and_removes_empty_db_on_no_hits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        let profile = vec![("Timestamp".to_string(), ".eventTime".to_string())];
        let sink = DuckDbSink::new(
            &path,
            &["Timestamp".to_string()],
            SuzakuMeta::new("aws-ct-timeline"),
        )
        .unwrap();
        // Opening the sink creates the database file.
        assert!(path.exists());

        let writers = Writers::new().with_duckdb(sink);
        let config = OutputConfig::new(true, false, false);
        let mut geo = None;
        let output_paths = vec![path.clone()];
        let mut ctx = OutputContext::new(&profile, &mut geo, &config, writers, &output_paths);

        // Never wrote a record, so flush_all takes the no-hit cleanup path.
        ctx.flush_all();

        assert!(
            ctx.writers.duckdb.is_none(),
            "the DuckDB sink must be dropped so its connection releases the file lock"
        );
        assert!(
            !path.exists(),
            "the empty .duckdb database must be removed when there are no hits"
        );
    }
}
