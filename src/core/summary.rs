use crate::core::color::SuzakuColor::{Cyan, Green, Orange, Red, White, Yellow};
use crate::core::color::{SuzakuColor, rgb};
use crate::core::util::p;
use chrono::{DateTime, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table, TableComponent};
use num_format::{Locale, ToFormattedString};
use std::collections::{HashMap, HashSet};

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

pub fn print_summary(sum: &DetectionSummary, no_color: bool) {
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

pub fn print_detected_rule_authors(
    rule_author_counter: &HashMap<String, i128>,
    table_column_num: usize,
    no_color: bool,
) {
    let mut sorted_authors: Vec<(&String, &i128)> = rule_author_counter.iter().collect();
    sorted_authors.sort_by(|a, b| (-a.1).cmp(&(-b.1)));
    let authors_num = sorted_authors.len();
    let div = if authors_num <= table_column_num {
        1
    } else if authors_num.is_multiple_of(4) {
        authors_num / table_column_num
    } else {
        authors_num / table_column_num + 1
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
