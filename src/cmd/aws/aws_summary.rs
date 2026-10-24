use crate::core::color::SuzakuColor::Red;
use crate::core::duckdb_out::{
    self, MULTI_VALUE_SEPARATOR, OUTCOME_ENUM, SuzakuMeta, list_expr, nullable, quote_ident,
    timestamp_expr,
};
use crate::core::errorlog::{log_error, log_warn};
use crate::core::log_source::LogSource;
use crate::core::scan::{load_aws_events_from_file, process_events_from_dir};
use crate::core::timeline_writer::resolve_output_targets;
use crate::core::util::{
    error_msg, fatal_error, get_writer, output_path_info, p, report_unreadable_input,
    sanitize_csv_field, upsert_count_entry,
};
use crate::option::cli::{InputOption, OutputFormat};
use crate::option::geoip::GeoIPSearch;
use crate::option::timefiler::filter_by_time;
use csv::ReaderBuilder;
use duckdb::Connection;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use serde::Serialize;
use serde_json::Value;
use sigma_rust::{Event, event_from_json};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

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
    /// Every identity type this principal was seen with. Previously a single `String` that each
    /// event overwrote, so a principal seen as both `AssumedRole` and `IAMUser` reported only
    /// whichever event happened to be processed last.
    user_types: BTreeSet<String>,
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
        access_key_id: Option<String>,
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

        upsert_count_entry(&mut self.aws_regions, aws_region, &event_time);
        upsert_count_entry(&mut self.src_ips, source_ip, &event_time);
        self.user_types.insert(user_type);
        if let Some(access_key_id) = access_key_id {
            upsert_count_entry(&mut self.access_key_ids, access_key_id, &event_time);
        }
        upsert_count_entry(&mut self.user_agents, user_agent, &event_time);

        if !abused_api_success.is_empty() {
            upsert_count_entry(
                &mut self.abused_api_success,
                abused_api_success,
                &event_time,
            );
        }
        if !abused_api_failed.is_empty() {
            upsert_count_entry(&mut self.abused_api_failed, abused_api_failed, &event_time);
        }
        if !other_api_success.is_empty() {
            upsert_count_entry(&mut self.other_api_success, other_api_success, &event_time);
        }
        if !other_api_failed.is_empty() {
            upsert_count_entry(&mut self.other_api_failed, other_api_failed, &event_time);
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
            user_types: summary
                .user_types
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(MULTI_VALUE_SEPARATOR),
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

/// Prefix of AWS STS temporary access key IDs (as opposed to `AKIA` long-term keys).
const STS_KEY_PREFIX: &str = "ASIA";

/// The access key ID to record for `event` in the per-principal access key list, or `None` to
/// leave the list untouched.
///
/// Temporary STS keys are noise in that list (a new key per session), so they are left out unless
/// `-s` is given -- but only the key is left out, the event itself is still summarized. `-s` used
/// to be an event filter, which hid nearly everything done through AssumeRole, SSO, console
/// sessions and instance/Lambda roles: most activity in a modern account, and where
/// compromised-role activity shows up. Same rule and same flag as `aws-ct-metrics`.
fn summary_access_key_id(event: &Event, include_sts: bool) -> Option<String> {
    match event.get("userIdentity.accessKeyId") {
        Some(access_key_id) => {
            let key = access_key_id.value_to_string();
            if !include_sts && key.starts_with(STS_KEY_PREFIX) {
                None
            } else {
                Some(key)
            }
        }
        None => Some("-".to_string()),
    }
}

/// The principal an event is summarized under: its `userIdentity.arn`, or `userIdentity.userName`
/// when the record has no ARN, or `None` to leave the event out.
///
/// The fallback exists for AWS Management Console CSV downloads, which carry the user name but no
/// ARN. For a role, that name is the session name, so different roles using the same session name
/// are summarized as one principal. An ARN and a user name can never be the same string (IAM user
/// and session names cannot contain `:`), so the two kinds of key do not collide.
fn summary_principal(event: &Event) -> Option<String> {
    event
        .get("userIdentity.arn")
        .or_else(|| event.get("userIdentity.userName"))
        .map(|v| v.value_to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn aws_summary(
    input_opt: &InputOption,
    output: &Path,
    no_color: bool,
    include_sts: &bool,
    hide_descriptions: &bool,
    geo_ip: &Option<PathBuf>,
    output_types: &[OutputFormat],
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
        let Some(user_identity_arn) = summary_principal(&event) else {
            return;
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
        let user_identity_access_key_id = summary_access_key_id(&event, *include_sts);
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
            log_error(&format!("Failed to scan directory {}: {e}", d.display()));
        }
        output_summary(
            &user_data,
            output,
            no_color,
            hide_descriptions,
            abused_aws_api_values,
            output_types,
            clobber,
            geo_ip.is_some(),
        );
    } else if let Some(f) = file {
        match load_aws_events_from_file(f) {
            Ok(events) => {
                summary_func(&events);
                output_summary(
                    &user_data,
                    output,
                    no_color,
                    hide_descriptions,
                    abused_aws_api_values,
                    output_types,
                    clobber,
                    geo_ip.is_some(),
                );
            }
            Err(e) => report_unreadable_input(no_color, f, &e),
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
    output_types: &[OutputFormat],
    clobber: bool,
    // Whether -G ran. This command has no geo columns — the enrichment is appended to the SrcIP
    // value itself — so it only reaches suzaku_meta, where it tells a consumer that those values
    // carry an "(ASN, city, country)" suffix.
    geo_enabled: bool,
) {
    if user_data.is_empty() {
        error_msg(no_color, "No events found.");
        return;
    }

    // One <base>.<ext> per requested format, deduped — the same resolution the
    // timeline commands use, so -o/-t behave identically across the tool.
    let targets = resolve_output_targets(output, output_types);
    let path_for = |format: OutputFormat| -> Option<PathBuf> {
        targets
            .iter()
            .find(|(fmt, _)| *fmt == format)
            .map(|(_, path)| path.clone())
    };
    let csv_path = path_for(OutputFormat::Csv);
    let json_path = path_for(OutputFormat::Json);
    let jsonl_path = path_for(OutputFormat::Jsonl);
    let duckdb_path = path_for(OutputFormat::Duckdb);

    if !clobber
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
            let user_types = summary
                .user_types
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(MULTI_VALUE_SEPARATOR);
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
                sanitize_csv_field(&user_types),
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

    // --- JSONL 出力 ---
    if let Some(jsonl_path) = jsonl_path {
        let records = build_json_records(user_data, *hide_descriptions);
        let file = File::create(&jsonl_path).unwrap_or_else(|e| {
            fatal_error(
                no_color,
                &format!("Cannot write to output file {}: {e}", jsonl_path.display()),
            )
        });
        let mut writer = BufWriter::new(file);
        for record in &records {
            let line = serde_json::to_string(record).unwrap();
            writeln!(writer, "{}", line).unwrap();
        }
        writer.flush().unwrap();
        output_paths.push(jsonl_path);
    }

    // --- DuckDB 出力 ---
    if let Some(duckdb_path) = duckdb_path {
        let records = build_json_records(user_data, *hide_descriptions);
        match write_duckdb_summary(&duckdb_path, &records, geo_enabled) {
            Ok(()) => output_paths.push(duckdb_path),
            Err(e) => fatal_error(no_color, &e),
        }
    }

    output_path_info(no_color, output_paths.as_slice(), true);
}

/// Split `"RunInstances (ec2.amazonaws.com)"` into its action and its service.
///
/// The two are already separate columns in the timeline output (`EventName`, `EventSource`), so
/// packing them back into one string here made the same two facts need string parsing in one
/// command and not the other. Returns an empty service when the value does not carry one.
fn split_api(api: &str) -> (&str, &str) {
    if let Some(open) = api.rfind(" (")
        && api.ends_with(')')
    {
        return (&api[..open], &api[open + 2..api.len() - 1]);
    }
    (api, "")
}

/// Write the summary to a DuckDB database as three related tables plus `suzaku_meta`.
///
/// The CSV output folds each user's API calls, regions, IPs, keys and agents
/// into multi-line text blobs, which is right for a spreadsheet and useless in
/// SQL. A database output only earns its place if the data is queryable, so the
/// nested structure of the JSON records is kept as relations instead:
///
///   summary             one row per principal
///   summary_api_calls   one row per (principal, API), with its abused/outcome axes
///   summary_attributes  one row per (principal, attribute value)
///
/// so questions the CSV cannot answer — which principals share an access key,
/// which source IPs called a given abused API — are ordinary joins.
///
/// Values are typed rather than rendered: timestamps are `TIMESTAMP`, counts `BIGINT`, the
/// old `Category` string is split into the two orthogonal axes it always encoded (`IsAbused`
/// boolean + `Outcome` enum), the `API` string into `API` + `EventSource`, and `UserTypes` into a
/// list. `-`/empty placeholders become NULL. See `core::duckdb_out` for why the rows go through a
/// staging table on the way in.
fn write_duckdb_summary(
    path: &Path,
    records: &[SummaryJsonRecord],
    geo_enabled: bool,
) -> Result<(), String> {
    let conn = Connection::open(path)
        .map_err(|e| format!("Cannot write to output file {}: {e}", path.display()))?;

    // Column expressions are shared between the three tables where the column means the same
    // thing, so a change to (say) how a timestamp is parsed cannot drift between them.
    let raw = |c: &str| quote_ident(c);
    let bigint = |c: &str| format!("TRY_CAST({} AS BIGINT)", nullable(&quote_ident(c)));

    let summary_rows: Vec<Vec<String>> = records
        .iter()
        .map(|r| {
            vec![
                r.user_arn.clone(),
                r.num_of_events.to_string(),
                r.first_timestamp.clone(),
                r.last_timestamp.clone(),
                r.user_types.clone(),
            ]
        })
        .collect();
    duckdb_out::stage_and_type(
        &conn,
        "summary",
        "UserARN VARCHAR NOT NULL,
         NumOfEvents BIGINT NOT NULL,
         FirstTimestamp TIMESTAMP,
         LastTimestamp TIMESTAMP,
         UserTypes VARCHAR[]",
        &[
            ("UserARN", raw("UserARN")),
            ("NumOfEvents", bigint("NumOfEvents")),
            (
                "FirstTimestamp",
                timestamp_expr(&quote_ident("FirstTimestamp")),
            ),
            (
                "LastTimestamp",
                timestamp_expr(&quote_ident("LastTimestamp")),
            ),
            ("UserTypes", list_expr(&quote_ident("UserTypes"))),
        ],
        &summary_rows,
    )?;

    let mut api_rows: Vec<Vec<String>> = Vec::new();
    for record in records {
        // The four buckets are two independent yes/no facts, not four categories: keeping them as
        // `abused_success`/`other_failed` strings forced `LIKE 'abused%'` for one and
        // `LIKE '%failed'` for the other.
        for (is_abused, outcome, entries) in [
            (true, "success", &record.abused_apis_success),
            (true, "failed", &record.abused_apis_failed),
            (false, "success", &record.other_apis_success),
            (false, "failed", &record.other_apis_failed),
        ] {
            for entry in entries {
                let (api, event_source) = split_api(&entry.api);
                api_rows.push(vec![
                    record.user_arn.clone(),
                    is_abused.to_string(),
                    outcome.to_string(),
                    api.to_string(),
                    event_source.to_string(),
                    entry.description.clone(),
                    entry.count.to_string(),
                    entry.first_seen.clone(),
                    entry.last_seen.clone(),
                ]);
            }
        }
    }
    duckdb_out::stage_and_type(
        &conn,
        "summary_api_calls",
        &format!(
            "UserARN VARCHAR NOT NULL,
             IsAbused BOOLEAN NOT NULL,
             Outcome {OUTCOME_ENUM} NOT NULL,
             API VARCHAR NOT NULL,
             EventSource VARCHAR,
             Description VARCHAR,
             Count BIGINT NOT NULL,
             FirstSeen TIMESTAMP,
             LastSeen TIMESTAMP"
        ),
        &[
            ("UserARN", raw("UserARN")),
            ("IsAbused", format!("CAST({} AS BOOLEAN)", raw("IsAbused"))),
            (
                "Outcome",
                format!("CAST({} AS {OUTCOME_ENUM})", raw("Outcome")),
            ),
            ("API", raw("API")),
            ("EventSource", nullable(&quote_ident("EventSource"))),
            ("Description", nullable(&quote_ident("Description"))),
            ("Count", bigint("Count")),
            ("FirstSeen", timestamp_expr(&quote_ident("FirstSeen"))),
            ("LastSeen", timestamp_expr(&quote_ident("LastSeen"))),
        ],
        &api_rows,
    )?;

    let mut attr_rows: Vec<Vec<String>> = Vec::new();
    for record in records {
        // Attribute labels are spelled exactly like the timeline columns holding the same fact,
        // so `src_ip` / `SrcIP` / `sourceIPAddress` stop being three names for one concept.
        for (attribute, entries) in [
            ("AwsRegion", &record.aws_regions),
            ("SrcIP", &record.src_ips),
            ("UserAccessKeyID", &record.user_access_key_ids),
            ("UserAgent", &record.user_agents),
        ] {
            for entry in entries {
                attr_rows.push(vec![
                    record.user_arn.clone(),
                    attribute.to_string(),
                    entry.value.clone(),
                    entry.count.to_string(),
                    entry.first_seen.clone(),
                    entry.last_seen.clone(),
                ]);
            }
        }
    }
    duckdb_out::stage_and_type(
        &conn,
        "summary_attributes",
        "UserARN VARCHAR NOT NULL,
         Attribute VARCHAR NOT NULL,
         Value VARCHAR,
         Count BIGINT NOT NULL,
         FirstSeen TIMESTAMP,
         LastSeen TIMESTAMP",
        &[
            ("UserARN", raw("UserARN")),
            ("Attribute", raw("Attribute")),
            ("Value", nullable(&quote_ident("Value"))),
            ("Count", bigint("Count")),
            ("FirstSeen", timestamp_expr(&quote_ident("FirstSeen"))),
            ("LastSeen", timestamp_expr(&quote_ident("LastSeen"))),
        ],
        &attr_rows,
    )?;

    let mut meta = SuzakuMeta::new("aws-ct-summary").with_geoip(geo_enabled);
    meta.output_rows = Some(records.len() as i64);
    duckdb_out::write_meta(&conn, &meta)?;
    duckdb_out::comment_on_table(&conn, "summary", "One row per principal (UserARN).")?;
    duckdb_out::comment_on_table(
        &conn,
        "summary_api_calls",
        "One row per (UserARN, API call). IsAbused and Outcome are the two axes the old Category \
         string packed together.",
    )?;
    duckdb_out::comment_on_table(
        &conn,
        "summary_attributes",
        "One row per (UserARN, attribute, value). Attribute names match the timeline columns \
         holding the same fact.",
    )?;
    duckdb_out::checkpoint(&conn)
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
            log_warn(&format!(
                "Could not open the abused-AWS-API list at '{file_path}' \
                 (run from the directory that contains ./rules, or after update-rules). \
                 All API calls will be classified as non-abused."
            ));
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
            Some("AKIAIOSFODNN7EXAMPLE".to_string()),
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
            Some("AKIAIOSFODNN7EXAMPLE".to_string()),
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

    #[test]
    fn test_ct_summary_event_without_access_key_is_still_counted() {
        // `None` leaves the access key list untouched but the event is summarized as usual.
        let mut summary = make_test_summary();
        summary.add_event(
            "2024-01-03T00:00:00Z".to_string(),
            "eu-west-1".to_string(),
            "9.9.9.9".to_string(),
            "AssumedRole".to_string(),
            None,
            "console.amazonaws.com".to_string(),
            "".to_string(),
            "".to_string(),
            "DescribeInstances (ec2.amazonaws.com)".to_string(),
            "".to_string(),
        );
        assert_eq!(summary.num_of_events, 3);
        assert_eq!(summary.last_timestamp, "2024-01-03T00:00:00Z");
        assert!(summary.aws_regions.contains_key("eu-west-1"));
        assert!(summary.src_ips.contains_key("9.9.9.9"));
        assert!(summary.user_types.contains("AssumedRole"));
        assert!(
            summary
                .other_api_success
                .contains_key("DescribeInstances (ec2.amazonaws.com)")
        );
        assert_eq!(summary.access_key_ids.len(), 1);
        assert_eq!(summary.access_key_ids["AKIAIOSFODNN7EXAMPLE"].0, 2);
    }

    // -----------------------------------------------------------------------
    // summary_access_key_id のテスト
    // -----------------------------------------------------------------------

    fn event_with_key(key: Option<&str>) -> Event {
        let json = match key {
            Some(k) => format!(
                r#"{{"eventName":"GetCallerIdentity","userIdentity":{{"arn":"arn:aws:sts::123456789012:assumed-role/r/s","accessKeyId":"{k}"}}}}"#
            ),
            None => r#"{"eventName":"ConsoleLogin","userIdentity":{"arn":"arn:aws:iam::123456789012:user/u"}}"#.to_string(),
        };
        event_from_json(&json).unwrap()
    }

    #[test]
    fn test_summary_access_key_id_omits_sts_key_by_default() {
        // The STS key is left out of the list, but `None` here does not drop the event: the
        // caller still passes it to `add_event` (previously the whole event was discarded).
        let event = event_with_key(Some("ASIAIOSFODNN7EXAMPLE"));
        assert_eq!(summary_access_key_id(&event, false), None);
    }

    #[test]
    fn test_summary_access_key_id_includes_sts_key_with_flag() {
        let event = event_with_key(Some("ASIAIOSFODNN7EXAMPLE"));
        assert_eq!(
            summary_access_key_id(&event, true),
            Some("ASIAIOSFODNN7EXAMPLE".to_string())
        );
    }

    #[test]
    fn test_summary_access_key_id_keeps_long_term_key() {
        let event = event_with_key(Some("AKIAIOSFODNN7EXAMPLE"));
        for include_sts in [false, true] {
            assert_eq!(
                summary_access_key_id(&event, include_sts),
                Some("AKIAIOSFODNN7EXAMPLE".to_string())
            );
        }
    }

    #[test]
    fn test_summary_access_key_id_records_missing_key_as_dash() {
        let event = event_with_key(None);
        for include_sts in [false, true] {
            assert_eq!(
                summary_access_key_id(&event, include_sts),
                Some("-".to_string())
            );
        }
    }

    // -----------------------------------------------------------------------
    // summary_principal のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_summary_principal_prefers_arn() {
        let event = event_from_json(
            r#"{"userIdentity":{"arn":"arn:aws:iam::123456789012:user/u","userName":"u"}}"#,
        )
        .unwrap();
        assert_eq!(
            summary_principal(&event),
            Some("arn:aws:iam::123456789012:user/u".to_string())
        );
    }

    #[test]
    fn test_summary_principal_falls_back_to_user_name() {
        // A console CSV row has a user name but no ARN.
        let event = event_from_json(r#"{"userIdentity":{"userName":"example-session"}}"#).unwrap();
        assert_eq!(
            summary_principal(&event),
            Some("example-session".to_string())
        );
    }

    #[test]
    fn test_summary_principal_none_without_arn_or_user_name() {
        let event = event_from_json(r#"{"eventName":"RotateKey"}"#).unwrap();
        assert_eq!(summary_principal(&event), None);
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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Csv],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Json],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Jsonl],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Csv, OutputFormat::Json],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Csv, OutputFormat::Jsonl],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Json],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Jsonl],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Json],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Csv, OutputFormat::Json],
            false,
            false,
        );

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

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Json],
            true,
            false,
        );

        let content = std::fs::read_to_string(&json_path).unwrap();
        assert_ne!(content, "original");
    }

    /// The relational shape is the point of the DuckDB output: the CSV folds
    /// each principal's APIs and attributes into multi-line text, which cannot
    /// be queried. These pin the schema and the row counts against the same
    /// records the JSON output is built from.
    #[test]
    fn duckdb_summary_writes_three_related_tables() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");
        let duckdb_path = tmp.path().join("result.duckdb");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Duckdb],
            false,
            false,
        );

        assert!(duckdb_path.exists(), "the .duckdb database must be written");
        let conn = Connection::open(&duckdb_path).unwrap();

        // Timestamps are real TIMESTAMPs, so they are rendered here rather than compared as text.
        let (arn, events, first, last): (String, i64, String, String) = conn
            .query_row(
                "SELECT UserARN, NumOfEvents,
                        strftime(FirstTimestamp, '%Y-%m-%d %H:%M:%S'),
                        strftime(LastTimestamp, '%Y-%m-%d %H:%M:%S')
                 FROM summary",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(arn, "arn::test");
        assert_eq!(events, user_data["arn::test"].num_of_events as i64);
        assert_eq!(first, "2024-01-01 00:00:00");
        assert_eq!(last, "2024-01-02 00:00:00");
        let ts_type: String = conn
            .query_row(
                "SELECT data_type FROM duckdb_columns()
                 WHERE table_name = 'summary' AND column_name = 'FirstTimestamp'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ts_type, "TIMESTAMP");

        // Every API and attribute entry becomes its own row, tagged with the
        // category/attribute it came from, rather than a text blob.
        let api_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM summary_api_calls", [], |r| r.get(0))
            .unwrap();
        assert!(api_rows > 0, "API entries must be exploded into rows");
        let attr_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM summary_attributes", [], |r| r.get(0))
            .unwrap();
        assert!(attr_rows > 0, "attributes must be exploded into rows");
    }

    #[test]
    fn duckdb_summary_row_counts_match_the_json_records() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::a".to_string(), make_test_summary());
        user_data.insert("arn::b".to_string(), make_test_summary());

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Duckdb],
            false,
            false,
        );

        // The two outputs are built from the same records, so a mismatch means
        // the relational fan-out dropped or duplicated something.
        let records = build_json_records(&user_data, false);
        let expected_apis: usize = records
            .iter()
            .map(|r| {
                r.abused_apis_success.len()
                    + r.abused_apis_failed.len()
                    + r.other_apis_success.len()
                    + r.other_apis_failed.len()
            })
            .sum();
        let expected_attrs: usize = records
            .iter()
            .map(|r| {
                r.aws_regions.len()
                    + r.src_ips.len()
                    + r.user_access_key_ids.len()
                    + r.user_agents.len()
            })
            .sum();

        let conn = Connection::open(tmp.path().join("result.duckdb")).unwrap();
        let summary_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM summary", [], |r| r.get(0))
            .unwrap();
        let api_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM summary_api_calls", [], |r| r.get(0))
            .unwrap();
        let attr_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM summary_attributes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(summary_rows, records.len() as i64);
        assert_eq!(api_rows, expected_apis as i64);
        assert_eq!(attr_rows, expected_attrs as i64);
    }

    /// The old `Category` column packed two orthogonal facts into one string, forcing
    /// `LIKE 'abused%'` for one and `LIKE '%failed'` for the other. They are now separate typed
    /// columns, and the service is split out of the API name.
    #[test]
    fn duckdb_summary_api_calls_split_their_packed_columns() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Duckdb],
            false,
            false,
        );

        let conn = Connection::open(tmp.path().join("result.duckdb")).unwrap();
        // `Category` is gone; asking "which abused calls succeeded" is now a plain conjunction.
        let (api, source, description): (String, String, String) = conn
            .query_row(
                "SELECT API, EventSource, Description FROM summary_api_calls
                 WHERE IsAbused AND Outcome = 'success'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(api, "ListBuckets");
        assert_eq!(source, "s3.amazonaws.com");
        assert_eq!(description, "List all S3 buckets");

        let (non_abused, outcomes): (i64, i64) = conn
            .query_row(
                "SELECT count(*) FILTER (WHERE NOT IsAbused),
                        count(DISTINCT Outcome)
                 FROM summary_api_calls",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(non_abused, 1);
        assert_eq!(outcomes, 1, "the fixture only has successful calls");

        // Attribute labels are spelled like the timeline columns holding the same fact.
        let mut stmt = conn
            .prepare("SELECT DISTINCT Attribute FROM summary_attributes ORDER BY 1")
            .unwrap();
        let attributes: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for attribute in &attributes {
            assert!(
                ["AwsRegion", "SrcIP", "UserAccessKeyID", "UserAgent"]
                    .contains(&attribute.as_str()),
                "unexpected attribute label: {attribute}"
            );
        }
    }

    /// `UserTypes` is plural, so it is a list. It also has to hold *every* identity type the
    /// principal was seen with, not just the one from the last event processed.
    #[test]
    fn duckdb_summary_user_types_is_a_list_of_every_type_seen() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut summary = make_test_summary();
        summary.add_event(
            "2024-01-03T00:00:00Z".to_string(),
            "us-east-1".to_string(),
            "1.2.3.4".to_string(),
            "AssumedRole".to_string(),
            Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            "aws-cli/2.0".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );
        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), summary);

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Duckdb],
            false,
            false,
        );

        let conn = Connection::open(tmp.path().join("result.duckdb")).unwrap();
        let (types, kind): (String, String) = conn
            .query_row(
                "SELECT list_aggregate(UserTypes, 'string_agg', ','),
                        (SELECT data_type FROM duckdb_columns()
                         WHERE table_name = 'summary' AND column_name = 'UserTypes')
                 FROM summary",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(kind, "VARCHAR[]");
        assert_eq!(types, "AssumedRole,IAMUser");
    }

    /// P1: the summary file says what produced it, like the timeline file does.
    #[test]
    fn duckdb_summary_writes_self_describing_metadata() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Duckdb],
            false,
            false,
        );

        let conn = Connection::open(tmp.path().join("result.duckdb")).unwrap();
        let (schema_version, command, rows): (i32, String, i64) = conn
            .query_row(
                "SELECT schema_version, command, output_rows FROM suzaku_meta",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(schema_version, duckdb_out::SCHEMA_VERSION);
        assert_eq!(command, "aws-ct-summary");
        assert_eq!(rows, 1);
    }

    #[test]
    fn split_api_separates_action_from_service() {
        assert_eq!(
            split_api("RunInstances (ec2.amazonaws.com)"),
            ("RunInstances", "ec2.amazonaws.com")
        );
        // A value that does not carry a service is left whole.
        assert_eq!(split_api("RunInstances"), ("RunInstances", ""));
        assert_eq!(split_api("-"), ("-", ""));
    }

    #[test]
    fn duckdb_summary_can_be_written_alongside_csv_and_json() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("result");

        let mut user_data = HashMap::new();
        user_data.insert("arn::test".to_string(), make_test_summary());

        output_summary(
            &user_data,
            &output_path,
            true,
            &false,
            vec![],
            &[OutputFormat::Csv, OutputFormat::Json, OutputFormat::Duckdb],
            false,
            false,
        );

        for extension in ["csv", "json", "duckdb"] {
            assert!(
                tmp.path().join(format!("result.{extension}")).exists(),
                "missing .{extension} output"
            );
        }
    }

    #[test]
    fn upsert_count_entry_tracks_per_key_time_range() {
        let mut m: HashMap<String, (usize, String, String)> = HashMap::new();
        // Same key seen at T1, then T3, then T2 (out of order).
        upsert_count_entry(&mut m, "us-east-1".to_string(), "2024-01-01T00:00:00Z");
        upsert_count_entry(&mut m, "us-east-1".to_string(), "2024-01-03T00:00:00Z");
        upsert_count_entry(&mut m, "us-east-1".to_string(), "2024-01-02T00:00:00Z");
        let e = &m["us-east-1"];
        assert_eq!(e.0, 3, "count");
        // first_seen/last_seen reflect THIS key's own earliest/latest event,
        // not the dataset-global min/max frozen at insertion time.
        assert_eq!(e.1, "2024-01-01T00:00:00Z");
        assert_eq!(e.2, "2024-01-03T00:00:00Z");
    }
}
