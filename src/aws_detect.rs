use crate::cmd::{AwsCtTimelineOptions, CommonOptions};
use crate::rules;
use crate::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::util::{get_json_writer, get_writer, p, s};
use chrono::{DateTime, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table, TableComponent};
use csv::Writer;
use krapslog::{build_sparkline, build_time_markers};
use num_format::{Locale, ToFormattedString};
use sigma_rust::{Event, Rule};
use std::cmp::min;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use terminal_size::{Width, terminal_size};

#[derive(Debug, Default)]
struct DetectionSummary {
    author_titles: HashMap<String, HashSet<String>>,
    timestamps: Vec<i64>,
    total_events: usize,
    event_with_hits: usize,
    dates_with_hits: HashMap<String, HashMap<String, usize>>,
    level_with_hits: HashMap<String, HashMap<String, usize>>,
    first_event_time: Option<DateTime<Utc>>,
    last_event_time: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub enum OutputType {
    Csv,
    Json,
    Jsonl,
    CsvAndJson,
    CsvAndJsonl,
}

impl OutputType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(OutputType::Csv),
            2 => Some(OutputType::Json),
            3 => Some(OutputType::Jsonl),
            4 => Some(OutputType::CsvAndJson),
            5 => Some(OutputType::CsvAndJsonl),
            _ => None,
        }
    }
}

