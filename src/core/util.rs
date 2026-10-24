use crate::core::color::SuzakuColor::{Green, Red};
use crate::core::errorlog::log_error;
use crate::core::log_source::LogSource;
use crate::option::geoip::GeoIPSearch;
use bytesize::ByteSize;
use csv::Writer;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs, io};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub fn get_writer(output: &Option<PathBuf>) -> Result<Writer<Box<dyn Write>>, String> {
    let wtr: Writer<Box<dyn io::Write>> = if let Some(output) = output {
        let file = fs::File::create(output)
            .map_err(|e| format!("Cannot write to output file {}: {e}", output.display()))?;
        Writer::from_writer(Box::new(file))
    } else {
        Writer::from_writer(Box::new(io::stdout()))
    };
    Ok(wtr)
}

pub fn get_json_writer(output: &Option<PathBuf>) -> Result<BufWriter<Box<dyn Write>>, String> {
    if let Some(output) = output {
        let file = File::create(output)
            .map_err(|e| format!("Cannot write to output file {}: {e}", output.display()))?;
        Ok(BufWriter::new(Box::new(file)))
    } else {
        Ok(BufWriter::new(Box::new(std::io::stdout())))
    }
}

/// Neutralizes CSV/spreadsheet formula injection (CWE-1236) for a single field.
///
/// Excel, LibreOffice Calc and Google Sheets treat any cell whose text begins with `=`, `+`,
/// `-`, `@`, a tab (0x09) or a carriage return (0x0D) as a formula. Suzaku's CSV fields carry
/// attacker-influenceable cloud-log values (`userAgent`, principal names, error strings, ...),
/// so such a value would be evaluated when an analyst opens the report. The `csv` crate only
/// escapes delimiters/quotes/newlines, so it does not prevent this. Prefix any dangerous value
/// with a single apostrophe, which spreadsheets treat as a "force text" marker and hide on
/// display. Use this for CSV output ONLY — never for JSON/JSONL or stdout.
pub fn sanitize_csv_field(field: &str) -> String {
    if field
        .chars()
        .next()
        .is_some_and(|c| matches!(c, '=' | '+' | '-' | '@' | '\t' | '\r'))
    {
        let mut s = String::with_capacity(field.len() + 1);
        s.push('\'');
        s.push_str(field);
        s
    } else {
        field.to_string()
    }
}

/// Prints a run-ending error in red. Every message the user sees just before suzaku
/// gives up — a bad path, an unusable combination of options, a missing rules folder,
/// a failed rule update — goes through here or `fatal_error`, so they all look the
/// same instead of some being red and some being the default terminal color, and so
/// they all honor `--no-color`.
pub fn error_msg(no_color: bool, msg: &str) {
    p(Red.rdg(no_color), msg, true);
}

/// Reports that the single input file given with `-f` could not be read as a log. Unlike a
/// skipped file in a `-d` scan this stays on the terminal as well as going to the error log: the
/// run has nothing else to process, so hiding it in a file would leave an empty result with no
/// visible reason.
pub fn report_unreadable_input(no_color: bool, path: &std::path::Path, e: &dyn std::fmt::Display) {
    let msg = format!("Could not read {}: {e}", path.display());
    log_error(&msg);
    error_msg(no_color, &msg);
}

/// Prints a fatal error in red and exits with a non-zero status, so that
/// input/filesystem failures end the run with a clean, actionable message
/// instead of a Rust panic and backtrace. Unlike ordinary warnings this stays on
/// the terminal as well as going to the error log: it is the last thing the run
/// prints, so hiding it in a file would leave the user with a silent exit.
pub fn fatal_error(no_color: bool, msg: &str) -> ! {
    log_error(msg);
    error_msg(no_color, msg);
    std::process::exit(1);
}

pub fn check_path_exists(
    filepath: Option<PathBuf>,
    dirpath: Option<PathBuf>,
    no_color: bool,
) -> bool {
    if let Some(file) = filepath {
        if !file.exists() {
            error_msg(no_color, &format!("File {file:?} does not exist."));
            return false;
        }
        if !file.is_file() {
            error_msg(
                no_color,
                &format!(
                    "Path {file:?} is not a file (it may be a directory or special file type)."
                ),
            );
            return false;
        }
    }

    if let Some(dir) = dirpath {
        if !dir.exists() {
            error_msg(no_color, &format!("Directory {dir:?} does not exist."));
            return false;
        }
        if !dir.is_dir() {
            error_msg(
                no_color,
                &format!(
                    "Path {dir:?} is not a directory (it may be a file or special file type)."
                ),
            );
            return false;
        }
    }
    true
}

