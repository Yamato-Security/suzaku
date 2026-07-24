use crate::core::color::SuzakuColor;
use crate::core::color::SuzakuColor::{Green, Orange, Red, White, Yellow};
use crate::core::util::{get_json_writer, get_writer, sanitize_csv_field};
use crate::option::cli::OutputFormat;
use crate::option::geoip::GeoIPSearch;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use csv::Writer;
use duckdb::{Connection, params_from_iter};
use itertools::Itertools;
use serde_json::Value;
use sigma_rust::{Event, Rule, SigmaCorrelationRule, TimestampedEvent};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufWriter, Write};
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
    let utc: Option<DateTime<Utc>> = DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
                .ok()
                .map(|ndt| Utc.from_utc_datetime(&ndt))
        });
    match utc {
        Some(u) => u
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S%:z")
            .to_string(),
        None => value.replace("T", " ").replace("Z", ""),
    }
}

pub struct Writers {
    csv: Option<Writer<Box<dyn Write>>>,
    json: Option<BufWriter<Box<dyn Write>>>,
    jsonl: Option<BufWriter<Box<dyn Write>>>,
    duckdb: Option<DuckDbSink>,
    std: Option<BufferWriter>,
}

/// DuckDB output sink: a `.duckdb` database file with a single `timeline` table whose columns
/// are the output-profile headers (all `VARCHAR`, since every Suzaku value is a string). Unlike
/// the CSV/JSON sinks this is not a byte stream, so it lives outside the `dyn Write` writers.
struct DuckDbSink {
    conn: Connection,
    insert_sql: String,
}

