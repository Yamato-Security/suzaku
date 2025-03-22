use crate::rules;
use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, s};
use sigma_rust::{Event, Rule};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn aws_detect(
    directory: &Option<PathBuf>,
    file: &Option<PathBuf>,
    output: &Option<PathBuf>,
    no_color: bool,
) {
    let profile = load_profile("config/aws_ct_timeline_default_profile.txt");
    let rules = rules::load_rules_from_dir("rules");
    println!("Total detection rules: {:?}", rules.len());

    let mut wtr = get_writer(output);
    let csv_header: Vec<&str> = profile.iter().map(|(k, _v)| k.as_str()).collect();
    wtr.write_record(&csv_header).unwrap();

    let scan_by_all_rules = |event| {
        for rule in &rules {
            if rule.is_match(&event) {
                let record: Vec<String> = profile
                    .iter()
                    .map(|(_k, v)| get_value_from_event(v, &event, rule))
                    .collect();
                wtr.write_record(&record).unwrap();
            }
        }
    };

    if let Some(d) = directory {
        process_events_from_dir(scan_by_all_rules, d, output.is_some(), no_color).unwrap();
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            events.into_iter().for_each(scan_by_all_rules);
        }
    }
    wtr.flush().ok();
}

fn load_profile(file_path: &str) -> Vec<(String, String)> {
    let file = File::open(file_path).expect("Unable to open profile file");
    let reader = BufReader::new(file);
    let mut profile = vec![];

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            profile.push((String::from(parts[0]), String::from(parts[1])));
        }
    }
    profile
}

fn get_value_from_event(key: &str, event: &Event, rule: &Rule) -> String {
    if key.starts_with("awsLog.") {
        let key = key.replace("awsLog.", "");
        if let Some(value) = event.get(key.as_str()) {
            s(format!("{:?}", value))
        } else {
            "".to_string()
        }
    } else if key.starts_with("sigmaRule.") {
        let key = key.replace("sigmaRule.", "");
        if key == "title" {
            rule.title.to_string()
        } else if key == "level" {
            format!("{:?}", rule.level.as_ref().unwrap())
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    }
}
