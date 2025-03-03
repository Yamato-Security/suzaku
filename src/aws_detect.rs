use crate::rules;
use crate::scan::{load_json_from_file, process_events_from_dir, read_gz_file};
use csv::Writer;
use std::path::PathBuf;
use std::{fs, io};

pub fn aws_detect(directory: &Option<PathBuf>, file: &Option<PathBuf>, output: &Option<PathBuf>) {
    let rules = rules::load_rules_from_dir("rules");
    println!("Total detection rules: {:?}", rules.len());

    let mut wtr: Writer<Box<dyn io::Write>> = if let Some(output) = output {
        Writer::from_writer(Box::new(fs::File::create(output).unwrap()))
    } else {
        Writer::from_writer(Box::new(io::stdout()))
    };

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
        let log_contents = if f.ends_with(".json") {
            fs::read_to_string(f).unwrap_or_default()
        } else if f.ends_with(".gz") {
            read_gz_file(f).unwrap_or_default()
        } else {
            "".to_string()
        };
        let events = load_json_from_file(&log_contents).unwrap();
        events.into_iter().for_each(scan_by_all_rules);
        println!("Scanning finished.");
    }
}

fn s(input: String) -> String {
    input.replace(r#"Value(String(""#, "").replace(r#""))"#, "")
}
