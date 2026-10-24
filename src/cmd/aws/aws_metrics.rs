use crate::core::color::SuzakuColor::Red;
use crate::core::duckdb_out::{self, SuzakuMeta, nullable, quote_ident, timestamp_expr};
use crate::core::errorlog::{log_error, log_warn};
use crate::core::log_source::LogSource;
use crate::core::scan::{load_aws_events_from_file, process_events_from_dir};
use crate::core::timeline_writer::{duckdb_column_name, resolve_output_targets};
use crate::core::util::{
    error_msg, fatal_error, get_writer, output_path_info, p, report_unreadable_input,
    sanitize_csv_field, upsert_count_entry,
};
use crate::option::cli::{MetricsOptions, OutputFormat};
use crate::option::geoip::GeoIPSearch;
use crate::option::timefiler::filter_by_time;
use comfy_table::{Cell, CellAlignment, Table};
use duckdb::Connection;
use serde::Serialize;
use serde_json::Value;
use sigma_rust::{Event, event_from_json};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Value shown for an event that has no value for the requested field. CloudTrail routinely
/// omits fields (`userIdentity.accessKeyId` is absent for console sessions, `errorCode` for
/// successful calls), and those events are part of the picture: counting them under `-`
/// keeps every field's total equal to the number of events scanned, so the percentages mean
/// "share of all events" rather than "share of the events that happened to have this field".
/// This matches how `aws-ct-summary` renders a missing field.
const NO_VALUE: &str = "";

/// Prefix of AWS STS temporary access key IDs (as opposed to `AKIA` long-term keys).
const STS_KEY_PREFIX: &str = "ASIA";

// ---------------------------------------------------------------------------
// 集計用内部構造体
// ---------------------------------------------------------------------------

/// `value -> (count, first_seen, last_seen)` for a single field.
type CountMap = HashMap<String, (usize, String, String)>;

/// Per-field counters. Every requested field is tallied in the same pass over the logs, so
/// aggregating five fields costs one scan instead of five.
#[derive(Default)]
struct FieldMetrics {
    counts: CountMap,
    total: usize,
}

struct Metrics {
    fields: Vec<String>,
    per_field: Vec<FieldMetrics>,
    include_sts: bool,
}

/// What the run covered, for `suzaku_meta`. Neither number is derivable from the output rows:
/// the rows are an aggregation, and every field's total excludes the events dropped by `-s`.
#[derive(Default, Clone, Copy)]
struct ScanStats {
    files: usize,
    events: usize,
}

impl Metrics {
    fn new(fields: &[String], include_sts: bool) -> Self {
        Metrics {
            fields: fields.to_vec(),
            per_field: fields.iter().map(|_| FieldMetrics::default()).collect(),
            include_sts,
        }
    }

    fn add_event(&mut self, event: &Event) {
        let event_time = match event.get("eventTime") {
            Some(time) => time.value_to_string(),
            None => NO_VALUE.to_string(),
        };
        for (i, field) in self.fields.iter().enumerate() {
            let value = match event.get(field) {
                Some(value) => value.value_to_string(),
                None => NO_VALUE.to_string(),
            };
            // Temporary STS keys are noise for most investigations (a new key per session
            // blows up the value count), so they are dropped unless -s is given. Same rule
            // and same flag as aws-ct-summary.
            if !self.include_sts
                && field.ends_with("accessKeyId")
                && value.starts_with(STS_KEY_PREFIX)
            {
                continue;
            }
            let metrics = &mut self.per_field[i];
            metrics.total += 1;
            upsert_count_entry(&mut metrics.counts, value, &event_time);
        }
    }

    fn is_empty(&self) -> bool {
        self.per_field.iter().all(|m| m.total == 0)
    }

