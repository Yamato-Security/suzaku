use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, output_path_info, p, s};
use itertools::Itertools;
use sigma_rust::Event;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use termcolor::Color;

#[derive(Default)]
struct CTSummary {
    num_of_events: usize,
    first_timestamp: String,
    last_timestamp: String,
    aws_regions: Vec<String>,
    event_names: Vec<String>,
    src_ips: Vec<String>,
    user_types: Vec<String>,
    access_key_ids: Vec<String>,
    user_agents: Vec<String>,
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

        if !aws_region.is_empty() && !self.aws_regions.contains(&aws_region) {
            self.aws_regions.push(aws_region);
        }
        if !event_name.is_empty() && !self.event_names.contains(&event_name) {
            self.event_names.push(event_name);
        }
        if !source_ip.is_empty() && !self.src_ips.contains(&source_ip) {
            self.src_ips.push(source_ip);
        }
        if !user_type.is_empty() && !self.user_types.contains(&user_type) {
            self.user_types.push(user_type);
        }
        if !access_key_id.is_empty() && !self.access_key_ids.contains(&access_key_id) {
            self.access_key_ids.push(access_key_id);
        }
        if !user_agent.is_empty() && !self.user_agents.contains(&user_agent) {
            self.user_agents.push(user_agent);
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

    for (user_arn, summary) in sorted_user_data.iter(){
        let num_of_events = summary.num_of_events.to_string();
        let first_timestamp = summary.first_timestamp.clone();
        let last_timestamp = summary.last_timestamp.clone();
        let aws_regions = summary.aws_regions.clone().into_iter().sorted().join("\n");
        let event_names = summary.event_names.clone().into_iter().sorted().join("\n");
        let src_ips = summary.src_ips.clone().into_iter().sorted().join("\n");
        let user_types = summary.user_types.clone().into_iter().sorted().join("\n");
        let access_key_ids = summary
            .access_key_ids
            .clone()
            .into_iter()
            .sorted()
            .join("\n");
        let user_agents = summary.user_agents.clone().into_iter().sorted().join("\n");
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
