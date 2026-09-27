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

/// `count` as a percentage of `total`, in floating point so fractions of a percent survive.
///
/// Integer division (`count * 100 / total`) truncated every share below 1% to `0`, and because
/// `Display` for integers ignores the precision option, `{:.2}` printed that as `0` rather than
/// `0.00`. Guarded against an empty denominator so the summary never prints `NaN%`.
fn percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 * 100.0 / total as f64
    }
}

/// Compute the "data reduction" count and percentage for the summary header.
///
/// The subtraction is saturating as a last line of defence: `event_with_hits` is counted once
/// per source event during the scan pass, so it should never exceed `total_events`, but an
/// underflow here panics in debug builds and wraps to ~1.8e19 in release.
fn data_reduction(total_events: usize, event_with_hits: usize) -> (usize, f64) {
    let reduction = total_events.saturating_sub(event_with_hits);
    (reduction, percentage(reduction, total_events))
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

/// Totals used as the denominators of the per-level percentages: every detection, and every
/// unique rule that fired.
fn detection_totals(sum: &DetectionSummary) -> (usize, usize) {
    let total: usize = sum.level_with_hits.values().flat_map(|h| h.values()).sum();
    let uniq: usize = sum.level_with_hits.values().map(|h| h.len()).sum();
    (total, uniq)
}

fn print_summary_levels(sum: &DetectionSummary, levels: &Vec<(&str, SuzakuColor)>) {
    // Each column is a share of its own total: this level's detections out of all detections,
    // and this level's unique rules out of all unique rules that fired. The old denominator for
    // both was the event count, which made the "Unique" share (a handful of rules over millions
    // of events) round to 0% for every level.
    let (total_detections, total_uniq) = detection_totals(sum);
    for (level, color) in levels {
        if let Some(hits) = sum.level_with_hits.get(*level) {
            let uniq_hits = hits.keys().len();
            let total_hits: usize = hits.values().sum();
            let msg = format!(
                "Total | Unique {} detections: {} ({:.2}%) | {} ({:.2}%)",
                level,
                total_hits.to_formatted_string(&Locale::en),
                percentage(total_hits, total_detections),
                uniq_hits.to_formatted_string(&Locale::en),
                percentage(uniq_hits, total_uniq)
            );
            p(color.rdg(false), &msg, true);
        } else {
            let msg = format!("Total | Unique {level} detections: 0 (0.00%) | 0 (0.00%)");
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

/// The date with the most detections. On a tie the earliest date wins, so the answer does not
/// depend on `HashMap` iteration order, which differs between identical runs.
fn busiest_date(dates: &HashMap<String, usize>) -> Option<(&String, usize)> {
    dates
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(date, &count)| (date, count))
}

/// The `n` rules with the most hits, most first. Equal counts are ordered by title: sorting on the
/// count alone left ties — and which of them made the cut — in `HashMap` order, so identical runs
/// printed different tables.
fn top_hits(hits: &HashMap<String, usize>, n: usize) -> Vec<(&String, usize)> {
    let mut hits: Vec<(&String, usize)> = hits.iter().map(|(rule, &count)| (rule, count)).collect();
    hits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    hits.truncate(n);
    hits
}

/// Rule authors by number of detected rules, most first, equal counts by name — for the same
/// reason as [`top_hits`].
fn sort_authors(counter: &HashMap<String, i128>) -> Vec<(&String, i128)> {
    let mut authors: Vec<(&String, i128)> = counter.iter().map(|(a, &n)| (a, n)).collect();
    authors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    authors
}

fn print_summary_dates_with_hits(sum: &DetectionSummary, levels: &Vec<(&str, SuzakuColor)>) {
    p(None, "Dates with most total detections:", true);
    for (level, color) in levels {
        if let Some(dates) = sum.dates_with_hits.get(*level) {
            if let Some((date, max_hits)) = busiest_date(dates) {
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
            let mut msgs: Vec<String> = top_hits(hits, 5)
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
    let sorted_authors = sort_authors(rule_author_counter);
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
    fn percentage_keeps_fractions_below_one_percent() {
        // The integer division this replaced truncated 0.51% to 0, and `{:.2}` on an integer
        // prints `0`, not `0.00` — so every sub-1% level was reported as "(0%)".
        assert!((percentage(11_807, 2_332_963) - 0.506_0).abs() < 0.001);
        assert_eq!(format!("{:.2}", percentage(11_807, 2_332_963)), "0.51");
        assert_eq!(percentage(1, 4), 25.0);
        // Empty denominator: no 0/0 NaN.
        assert_eq!(percentage(0, 0), 0.0);
    }

    #[test]
    fn detection_totals_sum_over_all_levels() {
        let mut sum = DetectionSummary::default();
        sum.level_with_hits.insert(
            "critical".to_string(),
            HashMap::from([("rule a".to_string(), 10usize)]),
        );
        sum.level_with_hits.insert(
            "low".to_string(),
            HashMap::from([("rule b".to_string(), 20usize), ("rule c".to_string(), 70)]),
        );
        // 3 unique rules firing 100 times in total; the unique share is per unique rule, so
        // critical is 1/3 rather than the ~0% the event-count denominator produced.
        assert_eq!(detection_totals(&sum), (100, 3));
        assert_eq!(percentage(10, 100), 10.0);
        assert!((percentage(1, 3) - 33.333).abs() < 0.01);
    }

    /// The same entries inserted in different orders. `HashMap` iteration order also depends on
    /// the per-map random seed, so several maps are built; with a count-only sort at least one of
    /// them came out in a different order.
    fn shuffled_maps<V: Copy>(entries: &[(&str, V)]) -> Vec<HashMap<String, V>> {
        (0..16)
            .map(|i| {
                let mut v = entries.to_vec();
                v.rotate_left(i % entries.len());
                if i % 2 == 1 {
                    v.reverse();
                }
                v.into_iter().map(|(k, n)| (k.to_string(), n)).collect()
            })
            .collect()
    }

    #[test]
    fn top_hits_breaks_count_ties_by_title() {
        let entries = [
            ("S3 Enum", 2usize),
            ("Many Recon Events", 9),
            ("AWS CloudWatchLogs CreateLogStream", 2),
            ("Attempt To Stop Logging", 2),
            ("AWS STS AssumeRole", 5),
            ("Zeta", 2),
            ("Alpha", 2),
        ];
        for hits in shuffled_maps(&entries) {
            let top: Vec<(&str, usize)> = top_hits(&hits, 5)
                .into_iter()
                .map(|(r, n)| (r.as_str(), n))
                .collect();
            // The cut at five falls inside the tie on 2: the first three titles by name make it.
            assert_eq!(
                top,
                [
                    ("Many Recon Events", 9),
                    ("AWS STS AssumeRole", 5),
                    ("AWS CloudWatchLogs CreateLogStream", 2),
                    ("Alpha", 2),
                    ("Attempt To Stop Logging", 2),
                ]
            );
        }
    }

    #[test]
    fn busiest_date_prefers_the_earliest_on_a_tie() {
        let entries = [("2024-01-03", 7usize), ("2024-01-01", 7), ("2024-01-02", 3)];
        for dates in shuffled_maps(&entries) {
            let (date, count) = busiest_date(&dates).unwrap();
            assert_eq!((date.as_str(), count), ("2024-01-01", 7));
        }
        assert_eq!(busiest_date(&HashMap::new()), None);
    }

    #[test]
    fn sort_authors_breaks_count_ties_by_name() {
        let entries = [("carol", 1i128), ("alice", 3), ("bob", 1), ("dave", 3)];
        for counter in shuffled_maps(&entries) {
            let sorted: Vec<(&str, i128)> = sort_authors(&counter)
                .into_iter()
                .map(|(a, n)| (a.as_str(), n))
                .collect();
            assert_eq!(
                sorted,
                [("alice", 3), ("dave", 3), ("bob", 1), ("carol", 1)]
            );
        }
    }

    #[test]
    fn data_reduction_handles_double_count_and_empty() {
        // Normal case.
        assert_eq!(data_reduction(100, 5), (95, 95.0));
        // event_with_hits > total_events must not underflow. The correlation double-count
        // that produced this is fixed, so the guard is defence in depth.
        assert_eq!(data_reduction(1, 2), (0, 0.0));
        // Empty dataset: no 0/0 NaN.
        let (n, pct) = data_reduction(0, 0);
        assert_eq!(n, 0);
        assert!(pct.is_finite() && pct == 0.0);
    }
}