impl DuckDbSink {
    fn new(path: &Path, columns: &[String]) -> Result<Self, String> {
        let conn = Connection::open(path)
            .map_err(|e| format!("Cannot write to output file {}: {e}", path.display()))?;
        let cols_ddl = columns
            .iter()
            .map(|c| format!("\"{}\" VARCHAR", c.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(", ");
        // CREATE OR REPLACE so re-running over an existing .duckdb file overwrites the table,
        // matching the truncate-on-write behavior of the CSV/JSON file writers.
        conn.execute_batch(&format!("CREATE OR REPLACE TABLE timeline ({cols_ddl});"))
            .map_err(|e| format!("Cannot create DuckDB table in {}: {e}", path.display()))?;
        let placeholders = vec!["?"; columns.len()].join(", ");
        Ok(Self {
            conn,
            insert_sql: format!("INSERT INTO timeline VALUES ({placeholders})"),
        })
    }

    fn append_row(&self, record: &[String]) {
        // Errors on a single row shouldn't abort the whole scan; drop the row (rare — schema is
        // fixed and every value is a string).
        let _ = self
            .conn
            .execute(&self.insert_sql, params_from_iter(record.iter()));
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
            let sigma_profile: Vec<(String, String)> = profile
                .iter()
                .filter(|(_, value)| value.starts_with("sigma."))
                .cloned()
                .collect();

            for (k, v) in sigma_profile {
                if let (Some(event), rule) = (event, rule) {
                    let value =
                        get_value_from_event(&v, event, rule, geo, localtime, src_ip_spec(profile));
                    json_record[k] = Value::String(value.to_string());
                }
            }

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
    if let Some(sink) = &context.writers.duckdb {
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
            let sigma_profile: Vec<(String, String)> = profile
                .iter()
                .filter(|(_, value)| value.starts_with("sigma."))
                .cloned()
                .collect();

            for (k, v) in sigma_profile {
                if let (Some(event), rule) = (event, rule) {
                    let value =
                        get_value_from_event(&v, event, rule, geo, localtime, src_ip_spec(profile));
                    json_record[k] = Value::String(value.to_string());
                }
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
    // `.callerIpAddress|.ClientIP|...` for Azure/M365) — NOT a hardcoded field name
    // — so Azure/M365 enrich just like AWS. A missing GeoIP DB, a missing source IP,
    // or a non-IP value (e.g. a service principal like "cloudtrail.amazonaws.com")
    // yields the "-" placeholder for those columns only — it must never overwrite
    // an unrelated column's value.
    if matches!(key, "SrcASN" | "SrcCity" | "SrcCountry") {
        if let Some(geo) = geo_ip {
            let ip_value = src_ip
                .split('|')
                .map(|k| k.trim_matches('.').trim())
                .filter(|k| !k.is_empty())
                .find_map(|k| event.get(k));
            if let Some(ip) = ip_value {
                let ip = ip.value_to_string();
                if let Some(ip) = geo.convert(ip.as_str()) {
                    return match key {
                        "SrcASN" => geo.get_asn(ip),
                        "SrcCity" => geo.get_city(ip),
                        _ => geo.get_country(ip),
                    };
                }
            }
        }
        return "-".to_string();
    }
    // イベントフィールド処理（共通）
    if key.starts_with(".") {
        let key_without_prefix = key.trim_start_matches('.').trim();
        let keys: Vec<&str> = key_without_prefix.split('|').collect();
        for k in keys {
            let k_trimmed = k.trim_matches('.').trim();
            if let Some(value) = event.get(k_trimmed) {
                return if k_trimmed.contains("eventTime")
                    || k_trimmed.contains("time")
                    || k_trimmed.contains("eventTimestamp")
                    || k_trimmed.contains("CreationTime")
                {
                    format_timestamp(&value.value_to_string(), localtime)
                } else {
                    value.value_to_string()
                };
            }
        }
        "-".to_string()
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
/// Single source of truth for both opening the writers and the `--clobber` preflight.
fn resolve_output_targets(
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
/// to several files at once. `profile` supplies the column names for the DuckDB table. With no
/// `output_path`, results go to the stdout table and `output_types` is ignored.
pub fn init_writers(
    output_path: Option<&PathBuf>,
    output_types: &[OutputFormat],
    profile: &[(String, String)],
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
                    writers.with_duckdb(DuckDbSink::new(&path, &columns)?)
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
    #[test]
    fn geoip_resolves_source_ip_via_profile_spec() {
        use crate::option::geoip::GeoIPSearch;
        use sigma_rust::{event_from_json, rule_from_yaml};
        use std::path::Path;

        let rule = rule_from_yaml(
            "title: t\nlogsource:\n    category: test\ndetection:\n    selection:\n        eventName: E\n    condition: selection\n",
        )
        .unwrap();
        let ip = "8.8.8.8";
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
        let aws_country = get_value_from_event(
            "SrcCountry",
            &aws_event,
            Some(&rule),
            &mut geo,
            false,
            ".sourceIPAddress",
        );
        let azure_country = get_value_from_event(
            "SrcCountry",
            &azure_event,
            Some(&rule),
            &mut geo,
            false,
            ".callerIpAddress",
        );
        // Same IP + same DB must enrich identically regardless of the field name.
        assert_eq!(aws_country, azure_country);
        // When the DB actually resolves the IP, the Azure field must enrich (not "-").
        if aws_country != "-" {
            assert_ne!(azure_country, "-");
        }
    }

    #[test]
    fn duckdb_sink_creates_named_table_and_appends_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.duckdb");
        // A column name with a hyphen must be quoted correctly in the DDL.
        let cols = vec![
            "Timestamp".to_string(),
            "RuleTitle".to_string(),
            "AWS-Region".to_string(),
        ];
        let sink = DuckDbSink::new(&path, &cols).unwrap();
        sink.append_row(&[
            "2024-01-01".to_string(),
            "Rule A".to_string(),
            "us-east-1".to_string(),
        ]);
        sink.append_row(&[
            "2024-01-02".to_string(),
            "Rule B".to_string(),
            "ap-northeast-1".to_string(),
        ]);
        drop(sink); // close the writer connection before re-opening for verification

        let conn = Connection::open(&path).unwrap();
        let rows: i64 = conn
            .query_row("SELECT count(*) FROM timeline", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2);
        let title: String = conn
            .query_row(
                "SELECT RuleTitle FROM timeline WHERE \"AWS-Region\" = 'ap-northeast-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title, "Rule B");
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
        let sink = DuckDbSink::new(&path, &["Timestamp".to_string()]).unwrap();
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
