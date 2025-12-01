use crate::option::cli::TimeOption;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

pub fn filter_by_time(opt: &TimeOption, value: &Value, ts_key: &str) -> bool {
    let keys: Vec<&str> = ts_key
        .split('|')
        .map(|k| k.trim_start_matches('.'))
        .collect();

    let event_time_str = keys
        .iter()
        .find_map(|k| value.get(k.trim()).and_then(|v| v.as_str()));

    let event_time_str = match event_time_str {
        Some(s) => s,
        None => return false,
    };
    let event_time = match ts_key {
        "eventTime" => match DateTime::parse_from_rfc3339(event_time_str) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return false,
        },
        "time" | "eventTimestamp" | "time|.eventTimestamp" => {
            match event_time_str.parse::<DateTime<Utc>>() {
                Ok(dt) => dt,
                Err(_) => return false,
            }
        }
        _ => return false,
    };
    if let Some(start) = &opt.timeline_start {
        let start_time = match DateTime::parse_from_rfc3339(start) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return false,
        };
        if event_time < start_time {
            return false;
        }
    }
    if let Some(end) = &opt.timeline_end {
        let end_time = match DateTime::parse_from_rfc3339(end) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return false,
        };
        if event_time > end_time {
            return false;
        }
    }
    if let Some(offset) = &opt.time_offset {
        let now = Utc::now();
        let duration = match parse_offset(offset) {
            Some(d) => d,
            None => return false,
        };
        if event_time < now - duration {
            return false;
        }
    }
    true
}
fn parse_offset(offset: &str) -> Option<Duration> {
    let (num, unit) = offset.trim().split_at(offset.len() - 1);
    let n: i64 = num.parse().ok()?;
    match unit {
        "y" => Some(Duration::days(n * 365)),
        "M" => Some(Duration::days(n * 30)),
        "d" => Some(Duration::days(n)),
        "h" => Some(Duration::hours(n)),
        "m" => Some(Duration::minutes(n)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_by_time_within_range() {
        let opt = TimeOption {
            timeline_start: Some("2024-08-18T12:00:00Z".to_string()),
            timeline_end: Some("2024-08-18T14:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "eventTime": "2024-08-18T13:00:00Z" });
        assert!(filter_by_time(&opt, &value, "eventTime"));
    }

    #[test]
    fn test_filter_by_time_outside_range() {
        let opt = TimeOption {
            timeline_start: Some("2024-08-18T12:00:00Z".to_string()),
            timeline_end: Some("2024-08-18T14:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "eventTime": "2024-08-18T15:00:00Z" });
        assert!(!filter_by_time(&opt, &value, "eventTime"));
    }

    #[test]
    fn test_filter_by_time_with_offset() {
        let opt = TimeOption {
            timeline_start: None,
            timeline_end: None,
            time_offset: Some("1h".to_string()),
        };
        let value = json!({ "eventTime": (Utc::now() - Duration::minutes(30)).to_rfc3339() });
        assert!(filter_by_time(&opt, &value, "eventTime"));
    }

    #[test]
    fn test_filter_by_time_with_invalid_event_time() {
        let opt = TimeOption {
            timeline_start: Some("2024-08-18T12:00:00Z".to_string()),
            timeline_end: Some("2024-08-18T14:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "eventTime": "invalid-date" });
        assert!(!filter_by_time(&opt, &value, "eventTime"));
    }

    #[test]
    fn test_filter_by_time_eventtime_format() {
        let opt = TimeOption {
            timeline_start: Some("2023-07-10T10:00:00Z".to_string()),
            timeline_end: Some("2023-07-10T12:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "eventTime": "2023-07-10T11:42:36Z" });
        assert!(filter_by_time(&opt, &value, "eventTime"));
    }

    #[test]
    fn test_filter_by_time_time_format() {
        let opt = TimeOption {
            timeline_start: Some("2022-05-13T13:00:00Z".to_string()),
            timeline_end: Some("2022-05-13T14:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "time": "2022-05-13T13:38:44.5454572Z" });
        assert!(filter_by_time(&opt, &value, "time"));
    }

    #[test]
    fn test_filter_by_time_event_timestamp_format() {
        let opt = TimeOption {
            timeline_start: Some("2025-11-30T01:00:00Z".to_string()),
            timeline_end: Some("2025-11-30T02:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "eventTimestamp": "2025-11-30T01:45:06.4650448Z" });
        assert!(filter_by_time(&opt, &value, "eventTimestamp"));
    }
}
