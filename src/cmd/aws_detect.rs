use crate::cmd::aws_detect_writer::{
    OutputConfig, OutputContext, init_writers, write_correlation_record, write_record,
};
use crate::core::color::SuzakuColor;
use crate::core::color::SuzakuColor::{Cyan, Green, Orange, Red, White, Yellow};
use crate::core::rules;
use crate::core::scan::{append_summary_data, scan_directory, scan_file};
use crate::core::util::{output_path_info, p};
use crate::option::cli::{AwsCtTimelineOptions, CommonOptions};
use crate::option::geoip::GeoIPSearch;
use chrono::{DateTime, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table, TableComponent};
use krapslog::{build_sparkline, build_time_markers};
use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use sigma_rust::{CorrelationEngine, Rule, TimestampedEvent, parse_rules_from_yaml};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use terminal_size::{Width, terminal_size};

#[derive(Debug, Default)]
pub struct DetectionSummary {
    pub author_titles: HashMap<String, HashSet<String>>,
    pub timestamps: Vec<i64>,
    pub total_events: usize,
    pub event_with_hits: usize,
    pub dates_with_hits: HashMap<String, HashMap<String, usize>>,
    pub level_with_hits: HashMap<String, HashMap<String, usize>>,
    pub first_event_time: Option<DateTime<Utc>>,
    pub last_event_time: Option<DateTime<Utc>>,
}

pub fn aws_detect(options: &AwsCtTimelineOptions, common_opt: &CommonOptions) {
    let no_color = common_opt.no_color;
    let mut geo_search = None;
    if let Some(path) = options.geo_ip.as_ref() {
        let res = GeoIPSearch::new(path);
        if let Ok(geo) = res {
            geo_search = Some(geo);
        } else {
            p(
                Red.rdg(no_color),
                "Could not find the appropriate MaxMind GeoIP .mmdb database files.\n",
                true,
            );
            return;
        }
    }
    let profile = load_profile("config/default_profile.yaml", &geo_search);
    let rules: Vec<Rule> = rules::load_rules_from_dir(&options.rules);
    let rules = rules::filter_rules_by_level(&rules, &options.min_level);
    let mut correlation_engine = CorrelationEngine::new();
    let correlation_rules = rules::load_correlation_yamls_from_dir(&options.rules);
    if rules.is_empty() && correlation_rules.is_empty() {
        p(
            Red.rdg(no_color),
            "Suzaku could not load any rules. Please download the rules with the update-rules command.\n",
            true,
        );
        return;
    }
    for yaml in &correlation_rules {
        match parse_rules_from_yaml(yaml.as_str()) {
            Ok(rules) => {
                let (correlation_rules, base_rules) = rules;
                for (name, rule) in base_rules {
                    correlation_engine.add_base_rule(name, rule);
                }
                for rule in correlation_rules {
                    correlation_engine.add_correlation_rule(rule);
                }
            }
            Err(e) => {
                p(
                    Red.rdg(no_color),
                    &format!("Error parsing correlation rule: {e}"),
                    true,
                );
                return;
            }
        }
    }

    p(Green.rdg(no_color), "Total detection rules: ", false);
    p(None, rules.len().to_string().as_str(), true);
    p(Green.rdg(no_color), "Total correlation rules: ", false);
    p(None, correlation_rules.len().to_string().as_str(), true);

    let (writers, output_pathes) = init_writers(options.output.as_ref(), options.output_type);
    let config = OutputConfig::new(no_color, options.raw_output);
    let mut context = OutputContext::new(&profile, &mut geo_search, &config, writers);
    let mut summary = DetectionSummary::default();
    let mut matched_correlation: Vec<TimestampedEvent> = Vec::new();
    context.write_header();

    if let Some(d) = &options.input_opt.directory {
        scan_directory(
            d,
            &mut context,
            &mut summary,
            options,
            &rules,
            &mut matched_correlation,
            &correlation_engine,
        );
    } else if let Some(f) = &options.input_opt.filepath {
        scan_file(
            f,
            &mut context,
            &mut summary,
            options,
            &rules,
            &mut matched_correlation,
            &correlation_engine,
        );
    }

    process_correlation_events(
        &mut context,
        &mut summary,
        &mut matched_correlation,
        &correlation_engine,
    );

    context.flush_all();
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

    print_detected_rule_authors(&authors_count, table_column_num, no_color);

    if !options.no_summary {
        print_summary(&summary, no_color);
    }

    if !output_pathes.is_empty() {
        output_path_info(no_color, &output_pathes);
    }
}