fn write_record(
    profile: &[(String, String)],
    event: &Event,
    rule: &Rule,
    csv_writer: &mut Option<Writer<Box<dyn Write>>>,
    json_writer: &mut Option<BufWriter<Box<dyn Write>>>,
    jsonl_writer: &mut Option<BufWriter<Box<dyn Write>>>,
    std_writer: &mut Option<BufferWriter>,
) {
    let record: Vec<String> = profile
        .iter()
        .map(|(_k, v)| get_value_from_event(v, event, rule))
        .collect();

    // 標準出力
    if let Some(writer) = std_writer {
        let level = &record[2];
        let color = if level == "critical" {
            Color::Rgb(255, 0, 0)
        } else if level == "high" {
            Color::Rgb(255, 175, 0)
        } else if level == "medium" {
            Color::Rgb(255, 255, 0)
        } else if level == "low" {
            Color::Rgb(0, 255, 0)
        } else {
            Color::Rgb(255, 255, 255)
        };
        let mut buf = writer.buffer();
        for (i, col) in record.iter().enumerate() {
            if i == 0 || i == 1 || i == 2 {
                buf.set_color(ColorSpec::new().set_fg(Some(color))).ok();
            } else {
                buf.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(255, 255, 255))))
                    .ok();
            }
            write!(buf, "{}", col).ok();
            if i != record.len() - 1 {
                buf.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(255, 175, 0))))
                    .ok();
                write!(buf, " · ").ok();
            }
        }
        write!(buf, "\n\n").ok();
        writer.print(&buf).ok();
    }

    // CSV出力
    if let Some(writer) = csv_writer {
        writer.write_record(&record).unwrap();
    }

    // JSON出力
    if let Some(writer) = json_writer {
        let mut json_record: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in profile {
            let value = get_value_from_event(v, event, rule);
            json_record.insert(k.clone(), value.to_string());
        }
        let rec = serde_json::to_string_pretty(&json_record);
        if let Ok(json_string) = rec {
            writer.write_all(json_string.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
    }

    // JSONL出力
    if let Some(writer) = jsonl_writer {
        let mut json_record: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in profile {
            let value = get_value_from_event(v, event, rule);
            json_record.insert(k.clone(), value.to_string());
        }
        if let Ok(json_string) = serde_json::to_string(&json_record) {
            writer.write_all(json_string.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
    }
}

pub fn aws_detect(options: &AwsCtTimelineOptions, common_opt: &CommonOptions) {
    let profile = load_profile("config/default_profile.yaml");
    let rules = rules::load_rules_from_dir(&options.rules);
    p(
        Some(Color::Rgb(0, 255, 0)),
        "Total detection rules: ",
        false,
    );
    p(None, rules.len().to_string().as_str(), true);

    let mut std_out = None;
    let mut csv_writer = None;
    let mut json_writer = None;
    let mut jsonl_writer = None;

    if let Some(output_path) = &options.output {
        let output_type = OutputType::from_u8(options.output_type).unwrap_or(OutputType::Csv);
        match output_type {
            OutputType::Csv | OutputType::CsvAndJson | OutputType::CsvAndJsonl => {
                let mut csv_path = output_path.clone();
                if csv_path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
                    csv_path.set_extension("csv");
                }
                csv_writer = Some(get_writer(&Some(csv_path)));
            }
            _ => {}
        }
        match output_type {
            OutputType::Json | OutputType::CsvAndJson => {
                let mut json_path = output_path.clone();
                if json_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    json_path.set_extension("json");
                }
                json_writer = Some(get_json_writer(&Some(json_path)));
            }
            OutputType::Jsonl | OutputType::CsvAndJsonl => {
                let mut jsonl_path = output_path.clone();
                if jsonl_path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                    jsonl_path.set_extension("jsonl");
                }
                jsonl_writer = Some(get_json_writer(&Some(jsonl_path)));
            }
            _ => {}
        }
    } else {
        let disp_wtr = BufferWriter::stdout(ColorChoice::Always);
        let mut disp_wtr_buf = disp_wtr.buffer();
        disp_wtr_buf
            .set_color(ColorSpec::new().set_fg(Some(Color::Rgb(0, 255, 0))))
            .ok();
        std_out = Some(disp_wtr);
    }

    if let Some(ref mut std_out) = std_out {
        let csv_header: Vec<&str> = profile.iter().map(|(k, _v)| k.as_str()).collect();
        let mut buf = std_out.buffer();
        buf.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(0, 255, 0))))
            .ok();
        writeln!(buf, "{}", csv_header.join(" · ")).ok();
    }

    if let Some(ref mut writer) = csv_writer {
        let csv_header: Vec<&str> = profile.iter().map(|(k, _v)| k.as_str()).collect();
        writer.write_record(&csv_header).unwrap();
    }

    let mut summary = DetectionSummary::default();
    let scan_by_all_rules = |event| {
        summary.total_events += 1;
        let mut counted = false;
        for rule in &rules {
            if rule.is_match(&event) {
                if !counted {
                    summary.event_with_hits += 1;
                    counted = true;
                }
                write_record(
                    &profile,
                    &event,
                    rule,
                    &mut csv_writer,
                    &mut json_writer,
                    &mut jsonl_writer,
                    &mut std_out,
                );

                if let Some(author) = &rule.author {
                    summary
                        .author_titles
                        .entry(author.clone())
                        .or_default()
                        .insert(rule.title.clone());
                }

                if let Some(level) = &rule.level {
                    let level = format!("{:?}", level).to_lowercase();
                    summary
                        .level_with_hits
                        .entry(level)
                        .or_default()
                        .entry(rule.title.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }

                if let Some(event_time) = event.get("eventTime") {
                    let event_time_str = s(format!("{:?}", event_time));
                    if let Ok(event_time) = event_time_str.parse::<DateTime<Utc>>() {
                        let unix_time = event_time.timestamp();
                        summary.timestamps.push(unix_time);
                        if summary.first_event_time.is_none()
                            || event_time < summary.first_event_time.unwrap()
                        {
                            summary.first_event_time = Some(event_time);
                        }
                        if summary.last_event_time.is_none()
                            || event_time > summary.last_event_time.unwrap()
                        {
                            summary.last_event_time = Some(event_time);
                        }
                        if let Some(level) = &rule.level {
                            let level = format!("{:?}", level).to_lowercase();
                            let date = event_time.date_naive().format("%Y-%m-%d").to_string();
                            summary
                                .dates_with_hits
                                .entry(level)
                                .or_default()
                                .entry(date)
                                .and_modify(|e| *e += 1)
                                .or_insert(1);
                        }
                    }
                }
            }
        }
    };

    if let Some(d) = &options.input_opt.directory {
        process_events_from_dir(
            scan_by_all_rules,
            d,
            options.output.is_some(),
            common_opt.no_color,
        )
        .unwrap();
    } else if let Some(f) = &options.input_opt.filepath {
        let log_contents = get_content(f);
        if let Ok(events) = load_json_from_file(&log_contents) {
            events.into_iter().for_each(scan_by_all_rules);
        }
    }
    if let Some(ref mut writer) = csv_writer {
        writer.flush().unwrap();
    }
    if let Some(ref mut writer) = json_writer {
        writer.flush().unwrap();
    }
    if let Some(ref mut writer) = jsonl_writer {
        writer.flush().unwrap();
    }
    println!();
    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 100,
    };
    if !options.no_frequency {
        print_timeline_hist(&summary.timestamps, terminal_width, 3);
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

    let authors_count: HashMap<String, i128> = summary
        .author_titles
        .iter()
        .map(|(author, rules)| (author.clone(), rules.len() as i128))
        .collect();

    print_detected_rule_authors(&authors_count, table_column_num);

    if !options.no_summary {
        print_summary(&summary);
    }
}

