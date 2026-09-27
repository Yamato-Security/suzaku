use crate::core::color::SuzakuColor::{Green, Red};
use crate::core::duckdb_out::SuzakuMeta;
use crate::core::errorlog::log_error;
use crate::core::log_source::LogSource;
use crate::core::rules;
use crate::core::scan::{append_summary_data, scan_directory, scan_file};
use crate::core::summary::{DetectionSummary, print_detected_rule_authors, print_summary};
use crate::core::timeline_writer::{
    OutputConfig, OutputContext, correlation_timestamp, init_writers, write_correlation_record,
    write_record,
};
use crate::core::util::{fatal_error, load_profile, output_path_info, p};
use crate::option::cli::{CommonOptions, TimelineOptions};
use crate::option::geoip::GeoIPSearch;
use chrono::{DateTime, Utc};
use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use sigma_rust::{
    CorrelationEngine, Rule, SigmaCorrelationRule, TimestampedEvent, parse_rules_from_yaml,
};
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};

pub fn make_timeline(options: &TimelineOptions, common_opt: &CommonOptions, log: LogSource) {
    let no_color = common_opt.no_color;
    let mut geo_search = None;
    if let Some(path) = options.output_opt.geo_ip.as_ref() {
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
    let profile = load_profile(&log, &geo_search, false);
    let rules: Vec<Rule> = rules::load_rules_from_dir(&options.rules, &log);
    // Skip rules listed in <rules-dir>/config/<log>_ignore_rule_list.txt (superseded/duplicate
    // rules that stay in the repo but should not be loaded).
    let ignore_ids =
        rules::load_ignore_rule_ids(&rules::ignore_rule_list_path(&options.rules, &log));
    let loaded_rule_count = rules.len();
    let rules: Vec<Rule> = rules::filter_ignored_rules(rules, &ignore_ids);
    let ignored_rule_count = loaded_rule_count - rules.len();
    let rules = rules::filter_rules_by_level(&rules, &options.min_level);
    let correlation_rules = rules::load_correlation_yamls_from_dir(&options.rules);
    if rules.is_empty() && correlation_rules.is_empty() {
        p(
            Red.rdg(no_color),
            "Suzaku could not load any rules. Please download the rules with the update-rules command.\n",
            true,
        );
        return;
    }
    let mut correlation_engine = CorrelationEngine::new();
    let mut total_correlation_rules = 0;
    let mut ignored_correlation_count = 0;
    for yaml in &correlation_rules {
        match parse_rules_from_yaml(yaml.as_str()) {
            Ok(rules) => {
                let (correlation_rules, base_rules) = rules;
                let total_base_rules = base_rules.len();
                let mut added_base_rules = 0;
                for (name, rule) in base_rules {
                    // Skip ignore-listed base rules; the correlation that depends on them is
                    // then skipped too (added_base_rules never reaches total_base_rules).
                    if rules::is_ignored(rule.id.as_deref(), &ignore_ids) {
                        ignored_correlation_count += 1;
                        continue;
                    }
                    if let Some(ref service) = rule.logsource.service
                        && log.supported_services().contains(&service.as_str())
                    {
                        correlation_engine.add_base_rule(name, rule);
                        added_base_rules += 1;
                    }
                }
                for rule in correlation_rules {
                    if rules::is_ignored(rule.id.as_deref(), &ignore_ids) {
                        ignored_correlation_count += 1;
                        continue;
                    }
                    if added_base_rules == total_base_rules {
                        correlation_engine.add_correlation_rule(rule);
                        total_correlation_rules += 1;
                    }
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
    let total_ignored = ignored_rule_count + ignored_correlation_count;
    if total_ignored > 0 {
        p(
            Green.rdg(no_color),
            "Rules skipped via ignore-list: ",
            false,
        );
        p(None, total_ignored.to_string().as_str(), true);
    }
    p(Green.rdg(no_color), "Total correlation rules: ", false);
    p(
        None,
        &total_correlation_rules.to_formatted_string(&Locale::en),
        true,
    );

    // Provenance for the DuckDB output: which command, which ruleset, which timezone. A DFIR
    // report has to be able to cite these, and today they are unknowable from the file.
    let meta = SuzakuMeta::new(log.command_name())
        .with_localtime(options.localtime)
        .with_geoip(geo_search.is_some())
        .with_rules(&options.rules, rules.len() + total_correlation_rules);
    let (writers, output_pathes) = init_writers(
        options.output_opt.output.as_ref(),
        &options.output_opt.output_types,
        &profile,
        meta,
    )
    .unwrap_or_else(|e| fatal_error(no_color, &e));
    let config = OutputConfig::new(
        no_color,
        options.output_opt.raw_output,
        options.localtime,
        options.sort,
    );
    let mut context =
        OutputContext::new(&profile, &mut geo_search, &config, writers, &output_pathes);
    let mut summary = DetectionSummary::default();
    let mut matched_correlation: Vec<TimestampedEvent> = Vec::new();
    context.write_header();

    let scanned_files = if let Some(d) = &options.input_opt.directory {
        scan_directory(
            d,
            &mut context,
            &mut summary,
            options,
            &rules,
            &mut matched_correlation,
            &correlation_engine,
            &log,
        )
    } else if let Some(f) = &options.input_opt.filepath {
        scan_file(
            f,
            &mut context,
            &mut summary,
            options,
            &rules,
            &mut matched_correlation,
            &correlation_engine,
            &log,
        );
        1
    } else {
        0
    };

    process_correlation_events(
        &mut context,
        &mut summary,
        &mut matched_correlation,
        &correlation_engine,
    );

    context.set_scan_stats(
        Some(scanned_files as i64),
        Some(summary.total_events as i64),
    );
    context.flush_all();
    println!();
    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        None => 100,
    };
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
        output_path_info(no_color, &output_pathes, context.has_written);
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
                            write_record(&event.event, &Value::Null, Some(event.rule), context);
                        }
                        // `event_with_hits` is not touched here: these events were already
                        // counted once in the scan pass (they matched a base rule there), and a
                        // single event reaches this loop once per firing correlation result it
                        // belongs to, so incrementing here double-counted it.
                        append_summary_data(summary, &event.event, event.rule, generate, context);
                    }
                    // One representative instant per result, shared by the row, the summary
                    // and the `--sort` position so they cannot disagree.
                    let timestamp = correlation_timestamp(&res.events);
                    write_correlation_record(&res.events, rule, timestamp, context);
                    record_correlation_summary(summary, rule, timestamp);
                }
            }
        }
        Err(e) => {
            log_error(&format!("Error processing correlation events: {e}"));
            return true;
        }
    }
    false
}

