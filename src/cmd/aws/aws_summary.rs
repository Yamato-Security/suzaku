use crate::core::color::SuzakuColor::Red;
use crate::core::log_source::LogSource;
use crate::core::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::core::util::{fatal_error, get_writer, output_path_info, p, sanitize_csv_field};
use crate::option::cli::InputOption;
use crate::option::geoip::GeoIPSearch;
use crate::option::timefiler::filter_by_time;
use csv::ReaderBuilder;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use serde::Serialize;
use serde_json::Value;
use sigma_rust::{Event, event_from_json};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use termcolor::Color;

// ---------------------------------------------------------------------------
// JSON 出力用データ構造
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, PartialEq)]
pub struct ApiEntry {
    pub api: String,
    pub description: String,
    pub count: usize,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CountEntry {
    pub value: String,
    pub count: usize,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Serialize, Debug)]
pub struct SummaryJsonRecord {
    pub user_arn: String,
    pub num_of_events: usize,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub abused_apis_success: Vec<ApiEntry>,
    pub abused_apis_failed: Vec<ApiEntry>,
    pub other_apis_success: Vec<ApiEntry>,
    pub other_apis_failed: Vec<ApiEntry>,
    pub aws_regions: Vec<CountEntry>,
    pub src_ips: Vec<CountEntry>,
    pub user_types: String,
    pub user_access_key_ids: Vec<CountEntry>,
    pub user_agents: Vec<CountEntry>,
}

// ---------------------------------------------------------------------------
// 集計用内部構造体
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CTSummary {
    num_of_events: usize,
    first_timestamp: String,
    last_timestamp: String,
    abused_api_success: HashMap<String, (usize, String, String)>,
    abused_api_failed: HashMap<String, (usize, String, String)>,
    other_api_success: HashMap<String, (usize, String, String)>,
    other_api_failed: HashMap<String, (usize, String, String)>,
    aws_regions: HashMap<String, (usize, String, String)>,
    src_ips: HashMap<String, (usize, String, String)>,
    user_types: String,
    access_key_ids: HashMap<String, (usize, String, String)>,
    user_agents: HashMap<String, (usize, String, String)>,
}

