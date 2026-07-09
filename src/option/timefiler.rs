use crate::option::cli::{FileDateOption, TimeOption};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

static DATE_PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[/\\](\d{4})[/\\](\d{2})[/\\](\d{2})[/\\]")
        .expect("DATE_PATH_RE regex pattern is invalid")
});

pub fn filter_by_time(opt: &TimeOption, value: &Value, ts_key: &str) -> bool {
    // With no time constraints, keep every event. Events whose timestamp field is
    // absent or in an unrecognized format must not be silently dropped just
    // because we could not read their time (e.g. M365 UAL uses `CreationTime`).
    if opt.timeline_start.is_none() && opt.timeline_end.is_none() && opt.time_offset.is_none() {
        return true;
    }

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
    let event_time = match parse_event_time(event_time_str) {
        Some(dt) => dt,
        None => return false,
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
/// Filter files by their path date structure (YYYY/MM/DD).
/// Intended for AWSLogs S3-compatible paths like `AWSLogs/{account}/{service}/{region}/YYYY/MM/DD/`.
/// If no date pattern is found in the path (e.g. Azure logs), the file is passed through (returns true).
pub fn filter_file_by_date_path(opt: &FileDateOption, path: &str) -> bool {
    if opt.file_date_from.is_none() && opt.file_date_to.is_none() {
        return true;
    }
    let Some(caps) = DATE_PATH_RE.captures(path) else {
        // No YYYY/MM/DD pattern found; pass through (e.g. Azure paths)
        return true;
    };
    // Compose as YYYYMMDD for direct lexicographic comparison with user input
    let file_date = format!("{}{}{}", &caps[1], &caps[2], &caps[3]);
    if let Some(from) = &opt.file_date_from
        && file_date.as_str() < from.as_str()
    {
        return false;
    }
    if let Some(to) = &opt.file_date_to
        && file_date.as_str() > to.as_str()
    {
        return false;
    }
    true
}

/// Parse an event timestamp string across the formats seen in Azure/M365 logs:
/// RFC3339 (`...Z`/offset), a fractional-second UTC form, and the timezone-less
/// naive form used by M365 Unified Audit Log `CreationTime` (assumed UTC).
fn parse_event_time(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    for fmt in [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
    ] {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc));
        }
    }
    None
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
    use crate::option::cli::FileDateOption;
    use serde_json::json;

    // --- filter_file_by_date_path tests ---

    #[test]
    fn test_filter_file_no_option_passes_through() {
        let opt = FileDateOption::default();
        assert!(filter_file_by_date_path(
            &opt,
            "AWSLogs/123/CloudTrail/us-east-1/2024/01/15/xxx.json.gz"
        ));
    }

    #[test]
    fn test_filter_file_within_range() {
        let opt = FileDateOption {
            file_date_from: Some("20240101".to_string()),
            file_date_to: Some("20240131".to_string()),
        };
        assert!(filter_file_by_date_path(
            &opt,
            "AWSLogs/123/CloudTrail/ap-northeast-1/2024/01/15/xxx.json.gz"
        ));
    }

    #[test]
    fn test_filter_file_before_range() {
        let opt = FileDateOption {
            file_date_from: Some("20240201".to_string()),
            file_date_to: None,
        };
        assert!(!filter_file_by_date_path(
            &opt,
            "AWSLogs/123/CloudTrail/ap-northeast-1/2024/01/15/xxx.json.gz"
        ));
    }

    #[test]
    fn test_filter_file_after_range() {
        let opt = FileDateOption {
            file_date_from: None,
            file_date_to: Some("20240110".to_string()),
        };
        assert!(!filter_file_by_date_path(
            &opt,
            "AWSLogs/123/CloudTrail/ap-northeast-1/2024/01/15/xxx.json.gz"
        ));
    }

    #[test]
    fn test_filter_file_on_boundary_from() {
        let opt = FileDateOption {
            file_date_from: Some("20240115".to_string()),
            file_date_to: None,
        };
        assert!(filter_file_by_date_path(
            &opt,
            "AWSLogs/123/CloudTrail/ap-northeast-1/2024/01/15/xxx.json.gz"
        ));
    }

    #[test]
    fn test_filter_file_on_boundary_to() {
        let opt = FileDateOption {
            file_date_from: None,
            file_date_to: Some("20240115".to_string()),
        };
        assert!(filter_file_by_date_path(
            &opt,
            "AWSLogs/123/CloudTrail/ap-northeast-1/2024/01/15/xxx.json.gz"
        ));
    }

    #[test]
    fn test_filter_file_no_date_pattern_passes_through() {
        // Azure-style path without YYYY/MM/DD structure passes through
        let opt = FileDateOption {
            file_date_from: Some("20240101".to_string()),
            file_date_to: Some("20241231".to_string()),
        };
        assert!(filter_file_by_date_path(
            &opt,
            "/logs/azure/auditlogs_2024_01_15.json"
        ));
    }

    #[test]
    fn test_filter_file_windows_path_separators() {
        // Windows-style path with backslash separators should match correctly
        let opt = FileDateOption {
            file_date_from: Some("20240101".to_string()),
            file_date_to: Some("20240131".to_string()),
        };
        assert!(filter_file_by_date_path(
            &opt,
            r"AWSLogs\123\CloudTrail\us-east-1\2024\01\15\xxx.json.gz"
        ));
    }

    // --- filter_by_time tests ---

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
    fn test_filter_by_time_no_filter_keeps_event_without_timestamp_field() {
        // With no time constraints, an event whose timestamp key is absent (e.g.
        // M365 UAL, which uses CreationTime) must still be kept, not dropped.
        let opt = TimeOption {
            timeline_start: None,
            timeline_end: None,
            time_offset: None,
        };
        let value = json!({ "Operation": "UserLoggedIn" });
        assert!(filter_by_time(
            &opt,
            &value,
            "time|.eventTimestamp|.CreationTime"
        ));
    }

    #[test]
    fn test_filter_by_time_creationtime_naive_format() {
        // M365 UAL CreationTime has no timezone; it is treated as UTC.
        let opt = TimeOption {
            timeline_start: Some("2023-11-21T23:00:00Z".to_string()),
            timeline_end: Some("2023-11-22T00:00:00Z".to_string()),
            time_offset: None,
        };
        let value = json!({ "CreationTime": "2023-11-21T23:44:05" });
        assert!(filter_by_time(
            &opt,
            &value,
            "time|.eventTimestamp|.CreationTime"
        ));
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