/// Adds one fired correlation result to the summary: its author, its level, and a detection on
/// the UTC date of its representative `timestamp` (see [`correlation_timestamp`]). The date stays
/// UTC under `--localtime`, like every other date in the summary. A result without a timestamp
/// still counts towards its level but is left out of the per-day table.
fn record_correlation_summary(
    summary: &mut DetectionSummary,
    rule: &SigmaCorrelationRule,
    timestamp: Option<DateTime<Utc>>,
) {
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
        if let Some(event_time) = timestamp {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn high_rule() -> SigmaCorrelationRule {
        SigmaCorrelationRule {
            title: "c".to_string(),
            level: Some("High".to_string()),
            ..Default::default()
        }
    }

    // A correlation whose members straddle midnight UTC counts on the day of its latest member,
    // the same instant its row shows — not on whichever member the scan reached last.
    #[test]
    fn correlation_summary_counts_the_utc_day_of_the_representative_time() {
        let mut summary = DetectionSummary::default();
        record_correlation_summary(
            &mut summary,
            &high_rule(),
            Some(utc("2024-01-02T00:01:00Z")),
        );
        // A +09:00 spelling of an instant on 2024-01-01 UTC stays on 2024-01-01.
        record_correlation_summary(
            &mut summary,
            &high_rule(),
            Some(utc("2024-01-02T08:00:00+09:00")),
        );
        let days = &summary.dates_with_hits["high"];
        assert_eq!(days.get("2024-01-02"), Some(&1));
        assert_eq!(days.get("2024-01-01"), Some(&1));
        assert_eq!(summary.level_with_hits["high"]["c"], 2);
    }

    #[test]
    fn correlation_summary_without_a_time_counts_the_level_but_no_day() {
        let mut summary = DetectionSummary::default();
        record_correlation_summary(&mut summary, &high_rule(), None);
        assert_eq!(summary.level_with_hits["high"]["c"], 1);
        assert!(summary.dates_with_hits.is_empty());
    }
}