impl CTSummary {
    #[allow(clippy::too_many_arguments)]
    fn add_event(
        &mut self,
        event_time: String,
        aws_region: String,
        source_ip: String,
        user_type: String,
        access_key_id: String,
        user_agent: String,
        abused_api_success: String,
        abused_api_failed: String,
        other_api_success: String,
        other_api_failed: String,
    ) {
        self.num_of_events += 1;

        if self.first_timestamp.is_empty() || event_time < self.first_timestamp {
            self.first_timestamp = event_time.clone();
        }
        if self.last_timestamp.is_empty() || event_time > self.last_timestamp {
            self.last_timestamp = event_time.clone();
        }

        let entry = self.aws_regions.entry(aws_region.clone()).or_insert((
            0,
            self.first_timestamp.clone(),
            self.last_timestamp.clone(),
        ));
        entry.0 += 1;
        let entry = self.src_ips.entry(source_ip.clone()).or_insert((
            0,
            self.first_timestamp.clone(),
            self.last_timestamp.clone(),
        ));
        entry.0 += 1;
        self.user_types = user_type.clone();
        let entry = self.access_key_ids.entry(access_key_id.clone()).or_insert((
            0,
            self.first_timestamp.clone(),
            self.last_timestamp.clone(),
        ));
        entry.0 += 1;
        let entry = self.user_agents.entry(user_agent.clone()).or_insert((
            0,
            self.first_timestamp.clone(),
            self.last_timestamp.clone(),
        ));
        entry.0 += 1;

        if !abused_api_success.is_empty() {
            let entry = self
                .abused_api_success
                .entry(abused_api_success.clone())
                .or_insert((0, self.first_timestamp.clone(), self.last_timestamp.clone()));
            entry.0 += 1;
        }

        if !abused_api_failed.is_empty() {
            let entry = self
                .abused_api_failed
                .entry(abused_api_failed.clone())
                .or_insert((0, self.first_timestamp.clone(), self.last_timestamp.clone()));
            entry.0 += 1;
        }
        if !other_api_success.is_empty() {
            let entry = self
                .other_api_success
                .entry(other_api_success.clone())
                .or_insert((0, self.first_timestamp.clone(), self.last_timestamp.clone()));
            entry.0 += 1;
        }

        if !other_api_failed.is_empty() {
            let entry = self
                .other_api_failed
                .entry(other_api_failed.clone())
                .or_insert((0, self.first_timestamp.clone(), self.last_timestamp.clone()));
            entry.0 += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// JSON ビルド用ヘルパー関数
// ---------------------------------------------------------------------------

/// `"EventName (source) - Description"` または `"EventName (source)"` 形式のキーを
/// `ApiEntry` に変換して件数降順で返す。
fn map_to_api_entries(
    map: &HashMap<String, (usize, String, String)>,
    hide_descriptions: bool,
) -> Vec<ApiEntry> {
    map.iter()
        .sorted_by(|a, b| b.1.0.cmp(&a.1.0))
        .map(|(key, (count, first, last))| {
            let (api, description) = if let Some(pos) = key.find(" - ") {
                let api = key[..pos].to_string();
                let desc = if hide_descriptions {
                    "".to_string()
                } else {
                    key[pos + 3..].to_string()
                };
                (api, desc)
            } else {
                (key.clone(), "".to_string())
            };
            ApiEntry {
                api,
                description,
                count: *count,
                first_seen: first.replace('T', " ").replace('Z', ""),
                last_seen: last.replace('T', " ").replace('Z', ""),
            }
        })
        .collect()
}

/// `HashMap<String, (usize, String, String)>` を件数降順の `CountEntry` リストに変換する。
fn map_to_count_entries(map: &HashMap<String, (usize, String, String)>) -> Vec<CountEntry> {
    map.iter()
        .sorted_by(|a, b| b.1.0.cmp(&a.1.0))
        .map(|(key, (count, first, last))| CountEntry {
            value: key.clone(),
            count: *count,
            first_seen: first.replace('T', " ").replace('Z', ""),
            last_seen: last.replace('T', " ").replace('Z', ""),
        })
        .collect()
}

/// `user_data` を JSON レコードのリスト（件数降順）に変換する。
fn build_json_records(
    user_data: &HashMap<String, CTSummary>,
    hide_descriptions: bool,
) -> Vec<SummaryJsonRecord> {
    let mut records: Vec<SummaryJsonRecord> = user_data
        .iter()
        .map(|(arn, summary)| SummaryJsonRecord {
            user_arn: arn.clone(),
            num_of_events: summary.num_of_events,
            first_timestamp: summary.first_timestamp.replace('T', " ").replace('Z', ""),
            last_timestamp: summary.last_timestamp.replace('T', " ").replace('Z', ""),
            abused_apis_success: map_to_api_entries(&summary.abused_api_success, hide_descriptions),
            abused_apis_failed: map_to_api_entries(&summary.abused_api_failed, hide_descriptions),
            other_apis_success: map_to_api_entries(&summary.other_api_success, false),
            other_apis_failed: map_to_api_entries(&summary.other_api_failed, false),
            aws_regions: map_to_count_entries(&summary.aws_regions),
            src_ips: map_to_count_entries(&summary.src_ips),
            user_types: summary.user_types.clone(),
            user_access_key_ids: map_to_count_entries(&summary.access_key_ids),
            user_agents: map_to_count_entries(&summary.user_agents),
        })
        .collect();

    records.sort_by_key(|r| std::cmp::Reverse(r.num_of_events));
    records
}

// ---------------------------------------------------------------------------
// パブリックエントリポイント
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn aws_summary(
    input_opt: &InputOption,
    output: &Path,
    no_color: bool,
    include_sts: &bool,
    hide_descriptions: &bool,
    geo_ip: &Option<PathBuf>,
    output_type: u8,
    clobber: bool,
) {
    let directory = &input_opt.directory;
    let file = &input_opt.filepath;
    let mut geo_search = None;
    if let Some(path) = geo_ip.as_ref() {
        let res = GeoIPSearch::new(path);
        if let Ok(geo) = res {
            geo_search = Some(geo);
        } else {
            p(
                Red.rdg(no_color),
                "Could not find the appropriate MaxMind GeoIP .mmdb database files.\n",
                true,
            );
            return;
        }
    }
    let abused_aws_api_calls = read_abused_aws_api_calls("rules/config/abused_aws_api_calls.csv");
    let mut user_data: HashMap<String, CTSummary> = HashMap::new();
    let mut single_summary_func = |json_value: &Value| {
        if !filter_by_time(&input_opt.time_opt, json_value, "eventTime") {
            return;
        }
        let event: Event = match event_from_json(json_value.to_string().as_str()) {
            Ok(event) => event,
            Err(_) => return,
        };
        let user_identity_arn = match event.get("userIdentity.arn") {
            Some(arn) => arn.value_to_string(),
            None => return,
        };
        let event_time = match event.get("eventTime") {
            Some(time) => time.value_to_string(),
            None => "-".to_string(),
        };
        let aws_region = match event.get("awsRegion") {
            Some(region) => region.value_to_string(),
            None => "-".to_string(),
        };

        let error_code = match event.get("errorCode") {
            Some(code) => code.value_to_string(),
            None => "-".to_string(),
        };
        let source_ipaddress = match event.get("sourceIPAddress") {
            Some(ip) => {
                let mut ip_str = ip.value_to_string();
                if let Some(geo) = geo_search.as_mut()
                    && let Some(ip) = geo.convert(ip_str.as_str())
                {
                    let asn = geo.get_asn(ip);
                    let country = geo.get_country(ip);
                    let city = geo.get_city(ip);
                    ip_str = format!("{ip_str} ({asn}, {city}, {country})");
                }
                ip_str
            }
            None => "-".to_string(),
        };
        let user_identity_type = match event.get("userIdentity.type") {
            Some(user_type) => user_type.value_to_string(),
            None => "-".to_string(),
        };
        let user_identity_access_key_id = match event.get("userIdentity.accessKeyId") {
            Some(access_key_id) => {
                let key = access_key_id.value_to_string();
                if !*include_sts && key.starts_with("ASIA") {
                    return;
                }
                key
            }
            None => "-".to_string(),
        };
        let user_agent = match event.get("userAgent") {
            Some(agent) => agent.value_to_string(),
            None => "-".to_string(),
        };

        let event_name = match event.get("eventName") {
            Some(name) => name.value_to_string(),
            None => "-".to_string(),
        };
        let event_source = match event.get("eventSource") {
            Some(source) => source.value_to_string(),
            None => "-".to_string(),
        };
        let mut abused_api_success = "".to_string();
        if let Some(desc) = abused_aws_api_calls.get(&event_name)
            && error_code != "AccessDenied"
        {
            abused_api_success = format!("{event_name} ({event_source}) - {desc}");
        };

        let mut abused_api_failed = "".to_string();
        if let Some(desc) = abused_aws_api_calls.get(&event_name)
            && error_code == "AccessDenied"
        {
            abused_api_failed = format!("{event_name} ({event_source}) - {desc}");
        };

        let mut other_api_success = "".to_string();
        if !abused_aws_api_calls.contains_key(&event_name) && error_code != "AccessDenied" {
            other_api_success = format!("{event_name} ({event_source})");
        };

        let mut other_api_failed = "".to_string();
        if !abused_aws_api_calls.contains_key(&event_name) && error_code == "AccessDenied" {
            other_api_failed = format!("{event_name} ({event_source})");
        };

        let entry = user_data.entry(user_identity_arn.clone()).or_default();
        entry.add_event(
            event_time,
            aws_region,
            source_ipaddress,
            user_identity_type,
            user_identity_access_key_id,
            user_agent,
            abused_api_success,
            abused_api_failed,
            other_api_success,
            other_api_failed,
        );
    };
    let mut summary_func = |json_values: &[Value]| {
        for json_value in json_values {
            single_summary_func(json_value);
        }
    };
    let abused_aws_api_values: Vec<String> = abused_aws_api_calls.values().cloned().collect();
    if let Some(d) = directory {
        if let Err(e) = process_events_from_dir(
            summary_func,
            d,
            true,
            no_color,
            &LogSource::Aws,
            &input_opt.file_date_opt,
        ) {
            p(
                Red.rdg(no_color),
                &format!("Failed to scan directory {}: {e}", d.display()),
                true,
            );
        }
        output_summary(
            &user_data,
            output,
            no_color,
            hide_descriptions,
            abused_aws_api_values,
            output_type,
            clobber,
        );
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents, &LogSource::Aws);
        if let Ok(events) = events {
            summary_func(&events);
            output_summary(
                &user_data,
                output,
                no_color,
                hide_descriptions,
                abused_aws_api_values,
                output_type,
                clobber,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 出力処理
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn output_summary(
    user_data: &HashMap<String, CTSummary>,
    output: &Path,
    no_color: bool,
    hide_descriptions: &bool,
    abused_aws_api_disc: Vec<String>,
    output_type: u8,
    clobber: bool,
) {
    if user_data.is_empty() {
        p(Some(Color::Rgb(255, 0, 0)), "No events found.", true);
        return;
    }

    let output_csv = matches!(output_type, 1 | 4 | 5);
    let output_json = matches!(output_type, 2 | 4);
    let output_jsonl = matches!(output_type, 3 | 5);

    let csv_path = output_csv.then(|| {
        let mut path = output.to_path_buf();
        path.set_extension("csv");
        path
    });
    let json_path = output_json.then(|| {
        let mut path = output.to_path_buf();
        path.set_extension("json");
        path
    });
    let jsonl_path = output_jsonl.then(|| {
        let mut path = output.to_path_buf();
        path.set_extension("jsonl");
        path
    });

    if !clobber
        && let Some(path) = [csv_path.as_ref(), json_path.as_ref(), jsonl_path.as_ref()]
            .into_iter()
            .flatten()
            .find(|path| path.exists())
    {
        p(
            Some(Color::Rgb(255, 0, 0)),
            &format!(
                "The file {} already exists. Use --clobber to overwrite.",
                path.display()
            ),
            true,
        );
        return;
    }

    let mut output_paths: Vec<PathBuf> = Vec::new();

    // --- CSV 出力 ---
    if let Some(csv_path) = csv_path {
        let fmt_key_total = |msg: &str, map: &HashMap<String, (usize, String, String)>| -> String {
            let total: usize = map.keys().len();
            let total = total.to_formatted_string(&Locale::en);
            let mut result = vec![format!("{}: {}", msg, total)];
            result.extend(map.iter().sorted_by(|a, b| b.1.cmp(a.1)).map(|(k, v)| {
                format!(
                    "{} - {} ({} ~ {})",
                    v.0.to_formatted_string(&Locale::en),
                    k,
                    v.1.replace('Z', "").replace('T', " "),
                    v.2.replace('Z', "").replace('T', " ")
                )
            }));
            result.join("\n")
        };

        let fmt_val_total = |msg: &str, map: &HashMap<String, (usize, String, String)>| -> String {
            let total: usize = map.values().map(|v| v.0).sum();
            let total = total.to_formatted_string(&Locale::en);
            format!("| {msg} {total}")
        };

        let mut csv_wtr =
            get_writer(&Some(csv_path.clone())).unwrap_or_else(|e| fatal_error(no_color, &e));
        let csv_header = vec![
            "UserARN",
            "NumOfEvents",
            "FirstTimestamp",
            "LastTimestamp",
            "AbusedAPIs-Success",
            "AbusedAPIs-Failed",
            "OtherAPIs-Success",
            "OtherAPIs-Failed",
            "AWS-Regions",
            "SrcIPs",
            "UserTypes",
            "UserAccessKeyIDs",
            "UserAgents",
        ];
        csv_wtr.write_record(&csv_header).unwrap();

        let mut sorted_user_data: Vec<_> = user_data.iter().collect();
        sorted_user_data.sort_by_key(|b| std::cmp::Reverse(b.1.num_of_events));

        for (user_arn, summary) in sorted_user_data.iter() {
            let num_of_events = summary.num_of_events.to_formatted_string(&Locale::en);
            let first_timestamp = summary
                .first_timestamp
                .clone()
                .replace("T", " ")
                .replace("Z", "");
            let last_timestamp = summary
                .last_timestamp
                .clone()
                .replace("T", " ")
                .replace("Z", "");
            let aws_regions = fmt_key_total("Total regions", &summary.aws_regions);
            let src_ips = fmt_key_total("Total source IPs", &summary.src_ips);
            let user_types = &summary.user_types;
            let access_key_ids = fmt_key_total("Total access key IDs", &summary.access_key_ids);
            let user_agents = fmt_key_total("Total user agents", &summary.user_agents);

            let mut abused_suc = fmt_key_total("Unique APIs", &summary.abused_api_success);
            if let Some(pos) = abused_suc.find('\n') {
                let abused_suc_val = fmt_val_total("Total APIs", &summary.abused_api_success);
                abused_suc.insert_str(pos, &format!(" {abused_suc_val}"));
            }
            let mut abused_fai = fmt_key_total("Unique APIs", &summary.abused_api_failed);
            if let Some(pos) = abused_fai.find('\n') {
                let abused_fai_val = fmt_val_total("Total APIs", &summary.abused_api_failed);
                abused_fai.insert_str(pos, &format!(" {abused_fai_val}"));
            }
            let mut other_suc = fmt_key_total("Unique APIs", &summary.other_api_success);
            if let Some(pos) = other_suc.find('\n') {
                let other_suc_val = fmt_val_total("Total APIs", &summary.other_api_success);
                other_suc.insert_str(pos, &format!(" {other_suc_val}"));
            }
            let mut other_fai = fmt_key_total("Unique APIs", &summary.other_api_failed);
            if let Some(pos) = other_fai.find('\n') {
                let other_fai_val = fmt_val_total("Total APIs", &summary.other_api_failed);
                other_fai.insert_str(pos, &format!(" {other_fai_val}"));
            }

            if *hide_descriptions {
                abused_aws_api_disc.iter().for_each(|disc| {
                    abused_suc = abused_suc.replace(disc, "");
                    abused_fai = abused_fai.replace(disc, "");
                });
                abused_suc = abused_suc.replace("-  (2", "(2");
                abused_fai = abused_fai.replace("-  (2", "(2");
            }

            let sanitized = vec![
                sanitize_csv_field(user_arn),
                sanitize_csv_field(&num_of_events),
                sanitize_csv_field(&first_timestamp),
                sanitize_csv_field(&last_timestamp),
                sanitize_csv_field(&abused_suc),
                sanitize_csv_field(&abused_fai),
                sanitize_csv_field(&other_suc),
                sanitize_csv_field(&other_fai),
                sanitize_csv_field(&aws_regions),
                sanitize_csv_field(&src_ips),
                sanitize_csv_field(user_types),
                sanitize_csv_field(&access_key_ids),
                sanitize_csv_field(&user_agents),
            ];
            csv_wtr.write_record(&sanitized).unwrap();
        }
        csv_wtr.flush().unwrap();
        output_paths.push(csv_path);
    }

    // --- JSON 出力 ---
    if let Some(json_path) = json_path {
        let records = build_json_records(user_data, *hide_descriptions);
        let file = File::create(&json_path).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &records).unwrap();
        writer.flush().unwrap();
        output_paths.push(json_path);
    }

    // --- JSONL 出力 ---
    if let Some(jsonl_path) = jsonl_path {
        let records = build_json_records(user_data, *hide_descriptions);
        let file = File::create(&jsonl_path).unwrap();
        let mut writer = BufWriter::new(file);
        for record in &records {
            let line = serde_json::to_string(record).unwrap();
            writeln!(writer, "{}", line).unwrap();
        }
        writer.flush().unwrap();
        output_paths.push(jsonl_path);
    }

    output_path_info(no_color, output_paths.as_slice(), true);
}

// ---------------------------------------------------------------------------
// abused_aws_api_calls.csv 読み込み
// ---------------------------------------------------------------------------

fn read_abused_aws_api_calls(file_path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let file = File::open(file_path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
            for record in csv_reader.records().flatten() {
                if let Some(event_name) = record.get(0)
                    && let Some(description) = record.get(1)
                {
                    map.insert(event_name.to_string(), description.to_string());
                }
            }
            map
        }
        Err(_) => {
            p(
                Some(Color::Rgb(255, 0, 0)),
                "Failed to open the abused AWS API calls file.",
                true,
            );
            HashMap::new()
        }
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// テスト用の CTSummary を生成するヘルパー
    fn make_test_summary() -> CTSummary {
        let mut s = CTSummary::default();
        s.add_event(
            "2024-01-01T00:00:00Z".to_string(),
            "us-east-1".to_string(),
            "1.2.3.4".to_string(),
            "IAMUser".to_string(),
            "AKIAIOSFODNN7EXAMPLE".to_string(),
            "aws-cli/2.0".to_string(),
            "ListBuckets (s3.amazonaws.com) - List all S3 buckets".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        s.add_event(
            "2024-01-02T00:00:00Z".to_string(),
            "us-west-2".to_string(),
            "5.6.7.8".to_string(),
            "IAMUser".to_string(),
            "AKIAIOSFODNN7EXAMPLE".to_string(),
            "aws-sdk/1.0".to_string(),
            "".to_string(),
            "".to_string(),
            "GetObject (s3.amazonaws.com)".to_string(),
            "".to_string(),
        );
        s
    }

    // -----------------------------------------------------------------------
    // CTSummary::add_event のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_ct_summary_num_of_events() {
        let summary = make_test_summary();
        assert_eq!(summary.num_of_events, 2);
    }

    #[test]
    fn test_ct_summary_timestamps() {
        let summary = make_test_summary();
        assert_eq!(summary.first_timestamp, "2024-01-01T00:00:00Z");
        assert_eq!(summary.last_timestamp, "2024-01-02T00:00:00Z");
    }

    #[test]
    fn test_ct_summary_regions() {
        let summary = make_test_summary();
        assert_eq!(summary.aws_regions.len(), 2);
        assert!(summary.aws_regions.contains_key("us-east-1"));
        assert!(summary.aws_regions.contains_key("us-west-2"));
    }

    #[test]
    fn test_ct_summary_abused_api_success() {
        let summary = make_test_summary();
        assert_eq!(summary.abused_api_success.len(), 1);
        assert!(
            summary
                .abused_api_success
                .contains_key("ListBuckets (s3.amazonaws.com) - List all S3 buckets")
        );
    }

    #[test]
    fn test_ct_summary_other_api_success() {
        let summary = make_test_summary();
        assert_eq!(summary.other_api_success.len(), 1);
        assert!(
            summary
                .other_api_success
                .contains_key("GetObject (s3.amazonaws.com)")
        );
    }

    // -----------------------------------------------------------------------
    // map_to_api_entries のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_map_to_api_entries_with_description() {
        let mut map = HashMap::new();
        map.insert(
            "ListBuckets (s3.amazonaws.com) - List all S3 buckets".to_string(),
            (
                3usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-02T00:00:00Z".to_string(),
            ),
        );
        let entries = map_to_api_entries(&map, false);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].api, "ListBuckets (s3.amazonaws.com)");
        assert_eq!(entries[0].description, "List all S3 buckets");
        assert_eq!(entries[0].count, 3);
        assert_eq!(entries[0].first_seen, "2024-01-01 00:00:00");
        assert_eq!(entries[0].last_seen, "2024-01-02 00:00:00");
    }

    #[test]
    fn test_map_to_api_entries_hide_description() {
        let mut map = HashMap::new();
        map.insert(
            "ListBuckets (s3.amazonaws.com) - List all S3 buckets".to_string(),
            (
                1usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        let entries = map_to_api_entries(&map, true);
        assert_eq!(entries[0].api, "ListBuckets (s3.amazonaws.com)");
        assert_eq!(entries[0].description, "");
    }

    #[test]
    fn test_map_to_api_entries_no_description() {
        let mut map = HashMap::new();
        map.insert(
            "GetObject (s3.amazonaws.com)".to_string(),
            (
                2usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        let entries = map_to_api_entries(&map, false);
        assert_eq!(entries[0].api, "GetObject (s3.amazonaws.com)");
        assert_eq!(entries[0].description, "");
    }

    #[test]
    fn test_map_to_api_entries_sorted_by_count_desc() {
        let mut map = HashMap::new();
        map.insert(
            "ApiA (src)".to_string(),
            (
                1usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        map.insert(
            "ApiB (src)".to_string(),
            (
                5usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        map.insert(
            "ApiC (src)".to_string(),
            (
                3usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        let entries = map_to_api_entries(&map, false);
        assert_eq!(entries[0].count, 5);
        assert_eq!(entries[1].count, 3);
        assert_eq!(entries[2].count, 1);
    }

    // -----------------------------------------------------------------------
    // map_to_count_entries のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_map_to_count_entries_basic() {
        let mut map = HashMap::new();
        map.insert(
            "us-east-1".to_string(),
            (
                10usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-02T00:00:00Z".to_string(),
            ),
        );
        let entries = map_to_count_entries(&map);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, "us-east-1");
        assert_eq!(entries[0].count, 10);
        assert_eq!(entries[0].first_seen, "2024-01-01 00:00:00");
        assert_eq!(entries[0].last_seen, "2024-01-02 00:00:00");
    }

    #[test]
    fn test_map_to_count_entries_sorted_by_count_desc() {
        let mut map = HashMap::new();
        map.insert(
            "us-east-1".to_string(),
            (
                2usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        map.insert(
            "eu-west-1".to_string(),
            (
                8usize,
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T00:00:00Z".to_string(),
            ),
        );
        let entries = map_to_count_entries(&map);
        assert_eq!(entries[0].value, "eu-west-1");
        assert_eq!(entries[1].value, "us-east-1");
    }

    // -----------------------------------------------------------------------
    // build_json_records のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_json_records_basic() {
        let mut user_data = HashMap::new();
        user_data.insert(
            "arn:aws:iam::123:user/alice".to_string(),
            make_test_summary(),
        );

        let records = build_json_records(&user_data, false);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].user_arn, "arn:aws:iam::123:user/alice");
        assert_eq!(records[0].num_of_events, 2);
        assert_eq!(records[0].first_timestamp, "2024-01-01 00:00:00");
        assert_eq!(records[0].last_timestamp, "2024-01-02 00:00:00");
        assert_eq!(records[0].aws_regions.len(), 2);
        assert_eq!(records[0].abused_apis_success.len(), 1);
        assert_eq!(records[0].other_apis_success.len(), 1);
    }

    #[test]
    fn test_build_json_records_sorted_by_events_desc() {
        let mut user_data = HashMap::new();
        let summary_alice = CTSummary {
            num_of_events: 5,
            ..Default::default()
        };
        let summary_bob = CTSummary {
            num_of_events: 20,
            ..Default::default()
        };

        user_data.insert("arn:aws:iam::123:user/alice".to_string(), summary_alice);
        user_data.insert("arn:aws:iam::123:user/bob".to_string(), summary_bob);

        let records = build_json_records(&user_data, false);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].num_of_events, 20); // bob が先
        assert_eq!(records[1].num_of_events, 5); // alice が後
    }

    #[test]
    fn test_build_json_records_hide_descriptions() {
        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        let records = build_json_records(&user_data, true);
        // hide_descriptions=true のとき description は空文字
        assert_eq!(records[0].abused_apis_success[0].description, "");
    }

    // -----------------------------------------------------------------------
    // output_summary のテスト (出力ファイル生成確認)
    // -----------------------------------------------------------------------

    #[test]
    fn test_output_type_1_creates_csv_only() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 1, false);

        assert!(tmp.path().join("result.csv").exists());
        assert!(!tmp.path().join("result.json").exists());
        assert!(!tmp.path().join("result.jsonl").exists());
    }

    #[test]
    fn test_output_type_2_creates_json_only() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 2, false);

        assert!(!tmp.path().join("result.csv").exists());
        assert!(tmp.path().join("result.json").exists());
        assert!(!tmp.path().join("result.jsonl").exists());
    }

    #[test]
    fn test_output_type_3_creates_jsonl_only() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 3, false);

        assert!(!tmp.path().join("result.csv").exists());
        assert!(!tmp.path().join("result.json").exists());
        assert!(tmp.path().join("result.jsonl").exists());
    }

    #[test]
    fn test_output_type_4_creates_csv_and_json() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 4, false);

