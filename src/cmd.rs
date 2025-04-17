use clap::{ArgAction, ArgGroup, Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Suzaku")]
#[command(version = "0.0.1")]
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

    /// Show the help menu
    #[clap(help_heading = Some("General Options"), short = 'h', long = "help", action = ArgAction::Help, required = false)]
    pub help: Option<bool>,
}

#[derive(Args, Clone, Debug, Default)]
#[clap(group(ArgGroup::new("input_filtering").args(["directory", "filepath"]).required(true)))]
pub struct InputOption {
    /// Directory of multiple gz/json files
    #[arg(help_heading = Some("Input"), short = 'd', long, value_name = "DIR", conflicts_with_all = ["filepath"])]
    pub directory: Option<PathBuf>,

    /// File path to one .gz/json file
    #[arg(help_heading = Some("Input"), short = 'f', long = "file", value_name = "FILE", conflicts_with_all = ["directory"])]
    pub filepath: Option<PathBuf>,
}

#[derive(Args, Clone, Debug, Default)]
pub struct AwsCtTimelineOptions {
    /// Specify a custom rule directory or file (default: ./rules)
    #[arg(help_heading = Some("General Options"), short = 'r', long, default_value = "./rules", hide_default_value = true, value_name = "DIR/FILE")]
    pub rules: PathBuf,

    #[clap(flatten)]
    pub input_opt: InputOption,

    /// Output CSV
    #[arg(help_heading = Some("Output"), short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output type 1: CSV (default), 2: JSON, 3: JSONL, 4: CSV & JSON, 5: CSV & JSONL
    #[arg(help_heading = Some("Output"), short = 't', long = "output-type", requires = "output", value_parser = clap::value_parser!(u8).range(1..=5), default_value = "1")]
    pub output_type: u8,

    /// Overwrite files when saving
    #[arg(help_heading = Some("Output"), short='C', long = "clobber", requires = "output")]
    pub clobber: bool,

    /// Disable event frequency timeline (terminal needs to support Unicode)
    #[arg(help_heading = Some("Display Settings"), short = 'T', long = "no-frequency-timeline", display_order = 3)]
    pub no_frequency: bool,

    /// Do not display result Summary for faster speed
    #[arg(help_heading = Some("Display Settings"), short = 'N', long = "no-summary", display_order = 2)]
    pub no_summary: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Creates a AWS CloudTrail log DFIR timeline",
        disable_help_flag = true
    )]
    AwsCtTimeline {
        #[clap(flatten)]
        options: AwsCtTimelineOptions,

        #[clap(flatten)]
        common_opt: CommonOptions,
    },

    #[command(
        about = "Generates metrics from AWS CloudTrail logs",
        disable_help_flag = true
    )]
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

    #[command(about = "Update rules", disable_help_flag = true)]
    UpdateRules {
        #[clap(flatten)]
        common_opt: CommonOptions,
    },
}
