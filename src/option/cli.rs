use chrono::NaiveDate;
use clap::{ArgAction, ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;

use const_format::concatcp;

pub const RELEASE_NAME: &str = "Dev Build";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const FULL_VERSION: &str = concatcp!(VERSION, " ", RELEASE_NAME);

/// Validate that the input is a valid date in YYYYMMDD format.
fn parse_file_date(s: &str) -> Result<String, String> {
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("'{}' is not in YYYYMMDD format (e.g. 20240115)", s));
    }
    let year: i32 = s[0..4].parse().unwrap();
    let month: u32 = s[4..6].parse().unwrap();
    let day: u32 = s[6..8].parse().unwrap();
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| format!("'{}' is not a valid date", s))?;
    Ok(s.to_string())
}

#[derive(Parser)]
#[command(name = "suzaku")]
#[command(version = VERSION)]
#[command(author = "Yamato Security (https://github.com/Yamato-Security/suzaku - @SecurityYamato)")]
#[command(about = "Cloud Log Threat Detection and Fast Forensics")]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) cmd: Commands,
}

#[derive(Copy, Args, Clone, Debug, Default)]
pub struct CommonOptions {
    /// Show the help menu
    #[clap(help_heading = Some("General Options"), short = 'h', long = "help", action = ArgAction::Help, required = false, display_order = 10)]
    pub help: Option<bool>,

    /// Disable color output
    #[arg(help_heading = Some("Display Settings"), short = 'K', long = "no-color", global = true, display_order = 400)]
    pub no_color: bool,

    /// Quiet mode: do not display the launch banner
    #[arg(help_heading = Some("Display Settings"), short, long, global = true, display_order = 402)]
    pub quiet: bool,

    /// Print debug information (memory usage, etc...)
    #[clap(long = "debug", global = true, hide = true)]
    pub debug: bool,
}

#[derive(Args, Clone, Debug, Default)]
pub struct TimeOption {
    /// Start time of the events to load (ex: "2022-02-22T23:59:59Z)
    #[arg(help_heading = Some("Filtering"), long = "timeline-start", value_name = "DATE", display_order = 210)]
    pub timeline_start: Option<String>,

    /// End time of the events to load (ex: "2020-02-22T00:00:00Z")
    #[arg(help_heading = Some("Filtering"), long = "timeline-end", value_name = "DATE", display_order = 211)]
    pub timeline_end: Option<String>,

    /// Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
    #[arg(help_heading = Some("Filtering"), long = "time-offset", value_name = "OFFSET", conflicts_with = "timeline_start", display_order = 212)]
    pub time_offset: Option<String>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct FileDateOption {
    /// Filter files by start date based on AWSLogs S3 path date structure (ex: "20240101")
    #[arg(help_heading = Some("Filtering"), long = "file-date-from", value_name = "DATE", value_parser = parse_file_date, display_order = 213)]
    pub file_date_from: Option<String>,

    /// Filter files by end date based on AWSLogs S3 path date structure (ex: "20241231")
    #[arg(help_heading = Some("Filtering"), long = "file-date-to", value_name = "DATE", value_parser = parse_file_date, display_order = 214)]
    pub file_date_to: Option<String>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct OutputOption {
    /// Overwrite files when saving
    #[arg(help_heading = Some("Output"), short='C', long = "clobber", requires = "output", display_order = 300)]
    pub clobber: bool,

    /// Add GeoIP (ASN, city, country) info to IP addresses
    #[arg(help_heading = Some("Output"), short = 'G', long = "geo-ip", value_name = "MAXMIND-DB-DIR", display_order = 301)]
    pub geo_ip: Option<PathBuf>,

    /// Save the results to a file
    #[arg(help_heading = Some("Output"), short, long, value_name = "FILE", display_order = 302)]
    pub output: Option<PathBuf>,

    /// Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL
    #[arg(help_heading = Some("Output"), short = 't', long = "output-type", value_parser = clap::value_parser!(u8).range(1..=5), default_value = "1", display_order = 303)]
    pub output_type: u8,

    /// Output the original JSON logs (only available in JSON formats or stdout)
    #[arg(help_heading = Some("Output"), long = "raw-output", display_order = 304)]
    pub raw_output: bool,

    /// Number of threads to use (default: same as CPU cores)
    #[arg(help_heading = Some("Output"), long = "threads", default_value = "0", hide_default_value = true, value_name = "THREAD NUMBER", display_order = 305)]
    pub thread_num: usize,
}

#[derive(Args, Clone, Debug, Default)]
pub struct SearchOptions {
    #[clap(flatten)]
    pub input_opt: InputOption,

    /// Filter by specific field(s)
    #[arg(help_heading = Some("Filtering"), short = 'F', long, value_name = "FILTER...", display_order = 200)]
    pub filter: Vec<String>,

    /// Case-sensitive keyword search
    #[arg(help_heading = Some("Filtering"), short = 'c', long = "preserve-case", display_order = 201)]
    pub preserve_case: bool,

    /// Search by keyword(s)
    #[arg(help_heading = Some("Filtering"), short = 'k', long, value_name = "KEYWORD...", display_order = 202)]
    pub keyword: Vec<String>,

    /// Search by regular expression
    #[arg(help_heading = Some("Filtering"), short = 'r', long = "regex", value_name = "REGEX", display_order = 203)]
    pub regex: Option<String>,

    #[clap(flatten)]
    pub output_opt: OutputOption,
}

#[derive(Args, Clone, Debug, Default)]
#[clap(group(ArgGroup::new("input_filtering").args(["directory", "filepath"]).required(true)))]
pub struct InputOption {
    /// Directory of multiple gz/json files
    #[arg(help_heading = Some("Input"), short = 'd', long, value_name = "DIR", conflicts_with_all = ["filepath"], display_order = 100)]
    pub directory: Option<PathBuf>,

    /// File path to one gz/json file
    #[arg(help_heading = Some("Input"), short = 'f', long = "file", value_name = "FILE", conflicts_with_all = ["directory"], display_order = 101)]
    pub filepath: Option<PathBuf>,

    #[clap(flatten)]
    pub time_opt: TimeOption,

    #[clap(flatten)]
    pub file_date_opt: FileDateOption,
}

#[derive(Args, Clone, Debug, Default)]
pub struct TimelineOptions {
    /// Specify a custom rule directory or file (default: ./rules)
    #[arg(help_heading = Some("General Options"), short = 'r', long, default_value = "./rules", hide_default_value = true, value_name = "DIR/FILE", display_order = 11)]
    pub rules: PathBuf,

    #[clap(flatten)]
    pub input_opt: InputOption,

    #[clap(flatten)]
    pub output_opt: OutputOption,

    /// Do not display results summary
    #[arg(help_heading = Some("Display Settings"), short = 'N', long = "no-summary", display_order = 401)]
    pub no_summary: bool,

    /// Minimum level for rules to load (default: informational)
    #[arg(help_heading = Some("Output"), short = 'm', long = "min-level", default_value = "informational", hide_default_value = true, value_name = "LEVEL", display_order = 302)]
    pub min_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        author = "Yamato Security (https://github.com/Yamato-Security/suzaku - @SecurityYamato)",
        version = FULL_VERSION,
        help_template = "\nVersion: {version}\n{author-with-newline}\n{usage-heading}\n  suzaku aws-ct-timeline <INPUT> [OPTIONS]\n\n{all-args}",
        disable_help_flag = true,
        disable_version_flag = true
    )]
    /// Creates an AWS CloudTrail DFIR timeline
    AwsCtTimeline {
        #[clap(flatten)]
        options: TimelineOptions,

        #[clap(flatten)]
        common_opt: CommonOptions,
    },

