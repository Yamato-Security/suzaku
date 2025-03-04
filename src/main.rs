use crate::aws_detect::aws_detect;
use crate::aws_metrics::aws_metrics;
use crate::cmd::Cli;
use crate::cmd::Commands::{AwsCtMetrics, AwsCtTimeline};
use chrono::Local;
use clap::Parser;
use std::fs;
use std::time::Instant;

mod aws_detect;
mod aws_metrics;
mod cmd;
mod rules;
mod scan;
mod util;

fn main() {
    let logo = fs::read_to_string("art/logo.txt").unwrap_or_default();
    println!("\x1b[38;2;0;255;0m{}\x1b[0m", logo);
    println!();

    let start = Instant::now();
    println!("Start time: {}\n", Local::now().format("%Y/%m/%d %H:%M"));

    let cmd = &Cli::parse().cmd;
    match cmd {
        AwsCtTimeline {
            directory,
            file,
            output,
        } => {
            if directory.is_none() && file.is_none() || directory.is_some() && file.is_some() {
                println!("Please specify either a directory or a file.");
                return;
            }
            aws_detect(directory, file, output);
        }
        AwsCtMetrics {
            directory,
            file,
            field_name,
            output,
        } => {
            if directory.is_none() && file.is_none() || directory.is_some() && file.is_some() {
                println!("Please specify either a directory or a file.");
                return;
            }
            aws_metrics(directory, file, field_name, output);
        }
    }

    let duration = start.elapsed();
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    println!("Elapsed time: {:02}:{:02}:{:02}\n", hours, minutes, seconds);
}