    /// Fields that no event had a value for, i.e. whose only tallied value is `-`. Almost always
    /// a misspelled or mis-cased `-F` (CloudTrail's field is `sourceIPAddress`, not
    /// `sourceIPaddress`), which would otherwise be indistinguishable from real data: counting
    /// missing values as `-` means a typo produces a plausible-looking 100% `-` table.
    fn fields_with_no_values(&self) -> Vec<&str> {
        self.fields
            .iter()
            .zip(self.per_field.iter())
            .filter(|(_, data)| data.counts.keys().all(|value| value == NO_VALUE))
            .map(|(field, _)| field.as_str())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 出力用データ構造
// ---------------------------------------------------------------------------

/// One output row: a single value of a single field. `percent` is that value's share of all
/// events counted for the field.
#[derive(Serialize, Debug, PartialEq)]
pub struct MetricRecord {
    pub field: String,
    pub value: String,
    pub count: usize,
    /// Events counted for this record's field, i.e. the denominator `percent` was derived from.
    /// Not serialized: it exists so the DuckDB output can carry the denominator (`sum(Percent)`
    /// over a field is 99.03, not 100, once every value has been rounded for display), and the
    /// CSV and JSON outputs stay exactly as they were.
    #[serde(skip)]
    pub field_total: usize,
    pub percent: f64,
    pub first_seen: String,
    pub last_seen: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_asn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_country: Option<String>,
}

impl MetricRecord {
    fn percent_str(&self) -> String {
        format!("{:.2}%", self.percent)
    }

    /// This value's share of its field, without the two-decimal rounding `percent` carries for
    /// display. Written to DuckDB, where the number is re-aggregated rather than read.
    fn exact_percent(&self) -> f64 {
        if self.field_total == 0 {
            0.0
        } else {
            self.count as f64 / self.field_total as f64 * 100.0
        }
    }

    /// The three GeoIP cells, or `-` when this value could not be enriched. Only called for
    /// the CSV and stdout outputs, which stay rectangular; JSON omits the fields entirely.
    fn geo_cells(&self) -> [String; 3] {
        let cell = |v: &Option<String>| v.clone().unwrap_or_else(|| NO_VALUE.to_string());
        [
            cell(&self.src_asn),
            cell(&self.src_city),
            cell(&self.src_country),
        ]
    }
}

// ---------------------------------------------------------------------------
// パブリックエントリポイント
// ---------------------------------------------------------------------------

pub fn aws_metrics(opt: &MetricsOptions, no_color: bool) {
    let directory = &opt.input_opt.directory;
    let file = &opt.input_opt.filepath;

    let mut geo_search = None;
    if let Some(path) = opt.geo_ip.as_ref() {
        match GeoIPSearch::new(path) {
            Ok(geo) => geo_search = Some(geo),
            Err(_) => {
                p(
                    Red.rdg(no_color),
                    "Could not find the appropriate MaxMind GeoIP .mmdb database files.\n",
                    true,
                );
                return;
            }
        }
    }

    let mut metrics = Metrics::new(&opt.field_names, opt.include_sts);
    // Coverage of the run, recorded in the DuckDB output's `suzaku_meta` so a report can state
    // what was aggregated. Counted before the time filter, like the other commands, so it means
    // "events read" rather than "events that survived -T/-t".
    let mut scan = ScanStats::default();
    let mut stats_func = |json_values: &[Value]| {
        for json_value in json_values {
            scan.events += 1;
            if !filter_by_time(&opt.input_opt.time_opt, json_value, "eventTime") {
                continue;
            }
            let event: Event = match event_from_json(json_value.to_string().as_str()) {
                Ok(event) => event,
                Err(_) => continue,
            };
            metrics.add_event(&event);
        }
    };

    if let Some(d) = directory {
        match process_events_from_dir(
            stats_func,
            d,
            true,
            no_color,
            &LogSource::Aws,
            &opt.input_opt.file_date_opt,
        ) {
            Ok(files) => scan.files = files,
            Err(e) => log_error(&format!("Failed to scan directory {}: {e}", d.display())),
        }
    } else if let Some(f) = file {
        match load_aws_events_from_file(f) {
            Ok(events) => {
                stats_func(&events);
                scan.files = 1;
            }
            Err(e) => {
                report_unreadable_input(no_color, f, &e);
                return;
            }
        }
    }

    output_metrics(&metrics, opt, geo_search.as_mut(), no_color, scan);
}

// ---------------------------------------------------------------------------
// レコード生成
// ---------------------------------------------------------------------------

/// Turn the counters into output rows, ordered by count (most frequent first) and GeoIP-enriched
/// when `-G` is given.
fn build_records(metrics: &Metrics, geo_search: Option<&mut GeoIPSearch>) -> Vec<MetricRecord> {
    let mut geo_search = geo_search;
    let mut records = Vec::new();

    for (field, data) in metrics.fields.iter().zip(metrics.per_field.iter()) {
        let mut entries: Vec<(&String, &(usize, String, String))> = data.counts.iter().collect();
        // Ties are broken by value so the output is stable across runs: a HashMap iterates in
        // an arbitrary order, which would otherwise shuffle equal-count rows between runs and
        // make a diff of two result files unreadable.
        entries.sort_by(|a, b| b.1.0.cmp(&a.1.0).then_with(|| a.0.cmp(b.0)));

        for (value, (count, first, last)) in entries {
            let percent = if data.total == 0 {
                0.0
            } else {
                (*count as f64 / data.total as f64) * 100.0
            };
            let (src_asn, src_city, src_country) = match geo_search.as_mut() {
                // Only values that parse as an IP address are enriched. Anything else — an
                // event name, or the `cloudtrail.amazonaws.com` that CloudTrail writes into
                // sourceIPAddress for AWS-service calls — is reported as `-`.
                Some(geo) => match geo.convert(value.as_str()) {
                    Some(ip) => (
                        Some(geo.get_asn(ip)),
                        Some(geo.get_city(ip)),
                        Some(geo.get_country(ip)),
                    ),
                    None => (
                        Some(NO_VALUE.to_string()),
                        Some(NO_VALUE.to_string()),
                        Some(NO_VALUE.to_string()),
                    ),
                },
                None => (None, None, None),
            };
            records.push(MetricRecord {
                field: field.clone(),
                value: value.clone(),
                count: *count,
                field_total: data.total,
                percent: (percent * 100.0).round() / 100.0,
                first_seen: render_time(first),
                last_seen: render_time(last),
                src_asn,
                src_city,
                src_country,
            });
        }
    }
    records
}

/// Render a CloudTrail RFC 3339 timestamp the way the rest of Suzaku does: `2024-01-02 03:04:05`.
fn render_time(time: &str) -> String {
    time.replace('T', " ").replace('Z', "")
}

// ---------------------------------------------------------------------------
// 出力処理
// ---------------------------------------------------------------------------

fn output_metrics(
    metrics: &Metrics,
    opt: &MetricsOptions,
    geo_search: Option<&mut GeoIPSearch>,
    no_color: bool,
    scan: ScanStats,
) {
    if metrics.is_empty() {
        error_msg(no_color, "No results found.");
        return;
    }

    // The field is spelled correctly (`-F` is validated up front) but these logs do not carry
    // it — `errorCode` in an export with no failed calls, say. Worth saying out loud, because
    // the resulting single `-` row at 100% otherwise reads like a real result.
    for field in metrics.fields_with_no_values() {
        error_msg(
            no_color,
            &format!(
                "No event had a value for the field \"{field}\", so every event is counted as \"-\"."
            ),
        );
    }

    let geo_enabled = geo_search.is_some();

    let Some(output) = opt.output.as_ref() else {
        let records = build_records(metrics, geo_search);
        print_tables(metrics, &records, geo_enabled);
        return;
    };

    // One <base>.<ext> per requested format, deduped — the same resolution the timeline and
    // summary commands use, so -o/-t behave identically across the tool.
    let targets = resolve_output_targets(output, &opt.output_types);
    let path_for = |format: OutputFormat| -> Option<PathBuf> {
        targets
            .iter()
            .find(|(fmt, _)| *fmt == format)
            .map(|(_, path)| path.clone())
    };

    // Checked before the records are built so an existing file fails immediately, without
    // first paying for the GeoIP lookups.
    if !opt.clobber
        && let Some(path) = targets
            .iter()
            .map(|(_, path)| path)
            .find(|path| path.exists())
    {
        error_msg(
            no_color,
            &format!(
                "The file {} already exists. Use --clobber to overwrite.",
                path.display()
            ),
        );
        return;
    }

    let records = build_records(metrics, geo_search);
    let mut output_paths: Vec<PathBuf> = Vec::new();

    if let Some(csv_path) = path_for(OutputFormat::Csv) {
        write_csv(&csv_path, &records, geo_enabled, no_color);
        output_paths.push(csv_path);
    }

    if let Some(json_path) = path_for(OutputFormat::Json) {
        let file = File::create(&json_path).unwrap_or_else(|e| {
            fatal_error(
                no_color,
                &format!("Cannot write to output file {}: {e}", json_path.display()),
            )
        });
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &records).unwrap();
        writer.flush().unwrap();
        output_paths.push(json_path);
    }

    if let Some(jsonl_path) = path_for(OutputFormat::Jsonl) {
        let file = File::create(&jsonl_path).unwrap_or_else(|e| {
            fatal_error(
                no_color,
                &format!("Cannot write to output file {}: {e}", jsonl_path.display()),
            )
        });
        let mut writer = BufWriter::new(file);
        for record in &records {
            writeln!(writer, "{}", serde_json::to_string(record).unwrap()).unwrap();
        }
        writer.flush().unwrap();
        output_paths.push(jsonl_path);
    }

    if let Some(duckdb_path) = path_for(OutputFormat::Duckdb) {
        match write_duckdb_metrics(&duckdb_path, &records, geo_enabled, scan) {
            Ok(()) => output_paths.push(duckdb_path),
            Err(e) => fatal_error(no_color, &e),
        }
    }

    output_path_info(no_color, output_paths.as_slice(), true);
}

/// Column names for the CSV output. The stdout tables replace `Field`/`Value` with a single
/// column named after the field itself, since they are printed one table per field.
fn csv_header(geo_enabled: bool) -> Vec<&'static str> {
    let mut header = vec![
        "Field",
        "Value",
        "Count",
        "Percent",
        "FirstSeen",
        "LastSeen",
    ];
    if geo_enabled {
        header.extend(["SrcASN", "SrcCity", "SrcCountry"]);
    }
    header
}