    #[command(
        author = "Yamato Security (https://github.com/Yamato-Security/suzaku - @SecurityYamato)",
        version = FULL_VERSION,
        help_template = "\nVersion: {version}\n{author-with-newline}\n{usage-heading}\n  suzaku aws-ct-search <INPUT> [OPTIONS]\n\n{all-args}",
        disable_help_flag = true,
        disable_version_flag = true
    )]
    /// Search AWS CloudTrail logs
    AwsCtSearch {
        #[clap(flatten)]
        common_opt: CommonOptions,

        #[clap(flatten)]
        options: SearchOptions,
    },

    #[command(
        author = "Yamato Security (https://github.com/Yamato-Security/suzaku - @SecurityYamato)",
        version = FULL_VERSION,
        help_template = "\nVersion {version}\n{author-with-newline}\n{usage-heading}\n  suzaku aws-ct-metrics <INPUT> [OPTIONS]\n\n{all-args}",
        disable_help_flag = true,
        disable_version_flag = true
    )]
    /// Generates metrics from AWS CloudTrail logs
    AwsCtMetrics {
        #[clap(flatten)]
        input_opt: InputOption,

        /// The field to generate metrics for
        #[arg(
            help_heading = Some("Output"),
            short = 'F',
            default_value = "eventName",
            long,
            value_name = "FIELD_NAME"
        )]
        field_name: String,

        /// Output CSV
        #[arg(help_heading = Some("Output"), short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        #[clap(flatten)]
        common_opt: CommonOptions,
    },

    #[command(
        author = "Yamato Security (https://github.com/Yamato-Security/suzaku - @SecurityYamato)",
        version = FULL_VERSION,
        help_template = "\nVersion {version}\n{author-with-newline}\n{usage-heading}\n  suzaku aws-ct-summary <INPUT> [OPTIONS]\n\n{all-args}",
        disable_help_flag = true,
        disable_version_flag = true
    )]
    /// Generates summary from AWS CloudTrail logs
    AwsCtSummary {
        #[clap(flatten)]
        input_opt: InputOption,

        /// Include temporary AWS STS access key IDs
        #[arg(help_heading = Some("Filtering"), short = 's', long = "include-sts-keys")]
        include_sts: bool,

        /// Output results to a CSV file
        #[arg(help_heading = Some("Output"), short, long, value_name = "FILE", required = true)]
        output: PathBuf,

        /// Hide description of the commonly abused API calls
        #[arg(help_heading = Some("Output"), short = 'D', long = "hide-descriptions")]
        hide_descriptions: bool,

        /// Add GeoIP (ASN, city, country) info to IP addresses
        #[arg(help_heading = Some("Output"), short = 'G', long = "GeoIP", value_name = "MAXMIND-DB-DIR")]
        geo_ip: Option<PathBuf>,

        #[clap(flatten)]
        common_opt: CommonOptions,
    },

    #[command(
        author = "Yamato Security (https://github.com/Yamato-Security/suzaku - @SecurityYamato)",
        version = FULL_VERSION,
        help_template = "\nVersion: {version}\n{author-with-newline}\n{usage-heading}\n  suzaku azure-ct-timeline <INPUT> [OPTIONS]\n\n{all-args}",
        disable_help_flag = true,
        disable_version_flag = true
    )]
    /// Creates an Azure DFIR timeline
    AzureTimeline {
        #[clap(flatten)]
        options: TimelineOptions,

        #[clap(flatten)]
        common_opt: CommonOptions,
    },

    #[command(about = "Update rules", disable_help_flag = true)]
    UpdateRules {
        #[clap(flatten)]
        common_opt: CommonOptions,
    },
}
