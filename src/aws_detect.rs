use crate::rules;
use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, s};
use chrono::{DateTime, Utc};
use krapslog::{build_sparkline, build_time_markers};
use sigma_rust::{Event, Rule};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use termcolor::{BufferWriter, ColorChoice, ColorSpec, WriteColor};
use terminal_size::{Width, terminal_size};

pub fn aws_detect(
    directory: &Option<PathBuf>,
    file: &Option<PathBuf>,
    output: &Option<PathBuf>,
    no_color: bool,
    no_frequency: bool,
) {
    let profile = load_profile("config/aws_ct_timeline_default_profile.txt");
    let rules = rules::load_rules_from_dir("rules");
    println!("Total detection rules: {:?}", rules.len());

    let mut wtr = get_writer(output);
    let csv_header: Vec<&str> = profile.iter().map(|(k, _v)| k.as_str()).collect();
    wtr.write_record(&csv_header).unwrap();

    let mut timestamps = vec![];
    let scan_by_all_rules = |event| {
        for rule in &rules {
            if rule.is_match(&event) {
                let record: Vec<String> = profile
                    .iter()
                    .map(|(_k, v)| get_value_from_event(v, &event, rule))
                    .collect();
                wtr.write_record(&record).unwrap();
                if let Some(event_time) = event.get("eventTime") {
                    let event_time_str = s(format!("{:?}", event_time));
                    if let Ok(event_time) = event_time_str.parse::<DateTime<Utc>>() {
                        let unix_time = event_time.timestamp();
                        timestamps.push(unix_time);
                    }
                }
            }
        }
    };

    if let Some(d) = directory {
        process_events_from_dir(scan_by_all_rules, d, output.is_some(), no_color).unwrap();
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        if let Ok(events) = load_json_from_file(&log_contents) {
            events.into_iter().for_each(scan_by_all_rules);
        }
    }
    wtr.flush().ok();

    if !no_frequency {
        let terminal_width = match terminal_size() {
            Some((Width(w), _)) => w as usize,
            None => 100,
        };
        print_timeline_hist(&timestamps, terminal_width, 3);
    }
}

fn print_timeline_hist(timestamps: &[i64], length: usize, side_margin_size: usize) {
    if timestamps.is_empty() {
        return;
    }

    let buf_wtr = BufferWriter::stdout(ColorChoice::Always);
    let mut wtr = buf_wtr.buffer();
    wtr.set_color(ColorSpec::new().set_fg(None)).ok();

    if timestamps.len() < 5 {
        writeln!(
            wtr,
            "Detection Frequency Timeline could not be displayed as there needs to be more than 5 events.",
        )
            .ok();
        buf_wtr.print(&wtr).ok();
        return;
    }

    let title = "Detection Frequency Timeline";
    let header_row_space = (length - title.len()) / 2;
    writeln!(wtr, "{}{}", " ".repeat(header_row_space), title).ok();
    println!();

    let timestamp_marker_max = if timestamps.len() < 2 {
        0
    } else {
        timestamps.len() - 2
    };
    let marker_num = min(timestamp_marker_max, 18);

    let (header_raw, footer_raw) =
        build_time_markers(timestamps, marker_num, length - (side_margin_size * 2));
    let sparkline = build_sparkline(timestamps, length - (side_margin_size * 2), 5_usize);
    for header_str in header_raw.lines() {
        writeln!(wtr, "{}{}", " ".repeat(side_margin_size - 1), header_str).ok();
    }
    for line in sparkline.lines() {
        writeln!(wtr, "{}{}", " ".repeat(side_margin_size - 1), line).ok();
    }
    for footer_str in footer_raw.lines() {
        writeln!(wtr, "{}{}", " ".repeat(side_margin_size - 1), footer_str).ok();
    }
    buf_wtr.print(&wtr).ok();
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
