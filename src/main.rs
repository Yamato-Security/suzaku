use crate::aws_detect::aws_detect;
use crate::aws_metrics::aws_metrics;
use crate::cmd::Commands::{AwsCtMetrics, AwsCtTimeline, UpdateRules};
use crate::cmd::{Cli, RELEASE_NAME, VERSION};
use crate::color::SuzakuColor::Green;
use crate::util::{check_path_exists, p};
use chrono::Local;
use clap::{CommandFactory, Parser};
use libmimalloc_sys::mi_stats_print_out;
use mimalloc::MiMalloc;
use std::ptr::null_mut;
use std::time::Instant;
use std::{env, fs};
use update::start_update_rules;

mod aws_detect;
mod aws_metrics;
mod cmd;
mod color;
mod rules;
mod scan;
mod update;
mod util;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    let no_color = match cmd {
        AwsCtTimeline { common_opt, .. } => common_opt.no_color,
        AwsCtMetrics { common_opt, .. } => common_opt.no_color,
        UpdateRules { common_opt } => common_opt.no_color,
    };
    match cmd {
        AwsCtTimeline {
            options,
            common_opt,
        } => {
            display_logo(common_opt.quiet, no_color, true, false);

            let dir = &options.input_opt.directory;
            let file = &options.input_opt.filepath;
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            if let Some(output) = &options.output {
                if !options.clobber && output.exists() {
                    let msg = format!(
                        "The file {} already exists. Please specify a different filename or add the -C, --clobber option to overwrite.",
                        output.display()
                    );
                    p(None, msg.as_str(), true);
                    return;
                }
            }
            if !options.rules.exists() {
                p(
                    Green.rdg(no_color),
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
            display_logo(common_opt.quiet, no_color, true, false);
            let dir = &input_opt.directory;
            let file = &input_opt.filepath;
            let field_name = field_name.as_ref();
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            aws_metrics(dir, file, field_name, output, no_color);
        }
        UpdateRules { common_opt } => {
            display_logo(common_opt.quiet, no_color, true, false);
            start_update_rules(no_color);
        }
    }

    let duration = start.elapsed();
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    p(Green.rdg(no_color), "Elapsed time: ", false);
    p(
        None,
        &format!("{:02}:{:02}:{:02}\n", hours, minutes, seconds),
        true,
    );
    let debug = match cmd {
        AwsCtTimeline { common_opt, .. } => common_opt.debug,
        AwsCtMetrics { common_opt, .. } => common_opt.debug,
        UpdateRules { common_opt } => common_opt.debug,
    };

    if matches!(cmd, AwsCtTimeline { .. }) {
        let mut msg = "Please report any issues with Suzaku rules to: ";
        p(Green.rdg(no_color), msg, false);
        msg = "https://github.com/Yamato-Security/suzaku-rules/issues";
        p(None, msg, true);
        msg = "Please report any false positives with Sigma rules to: ";
        p(Green.rdg(no_color), msg, false);
        msg = "https://github.com/SigmaHQ/sigma/issues";
        p(None, msg, true);
        msg = "Please submit new Sigma rules with pull requests to: ";
        p(Green.rdg(no_color), msg, false);
        msg = "https://github.com/SigmaHQ/sigma/pulls";
        p(None, msg, true);
        println!()
    }

    if debug {
        println!("Memory usage stats:");
        unsafe {
            mi_stats_print_out(None, null_mut());
        }
    }
}

fn display_logo(quiet: bool, no_color: bool, time: bool, help: bool) {
    if !quiet {
        let logo = fs::read_to_string("art/logo.txt").unwrap_or_default();
        p(Green.rdg(no_color), &logo, true);
        println!();
    }
    if time {
        let msg = Local::now().format("%Y/%m/%d %H:%M").to_string();
        p(Green.rdg(no_color), "Start time: ", false);
        p(None, msg.as_str(), true);
    }
    if help {
        let msg = format!("Version: {} ({})", VERSION, RELEASE_NAME);
        p(None, msg.as_str(), true);
    } else {
        p(Green.rdg(no_color), "Version: ", false);
        p(None, &format!("{} ({})\n", VERSION, RELEASE_NAME), false);
    }
    println!()
}
