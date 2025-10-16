use crate::core::color::SuzakuColor::{Green, Red};
use crate::core::log_source::LogSource;
use crate::core::rules;
use crate::core::scan::{append_summary_data, scan_directory, scan_file};
use crate::core::summary::{DetectionSummary, print_detected_rule_authors, print_summary};
use crate::core::timeline_writer::{
    OutputConfig, OutputContext, init_writers, write_correlation_record, write_record,
};
use crate::core::util::{output_path_info, p};
use crate::option::cli::{CommonOptions, TimelineOptions};
use crate::option::geoip::GeoIPSearch;
use chrono::{DateTime, Utc};
use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use sigma_rust::{CorrelationEngine, Rule, TimestampedEvent, parse_rules_from_yaml};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use terminal_size::{Width, terminal_size};

pub fn make_timeline(options: &TimelineOptions, common_opt: &CommonOptions, log: LogSource) {
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
    let profile = load_profile(&log, &geo_search);
    let rules: Vec<Rule> = rules::load_rules_from_dir(&options.rules, &log);
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
    for yaml in &correlation_rules {
        match parse_rules_from_yaml(yaml.as_str()) {
            Ok(rules) => {
                let (correlation_rules, base_rules) = rules;
                let total_base_rules = base_rules.len();
                let mut added_base_rules = 0;
                for (name, rule) in base_rules {
                    if let Some(ref service) = rule.logsource.service
                        && log.supported_services().contains(&service.as_str())
                    {
                        correlation_engine.add_base_rule(name, rule);
                        added_base_rules += 1;
                    }
                }
                for rule in correlation_rules {
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
    p(Green.rdg(no_color), "Total correlation rules: ", false);
    p(
        None,
        &total_correlation_rules.to_formatted_string(&Locale::en),
        true,
    );

    let (writers, output_pathes) = init_writers(options.output.as_ref(), options.output_type);
    let config = OutputConfig::new(no_color, options.raw_output);
    let mut context =
        OutputContext::new(&profile, &mut geo_search, &config, writers, &output_pathes);
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
            &log,
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
            &log,
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
                            write_record(&event.event, &Value::Null, event.rule, context);
                        }
                        summary.event_with_hits += 1;
                        append_summary_data(summary, &event.event, event.rule, generate, context);
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
                        if let Some(event_time) = event.get(context.prof_ts_key) {
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

fn load_profile(log: &LogSource, geo_search: &Option<GeoIPSearch>) -> Vec<(String, String)> {
    let file = File::open(log.profile_path()).expect("Unable to open profile file");
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
