use crate::core::color::SuzakuColor;
use crate::core::color::SuzakuColor::{Green, Orange, Red, White, Yellow};
use crate::core::util::{get_json_writer, get_writer};
use crate::option::geoip::GeoIPSearch;
use csv::Writer;
use itertools::Itertools;
use serde_json::Value;
use sigma_rust::{Event, Rule, SigmaCorrelationRule, TimestampedEvent};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use termcolor::{BufferWriter, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
pub struct OutputConfig {
    pub no_color: bool,
    pub raw_output: bool,
}

pub struct Writers {
    csv: Option<Writer<Box<dyn Write>>>,
    json: Option<BufWriter<Box<dyn Write>>>,
    jsonl: Option<BufWriter<Box<dyn Write>>>,
    std: Option<BufferWriter>,
}

pub struct OutputContext<'a> {
    pub profile: &'a [(String, String)],
    pub geo: &'a mut Option<GeoIPSearch>,
    pub config: &'a OutputConfig,
    pub writers: Writers,
}

pub fn write_record(event: &Event, json: &Value, rule: &Rule, context: &mut OutputContext) {
    let mut record: Vec<String> = context
        .profile
        .iter()
        .map(|(_k, v)| get_value_from_event(v, event, rule, context.geo))
        .collect();
    write_to_stdout(&mut record, context);
    write_to_csv(&record, context);
    write_to_json(&record, json, Some(event), Some(rule), context);
    write_to_jsonl(&record, json, Some(event), Some(rule), context);
}

pub fn write_correlation_record(
    events: &Vec<&TimestampedEvent>,
    rule: &SigmaCorrelationRule,
    context: &mut OutputContext,
) {
    let mut record: Vec<String> = build_correlation_record(events, rule, context);
    write_to_stdout(&mut record, context);
    write_to_csv(&record, context);
    write_to_json(&record, &Value::Null, None, None, context);
    write_to_jsonl(&record, &Value::Null, None, None, context);
}