fn write_csv(path: &Path, records: &[MetricRecord], geo_enabled: bool, no_color: bool) {
    let mut wtr =
        get_writer(&Some(path.to_path_buf())).unwrap_or_else(|e| fatal_error(no_color, &e));
    wtr.write_record(csv_header(geo_enabled)).unwrap();
    for record in records {
        let mut row = vec![
            record.field.clone(),
            record.value.clone(),
            record.count.to_string(),
            record.percent_str(),
            record.first_seen.clone(),
            record.last_seen.clone(),
        ];
        if geo_enabled {
            row.extend(record.geo_cells());
        }
        let sanitized: Vec<String> = row.iter().map(|f| sanitize_csv_field(f)).collect();
        wtr.write_record(&sanitized).unwrap();
    }
    wtr.flush().ok();
}

/// CloudTrail field path -> the DuckDB timeline column holding the same fact, e.g.
/// `sourceIPAddress` -> `SrcIP`.
///
/// `Field` keeps the path the user typed after `-F` — it is the command's input, it round-trips,
/// and since `-F` accepts paths into the API-specific containers (`requestParameters.bucketName`)
/// it has no timeline counterpart in the general case. Naming that counterpart in a second column
/// is what stops `sourceIPAddress` / `SrcIP` from being two unrelatable spellings of one fact
/// across two commands of the same tool.
///
/// The mapping is not new knowledge: the output profile already states it, so it is inverted here
/// rather than copied. A missing profile leaves the column NULL instead of stopping a command that
/// otherwise needs no configuration.
fn timeline_column_map() -> HashMap<String, String> {
    let profile_path = LogSource::Aws.profile_path();
    let Ok(profile) = std::fs::read_to_string(profile_path) else {
        log_warn(&format!(
            "Could not open the output profile at '{profile_path}' \
             (run from the directory that contains ./config). \
             The DuckDB TimelineColumn column will be NULL for every row."
        ));
        return HashMap::new();
    };
    profile
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter_map(|(column, source)| {
            // Only the entries fed by a CloudTrail path: `sigma.title` and friends come from the
            // rule that matched, which a metric over raw events has no equivalent of.
            let field = source.trim().trim_matches('\'').strip_prefix('.')?;
            Some((field.to_string(), duckdb_column_name(column.trim())))
        })
        .collect()
}