        assert!(tmp.path().join("result.csv").exists());
        assert!(tmp.path().join("result.json").exists());
        assert!(!tmp.path().join("result.jsonl").exists());
    }

    #[test]
    fn test_output_type_5_creates_csv_and_jsonl() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 5, false);

        assert!(tmp.path().join("result.csv").exists());
        assert!(!tmp.path().join("result.json").exists());
        assert!(tmp.path().join("result.jsonl").exists());
    }

    #[test]
    fn test_output_json_valid_structure() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert(
            "arn:aws:iam::123:user/alice".to_string(),
            make_test_summary(),
        );

        output_summary(&user_data, &output_path, true, &false, vec![], 2, false);

        let content = std::fs::read_to_string(tmp.path().join("result.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["user_arn"], "arn:aws:iam::123:user/alice");
        assert_eq!(arr[0]["num_of_events"], 2);
    }

    #[test]
    fn test_output_jsonl_each_line_is_valid_json() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::a".to_string(), make_test_summary());
        user_data.insert("arn::b".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 3, false);

        let content = std::fs::read_to_string(tmp.path().join("result.jsonl")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
        }
    }

    #[test]
    fn test_clobber_false_does_not_overwrite_existing_json() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");
        let json_path = tmp.path().join("result.json");

        // 先にファイルを作成
        std::fs::write(&json_path, "original").unwrap();

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 2, false);

        // 上書きされていないこと
        let content = std::fs::read_to_string(&json_path).unwrap();
        assert_eq!(content, "original");
    }

    #[test]
    fn test_clobber_false_preflights_all_output_paths_for_type_4() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");
        let csv_path = tmp.path().join("result.csv");
        let json_path = tmp.path().join("result.json");

        std::fs::write(&json_path, "original").unwrap();

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 4, false);

        assert_eq!(std::fs::read_to_string(&json_path).unwrap(), "original");
        assert!(!csv_path.exists());
    }

    #[test]
    fn test_clobber_true_overwrites_existing_json() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");
        let json_path = tmp.path().join("result.json");

        std::fs::write(&json_path, "original").unwrap();

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(&user_data, &output_path, true, &false, vec![], 2, true);

        let content = std::fs::read_to_string(&json_path).unwrap();
        assert_ne!(content, "original");
    }
}
