use crate::rules;
use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_writer, s, write_color_buffer};
use chrono::{DateTime, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Table, TableComponent};
use krapslog::{build_sparkline, build_time_markers};
use sigma_rust::{Event, Rule};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
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
    let mut author_titles: HashMap<String, HashSet<String>> = HashMap::new();
    let scan_by_all_rules = |event| {
        for rule in &rules {
            if rule.is_match(&event) {
                let record: Vec<String> = profile
                    .iter()
                    .map(|(_k, v)| get_value_from_event(v, &event, rule))
                    .collect();
                wtr.write_record(&record).unwrap();
                if let Some(author) = &rule.author {
                    author_titles
                        .entry(author.clone())
                        .or_default()
                        .insert(rule.title.clone());
                }
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
    println!();
    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 100,
    };
    if !no_frequency {
        print_timeline_hist(&timestamps, terminal_width, 3);
    }
    let table_column_num = if terminal_width <= 105 {
        2
    } else if terminal_width < 140 {
        3
    } else if terminal_width < 175 {
        4
    } else if terminal_width <= 210 {
        5
    } else {
        6
    };
    let mut authors_count: HashMap<String, i128> = HashMap::new();
    for (author, rules) in author_titles.iter() {
        let count = rules.len() as i128;
        authors_count.insert(author.clone(), count);
    }
    print_detected_rule_authors(&authors_count, table_column_num);
}

fn print_detected_rule_authors(
    rule_author_counter: &HashMap<String, i128>,
    table_column_num: usize,
) {
    let mut sorted_authors: Vec<(&String, &i128)> = rule_author_counter.iter().collect();
    sorted_authors.sort_by(|a, b| (-a.1).cmp(&(-b.1)));
    let authors_num = sorted_authors.len();
    let div = if authors_num <= table_column_num {
        1
    } else if authors_num % 4 != 0 {
        authors_num / table_column_num + 1
    } else {
        authors_num / table_column_num
    };
    let mut tb = Table::new();
    tb.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_style(TableComponent::VerticalLines, ' ');
    let mut stored_by_column = vec![];
    let hlch = tb.style(TableComponent::HorizontalLines).unwrap();
    let tbch = tb.style(TableComponent::TopBorder).unwrap();
    for x in 0..table_column_num {
        let mut tmp = Vec::new();
        for y in 0..div {
            if y * table_column_num + x < sorted_authors.len() {
                // Limit length to 27 to prevent the table from wrapping
                let filter_author = if sorted_authors[y * table_column_num + x].0.len() <= 27 {
                    sorted_authors[y * table_column_num + x].0.to_string()
                } else {
                    format!("{}...", &sorted_authors[y * table_column_num + x].0[0..24])
                };
                tmp.push(format!(
                    "{} ({})",
                    filter_author,
                    sorted_authors[y * table_column_num + x].1
                ));
            }
        }
        if !tmp.is_empty() {
            stored_by_column.push(tmp);
        }
    }
    let mut output = vec![];
    for col_data in stored_by_column {
        output.push(col_data.join("\n"));
    }
    if !output.is_empty() {
        tb.add_row(output)
            .set_style(TableComponent::MiddleIntersections, hlch)
            .set_style(TableComponent::TopBorderIntersections, tbch)
            .set_style(TableComponent::BottomBorderIntersections, hlch);
    }
    println!("Rule Authors:");
    println!("{tb}");
    println!();
}

fn print_timeline_hist(timestamps: &[i64], length: usize, side_margin_size: usize) {
    if timestamps.is_empty() {
        return;
    }

    let buf_wtr = BufferWriter::stdout(ColorChoice::Always);
    let mut wtr = buf_wtr.buffer();
    wtr.set_color(ColorSpec::new().set_fg(None)).ok();

    if timestamps.len() < 5 {
        let msg = "Detection Frequency Timeline could not be displayed as there needs to be more than 5 events.";
        write_color_buffer(&buf_wtr, Some(Color::Rgb(255, 0, 0)), msg, false).ok();
        write_color_buffer(&buf_wtr, None, "\n", true).ok();
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
    println!();
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