fn process_correlation_events(
    context: &mut OutputContext,
    summary: &mut DetectionSummary,
    matched_correlation: &mut Vec<TimestampedEvent>,
    correlation_engine: &CorrelationEngine,
) -> bool {
    let results = correlation_engine.process_events(matched_correlation);
    match results {
        Ok(results) => {
            for res in results.iter() {
                let rule = res.rule;
                if res.matched {
                    for event in &res.events {
                        let generate = rule.correlation.generate.unwrap_or(false);
                        if generate {
                            write_record(&event.event, &Value::Null, event.rule, context);
                        }
                        summary.event_with_hits += 1;
                        append_summary_data(summary, &event.event, event.rule, generate);
                    }
                    write_correlation_record(&res.events, rule, context);
                    if let Some(author) = &rule.author {
                        summary
                            .author_titles
                            .entry(author.clone())
                            .or_default()
                            .insert(rule.title.clone());
                    }
                    if let Some(level) = &rule.level {
                        let level = level.to_lowercase();
                        summary
                            .level_with_hits
                            .entry(level.clone())
                            .or_default()
                            .entry(rule.title.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                        let event = &res.events.last().unwrap().event;
                        if let Some(event_time) = event.get("eventTime") {
                            let event_time_str = event_time.value_to_string();
                            if let Ok(event_time) = event_time_str.parse::<DateTime<Utc>>() {
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
        }
        Err(e) => {
            p(
                Red.rdg(context.config.no_color),
                &format!("Error processing correlation events: {e}"),
                true,
            );
            return true;
        }
    }
    false
}

fn print_summary(sum: &DetectionSummary, no_color: bool) {
    let levels = if no_color {
        vec![
            ("critical", White),
            ("high", White),
            ("medium", White),
            ("low", White),
            ("informational", White),
        ]
    } else {
        vec![
            ("critical", Red),
            ("high", Orange),
            ("medium", Yellow),
            ("low", Green),
            ("informational", White),
        ]
    };
    print_summary_header(sum, no_color);
    print_summary_levels(sum, &levels);
    print_summary_event_times(sum);
    print_summary_dates_with_hits(sum, &levels);
    print_summary_table(sum, &levels);
}

fn print_summary_header(sum: &DetectionSummary, no_color: bool) {
    p(Green.rdg(no_color), "Results Summary:", true);
    p(None, "", false);
    p(Green.rdg(no_color), "Events with hits", false);
    p(None, " / ", false);
    p(Green.rdg(no_color), "Total events: ", false);
    let msg = sum.event_with_hits.to_formatted_string(&Locale::en);
    p(Yellow.rdg(no_color), msg.as_str(), false);
    p(None, " / ", false);
    let msg = sum.total_events.to_formatted_string(&Locale::en);
    p(Cyan.rdg(no_color), msg.as_str(), false);
    p(None, " (", false);
    p(
        Green.rdg(no_color),
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

fn print_summary_levels(sum: &DetectionSummary, levels: &Vec<(&str, SuzakuColor)>) {
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
            p(color.rdg(false), &msg, true);
        } else {
            let msg = format!("Total | Unique {level} detections: 0 (0%) | 0 (0%)");
            p(color.rdg(false), &msg, true);
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

fn print_summary_dates_with_hits(sum: &DetectionSummary, levels: &Vec<(&str, SuzakuColor)>) {
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
                p(color.rdg(false), &msg, false);
            }
        } else {
            p(color.rdg(false), &format!("{level}: n/a"), false);
        }
        if *level != "informational" {
            p(None, ", ", false);
        }
    }
    println!();
}

fn print_summary_table(sum: &DetectionSummary, levels: &Vec<(&str, SuzakuColor)>) {
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
            table_data.push((*level, (color.rdg(false), msgs)));
        } else {
            let data = vec!["n/a".to_string(); 5];
            table_data.push((*level, (color.rdg(false), data)));
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
            .map(|(level, (color, _))| Cell::new(format!("Top {level} alerts:")).fg(rgb(color)))
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
    no_color: bool,
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
    p(Green.rdg(no_color), "Rule Authors:", true);
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

fn load_profile(file_path: &str, geo_search: &Option<GeoIPSearch>) -> Vec<(String, String)> {
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
            if key == "SrcIP" && geo_search.is_some() {
                profile.push(("SrcASN".to_string(), "SrcASN".to_string()));
                profile.push(("SrcCity".to_string(), "SrcCity".to_string()));
                profile.push(("SrcCountry".to_string(), "SrcCountry".to_string()));
            }
        }
    }
    profile
}