pub fn p(color: Option<Color>, msg: &str, newline: bool) {
    let wtr = BufferWriter::stdout(ColorChoice::Always);
    let mut buf = wtr.buffer();
    buf.set_color(ColorSpec::new().set_fg(color)).ok();
    if newline {
        writeln!(buf, "{msg}").ok();
    } else {
        write!(buf, "{msg}").ok();
    }
    wtr.print(&buf).ok();
}

pub fn output_path_info(no_color: bool, output_paths: &[PathBuf], has_detect: bool) {
    p(Green.rdg(no_color), "Results saved: ", false);
    if !has_detect {
        p(None, "None", true);
        return;
    }
    for (i, path) in output_paths.iter().enumerate() {
        if let Ok(metadata) = path.metadata() {
            let size = ByteSize::b(metadata.len()).display();
            p(None, &format!("{} ({})", path.display(), size), false);
        }
        if i < output_paths.len() - 1 {
            p(None, " and ", false);
        }
    }
    println!();
}

/**
 * Set the global thread number for rayon.
 * @param val The number of threads.
 */
pub fn set_rayon_threat_number(val: usize) {
    if val == 0 {
        return;
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(val)
        .build_global()
        .unwrap();
}

/// Insert or update a per-key `(count, first_seen, last_seen)` entry, tracking the first/last
/// event time seen for THAT key. Shared by `aws-ct-summary` and `aws-ct-metrics` so both report
/// the same per-value time window. Times are compared as strings, which is correct for the
/// RFC 3339 UTC timestamps CloudTrail writes (`2024-01-02T03:04:05Z`) because they are fixed
/// width and lexicographically ordered.
pub fn upsert_count_entry(
    map: &mut HashMap<String, (usize, String, String)>,
    key: String,
    event_time: &str,
) {
    let entry = map
        .entry(key)
        .or_insert_with(|| (0, event_time.to_string(), event_time.to_string()));
    entry.0 += 1;
    if event_time < entry.1.as_str() {
        entry.1 = event_time.to_string();
    }
    if event_time > entry.2.as_str() {
        entry.2 = event_time.to_string();
    }
}

pub fn load_profile(
    log: &LogSource,
    geo_search: &Option<GeoIPSearch>,
    skip_sigma: bool,
) -> Vec<(String, String)> {
    let file = File::open(log.profile_path()).expect("Unable to open profile file");
    let reader = BufReader::new(file);
    let mut profile = vec![];

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val = parts[1].trim().trim_matches('\'');
            if skip_sigma && val.contains("sigma") {
                continue;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_csv_field_neutralizes_formula_triggers() {
        // Leading formula characters get an apostrophe prefix.
        for f in ["=1+1", "+cmd", "-2+3", "@SUM(A1)", "\tx", "\rx"] {
            let out = sanitize_csv_field(f);
            assert!(out.starts_with('\''), "{f:?} -> {out:?}");
            assert_eq!(&out[1..], f);
        }
    }

    #[test]
    fn sanitize_csv_field_leaves_safe_values_unchanged() {
        for f in [
            "cloudtrail.amazonaws.com",
            "ListBuckets",
            "192.168.0.1",
            "",
            "abc-def",
        ] {
            assert_eq!(sanitize_csv_field(f), f);
        }
    }

    // An unwritable --output path must yield a clean error, not a panic (issue #149, case 3).
    #[test]
    fn get_writer_errors_on_unwritable_output_path() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("missing_subdir").join("out.csv");
        match get_writer(&Some(bad)) {
            Ok(_) => panic!("expected an error for an unwritable output path"),
            Err(e) => assert!(e.contains("Cannot write to output file"), "got: {e}"),
        }
    }

    #[test]
    fn get_json_writer_errors_on_unwritable_output_path() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("missing_subdir").join("out.json");
        match get_json_writer(&Some(bad)) {
            Ok(_) => panic!("expected an error for an unwritable output path"),
            Err(e) => assert!(e.contains("Cannot write to output file"), "got: {e}"),
        }
    }

    #[test]
    fn writers_default_to_stdout_when_no_output() {
        assert!(get_writer(&None).is_ok());
        assert!(get_json_writer(&None).is_ok());
    }
}
