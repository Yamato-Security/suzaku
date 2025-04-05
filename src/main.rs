use crate::aws_detect::aws_detect;
use crate::aws_metrics::aws_metrics;
use crate::cmd::Cli;
use crate::cmd::Commands::{AwsCtMetrics, AwsCtTimeline, UpdateRules};
use crate::util::{check_path_exists, p};
use chrono::Local;
use clap::{CommandFactory, Parser};
use std::time::Instant;
use std::{env, fs};
use termcolor::Color;
use update::start_update_rules;

mod aws_detect;
mod aws_metrics;
mod cmd;
mod rules;
mod scan;
mod update;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1
        || args.len() == 2
            && (args.contains(&String::from("-h")) || args.contains(&String::from("--help")))
    {
        display_logo(false, false);
        Cli::command().print_help().unwrap();
        return;
    }
    let start = Instant::now();
    let cmd = &Cli::parse().cmd;
    match cmd {
        AwsCtTimeline {
            input_opt,
            output,
            common_opt,
            no_frequency,
            no_summary,
        } => {
            display_logo(common_opt.quiet, common_opt.no_color);
            let dir = &input_opt.directory;
            let file = &input_opt.filepath;
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            aws_detect(
                dir,
                file,
                output,
                common_opt.no_color,
                *no_frequency,
                *no_summary,
            );
        }
        AwsCtMetrics {
            input_opt,
            output,
            field_name,
            common_opt,
        } => {
            display_logo(common_opt.quiet, common_opt.no_color);
            let dir = &input_opt.directory;
            let file = &input_opt.filepath;
            let field_name = field_name.as_ref();
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            aws_metrics(dir, file, field_name, output, common_opt.no_color);
        }
        UpdateRules { common_opt } => {
            display_logo(common_opt.quiet, common_opt.no_color);
            start_update_rules();
        }
    }

    let duration = start.elapsed();
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    p(Some(Color::Rgb(0, 255, 0)), "Elapsed time: ", false);
    p(
        None,
        &format!("{:02}:{:02}:{:02}\n", hours, minutes, seconds),
        true,
    );
}

fn display_logo(quiet: bool, no_color: bool) {
    if !quiet {
        let logo = fs::read_to_string("art/logo.txt").unwrap_or_default();
        if no_color {
            println!("{}", logo);
        } else {
            p(Some(Color::Rgb(0, 255, 0)), &logo, true);
        }
        println!();
    }
    p(Some(Color::Rgb(0, 255, 0)), "Start time: ", false);
    p(
        None,
        Local::now().format("%Y/%m/%d %H:%M").to_string().as_str(),
        true,
    );
    println!();
}
