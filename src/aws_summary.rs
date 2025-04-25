use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, output_path_info, p, s};
use csv::ReaderBuilder;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use sigma_rust::Event;
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
    aws_regions: HashMap<String, (usize, String, String)>,
    event_names: HashMap<String, (usize, String, String)>,
    src_ips: HashMap<String, (usize, String, String)>,
    user_types: HashMap<String, (usize, String, String)>,
    access_key_ids: HashMap<String, (usize, String, String)>,
    user_agents: HashMap<String, (usize, String, String)>,
}

impl CTSummary {
    #[allow(clippy::too_many_arguments)]
    fn add_event(
        &mut self,
        event_time: String,
        aws_region: String,
        event_name: String,
        source_ip: String,
        user_type: String,
        access_key_id: String,
        user_agent: String,
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
        let entry = self.event_names.entry(event_name.clone()).or_insert((
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
        let entry = self.user_types.entry(user_type.clone()).or_insert((
            0,
            self.first_timestamp.clone(),
            self.last_timestamp.clone(),
        ));
        entry.0 += 1;
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
    }
}

pub fn aws_summary(
    directory: &Option<PathBuf>,
    file: &Option<PathBuf>,
    output: &Path,
    no_color: bool,
    filter_sts: &Option<String>,
    hide_descriptions: &bool,
) {
    let mut user_data: HashMap<String, CTSummary> = HashMap::new();
    let summary_func = |event: Event| {
        let user_identity_arn = match event.get("userIdentity.arn") {
            Some(arn) => s(format!("{:?}", arn)),
            None => return,
        };
        let event_time = match event.get("eventTime") {
            Some(time) => s(format!("{:?}", time)),
            None => "-".to_string(),
        };
        let aws_region = match event.get("awsRegion") {
            Some(region) => s(format!("{:?}", region)),
            None => "-".to_string(),
        };
        let event_name = match event.get("eventName") {
            Some(name) => s(format!("{:?}", name)),
            None => "-".to_string(),
        };
        let source_ipaddress = match event.get("sourceIPAddress") {
            Some(ip) => s(format!("{:?}", ip)),
            None => "-".to_string(),
        };
        let user_identity_type = match event.get("userIdentity.type") {
            Some(user_type) => s(format!("{:?}", user_type)),
            None => "-".to_string(),
        };
        let user_identity_access_key_id = match event.get("userIdentity.accessKeyId") {
            Some(access_key_id) => {
                let key = s(format!("{:?}", access_key_id));
                if let Some(filter) = filter_sts {
                    if key.contains(filter) {
                        return;
                    }
                }
                key
            }
            None => "-".to_string(),
        };
        let user_agent = match event.get("userAgent") {
            Some(agent) => s(format!("{:?}", agent)),
            None => "-".to_string(),
        };

        let entry = user_data.entry(user_identity_arn.clone()).or_default();
        entry.add_event(
            event_time,
            aws_region,
            event_name,
            source_ipaddress,
            user_identity_type,
            user_identity_access_key_id,
            user_agent,
        );
    };
    let abused_aws_api_calls = read_abused_aws_api_calls("config/abused_aws_api_calls.csv");
    if let Some(d) = directory {
        process_events_from_dir(summary_func, d, true, no_color).unwrap();
        output_summary(
            &user_data,
            &abused_aws_api_calls,
            output,
            no_color,
            hide_descriptions,
        );
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            events.into_iter().for_each(summary_func);
            output_summary(
                &user_data,
                &abused_aws_api_calls,
                output,
                no_color,
                hide_descriptions,
            );
        }
    }
}

fn output_summary(
    user_data: &HashMap<String, CTSummary>,
    api_calls: &HashMap<String, String>,
    output: &Path,
    no_color: bool,
    hide_descriptions: &bool,
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
        "AWS-Regions",
        "EventNames",
        "SrcIPs",
        "UserTypes",
        "UserAccessKeyIDs",
        "UserAgents",
    ];

    csv_wtr.write_record(&csv_header).unwrap();

    let mut sorted_user_data: Vec<_> = user_data.iter().collect();
    sorted_user_data.sort_by(|a, b| b.1.num_of_events.cmp(&a.1.num_of_events));

    let fmt_and_sort = |key: &str, map: &HashMap<String, (usize, String, String)>| -> String {
        let total: usize = map.values().map(|v| v.0).sum();
        let total = total.to_formatted_string(&Locale::en);
        let mut result = vec![format!("Total {}s: {}", key, total)];
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
        let aws_regions = fmt_and_sort("region", &summary.aws_regions);
        let event_names = if *hide_descriptions {
            fmt_and_sort("event name", &summary.event_names)
        } else {
            let rep_event_names = replace_with_descriptions(&summary.event_names, api_calls);
            fmt_and_sort("event name", &rep_event_names)
        };
        let src_ips = fmt_and_sort("src ip", &summary.src_ips);
        let user_types = fmt_and_sort("user type", &summary.user_types);
        let access_key_ids = fmt_and_sort("access key id", &summary.access_key_ids);
        let user_agents = fmt_and_sort("user agent", &summary.user_agents);
        csv_wtr
            .write_record(vec![
                user_arn,
                &num_of_events,
                &first_timestamp,
                &last_timestamp,
                &aws_regions,
                &event_names,
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
            for result in csv_reader.records() {
                let record = result.expect("Failed to read record");
                let event_name = record.get(0).expect("Missing EventName").to_string();
                let description = record.get(1).expect("Missing Description").to_string();
                map.insert(event_name, description);
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

fn replace_with_descriptions(
    event_names: &HashMap<String, (usize, String, String)>,
    api_calls: &HashMap<String, String>,
) -> HashMap<String, (usize, String, String)> {
    event_names
        .iter()
        .map(|(key, value)| {
            let new_key = api_calls
                .get(key)
                .unwrap_or(&"OtherAPIs".to_string())
                .to_string();
            (new_key, value.clone())
        })
        .collect()
}
