use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Suzaku")]
#[command(version = "0.0.1")]
#[command(author = "Yamato Security @SecurityYamato")]
#[command(about = "CloudTrail Threat Detection")]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Scans AWS CloudTrail logs using Sigma rules to detect potential security threats."
    )]
    AwsDetect {
        #[arg(
            short,
            long,
            value_name = "DIR",
            help = "Directory of multiple log files"
        )]
        directory: Option<PathBuf>,

        #[arg(
            short,
            long,
            value_name = "FILE",
            help = "The log file to scan(json/gz)"
        )]
        file: Option<PathBuf>,

        #[arg(short, long, value_name = "FILE", help = "Output CSV")]
        output: Option<PathBuf>,
    },

    #[command(about = "Generates metrics from AWS CloudTrail logs.")]
    AwsCloudTrailMetrics {
        #[arg(
            short,
            long,
            value_name = "DIR",
            help = "Directory of multiple log files"
        )]
        directory: Option<PathBuf>,

        #[arg(
            short,
            long,
            value_name = "FILE",
            help = "The log file to scan(json/gz)"
        )]
        file: Option<PathBuf>,

        #[arg(
            short = 'F',
            default_value = "eventName",
            long,
            value_name = "FIELD",
            help = "The field to generate metrics for"
        )]
        field: String,

        #[arg(short, long, value_name = "FILE", help = "Output CSV")]
        output: Option<PathBuf>,
    },
}