fn write_to_stdout(record: &mut [String], context: &mut OutputContext) {
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

fn write_to_csv(record: &[String], context: &mut OutputContext) {
    if let Some(writer) = &mut context.writers.csv {
        writer.write_record(record).unwrap();
    }
}

fn write_to_json(
    record: &[String],
    json: &Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
    context: &mut OutputContext,
) {
    let raw_output = context.config.raw_output;
    // writerの借用を分離してスコープを限定
    if raw_output {
        // raw_outputの場合、contextを分割借用
        let profile = context.profile;
        let geo = &mut context.geo;

        if let Some(writer) = &mut context.writers.json {
            let mut json_record = json.clone();
            let sigma_profile: Vec<(String, String)> = profile
                .iter()
                .filter(|(_, value)| value.starts_with("sigma."))
                .cloned()
                .collect();

            for (k, v) in sigma_profile {
                if event.is_some() && rule.is_some() {
                    let value = get_value_from_event(&v, event.unwrap(), rule.unwrap(), geo);
                    json_record[k] = Value::String(value.to_string());
                }
            }

            if let Ok(json_string) = serde_json::to_string_pretty(&json_record) {
                writer.write_all(json_string.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            }
        }
    } else {
        // formatted outputの場合
        if let Some(writer) = &mut context.writers.json {
            let mut json_record: BTreeMap<String, String> = BTreeMap::new();
            for ((k, _), value) in context.profile.iter().zip(record.iter()) {
                json_record.insert(k.clone(), value.clone());
            }

            if let Ok(json_string) = serde_json::to_string_pretty(&json_record) {
                writer.write_all(json_string.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            }
        }
    }
}

fn write_to_jsonl(
    record: &[String],
    json: &Value,
    event: Option<&Event>,
    rule: Option<&Rule>,
    context: &mut OutputContext,
) {
    let raw_output = context.config.raw_output;

    // writerの借用を分離してスコープを限定
    if raw_output {
        // raw_outputの場合、contextを分割借用
        let profile = context.profile;
        let geo = &mut context.geo;

        if let Some(writer) = &mut context.writers.jsonl {
            let mut json_record = json.clone();
            let sigma_profile: Vec<(String, String)> = profile
                .iter()
                .filter(|(_, value)| value.starts_with("sigma."))
                .cloned()
                .collect();

            for (k, v) in sigma_profile {
                if event.is_some() && rule.is_some() {
                    let value = get_value_from_event(&v, event.unwrap(), rule.unwrap(), geo);
                    json_record[k] = Value::String(value.to_string());
                }
            }

            if let Ok(json_string) = serde_json::to_string(&json_record) {
                writer.write_all(json_string.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            }
        }
    } else {
        // formatted outputの場合
        if let Some(writer) = &mut context.writers.jsonl {
            let mut json_record: BTreeMap<String, String> = BTreeMap::new();
            for ((k, _), value) in context.profile.iter().zip(record.iter()) {
                json_record.insert(k.clone(), value.clone());
            }

            if let Ok(json_string) = serde_json::to_string(&json_record) {
                writer.write_all(json_string.as_bytes()).unwrap();
                writer.write_all(b"\n").unwrap();
            }
        }
    }
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

fn build_correlation_record(
    events: &Vec<&TimestampedEvent>,
    rule: &SigmaCorrelationRule,
    context: &mut OutputContext,
) -> Vec<String> {
    let events: Vec<Event> = events.iter().map(|e| e.event.clone()).collect();
    let profile = &context.profile;
    let mut correlation_map: HashMap<String, String> = HashMap::new();
    for (_, profile_value) in profile.iter() {
        let mut values = HashSet::new();
        for (i, event) in events.iter().enumerate() {
            if profile_value == ".eventTime" && i < events.len() - 1 {
                continue;
            }
            let value = get_value_from_correlation_event(profile_value, event, rule, context.geo);
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

fn get_value_from_correlation_event(
    key: &str,
    event: &Event,
    rule: &SigmaCorrelationRule,
    geo_ip: &mut Option<GeoIPSearch>,
) -> String {
    if let Some(geo) = geo_ip {
        if let Some(ip) = event.get("sourceIPAddress") {
            let ip = ip.value_to_string();
            if let Some(ip) = geo.convert(ip.as_str()) {
                if key == "SrcASN" {
                    return geo.get_asn(ip);
                } else if key == "SrcCity" {
                    return geo.get_city(ip);
                } else if key == "SrcCountry" {
                    return geo.get_country(ip);
                }
            } else {
                return ip;
            }
        }
    }
    if key.starts_with(".") {
        let key = key.strip_prefix(".").unwrap();
        if let Some(value) = event.get(key) {
            if key == "eventTime" {
                value.value_to_string().replace("T", " ").replace("Z", "")
            } else {
                value.value_to_string()
            }
        } else {
            "-".to_string()
        }
    } else if key.starts_with("sigma.") {
        let key = key.replace("sigma.", "");
        if key == "title" {
            rule.title.to_string()
        } else if key == "id"
            && let Some(id) = &rule.id
        {
            id.to_string()
        } else if key == "status"
            && let Some(status) = &rule.status
        {
            format!("{status:?}").to_lowercase()
        } else if key == "author"
            && let Some(author) = &rule.author
        {
            author.to_string()
        } else if key == "description"
            && let Some(desc) = &rule.description
        {
            desc.to_string()
        } else if key == "references"
            && let Some(reference) = &rule.references
        {
            format!("{reference:?}")
        } else if key == "date"
            && let Some(date) = &rule.date
        {
            date.to_string()
        } else if key == "modified"
            && let Some(modified) = &rule.date
        {
            modified.to_string()
        } else if key == "tags"
            && let Some(tag) = &rule.tags
        {
            format!("{tag:?}")
        } else if key == "falsepositives"
            && let Some(fp) = &rule.falsepositives
        {
            format!("{fp:?}")
        } else if key == "level"
            && let Some(level) = &rule.level
        {
            level.to_lowercase()
        } else {
            "-".to_string()
        }
    } else {
        "-".to_string()
    }
}

fn get_value_from_event(
    key: &str,
    event: &Event,
    rule: &Rule,
    geo_ip: &mut Option<GeoIPSearch>,
) -> String {
    if let Some(geo) = geo_ip {
        if let Some(ip) = event.get("sourceIPAddress") {
            let ip = ip.value_to_string();
            if let Some(ip) = geo.convert(ip.as_str()) {
                if key == "SrcASN" {
                    return geo.get_asn(ip);
                } else if key == "SrcCity" {
                    return geo.get_city(ip);
                } else if key == "SrcCountry" {
                    return geo.get_country(ip);
                }
            } else {
                return ip;
            }
        }
    }
    if key.starts_with(".") {
        let key = key.strip_prefix(".").unwrap();
        if let Some(value) = event.get(key) {
            if key == "eventTime" {
                value.value_to_string().replace("T", " ").replace("Z", "")
            } else {
                value.value_to_string()
            }
        } else {
            "-".to_string()
        }
    } else if key.starts_with("sigma.") {
        let key = key.replace("sigma.", "");
        if key == "title" {
            rule.title.to_string()
        } else if key == "id"
            && let Some(id) = &rule.id
        {
            id.to_string()
        } else if key == "status"
            && let Some(status) = &rule.status
        {
            format!("{status:?}").to_lowercase()
        } else if key == "author"
            && let Some(author) = &rule.author
        {
            author.to_string()
        } else if key == "description"
            && let Some(desc) = &rule.description
        {
            desc.to_string()
        } else if key == "references"
            && let Some(reference) = &rule.references
        {
            format!("{reference:?}")
        } else if key == "date"
            && let Some(date) = &rule.date
        {
            date.to_string()
        } else if key == "modified"
            && let Some(modified) = &rule.modified
        {
            modified.to_string()
        } else if key == "tags"
            && let Some(tag) = &rule.tags
        {
            format!("{tag:?}")
        } else if key == "falsepositives"
            && let Some(fp) = &rule.falsepositives
        {
            format!("{fp:?}")
        } else if key == "level"
            && let Some(level) = &rule.level
        {
            format!("{level:?}").to_lowercase()
        } else {
            "-".to_string()
        }
    } else {
        "-".to_string()
    }
}

// 使用例
impl OutputConfig {
    pub fn new(no_color: bool, raw_output: bool) -> Self {
        Self {
            no_color,
            raw_output,
        }
    }
}

impl Writers {
    pub fn new() -> Self {
        Self {
            csv: None,
            json: None,
            jsonl: None,
            std: None,
        }
    }

    pub fn with_csv(mut self, writer: Writer<Box<dyn Write>>) -> Self {
        self.csv = Some(writer);
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
    ) -> Self {
        Self {
            profile,
            geo,
            config,
            writers,
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

#[derive(Debug)]
pub enum OutputType {
    Csv,
    Json,
    Jsonl,
    CsvAndJson,
    CsvAndJsonl,
}

impl OutputType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(OutputType::Csv),
            2 => Some(OutputType::Json),
            3 => Some(OutputType::Jsonl),
            4 => Some(OutputType::CsvAndJson),
            5 => Some(OutputType::CsvAndJsonl),
            _ => None,
        }
    }
}
pub fn init_writers(output_path: Option<&PathBuf>, output_type: u8) -> (Writers, Vec<PathBuf>) {
    let mut output_pathes = vec![];
    let mut writers = Writers::new();

    if let Some(output_path) = output_path {
        let output_type = OutputType::from_u8(output_type).unwrap_or(OutputType::Csv);

        match output_type {
            OutputType::Csv | OutputType::CsvAndJson | OutputType::CsvAndJsonl => {
                let mut csv_path = output_path.clone();
                if csv_path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
                    csv_path.set_extension("csv");
                }
                output_pathes.push(csv_path.clone());
                writers = writers.with_csv(get_writer(&Some(csv_path)));
            }
            _ => {}
        }

        match output_type {
            OutputType::Json | OutputType::CsvAndJson => {
                let mut json_path = output_path.clone();
                if json_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    json_path.set_extension("json");
                }
                output_pathes.push(json_path.clone());
                writers = writers.with_json(get_json_writer(&Some(json_path)));
            }
            OutputType::Jsonl | OutputType::CsvAndJsonl => {
                let mut jsonl_path = output_path.clone();
                if jsonl_path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                    jsonl_path.set_extension("jsonl");
                }
                output_pathes.push(jsonl_path.clone());
                writers = writers.with_jsonl(get_json_writer(&Some(jsonl_path)));
            }
            _ => {}
        }
    } else {
        let disp_wtr = BufferWriter::stdout(ColorChoice::Always);
        let mut disp_wtr_buf = disp_wtr.buffer();
        disp_wtr_buf.set_color(ColorSpec::new().set_fg(None)).ok();
        writers = writers.with_stdout(disp_wtr);
    }

    (writers, output_pathes)
}
