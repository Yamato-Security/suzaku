use crate::aws_detect::aws_detect;
use crate::aws_metrics::aws_metrics;
use crate::cmd::Commands::{AwsCtMetrics, AwsCtTimeline, UpdateRules};
use crate::cmd::{Cli, RELEASE_NAME, VERSION};
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
        display_logo(false, false, false, true);
        Cli::command().print_help().unwrap();
        return;
    }
    let start = Instant::now();
    let cmd = &Cli::parse().cmd;
    match cmd {
        AwsCtTimeline {
            options,
            common_opt,
        } => {
            display_logo(common_opt.quiet, common_opt.no_color, true, false);

            let dir = &options.input_opt.directory;
            let file = &options.input_opt.filepath;
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            if let Some(output) = &options.output {
                if !options.clobber && output.exists() {
                    p(
                        None,
                        &format!(
                            "The file {} already exists. Please specify a different filename or add the -C, --clobber option to overwrite.",
                            output.display()
                        ),
                        true,
                    );
                    return;
                }
            }
            if !options.rules.exists() {
                p(
                    Some(Color::Red),
                    &format!("Rule file or directory does not exist: {:?}", options.rules),
                    true,
                );
                return;
            }
            aws_detect(options, common_opt);
        }
        AwsCtMetrics {
            input_opt,
            output,
            field_name,
            common_opt,
        } => {
            display_logo(common_opt.quiet, common_opt.no_color, true, false);
            let dir = &input_opt.directory;
            let file = &input_opt.filepath;
            let field_name = field_name.as_ref();
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            aws_metrics(dir, file, field_name, output, common_opt.no_color);
        }
        UpdateRules { common_opt } => {
            display_logo(common_opt.quiet, common_opt.no_color, true, false);
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

fn display_logo(quiet: bool, no_color: bool, time: bool, help: bool) {
    if !quiet {
        let logo = fs::read_to_string("art/logo.txt").unwrap_or_default();
        if no_color {
            p(None, &logo, true);
        } else {
            p(Some(Color::Rgb(0, 255, 0)), &logo, true);
        }
        println!();
    }
    if help {
        p(
            None,
            &format!("Suzaku: v{} ({})", VERSION, RELEASE_NAME),
            true,
        );
    } else {
        p(Some(Color::Rgb(0, 255, 0)), "Version: ", false);
        p(None, &format!("{} ({})\n", VERSION, RELEASE_NAME), false);
    }
    if time {
        if no_color {
            p(None, "Start time: ", false);
        } else {
            p(Some(Color::Rgb(0, 255, 0)), "Start time: ", false);
        }
        p(
            None,
            Local::now().format("%Y/%m/%d %H:%M").to_string().as_str(),
            true,
        );
        println!()
    }
}
