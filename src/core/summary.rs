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

/// Compute the "data reduction" count and percentage for the summary header.
///
/// `event_with_hits` can exceed `total_events` when correlation results re-count
/// events already tallied by the base scan, so the subtraction is saturating and
/// the percentage is guarded against an empty dataset (0/0) — otherwise the
/// header panics on subtract-overflow in debug builds, wraps to ~1.8e19 in
/// release, or prints `NaN%` on empty input.
fn data_reduction(total_events: usize, event_with_hits: usize) -> (usize, f64) {
    let reduction = total_events.saturating_sub(event_with_hits);
    let pct = if total_events == 0 {
        0.0
    } else {
        reduction as f64 * 100.0 / total_events as f64
    };
    (reduction, pct)
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
    let (reduction, reduction_pct) = data_reduction(sum.total_events, sum.event_with_hits);
    p(
        Green.rdg(no_color),
        &format!(
            "Data reduction: {} events ({:.2}%)",
            reduction.to_formatted_string(&Locale::en),
            reduction_pct
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

/// Truncates a rule author name to at most 27 characters for the summary table.
///
/// Slicing by byte index panics ("byte index N is not a char boundary") when a multibyte
/// UTF-8 codepoint straddles the cut — routine for Japanese/CJK and accented author names.
/// Counting and truncating by `chars()` keeps the ~27-char display budget and never slices
/// mid-codepoint.
fn truncate_author(name: &str) -> String {
    if name.chars().count() <= 27 {
        name.to_string()
    } else {
        format!("{}...", name.chars().take(24).collect::<String>())
    }
}

pub fn print_detected_rule_authors(
    rule_author_counter: &HashMap<String, i128>,
    table_column_num: usize,
    no_color: bool,
) {
    let mut sorted_authors: Vec<(&String, &i128)> = rule_author_counter.iter().collect();
    sorted_authors.sort_by_key(|a| -a.1);
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
                let filter_author = truncate_author(sorted_authors[y * table_column_num + x].0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_author_short_name_unchanged() {
        // <= 27 chars is returned verbatim, including short multibyte names.
        assert_eq!(truncate_author("Zach Mathis"), "Zach Mathis");
        assert_eq!(truncate_author("山本太郎"), "山本太郎");
    }

    #[test]
    fn truncate_author_long_ascii_is_truncated_by_chars() {
        let out = truncate_author(&"a".repeat(40));
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), 27); // 24 chars + "..."
    }

    #[test]
    fn truncate_author_multibyte_does_not_panic() {
        // 22 ASCII + 10 three-byte kanji => 32 chars, and byte index 24 lands in the
        // interior of the first kanji, so the old `&name[0..24]` byte slice panicked with
        // "byte index 24 is not a char boundary". The char-based version must not panic.
        let name = format!("{}{}", "x".repeat(22), "あ".repeat(10));
        assert!(!name.is_char_boundary(24)); // reproduces the exact panic condition
        let out = truncate_author(&name); // must not panic
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), 27);
    }

    #[test]
    fn data_reduction_handles_double_count_and_empty() {
        // Normal case.
        assert_eq!(data_reduction(100, 5), (95, 95.0));
        // event_with_hits > total_events (correlation double-count): no underflow.
        assert_eq!(data_reduction(1, 2), (0, 0.0));
        // Empty dataset: no 0/0 NaN.
        let (n, pct) = data_reduction(0, 0);
        assert_eq!(n, 0);
        assert!(pct.is_finite() && pct == 0.0);
    }
}
