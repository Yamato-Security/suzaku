use crate::cmd::aws_detect::DetectionSummary;
use crate::cmd::aws_detect_writer::{OutputContext, write_record};
use crate::core::color::SuzakuColor::{Green, Orange};
use crate::core::util::p;
use crate::option::cli::{AwsCtTimelineOptions, TimeOption};
use crate::option::timefiler::filter_by_time;
use bytesize::ByteSize;
use chrono::{DateTime, Utc};
use colored::Colorize;
use console::style;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use num_format::{Locale, ToFormattedString};
use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use serde_json::Value;
use sigma_rust::{CorrelationEngine, Event, Rule, TimestampedEvent, event_from_json};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

pub fn scan_file<'a>(
    f: &PathBuf,
    context: &mut OutputContext<'a>,
    summary: &mut DetectionSummary,
    options: &AwsCtTimelineOptions,
    rules: &Vec<&Rule>,
    matched_correlation: &mut Vec<TimestampedEvent<'a>>,
    correlation_engine: &'a CorrelationEngine,
) {
    let log_contents = get_content(f);
    let events = match load_json_from_file(&log_contents) {
        Ok(value) => value,
        Err(_e) => return,
    };

    detect_events(
        &events,
        context,
        summary,
        options,
        rules,
        matched_correlation,
        correlation_engine,
    );
}

pub fn scan_directory<'a>(
    d: &PathBuf,
    context: &mut OutputContext<'a>,
    summary: &mut DetectionSummary,
    options: &AwsCtTimelineOptions,
    rules: &Vec<&Rule>,
    matched_correlation: &mut Vec<TimestampedEvent<'a>>,
    correlation_engine: &'a CorrelationEngine,
) {
    let no_color = context.config.no_color;
    let process_events = |events: &[Value]| {
        detect_events(
            events,
            context,
            summary,
            options,
            rules,
            matched_correlation,
            correlation_engine,
        );
    };
    process_events_from_dir(process_events, d, options.output.is_some(), no_color).unwrap();
}

pub fn process_events_from_dir<F>(
    mut process_events: F,
    directory: &PathBuf,
    show_progress: bool,
    no_color: bool,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&[Value]),
{
    let (count, file_paths, total_size) = count_files_recursive(directory)?;
    let size = ByteSize::b(total_size).display().to_string();

    p(Green.rdg(no_color), "Total log files: ", false);
    p(None, &count.to_formatted_string(&Locale::en), true);
    p(Green.rdg(no_color), "Total file size: ", false);
    p(None, size.to_string().as_str(), true);
    println!();

    p(Orange.rdg(no_color), "Scanning now. Please wait.", true);
    println!();

    let template = if no_color {
        "[{elapsed_precise}] {human_pos} / {human_len} {spinner} [{bar:40}] {percent}%\r\n\r\n{msg}"
            .to_string()
    } else {
        format!(
            "[{{elapsed_precise}}] {{human_pos}} / {{human_len}} {} [{}] {{percent}}%\r\n\r\n{{msg}}",
            "{spinner}".truecolor(0, 255, 0),
            "{bar:40}".truecolor(0, 255, 0)
        )
    };
    let pb_style = ProgressStyle::with_template(&template)
        .unwrap()
        .progress_chars("=> ");
    let pb =
        ProgressBar::with_draw_target(Some(count as u64), ProgressDrawTarget::stdout_with_hz(10))
            .with_tab_width(55);
    pb.set_style(pb_style);
    if show_progress {
        pb.enable_steady_tick(Duration::from_millis(300));
    }

    for path in file_paths {
        if show_progress {
            let size = fs::metadata(&path).unwrap().len();
            let size = ByteSize::b(size).display().to_string();
            let pb_msg = format!("{path} ({size})");
            pb.set_message(pb_msg);
        }
        let log_contents = if path.ends_with("json") {
            fs::read_to_string(&path)?
        } else if path.ends_with("gz") {
            read_gz_file(&PathBuf::from(&path))?
        } else {
            pb.inc(1);
            continue;
        };

        let events = log_contents_to_events(&log_contents);
        process_events(&events);

        if show_progress {
            pb.inc(1);
        }
    }
    if show_progress {
        if no_color {
            pb.finish_with_message("Scanning finished.\n");
        } else {
            pb.finish_with_message(style("Scanning finished.\n").color256(214).to_string());
        }
    }
    Ok(())
}

