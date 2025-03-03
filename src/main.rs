use crate::cmd::{Cli, Commands};
use crate::result::s;
use crate::scan::{load_json_from_file, process_events_from_dir, read_gz_file};
use chrono::Local;
use clap::Parser;
use csv::Writer;
use std::fs;
use std::time::Instant;

mod cmd;
mod result;
mod rules;
mod scan;

fn main() {
    let logo = fs::read_to_string("art/logo.txt").unwrap();
    println!("\x1b[38;2;0;255;0m{}\x1b[0m", logo);
    println!();

    let cli = Cli::parse();
    match &cli.cmd {
        Commands::AwsDetect {
            directory,
            file,
            output,
        } => {
            let start = Instant::now();
            println!("Start time: {}\n", Local::now().format("%Y/%m/%d %H:%M"));

            let rules = rules::load_rules_from_dir("rules");
            println!("Total detection rules: {:?}", rules.len());

            let mut wtr = Writer::from_path(output).unwrap();
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
                process_events_from_dir(d, scan_by_all_rules).unwrap();
            } else if let Some(f) = file {
                if f.ends_with(".json") {
                    let log_contents = fs::read_to_string(f).unwrap();
                    let events = load_json_from_file(&log_contents).unwrap();
                    events.into_iter().for_each(scan_by_all_rules);
                } else if f.ends_with(".gz") {
                    let log_contents = read_gz_file(f).unwrap();
                    let events = load_json_from_file(&log_contents).unwrap();
                    events.into_iter().for_each(scan_by_all_rules);
                }
                println!("Scanning finished.");
            }

            let duration = start.elapsed();
            let hours = duration.as_secs() / 3600;
            let minutes = (duration.as_secs() % 3600) / 60;
            let seconds = duration.as_secs() % 60;
            println!("Elapsed time: {:02}:{:02}:{:02}\n", hours, minutes, seconds);
        }
    }
}