/// Write the metrics to a DuckDB database as a single `metrics` table plus `suzaku_meta`. Unlike
/// the summary output there is no nesting to preserve here: one row per (field, value) is already
/// the relational shape, so `GROUP BY Field` / `WHERE Value = ...` work directly.
///
/// Values are typed rather than rendered, as everywhere else in the DuckDB output: `First/LastSeen`
/// are `TIMESTAMP`, a missing value is `NULL` rather than the text placeholder, and `Percent` is
/// stored at full precision alongside the `FieldTotal` it was derived from — the CSV's two-decimal
/// rendering makes `sum(Percent)` 99.03 instead of 100, and the denominator needed to recompute it
/// is per field, so it cannot live in `suzaku_meta`. The GeoIP columns are always present, unlike
/// in the CSV output — a column that comes and goes with `-G` turns one query into a binder error
/// on half the files — and `suzaku_meta.geoip_enabled` says whether they are NULL because
/// enrichment was off or because the value was not an IP.
fn write_duckdb_metrics(
    path: &Path,
    records: &[MetricRecord],
    geo_enabled: bool,
    scan: ScanStats,
) -> Result<(), String> {
    let conn = Connection::open(path)
        .map_err(|e| format!("Cannot write to output file {}: {e}", path.display()))?;

    let timeline_columns = timeline_column_map();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(records.len());
    for record in records {
        let mut row = vec![
            record.field.clone(),
            timeline_columns
                .get(&record.field)
                .cloned()
                .unwrap_or_default(),
            record.value.clone(),
            record.count.to_string(),
            record.field_total.to_string(),
            record.exact_percent().to_string(),
            record.first_seen.clone(),
            record.last_seen.clone(),
        ];
        // Unenriched cells are the `-` placeholder here and `NULL` in the table: `geo_cells`
        // already renders an absent value that way, whether it is absent because `-G` was not
        // given or because the value is not an IP address.
        row.extend(record.geo_cells());
        rows.push(row);
    }

    let raw = |c: &str| quote_ident(c);
    let text = |c: &str| nullable(&quote_ident(c));
    let bigint = |c: &str| format!("TRY_CAST({} AS BIGINT)", nullable(&quote_ident(c)));

    let mut ddl = String::from(
        "Field VARCHAR NOT NULL,
         TimelineColumn VARCHAR,
         Value VARCHAR,
         Count BIGINT NOT NULL,
         FieldTotal BIGINT NOT NULL,
         Percent DOUBLE NOT NULL,
         FirstSeen TIMESTAMP,
         LastSeen TIMESTAMP",
    );
    let mut columns: Vec<(&str, String)> = vec![
        ("Field", raw("Field")),
        ("TimelineColumn", text("TimelineColumn")),
        ("Value", text("Value")),
        ("Count", bigint("Count")),
        ("FieldTotal", bigint("FieldTotal")),
        ("Percent", format!("TRY_CAST({} AS DOUBLE)", raw("Percent"))),
        ("FirstSeen", timestamp_expr(&quote_ident("FirstSeen"))),
        ("LastSeen", timestamp_expr(&quote_ident("LastSeen"))),
    ];
    for column in duckdb_out::GEO_COLUMNS {
        ddl.push_str(&format!(",\n         {column} VARCHAR"));
        columns.push((column, text(column)));
    }
    duckdb_out::stage_and_type(&conn, "metrics", &ddl, &columns, &rows)?;

    let mut meta = SuzakuMeta::new("aws-ct-metrics").with_geoip(geo_enabled);
    meta.scanned_files = Some(scan.files as i64);
    meta.scanned_events = Some(scan.events as i64);
    meta.output_rows = Some(records.len() as i64);
    duckdb_out::write_meta(&conn, &meta)?;
    duckdb_out::comment_on_table(
        &conn,
        "metrics",
        "One row per (Field, Value). Percent is that value's share of FieldTotal, the events \
         counted for its Field; events that carried no value for the field are counted too, under \
         Value IS NULL. Rows are an aggregation, so (Field, Value) is unique.",
    )?;
    for column in duckdb_out::GEO_COLUMNS {
        duckdb_out::comment_on_column(&conn, "metrics", column, duckdb_out::GEO_COLUMN_COMMENT)?;
    }
    duckdb_out::checkpoint(&conn)
}