fn log_contents_to_events(log_contents: &str) -> Vec<Value> {
    let json_value: Result<Value, _> = serde_json::from_str(log_contents);
    match json_value {
        Ok(json_value) => {
            match json_value {
                Value::Array(json_array) => json_array,
                Value::Object(mut json_map) => {
                    // use json_map.remove to get json_array
                    if let Some(json_array) = json_map.remove("Records") {
                        match json_array {
                            Value::Array(json_array) => json_array,
                            _ => vec![],
                        }
                    } else {
                        vec![]
                    }
                }
                _ => {
                    // TODO: Handle unexpected JSON structure
                    vec![]
                }
            }
        }
        Err(_) => {
            // TODO: Handle unexpected JSON structure
            vec![]
        }
    }
}

fn detect_events<'a>(
    events: &[Value],
    context: &mut OutputContext<'a>,
    summary: &mut DetectionSummary,
    options: &AwsCtTimelineOptions,
    rules: &Vec<&Rule>,
    matched_correlation: &mut Vec<TimestampedEvent<'a>>,
    engine: &'a CorrelationEngine,
) {
    // If all the events are loaded at once, it can consume too much memory.
    // To avoid the problem, we split the events into chunks.
    const CHUNK_SIZE: usize = 1000;
    for event_chunks in events.chunks(CHUNK_SIZE) {
        // Convert loaded events into JSON
        // I call the collect() function at the end of this block due to a lifetime issue of json_event.
        // The ownership of json_event's reference is going to be moved in the next code block, so I ensure that the lifetime of json_event is longer than the next code block.
        let repeated_time_opt: rayon::iter::RepeatN<&TimeOption> =
            rayon::iter::repeat(&options.input_opt.time_opt).take(events.len());
        let json_events: Vec<(&Value, Event)> = event_chunks
            .par_iter()
            .zip(repeated_time_opt.into_par_iter())
            .filter_map(|(event, time_opt)| {
                if filter_by_time(time_opt, event) {
                    Some(event)
                } else {
                    None
                }
            })
            .filter_map(|event| match event_from_json(event.to_string().as_str()) {
                Ok(json_event) => Some((event, json_event)),
                Err(_) => None,
            })
            .collect();

        // conduct rule's matches and return pairs of json_event and matched_rules
        let results: Vec<(&Value, &Event, Vec<&Rule>)> = json_events
            .par_iter()
            .map(|(event, json_event)| {
                let matched_rules: Vec<&Rule> = rules
                    .par_iter()
                    .filter(move |rule| rule.is_match(json_event))
                    .map(|rule| *rule)
                    .collect();
                (*event, json_event, matched_rules)
            })
            .collect();

        // perform post-processing
        // calculate some statistics values
        summary.event_with_hits += results
            .iter()
            .filter(|(_, _, matched_rules)| !matched_rules.is_empty())
            .count();
        summary.total_events += json_events.len();

        // The post-processing contains codes that shouldn't be executed in parallel, like setting values to variable summary, so please don't use rayon here.
        for (event, json_event, matched_rules) in results {
            for rule in matched_rules {
                // write to console
                write_record(json_event, event, rule, context);
                append_summary_data(summary, json_event, rule, true);
            }
        }

        // process correlation base rules
        let base_rule_matched: Vec<TimestampedEvent> =
            process_correlation_base_rule(engine, json_events);
        matched_correlation.extend(base_rule_matched);
    }
}

