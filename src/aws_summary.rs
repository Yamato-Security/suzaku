use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, output_path_info, p, s};
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use sigma_rust::Event;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use termcolor::Color;

#[derive(Default)]
struct CTSummary {
    num_of_events: usize,
    first_timestamp: String,
    last_timestamp: String,
    aws_regions: HashMap<String, usize>,
    event_names: HashMap<String, usize>,
    src_ips: HashMap<String, usize>,
    user_types: HashMap<String, usize>,
    access_key_ids: HashMap<String, usize>,
    user_agents: HashMap<String, usize>,
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

        if !aws_region.is_empty() {
            *self.aws_regions.entry(aws_region).or_insert(0) += 1;
        }
        if !event_name.is_empty() {
            *self.event_names.entry(event_name).or_insert(0) += 1;
        }
        if !source_ip.is_empty() {
            *self.src_ips.entry(source_ip).or_insert(0) += 1;
        }
        if !user_type.is_empty() {
            *self.user_types.entry(user_type).or_insert(0) += 1;
        }
        if !access_key_id.is_empty() {
            *self.access_key_ids.entry(access_key_id).or_insert(0) += 1;
        }
        if !user_agent.is_empty() {
            *self.user_agents.entry(user_agent).or_insert(0) += 1;
        }
    }
}

pub fn aws_summary(
    directory: &Option<PathBuf>,
    file: &Option<PathBuf>,
    output: &Path,
    no_color: bool,
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
        let event_source = match event.get("eventSource") {
            Some(source) => s(format!("{:?}", source)),
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
            Some(access_key_id) => s(format!("{:?}", access_key_id)),
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
            event_source,
            source_ipaddress,
            user_identity_type,
            user_identity_access_key_id,
            user_agent,
        );
    };
    if let Some(d) = directory {
        process_events_from_dir(summary_func, d, true, no_color).unwrap();
        output_summary(&user_data, output, no_color);
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            events.into_iter().for_each(summary_func);
            output_summary(&user_data, output, no_color);
        }
    }
}

fn output_summary(user_data: &HashMap<String, CTSummary>, output: &Path, no_color: bool) {
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

    let fmt_and_sort = |map: &HashMap<String, usize>| -> String {
        map.iter()
            .sorted_by(|a, b| b.1.cmp(a.1)) // 件数の多い順にソート
            .map(|(key, count)| format!("{} - {}", count.to_formatted_string(&Locale::en), key))
            .join("\n")
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
        let aws_regions = fmt_and_sort(&summary.aws_regions);
        let event_names = fmt_and_sort(&summary.event_names);
        let src_ips = fmt_and_sort(&summary.src_ips);
        let user_types = fmt_and_sort(&summary.user_types);
        let access_key_ids = fmt_and_sort(&summary.access_key_ids);
        let user_agents = fmt_and_sort(&summary.user_agents);
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
