use clap::{ArgAction, ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;

use const_format::concatcp;

pub const RELEASE_NAME: &str = "Black Hat Arsenal USA 2025 Release";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const FULL_VERSION: &str = concatcp!(VERSION, " ", RELEASE_NAME);

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
    /// Disable color output
    #[arg(help_heading = Some("Display Settings"), short = 'K', long = "no-color", global = true, display_order = 1)]
    pub no_color: bool,

    /// Quiet mode: do not display the launch banner
    #[arg(help_heading = Some("Display Settings"), short, long, global = true,  display_order = 10)]
    pub quiet: bool,

    /// Print debug information (memory usage, etc...)
    #[clap(long = "debug", global = true, hide = true)]
    pub debug: bool,

    /// Show the help menu
    #[clap(help_heading = Some("General Options"), short = 'h', long = "help", action = ArgAction::Help, required = false)]
    pub help: Option<bool>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct TimeOption {
    /// Start time of the events to load (ex: "2022-02-22T23:59:59Z)
    #[arg(help_heading = Some("Filtering"), long = "timeline-start", value_name = "DATE")]
    pub timeline_start: Option<String>,

    /// End time of the events to load (ex: "2020-02-22T00:00:00Z")
    #[arg(help_heading = Some("Filtering"), long = "timeline-end", value_name = "DATE")]
    pub timeline_end: Option<String>,

    /// Scan recent events based on an offset (ex: 1y, 3M, 30d, 24h, 30m)
    #[arg(help_heading = Some("Filtering"), long = "time-offset", value_name = "OFFSET", conflicts_with = "timeline_start")]
    pub time_offset: Option<String>,
}

#[derive(Args, Clone, Debug, Default)]
#[clap(group(ArgGroup::new("input_filtering").args(["directory", "filepath"]).required(true)))]
pub struct InputOption {
    /// Directory of multiple gz/json files
    #[arg(help_heading = Some("Input"), short = 'd', long, value_name = "DIR", conflicts_with_all = ["filepath"])]
    pub directory: Option<PathBuf>,

    /// File path to one gz/json file
    #[arg(help_heading = Some("Input"), short = 'f', long = "file", value_name = "FILE", conflicts_with_all = ["directory"])]
    pub filepath: Option<PathBuf>,

    #[clap(flatten)]
    pub time_opt: TimeOption,
}

#[derive(Args, Clone, Debug, Default)]
pub struct AwsCtTimelineOptions {
    /// Specify a custom rule directory or file (default: ./rules)
    #[arg(help_heading = Some("General Options"), short = 'r', long, default_value = "./rules", hide_default_value = true, value_name = "DIR/FILE")]
    pub rules: PathBuf,

    #[clap(flatten)]
    pub input_opt: InputOption,

    /// Save the results to a file
    #[arg(help_heading = Some("Output"), short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL
    #[arg(help_heading = Some("Output"), short = 't', long = "output-type", requires = "output", value_parser = clap::value_parser!(u8).range(1..=5), default_value = "1")]
    pub output_type: u8,

    /// Overwrite files when saving
    #[arg(help_heading = Some("Output"), short='C', long = "clobber", requires = "output", display_order = 1)]
    pub clobber: bool,

    /// Disable event frequency timeline (terminal needs to support Unicode)
    #[arg(help_heading = Some("Display Settings"), short = 'T', long = "no-frequency-timeline", display_order = 3)]
    pub no_frequency: bool,

    /// Do not display results summary
    #[arg(help_heading = Some("Display Settings"), short = 'N', long = "no-summary", display_order = 2)]
    pub no_summary: bool,

    /// Add GeoIP (ASN, city, country) info to IP addresses
    #[arg(help_heading = Some("Output"), short = 'G', long = "geo-ip", value_name = "MAXMIND-DB-DIR", display_order = 2)]
    pub geo_ip: Option<PathBuf>,

    /// Output the original JSON logs (only available in JSON formats)
    #[arg(help_heading = Some("Output"), short = 'R', long = "raw-output")]
    pub raw_output: bool,

    /// Minimum level for rules to load (default: informational)
    #[arg(help_heading = Some("Output"), short = 'm', long = "min-level", default_value = "informational", hide_default_value = true, value_name = "LEVEL", display_order = 3)]
    pub min_level: String,

    /// Number of threads to use (default: same as CPU cores)
    #[arg(help_heading = Some("Output"), long = "threads", default_value = "0", hide_default_value = true, value_name = "THREAD NUMBER",)]
    pub threat_num: usize,
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
        options: AwsCtTimelineOptions,

        #[clap(flatten)]
        common_opt: CommonOptions,
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

    #[command(about = "Update rules", disable_help_flag = true)]
    UpdateRules {
        #[clap(flatten)]
        common_opt: CommonOptions,
    },
}
