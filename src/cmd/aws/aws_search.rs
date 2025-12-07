use crate::core::color::SuzakuColor::Green;
use crate::core::log_source::LogSource;
use crate::core::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::core::timeline_writer::{OutputConfig, OutputContext, init_writers, write_record};
use crate::core::util::{load_profile, output_path_info, p};
use crate::option::cli::{CommonOptions, SearchOptions};
use crate::option::timefiler::filter_by_time;
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use serde_json::Value;
use sigma_rust::{event_from_json, rule_from_yaml};

pub fn aws_search(options: &SearchOptions, common_opt: &CommonOptions) {
    let no_color = common_opt.no_color;
    let directory = &options.input_opt.directory;
    let file = &options.input_opt.filepath;

    let mut matched_events = 0;
    let mut total_events = 0;

    let profile = load_profile(&LogSource::Aws, &None, true);
    let (writers, output_pathes) = init_writers(
        options.output_opt.output.as_ref(),
        options.output_opt.output_type,
    );
    let config = OutputConfig::new(no_color, options.output_opt.raw_output);
    let none = &mut None;
    let mut context = OutputContext::new(&profile, none, &config, writers, &output_pathes);

    let filter_conditions = parse_filter_conditions(&options.filter);

    let regex_pattern = options
        .regex
        .as_ref()
        .map(|pattern| Regex::new(pattern).expect("Invalid regex pattern"));

    let sigma_rule_content = r"title: Dummy rule for Search command
id: 2a2466e1-c0da-434b-a327-57da46cc8dae
status: test
description: ''
author: YamatoSecurity
date: 2025-12-24
logsource:
    product: aws
    service: cloudtrail
detection:
    selection:
        eventName: 'sample'
    condition: selection
level: informational";
    let rule = rule_from_yaml(sigma_rule_content).unwrap();

    context.write_header();

    let mut search_func = |json_values: &[Value]| {
        for json_value in json_values {
            total_events += 1;
            if !filter_by_time(&options.input_opt.time_opt, json_value, "eventTime") {
                continue;
            }

            if !filter_conditions.is_empty() && !matches_filters(json_value, &filter_conditions) {
                continue;
            }

            if !options.keyword.is_empty() {
                let json_str = json_value.to_string();
                let found = if options.preserve_case {
                    options.keyword.iter().any(|k| json_str.contains(k))
                } else {
                    let json_str_lower = json_str.to_lowercase();
                    options
                        .keyword
                        .iter()
                        .any(|k| json_str_lower.contains(&k.to_lowercase()))
                };
                if !found {
                    continue;
                }
            }

            if let Some(ref pattern) = regex_pattern {
                let json_str = json_value.to_string();
                if !pattern.is_match(&json_str) {
                    continue;
                }
            }
            let event = event_from_json(json_value.to_string().as_str());
            if let Ok(event) = event {
                write_record(&event, json_value, Some(&rule), &mut context);
                matched_events += 1;
            }
        }
    };

    if let Some(d) = directory {
        process_events_from_dir(
            search_func,
            d,
            options.output_opt.output.is_some(),
            no_color,
            &LogSource::Aws,
        )
        .unwrap();
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents, &LogSource::Aws);
        if let Ok(events) = events {
            search_func(&events);
        }
    }

    display_results(matched_events, total_events, no_color);
    if !output_pathes.is_empty() {
        output_path_info(no_color, &output_pathes, context.has_written);
    }
}

fn parse_filter_conditions(filters: &[String]) -> Vec<(String, String)> {
    let mut conditions = Vec::new();
    for filter in filters {
        if let Some((field, value)) = filter.split_once(':') {
            let value = value.trim_matches('"').trim_matches('\'');
            conditions.push((field.to_string(), value.to_string()));
        }
    }
    conditions
}

