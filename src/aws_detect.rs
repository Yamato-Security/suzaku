use crate::rules;
use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, s};
use std::path::PathBuf;

pub fn aws_detect(directory: &Option<PathBuf>, file: &Option<PathBuf>, output: &Option<PathBuf>) {
    let rules = rules::load_rules_from_dir("rules");
    println!("Total detection rules: {:?}", rules.len());

    let mut wtr = get_writer(output);
    let csv_header = vec![
        "eventTime",
        "Rule Title",
        "Level",
        "awsRegion",
        "eventName",
        "eventSource",
        "eventID",
    ];
    wtr.write_record(csv_header).unwrap();

    let scan_by_all_rules = |event| {
        for rule in &rules {
            if rule.is_match(&event) {
                let record = vec![
                    s(format!("{:?}", event.get("eventTime").unwrap())),
                    rule.title.to_string(),
                    format!("{:?}", rule.level.as_ref().unwrap()),
                    s(format!("{:?}", event.get("awsRegion").unwrap())),
                    s(format!("{:?}", event.get("eventName").unwrap())),
                    s(format!("{:?}", event.get("eventSource").unwrap())),
                    s(format!("{:?}", event.get("eventID").unwrap())),
                ];
                wtr.write_record(&record).unwrap();
            }
        }
    };
    if let Some(d) = directory {
        process_events_from_dir(d, output.is_some(), scan_by_all_rules).unwrap();
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            events.into_iter().for_each(scan_by_all_rules);
        }
    }
    wtr.flush().ok();
}