/// Print one table per field to stdout. The first column is named after the field, which is
/// what the hardcoded `EventName` header was always meant to be.
fn print_tables(metrics: &Metrics, records: &[MetricRecord], geo_enabled: bool) {
    for field in &metrics.fields {
        let mut header = vec![field.as_str(), "Count", "Percent", "FirstSeen", "LastSeen"];
        if geo_enabled {
            header.extend(["SrcASN", "SrcCity", "SrcCountry"]);
        }
        let mut table = Table::new();
        table.set_header(
            header
                .iter()
                .map(|s| Cell::new(s).set_alignment(CellAlignment::Center))
                .collect::<Vec<Cell>>(),
        );
        for record in records.iter().filter(|r| &r.field == field) {
            let mut row = vec![
                record.value.clone(),
                record.count.to_string(),
                record.percent_str(),
                record.first_seen.clone(),
                record.last_seen.clone(),
            ];
            if geo_enabled {
                row.extend(record.geo_cells());
            }
            table.add_row(row.iter().map(Cell::new));
        }
        println!("{table}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics_from(fields: &[&str], events: &[&str], include_sts: bool) -> Metrics {
        let fields: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
        let mut metrics = Metrics::new(&fields, include_sts);
        for json in events {
            metrics.add_event(&event_from_json(json).unwrap());
        }
        metrics
    }

    fn records_of<'a>(records: &'a [MetricRecord], field: &str) -> Vec<&'a MetricRecord> {
        records.iter().filter(|r| r.field == field).collect()
    }

    const EVENTS: [&str; 3] = [
        r#"{"eventTime":"2024-01-02T00:00:00Z","eventName":"ListBuckets","sourceIPAddress":"1.1.1.1","awsRegion":"us-east-1","userIdentity":{"accessKeyId":"AKIA1"}}"#,
        r#"{"eventTime":"2024-01-01T00:00:00Z","eventName":"ListBuckets","sourceIPAddress":"2.2.2.2","awsRegion":"us-east-1","userIdentity":{"accessKeyId":"ASIA1"}}"#,
        r#"{"eventTime":"2024-01-03T00:00:00Z","eventName":"DeleteTrail","sourceIPAddress":"1.1.1.1","awsRegion":"ap-northeast-1"}"#,
    ];

    #[test]
    fn counts_every_requested_field_in_one_pass() {
        let metrics = metrics_from(
            &["eventName", "sourceIPAddress", "awsRegion"],
            &EVENTS,
            true,
        );
        let records = build_records(&metrics, None);

        let event_names = records_of(&records, "eventName");
        assert_eq!(event_names.len(), 2);
        assert_eq!(event_names[0].value, "ListBuckets");
        assert_eq!(event_names[0].count, 2);

        let ips = records_of(&records, "sourceIPAddress");
        assert_eq!(ips[0].value, "1.1.1.1");
        assert_eq!(ips[0].count, 2);

        let regions = records_of(&records, "awsRegion");
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].value, "us-east-1");
    }

    // A mis-cased field name (CloudTrail's field is `sourceIPAddress`) tallies every event as
    // "-", which looks like real data, so it has to be reported back to the user.
    #[test]
    fn a_field_no_event_has_is_reported() {
        let metrics = metrics_from(&["sourceIPaddress", "eventName"], &EVENTS, true);
        assert_eq!(metrics.fields_with_no_values(), ["sourceIPaddress"]);

        let metrics = metrics_from(&["sourceIPAddress", "eventName"], &EVENTS, true);
        assert!(metrics.fields_with_no_values().is_empty());
    }

    // A field only *some* events have is normal (`userIdentity.accessKeyId` is absent for
    // console sessions) and must not be reported.
    #[test]
    fn a_partially_present_field_is_not_reported() {
        let metrics = metrics_from(&["userIdentity.accessKeyId"], &EVENTS, true);
        assert!(metrics.fields_with_no_values().is_empty());
    }

    // Each value carries the time window in which THAT value was seen, not the dataset's.
    #[test]
    fn tracks_first_and_last_seen_per_value() {
        let metrics = metrics_from(&["sourceIPAddress"], &EVENTS, true);
        let records = build_records(&metrics, None);
        let top = records_of(&records, "sourceIPAddress")[0];
        assert_eq!(top.value, "1.1.1.1");
        assert_eq!(top.first_seen, "2024-01-02 00:00:00");
        assert_eq!(top.last_seen, "2024-01-03 00:00:00");
    }

    // An event missing the field is counted as "-" so percentages stay a share of all events.
    #[test]
    fn missing_field_is_counted_as_no_value() {
        let metrics = metrics_from(&["userIdentity.accessKeyId"], &EVENTS, true);
        let records = build_records(&metrics, None);
        let rows = records_of(&records, "userIdentity.accessKeyId");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows.iter().map(|r| r.count).sum::<usize>(), EVENTS.len());
        assert!(rows.iter().any(|r| r.value == NO_VALUE && r.count == 1));
        for row in rows {
            assert!((row.percent - 33.33).abs() < 0.01, "{}", row.percent);
        }
    }

    #[test]
    fn sts_keys_are_excluded_unless_requested() {
        let without = metrics_from(&["userIdentity.accessKeyId"], &EVENTS, false);
        let records = build_records(&without, None);
        let rows = records_of(&records, "userIdentity.accessKeyId");
        assert!(!rows.iter().any(|r| r.value.starts_with(STS_KEY_PREFIX)));
        // The dropped event is not counted at all, so the remaining two share 100%.
        assert_eq!(rows.iter().map(|r| r.count).sum::<usize>(), 2);

        let with = metrics_from(&["userIdentity.accessKeyId"], &EVENTS, true);
        let records = build_records(&with, None);
        assert!(
            records_of(&records, "userIdentity.accessKeyId")
                .iter()
                .any(|r| r.value == "ASIA1")
        );
    }

    #[test]
    fn values_are_ordered_by_count_within_each_field() {
        let metrics = metrics_from(&["eventName", "sourceIPAddress"], &EVENTS, true);
        let records = build_records(&metrics, None);

        for field in ["eventName", "sourceIPAddress"] {
            let counts: Vec<usize> = records_of(&records, field)
                .iter()
                .map(|r| r.count)
                .collect();
            assert!(
                counts.windows(2).all(|w| w[0] >= w[1]),
                "{field}: {counts:?}"
            );
        }
        assert!((records_of(&records, "eventName")[0].percent - 66.67).abs() < 0.01);
    }

    #[test]
    fn geoip_enriches_ip_values_and_leaves_others_blank() {
        // Small GeoLite2 test databases shipped under test_files/mmdb/.
        let mut geo = GeoIPSearch::new(Path::new("test_files/mmdb"))
            .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/");
        let events = [
            // 81.2.69.142 is one of the addresses the GeoLite2 test databases resolve.
            r#"{"eventTime":"2024-01-01T00:00:00Z","sourceIPAddress":"81.2.69.142"}"#,
            r#"{"eventTime":"2024-01-01T00:00:00Z","sourceIPAddress":"cloudtrail.amazonaws.com"}"#,
        ];
        let metrics = metrics_from(&["sourceIPAddress"], &events, true);
        let records = build_records(&metrics, Some(&mut geo));

        let service = records
            .iter()
            .find(|r| r.value == "cloudtrail.amazonaws.com")
            .unwrap();
        assert_eq!(service.geo_cells(), [NO_VALUE, NO_VALUE, NO_VALUE]);

        let enriched = records.iter().find(|r| r.value == "81.2.69.142").unwrap();
        assert_eq!(enriched.src_city.as_deref(), Some("London"));
        assert_eq!(enriched.src_country.as_deref(), Some("United Kingdom"));
    }

    // Without -G the GeoIP columns are absent entirely (and omitted from JSON).
    #[test]
    fn no_geoip_columns_without_the_flag() {
        let metrics = metrics_from(&["sourceIPAddress"], &EVENTS, true);
        let records = build_records(&metrics, None);
        assert!(records.iter().all(|r| r.src_asn.is_none()));
        let json = serde_json::to_string(&records[0]).unwrap();
        assert!(!json.contains("src_asn"), "{json}");
    }

    #[test]
    fn csv_header_matches_the_geoip_setting() {
        assert_eq!(csv_header(false).len(), 6);
        assert_eq!(csv_header(true).len(), 9);
        assert_eq!(csv_header(true)[8], "SrcCountry");
    }

    // -----------------------------------------------------------------------
    // DuckDB 出力
    // -----------------------------------------------------------------------

    /// Write `fields` from [`EVENTS`] to a `.duckdb` in a temp dir and hand back the connection.
    /// The `TempDir` comes back too because dropping it deletes the database.
    fn duckdb_of(
        fields: &[&str],
        geo: Option<&mut GeoIPSearch>,
    ) -> (tempfile::TempDir, Connection) {
        let events: Vec<&str> = EVENTS.to_vec();
        duckdb_of_events(fields, &events, geo)
    }

    fn duckdb_of_events(
        fields: &[&str],
        events: &[&str],
        geo: Option<&mut GeoIPSearch>,
    ) -> (tempfile::TempDir, Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("result.duckdb");
        let metrics = metrics_from(fields, events, true);
        let geo_enabled = geo.is_some();
        let records = build_records(&metrics, geo);
        let scan = ScanStats {
            files: 2,
            events: events.len(),
        };
        write_duckdb_metrics(&path, &records, geo_enabled, scan).unwrap();
        let conn = Connection::open(&path).unwrap();
        (tmp, conn)
    }

    fn column_type(conn: &Connection, column: &str) -> Option<String> {
        conn.query_row(
            "SELECT data_type FROM duckdb_columns()
             WHERE table_name = 'metrics' AND column_name = ?",
            [column],
            |r| r.get(0),
        )
        .ok()
    }

    /// P3: the two time columns are temporal, not text that happens to sort.
    #[test]
    fn duckdb_metrics_types_the_seen_timestamps() {
        let (_tmp, conn) = duckdb_of(&["sourceIPAddress"], None);
        assert_eq!(
            column_type(&conn, "FirstSeen").as_deref(),
            Some("TIMESTAMP")
        );
        assert_eq!(column_type(&conn, "LastSeen").as_deref(), Some("TIMESTAMP"));

        let (first, last): (String, String) = conn
            .query_row(
                "SELECT strftime(FirstSeen, '%Y-%m-%d %H:%M:%S'),
                        strftime(LastSeen, '%Y-%m-%d %H:%M:%S')
                 FROM metrics WHERE Value = '1.1.1.1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(first, "2024-01-02 00:00:00");
        assert_eq!(last, "2024-01-03 00:00:00");
    }

    /// P2: an event with no value for the field is NULL, not a placeholder that every consumer
    /// has to remember to exclude. The row still exists — it is part of the denominator.
    #[test]
    fn duckdb_metrics_writes_null_for_a_missing_value() {
        let (_tmp, conn) = duckdb_of(&["userIdentity.accessKeyId"], None);
        let (nulls, placeholders): (i64, i64) = conn
            .query_row(
                "SELECT count(*) FILTER (WHERE Value IS NULL),
                        count(*) FILTER (WHERE Value IN ('', '-'))
                 FROM metrics",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(nulls, 1, "the event without an access key must be NULL");
        assert_eq!(placeholders, 0, "no text placeholder may survive");
    }

    /// P7: the CSV's two-decimal rendering makes `sum(Percent)` 99.03 instead of 100. The DuckDB
    /// output stores the exact share and the denominator it came from.
    #[test]
    fn duckdb_metrics_percent_is_exact_and_carries_its_denominator() {
        // Three events, so a rounded percentage cannot sum back to 100.
        let (_tmp, conn) = duckdb_of(&["userIdentity.accessKeyId"], None);
        let (total, rows, sum): (i64, i64, f64) = conn
            .query_row(
                "SELECT any_value(FieldTotal), count(*), sum(Percent) FROM metrics",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(total, EVENTS.len() as i64);
        assert_eq!(rows, 3);
        assert!((sum - 100.0).abs() < 1e-9, "sum(Percent) = {sum}");

        // And Count is recoverable from Percent, which is what "full precision" has to mean.
        let mismatched: i64 = conn
            .query_row(
                "SELECT count(*) FROM metrics
                 WHERE abs(Count - FieldTotal * Percent / 100.0) > 1e-9",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(mismatched, 0);
    }

    /// P6: `Field` keeps the CloudTrail path the user asked for, and names the timeline column
    /// holding the same fact instead of being renamed to it.
    #[test]
    fn duckdb_metrics_names_the_timeline_column_for_each_field() {
        let (_tmp, conn) = duckdb_of(&["sourceIPAddress", "eventName", "awsRegion"], None);
        let mut stmt = conn
            .prepare("SELECT DISTINCT Field, TimelineColumn FROM metrics ORDER BY Field")
            .unwrap();
        let pairs: Vec<(String, Option<String>)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            pairs,
            vec![
                // The hyphen fix carries over: AWS-Region in the profile, AwsRegion in DuckDB.
                ("awsRegion".to_string(), Some("AwsRegion".to_string())),
                ("eventName".to_string(), Some("EventName".to_string())),
                ("sourceIPAddress".to_string(), Some("SrcIP".to_string())),
            ]
        );
    }

    /// A path into an API-specific container has no timeline column, and saying so with NULL is
    /// the whole reason `Field` was not renamed.
    #[test]
    fn duckdb_metrics_leaves_timeline_column_null_when_there_is_none() {
        let events =
            [r#"{"eventTime":"2024-01-01T00:00:00Z","requestParameters":{"bucketName":"logs"}}"#];
        let (_tmp, conn) = duckdb_of_events(&["requestParameters.bucketName"], &events, None);
        let column: Option<String> = conn
            .query_row("SELECT TimelineColumn FROM metrics", [], |r| r.get(0))
            .unwrap();
        assert_eq!(column, None);
    }

    /// P8: the geo columns are part of the schema whether or not `-G` ran, so one query works
    /// against every file. `suzaku_meta.geoip_enabled` is what says why they are NULL.
    #[test]
    fn duckdb_metrics_always_has_geo_columns() {
        let (_tmp, conn) = duckdb_of(&["sourceIPAddress"], None);
        assert_eq!(column_type(&conn, "SrcCountry").as_deref(), Some("VARCHAR"));
        let (enriched, geoip_enabled): (i64, bool) = conn
            .query_row(
                "SELECT (SELECT count(*) FROM metrics WHERE SrcCountry IS NOT NULL),
                        (SELECT geoip_enabled FROM suzaku_meta)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(enriched, 0);
        assert!(!geoip_enabled);

        let mut geo = GeoIPSearch::new(Path::new("test_files/mmdb"))
            .expect("GeoLite2 test .mmdb files must be present under test_files/mmdb/");
        let events = [
            r#"{"eventTime":"2024-01-01T00:00:00Z","sourceIPAddress":"81.2.69.142"}"#,
            r#"{"eventTime":"2024-01-01T00:00:00Z","sourceIPAddress":"cloudtrail.amazonaws.com"}"#,
        ];
        let (_tmp, conn) = duckdb_of_events(&["sourceIPAddress"], &events, Some(&mut geo));
        let geoip_enabled: bool = conn
            .query_row("SELECT geoip_enabled FROM suzaku_meta", [], |r| r.get(0))
            .unwrap();
        assert!(geoip_enabled);

        let country: Option<String> = conn
            .query_row(
                "SELECT SrcCountry FROM metrics WHERE Value = '81.2.69.142'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(country.as_deref(), Some("United Kingdom"));
        // A value that is not an IP could not be enriched: NULL, not a placeholder.
        let unenriched: Option<String> = conn
            .query_row(
                "SELECT SrcCountry FROM metrics WHERE Value = 'cloudtrail.amazonaws.com'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unenriched, None);
    }

    /// P1: the file states which command, version and coverage produced it.
    #[test]
    fn duckdb_metrics_writes_self_describing_metadata() {
        let (_tmp, conn) = duckdb_of(&["eventName"], None);
        let (version, command, tz, files, events, rows, dupes): (
            i32,
            String,
            String,
            i64,
            i64,
            i64,
            Option<i64>,
        ) = conn
            .query_row(
                "SELECT schema_version, command, timestamp_tz, scanned_files, scanned_events,
                        output_rows, duplicate_rows_removed
                 FROM suzaku_meta",
                [],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(version, duckdb_out::SCHEMA_VERSION);
        assert_eq!(command, "aws-ct-metrics");
        assert_eq!(tz, "UTC");
        assert_eq!(files, 2);
        assert_eq!(events, EVENTS.len() as i64);
        assert_eq!(rows, 2, "ListBuckets and DeleteTrail");
        // The rows are an aggregation, so there is nothing to deduplicate.
        assert_eq!(dupes, None);

        let meta_rows: i64 = conn
            .query_row("SELECT count(*) FROM suzaku_meta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(meta_rows, 1);

        // P9: the grain is readable from the file itself, not only from the docs.
        let comment: Option<String> = conn
            .query_row(
                "SELECT comment FROM duckdb_tables() WHERE table_name = 'metrics'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            comment.unwrap_or_default().contains("(Field, Value)"),
            "the metrics table must document its grain"
        );
    }

    /// P10: a database left with an unreplayed WAL cannot be opened from a read-only mount.
    #[test]
    fn duckdb_metrics_leaves_no_wal_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("result.duckdb");
        let metrics = metrics_from(&["eventName"], &EVENTS, true);
        let records = build_records(&metrics, None);
        write_duckdb_metrics(&path, &records, false, ScanStats::default()).unwrap();

        let wal = tmp.path().join("result.duckdb.wal");
        assert!(
            !wal.exists() || std::fs::metadata(&wal).unwrap().len() == 0,
            "the checkpoint on exit must leave no work in the WAL"
        );
        let cfg = duckdb::Config::default()
            .access_mode(duckdb::AccessMode::ReadOnly)
            .unwrap();
        let ro = Connection::open_with_flags(&path, cfg).unwrap();
        assert_eq!(
            ro.query_row("SELECT count(*) FROM metrics", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
    }

    /// The DuckDB output is a different rendering of the same records, not a different dataset:
    /// unlike the timeline it drops nothing, so the row counts must agree with the CSV's.
    #[test]
    fn duckdb_metrics_row_count_matches_the_records() {
        let metrics = metrics_from(&["eventName", "sourceIPAddress"], &EVENTS, true);
        let expected = build_records(&metrics, None).len() as i64;
        let (_tmp, conn) = duckdb_of(&["eventName", "sourceIPAddress"], None);
        let rows: i64 = conn
            .query_row("SELECT count(*) FROM metrics", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, expected);
    }

    #[test]
    fn equal_counts_sort_by_value_for_stable_output() {
        let events = [
            r#"{"eventTime":"2024-01-01T00:00:00Z","awsRegion":"us-east-1"}"#,
            r#"{"eventTime":"2024-01-01T00:00:00Z","awsRegion":"ap-northeast-1"}"#,
            r#"{"eventTime":"2024-01-01T00:00:00Z","awsRegion":"eu-west-1"}"#,
        ];
        let metrics = metrics_from(&["awsRegion"], &events, true);
        let values: Vec<String> = build_records(&metrics, None)
            .iter()
            .map(|r| r.value.clone())
            .collect();
        assert_eq!(values, ["ap-northeast-1", "eu-west-1", "us-east-1"]);
    }
}