fn matches_filters(event: &Value, filters: &[(String, String)]) -> bool {
    for (field, expected_value) in filters {
        let actual_value = get_nested_value(event, field);
        match actual_value {
            Some(val) => {
                let val_str = match val {
                    Value::String(s) => s.clone(),
                    _ => val.to_string().trim_matches('"').to_string(),
                };
                if val_str != *expected_value {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

fn get_nested_value(event: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.trim_start_matches('.').split('.').collect();
    let mut current = event;

    for part in parts {
        match current.get(part) {
            Some(val) => current = val,
            None => return None,
        }
    }

    Some(current.clone())
}

fn display_results(matched_events: usize, total_events: usize, no_color: bool) {
    p(Green.rdg(no_color), "Total events scanned: ", false);
    p(None, &total_events.to_formatted_string(&Locale::en), true);
    p(Green.rdg(no_color), "Matching events: ", false);
    p(None, &matched_events.to_formatted_string(&Locale::en), true);
    println!();

    if matched_events == 0 {
        p(None, "No matching events found.", true);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_event(event_name: &str, user_arn: &str, event_time: &str) -> Value {
        json!({
            "eventVersion": "1.08",
            "userIdentity": {
                "type": "Root",
                "principalId": "123456789012",
                "arn": user_arn,
                "accountId": "123456789012"
            },
            "eventTime": event_time,
            "eventSource": "iam.amazonaws.com",
            "eventName": event_name,
            "awsRegion": "us-east-1",
            "sourceIPAddress": "192.168.1.1",
            "requestParameters": {},
            "responseElements": null
        })
    }

    #[test]
    fn test_parse_filter_conditions() {
        let filters = vec![
            ".eventName:CreateUser".to_string(),
            ".awsRegion:us-east-1".to_string(),
            "nested.field:value".to_string(),
        ];

        let conditions = parse_filter_conditions(&filters);

        assert_eq!(conditions.len(), 3);
        assert_eq!(conditions[0].0, ".eventName");
        assert_eq!(conditions[0].1, "CreateUser");
        assert_eq!(conditions[1].0, ".awsRegion");
        assert_eq!(conditions[1].1, "us-east-1");
    }

    #[test]
    fn test_parse_filter_conditions_with_quotes() {
        let filters = vec![r#".userIdentity.arn:"arn:aws:iam::123456:user/test""#.to_string()];

        let conditions = parse_filter_conditions(&filters);

        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0].0, ".userIdentity.arn");
        assert_eq!(conditions[0].1, "arn:aws:iam::123456:user/test");
    }

    #[test]
    fn test_get_nested_value() {
        let event = json!({
            "eventName": "CreateUser",
            "userIdentity": {
                "type": "Root",
                "arn": "arn:aws:iam::123456:root"
            },
            "nested": {
                "deep": {
                    "value": "test"
                }
            }
        });

        // Test simple field
        let value = get_nested_value(&event, "eventName");
        assert_eq!(value, Some(json!("CreateUser")));

        // Test nested field
        let value = get_nested_value(&event, "userIdentity.type");
        assert_eq!(value, Some(json!("Root")));

        // Test deeply nested field
        let value = get_nested_value(&event, "nested.deep.value");
        assert_eq!(value, Some(json!("test")));

        // Test with leading dot
        let value = get_nested_value(&event, ".userIdentity.arn");
        assert_eq!(value, Some(json!("arn:aws:iam::123456:root")));

        // Test non-existent field
        let value = get_nested_value(&event, "nonexistent");
        assert_eq!(value, None);
    }

    #[test]
    fn test_matches_filters_single_field() {
        let event = create_test_event(
            "CreateUser",
            "arn:aws:iam::123456789012:root",
            "2021-07-05T13:03:37Z",
        );

        let filters = vec![("eventName".to_string(), "CreateUser".to_string())];

        assert!(matches_filters(&event, &filters));

        let filters = vec![("eventName".to_string(), "DeleteUser".to_string())];

        assert!(!matches_filters(&event, &filters));
    }

    #[test]
    fn test_matches_filters_nested_field() {
        let event = create_test_event(
            "CreateUser",
            "arn:aws:iam::123456789012:root",
            "2021-07-05T13:03:37Z",
        );

        let filters = vec![("userIdentity.type".to_string(), "Root".to_string())];

        assert!(matches_filters(&event, &filters));

        let filters = vec![("userIdentity.type".to_string(), "IAMUser".to_string())];

        assert!(!matches_filters(&event, &filters));
    }

    #[test]
    fn test_matches_filters_multiple_conditions() {
        let event = create_test_event(
            "CreateUser",
            "arn:aws:iam::123456789012:root",
            "2021-07-05T13:03:37Z",
        );

        let filters = vec![
            ("eventName".to_string(), "CreateUser".to_string()),
            ("awsRegion".to_string(), "us-east-1".to_string()),
        ];

        assert!(matches_filters(&event, &filters));

        let filters = vec![
            ("eventName".to_string(), "CreateUser".to_string()),
            ("awsRegion".to_string(), "eu-west-1".to_string()),
        ];

        assert!(!matches_filters(&event, &filters));
    }

    #[test]
    fn test_matches_filters_with_arn() {
        let event = create_test_event(
            "CreateUser",
            "arn:aws:iam::143434273843:user/christophe",
            "2021-07-05T13:03:37Z",
        );

        let filters = vec![(
            "userIdentity.arn".to_string(),
            "arn:aws:iam::143434273843:user/christophe".to_string(),
        )];

        assert!(matches_filters(&event, &filters));
    }

    #[test]
    fn test_parse_filter_conditions_empty() {
        let filters: Vec<String> = vec![];
        let conditions = parse_filter_conditions(&filters);
        assert_eq!(conditions.len(), 0);
    }

    #[test]
    fn test_matches_filters_empty() {
        let event = create_test_event(
            "CreateUser",
            "arn:aws:iam::123456789012:root",
            "2021-07-05T13:03:37Z",
        );

        let filters: Vec<(String, String)> = vec![];
        assert!(matches_filters(&event, &filters));
    }

    #[test]
    fn test_get_nested_value_array() {
        let event = json!({
            "items": [
                {"name": "item1"},
                {"name": "item2"}
            ]
        });

        let value = get_nested_value(&event, "items");
        assert!(value.is_some());
        assert!(value.unwrap().is_array());
    }

    #[test]
    fn test_matches_filters_numeric_value() {
        let event = json!({
            "eventName": "CreateUser",
            "responseElements": {
                "user": {
                    "userId": 12345
                }
            }
        });

        // Numeric values should be stringified for comparison
        let filters = vec![(
            "responseElements.user.userId".to_string(),
            "12345".to_string(),
        )];

        assert!(matches_filters(&event, &filters));
    }

    #[test]
    fn test_matches_filters_boolean_value() {
        let event = json!({
            "eventName": "CreateUser",
            "readOnly": true
        });

        let filters = vec![("readOnly".to_string(), "true".to_string())];

        assert!(matches_filters(&event, &filters));
    }

    #[test]
    fn test_parse_filter_invalid_format() {
        let filters = vec!["invalidformat".to_string(), "another:valid:one".to_string()];

        let conditions = parse_filter_conditions(&filters);

        // Only the valid one with a colon should be parsed
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0].0, "another");
        assert_eq!(conditions[0].1, "valid:one");
    }
}