fn process_correlation_base_rule<'a>(
    engine: &'a CorrelationEngine,
    json_events: Vec<(&Value, Event)>,
) -> Vec<TimestampedEvent<'a>> {
    json_events
        .par_iter()
        .flat_map(|(_, event)| {
            engine
                .base_rules
                .values()
                .filter_map(|rule| {
                    if rule.is_match(event)
                        && let Some(timestamp_field) = event.get("eventTime")
                    {
                        let ts = timestamp_field.value_to_string();
                        if let Ok(parsed_time) = DateTime::parse_from_rfc3339(&ts) {
                            let utc_time = parsed_time.with_timezone(&Utc);
                            return Some(TimestampedEvent {
                                event: event.clone(),
                                timestamp: utc_time,
                                rule,
                            });
                        }
                    }
                    None
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn append_summary_data(
    summary: &mut DetectionSummary,
    event: &Event,
    rule: &Rule,
    generate: bool,
) {
    // add information to summary
    if generate {
        if let Some(author) = &rule.author {
            summary
                .author_titles
                .entry(author.clone())
                .or_default()
                .insert(rule.title.clone());
        }

        if let Some(level) = &rule.level {
            let level = format!("{level:?}").to_lowercase();
            summary
                .level_with_hits
                .entry(level)
                .or_default()
                .entry(rule.title.clone())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }
    if let Some(event_time) = event.get("eventTime") {
        let event_time_str = event_time.value_to_string();
        if let Ok(event_time) = event_time_str.parse::<DateTime<Utc>>() {
            let unix_time = event_time.timestamp();
            summary.timestamps.push(unix_time);
            if summary.first_event_time.is_none() || event_time < summary.first_event_time.unwrap()
            {
                summary.first_event_time = Some(event_time);
            }
            if summary.last_event_time.is_none() || event_time > summary.last_event_time.unwrap() {
                summary.last_event_time = Some(event_time);
            }
            if let Some(level) = &rule.level
                && generate
            {
                let level = format!("{level:?}").to_lowercase();
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

fn count_files_recursive(directory: &PathBuf) -> Result<(usize, Vec<String>, u64), Box<dyn Error>> {
    let mut count = 0;
    let mut paths = Vec::new();
    let mut total_size = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str())
                && (ext == "json" || ext == "gz")
            {
                count += 1;
                total_size += fs::metadata(&path)?.len();
                paths.push(path.to_str().unwrap().to_string());
            }
        } else if path.is_dir() {
            let (sub_count, sub_paths, sub_size) = count_files_recursive(&path)?;
            count += sub_count;
            total_size += sub_size;
            paths.extend(sub_paths);
        }
    }
    Ok((count, paths, total_size))
}

pub fn read_gz_file(file_path: &PathBuf) -> io::Result<String> {
    let file = File::open(file_path)?;
    let mut decoder = GzDecoder::new(BufReader::new(file));
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    Ok(contents)
}
pub fn load_json_from_file(log_contents: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut events = Vec::new();
    let json_value: Value = serde_json::from_str(log_contents)?;
    match json_value {
        Value::Array(json_array) => {
            for json_value in json_array {
                events.push(json_value);
            }
        }
        Value::Object(json_map) => {
            if let Some(json_array) = json_map.get("Records") {
                for json_value in json_array.as_array().unwrap() {
                    events.push(json_value.clone());
                }
            }
        }
        _ => {
            eprintln!("Unexpected JSON structure in file:");
        }
    }
    Ok(events)
}

pub fn get_content(f: &PathBuf) -> String {
    let path = f.display().to_string();
    if path.ends_with(".json") {
        fs::read_to_string(f).unwrap_or_default()
    } else if path.ends_with(".gz") {
        read_gz_file(f).unwrap_or_default()
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_event_from_file() {
        let test_file = "test_files/json/DeleteTrail.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 1);
    }

    #[test]
    fn test_load_event_from_file_record() {
        let test_file = "test_files/json/test.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 29);
    }
}