fn print_summary(sum: &DetectionSummary) {
    let levels = vec![
        ("critical", Some(Color::Rgb(255, 0, 0))),
        ("high", Some(Color::Rgb(255, 175, 0))),
        ("medium", Some(Color::Rgb(255, 255, 0))),
        ("low", Some(Color::Rgb(0, 255, 0))),
        ("informational", None),
    ];
    print_summary_header(sum);
    print_summary_levels(sum, &levels);
    print_summary_event_times(sum);
    print_summary_dates_with_hits(sum, &levels);
    print_summary_table(sum, &levels);
}

fn print_summary_header(sum: &DetectionSummary) {
    p(Some(Color::Rgb(0, 255, 0)), "Results Summary:", true);
    p(None, "", false);
    p(Some(Color::Rgb(255, 255, 0)), "Events with hits", false);
    p(None, " / ", false);
    p(Some(Color::Rgb(0, 255, 255)), "Total events: ", false);
    p(
        Some(Color::Rgb(255, 255, 0)),
        &sum.event_with_hits.to_formatted_string(&Locale::en),
        false,
    );
    p(None, " / ", false);
    p(
        Some(Color::Rgb(0, 255, 255)),
        &sum.total_events.to_formatted_string(&Locale::en),
        false,
    );
    p(None, " (", false);
    p(
        Some(Color::Rgb(0, 255, 0)),
        &format!(
            "Data reduction: {} events ({:.2}%)",
            (sum.total_events - sum.event_with_hits).to_formatted_string(&Locale::en),
            (sum.total_events - sum.event_with_hits) as f64 * 100.0 / sum.total_events as f64
        ),
        false,
    );
    p(None, ")", false);
    println!();
}

fn print_summary_levels(sum: &DetectionSummary, levels: &Vec<(&str, Option<Color>)>) {
    for (level, color) in levels {
        if let Some(hits) = sum.level_with_hits.get(*level) {
            let uniq_hits = hits.keys().len();
            let total_hits: usize = hits.values().sum();
            let msg = format!(
                "Total | Unique {} detections: {} ({:.2}%) | {} ({:.2}%)",
                level,
                total_hits.to_formatted_string(&Locale::en),
                total_hits * 100 / sum.event_with_hits,
                uniq_hits,
                uniq_hits * 100 / sum.event_with_hits
            );
            p(*color, &msg, true);
        } else {
            let msg = format!("Total | Unique {} detections: 0 (0%) | 0 (0%)", level);
            p(*color, &msg, true);
        }
    }
    println!();
}

fn print_summary_event_times(sum: &DetectionSummary) {
    if let Some(first_event_time) = sum.first_event_time {
        p(None, "First event time: ", false);
        p(None, &first_event_time.to_string(), true);
    }
    if let Some(last_event_time) = sum.last_event_time {
        p(None, "Last event time: ", false);
        p(None, &last_event_time.to_string(), true);
    }
    println!();
}

fn print_summary_dates_with_hits(sum: &DetectionSummary, levels: &Vec<(&str, Option<Color>)>) {
    p(None, "Dates with most total detections:", true);
    for (level, color) in levels {
        if let Some(dates) = sum.dates_with_hits.get(*level) {
            if let Some((date, &max_hits)) = dates.iter().max_by_key(|&(_, &count)| count) {
                let msg = format!(
                    "{}: {} ({})",
                    level,
                    date,
                    max_hits.to_formatted_string(&Locale::en)
                );
                p(*color, &msg, false);
            }
        } else {
            p(*color, &format!("{}: n/a", level), false);
        }
        if *level != "informational" {
            p(None, ", ", false);
        }
    }
    println!();
}

