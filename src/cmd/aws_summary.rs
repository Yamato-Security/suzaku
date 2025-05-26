use crate::core::color::SuzakuColor::Red;
use crate::core::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::core::util::{get_writer, output_path_info, p};
use crate::option::geoip::GeoIPSearch;
use csv::ReaderBuilder;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use sigma_rust::{Event, event_from_json};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use termcolor::Color;

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

pub fn aws_summary(
    directory: &Option<PathBuf>,
    file: &Option<PathBuf>,
    output: &Path,
    no_color: bool,
    include_sts: &bool,
    hide_descriptions: &bool,
    geo_ip: &Option<PathBuf>,
) {
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
    let mut summary_func = |json_value: &Value| {
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
                if let Some(geo) = geo_search.as_mut() {
                    if let Some(ip) = geo.convert(ip_str.as_str()) {
                        let asn = geo.get_asn(ip);
                        let country = geo.get_country(ip);
                        let city = geo.get_city(ip);
                        ip_str = format!("{} ({}, {}, {})", ip_str, asn, city, country);
                    }
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
        if let Some(desc) = abused_aws_api_calls.get(&event_name) {
            if error_code != "AccessDenied" {
                abused_api_success = format!("{} ({}) - {}", event_name, event_source, desc);
            }
        };

        let mut abused_api_failed = "".to_string();
        if let Some(desc) = abused_aws_api_calls.get(&event_name) {
            if error_code == "AccessDenied" {
                abused_api_failed = format!("{} ({}) - {}", event_name, event_source, desc);
            }
        };

        let mut other_api_success = "".to_string();
        if !abused_aws_api_calls.contains_key(&event_name) && error_code != "AccessDenied" {
            other_api_success = format!("{} ({})", event_name, event_source);
        };

        let mut other_api_failed = "".to_string();
        if !abused_aws_api_calls.contains_key(&event_name) && error_code == "AccessDenied" {
            other_api_failed = format!("{} ({})", event_name, event_source);
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
    let abused_aws_api_values: Vec<String> = abused_aws_api_calls.values().cloned().collect();
    if let Some(d) = directory {
        process_events_from_dir(summary_func, d, true, no_color).unwrap();
        output_summary(
            &user_data,
            output,
            no_color,
            hide_descriptions,
            abused_aws_api_values,
        );
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            for event in events {
                summary_func(&event);
            }
            output_summary(
                &user_data,
                output,
                no_color,
                hide_descriptions,
                abused_aws_api_values,
            );
        }
    }
}

fn output_summary(
    user_data: &HashMap<String, CTSummary>,
    output: &Path,
    no_color: bool,
    hide_descriptions: &bool,
    abused_aws_api_disc: Vec<String>,
) {
    if user_data.is_empty() {
        p(Some(Color::Rgb(255, 0, 0)), "No events found.", true);
        return;
    }

    let mut csv_path = output.to_path_buf();
    if csv_path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        csv_path.set_extension("csv");
    }
    let mut csv_wtr = get_writer(&Some(csv_path.clone()));
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
    sorted_user_data.sort_by(|a, b| b.1.num_of_events.cmp(&a.1.num_of_events));

    let fmt_key_total = |msg: &str, map: &HashMap<String, (usize, String, String)>| -> String {
        let total: usize = map.keys().len();
        let total = total.to_formatted_string(&Locale::en);
        let mut result = vec![format!("{}: {}", msg, total)];
        result.extend(
            map.iter()
                .sorted_by(|a, b| b.1.cmp(a.1)) // 件数の多い順にソート
                .map(|(k, v)| {
                    format!(
                        "{} - {} ({} ~ {})",
                        v.0.to_formatted_string(&Locale::en),
                        k,
                        v.1.replace('Z', "").replace('T', " "),
                        v.2.replace('Z', "").replace('T', " ")
                    )
                }),
        );
        result.join("\n")
    };

    let fmt_val_total = |msg: &str, map: &HashMap<String, (usize, String, String)>| -> String {
        let total: usize = map.values().map(|v| v.0).sum();
        let total = total.to_formatted_string(&Locale::en);
        format!("| {} {}", msg, total)
    };

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
        let src_ips = fmt_key_total("Total source IDs", &summary.src_ips);
        let user_types = &summary.user_types;
        let access_key_ids = fmt_key_total("Total access key IDs", &summary.access_key_ids);
        let user_agents = fmt_key_total("Total user agents", &summary.user_agents);

        let mut abused_suc = fmt_key_total("Unique APIs", &summary.abused_api_success);
        if let Some(pos) = abused_suc.find('\n') {
            let abused_suc_val = fmt_val_total("Total APIs", &summary.abused_api_success);
            abused_suc.insert_str(pos, &format!(" {}", abused_suc_val));
        }
        let mut abused_fai = fmt_key_total("Unique APIs", &summary.abused_api_failed);
        if let Some(pos) = abused_fai.find('\n') {
            let abused_fai_val = fmt_val_total("Total APIs", &summary.abused_api_failed);
            abused_fai.insert_str(pos, &format!(" {}", abused_fai_val));
        }
        let mut other_suc = fmt_key_total("Unique APIs", &summary.other_api_success);
        if let Some(pos) = other_suc.find('\n') {
            let other_suc_val = fmt_val_total("Total APIs", &summary.other_api_success);
            other_suc.insert_str(pos, &format!(" {}", other_suc_val));
        }
        let mut other_fai = fmt_key_total("Unique APIs", &summary.other_api_failed);
        if let Some(pos) = other_fai.find('\n') {
            let other_fai_val = fmt_val_total("Total APIs", &summary.other_api_failed);
            other_fai.insert_str(pos, &format!(" {}", other_fai_val));
        }

        if *hide_descriptions {
            abused_aws_api_disc.iter().for_each(|disc| {
                abused_suc = abused_suc.replace(disc, "");
                abused_fai = abused_fai.replace(disc, "");
            });
            abused_suc = abused_suc.replace("-  (2", "(2");
            abused_fai = abused_fai.replace("-  (2", "(2");
        }

        csv_wtr
            .write_record(vec![
                user_arn,
                &num_of_events,
                &first_timestamp,
                &last_timestamp,
                &abused_suc,
                &abused_fai,
                &other_suc,
                &other_fai,
                &aws_regions,
                &src_ips,
                &user_types,
                &access_key_ids,
                &user_agents,
            ])
            .unwrap();
    }
    csv_wtr.flush().unwrap();
    output_path_info(no_color, [csv_path].as_slice());
}

fn read_abused_aws_api_calls(file_path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let file = File::open(file_path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
            for record in csv_reader.records().flatten() {
                if let Some(event_name) = record.get(0) {
                    if let Some(description) = record.get(1) {
                        map.insert(event_name.to_string(), description.to_string());
                    }
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