fn print_summary_table(sum: &DetectionSummary, levels: &Vec<(&str, Option<Color>)>) {
    let mut table_data = vec![];
    for (level, color) in levels {
        if let Some(hits) = sum.level_with_hits.get(*level) {
            let mut hits_vec: Vec<(&String, &usize)> = hits.iter().collect();
            hits_vec.sort_by(|a, b| b.1.cmp(a.1));
            let top_hits: Vec<(&String, &usize)> = hits_vec.into_iter().take(5).collect();
            let mut msgs: Vec<String> = top_hits
                .into_iter()
                .map(|(rule, count)| {
                    format!("{} ({})", rule, count.to_formatted_string(&Locale::en))
                })
                .collect();
            while msgs.len() < 5 {
                msgs.push("n/a".to_string());
            }
            table_data.push((*level, (*color, msgs)));
        } else {
            let data = vec!["n/a".to_string(); 5];
            table_data.push((*level, (*color, data)));
        }
    }
    let mut tb = Table::new();
    tb.load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_style(TableComponent::VerticalLines, ' ');
    let hlch = tb.style(TableComponent::HorizontalLines).unwrap();
    let tbch = tb.style(TableComponent::TopBorder).unwrap();
    for chunk in table_data.chunks(2) {
        let heads = chunk
            .iter()
            .map(|(level, (color, _))| Cell::new(format!("Top {} alerts:", level)).fg(rgb(color)))
            .collect::<Vec<_>>();
        let columns = chunk
            .iter()
            .map(|(_, (color, msgs))| {
                let msg = msgs
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                Cell::new(msg).fg(rgb(color))
            })
            .collect::<Vec<_>>();
        tb.add_row(heads)
            .set_style(TableComponent::MiddleIntersections, hlch)
            .set_style(TableComponent::TopBorderIntersections, tbch)
            .set_style(TableComponent::BottomBorderIntersections, hlch);
        tb.add_row(columns);
    }
    println!("{tb}");
    println!();
}

fn rgb(color: &Option<Color>) -> comfy_table::Color {
    match color {
        Some(Color::Rgb(255, 0, 0)) => comfy_table::Color::Rgb { r: 255, g: 0, b: 0 },
        Some(Color::Rgb(255, 175, 0)) => comfy_table::Color::Rgb {
            r: 255,
            g: 175,
            b: 0,
        },
        Some(Color::Rgb(255, 255, 0)) => comfy_table::Color::Rgb {
            r: 255,
            g: 255,
            b: 0,
        },
        Some(Color::Rgb(0, 255, 0)) => comfy_table::Color::Rgb { r: 0, g: 255, b: 0 },
        _ => comfy_table::Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
    }
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
    p(Some(Color::Rgb(0, 255, 0)), "Rule Authors:", true);
    p(None, &format!("{tb}"), true);
    println!();
}

fn print_timeline_hist(timestamps: &[i64], length: usize, side_margin_size: usize) {
    if timestamps.is_empty() {
        return;
    }
    if timestamps.len() < 5 {
        let msg = "Detection Frequency Timeline could not be displayed as there needs to be more than 5 events.";
        p(Some(Color::Rgb(255, 0, 0)), msg, false);
        p(None, "\n", true);
        return;
    }

    let title = "Detection Frequency Timeline";
    let header_row_space = (length - title.len()) / 2;
    let buf_wtr = BufferWriter::stdout(ColorChoice::Always);
    let mut wtr = buf_wtr.buffer();
    wtr.set_color(ColorSpec::new().set_fg(None)).ok();
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
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val = parts[1].trim().trim_matches('\'');
            profile.push((key.to_string(), val.to_string()));
        }
    }
    profile
}

fn get_value_from_event(key: &str, event: &Event, rule: &Rule) -> String {
    if key.starts_with(".") {
        let key = key.strip_prefix(".").unwrap();
        if let Some(value) = event.get(key) {
            if key == "eventTime" {
                s(format!("{:?}", value)).replace("T", " ").replace("Z", "")
            } else {
                s(format!("{:?}", value))
            }
        } else {
            "-".to_string()
        }
    } else if key.starts_with("sigma.") {
        let key = key.replace("sigma.", "");
        if key == "title" {
            rule.title.to_string()
        } else if key == "level" {
            format!("{:?}", rule.level.as_ref().unwrap()).to_lowercase()
        } else {
            "-".to_string()
        }
    } else {
        "-".to_string()
    }
}
