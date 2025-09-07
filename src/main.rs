use crate::cmd::aws::aws_metrics::aws_metrics;
use crate::cmd::aws::aws_summary::aws_summary;
use crate::cmd::aws::aws_timeline::aws_timeline;

use crate::cmd::azure::azure_timeline::azure_timeline;
use chrono::Local;
use clap::{CommandFactory, Parser};
use cmd::update::start_update_rules;
use core::color::SuzakuColor::Green;
use core::util::{check_path_exists, p, set_rayon_threat_number};
use libmimalloc_sys::mi_stats_print_out;
use mimalloc::MiMalloc;
use option::cli::Commands::{
    AwsCtMetrics, AwsCtSummary, AwsCtTimeline, AzureTimeline, UpdateRules,
};
use option::cli::{Cli, RELEASE_NAME, VERSION};
use std::ptr::null_mut;
use std::time::Instant;
use std::{env, fs};

mod cmd;
mod core;
mod option;

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
        AwsCtTimeline { common_opt, .. }
        | AwsCtMetrics { common_opt, .. }
        | AwsCtSummary { common_opt, .. }
        | UpdateRules { common_opt }
        | AzureTimeline { common_opt, .. } => common_opt.no_color,
    };

    match cmd {
        AzureTimeline {
            options,
            common_opt,
        }
        | AwsCtTimeline {
            options,
            common_opt,
        } => {
            display_logo(common_opt.quiet, no_color, true, false);
            set_rayon_threat_number(options.threat_num);

            // Common validation for timeline commands
            if !check_path_exists(
                options.input_opt.filepath.clone(),
                options.input_opt.directory.clone(),
            ) {
                return;
            }

            if let Some(output) = &options.output
                && !options.clobber
                && output.exists()
            {
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

            if !options.rules.exists() {
                p(
                    None,
                    &format!("Rule file or directory does not exist: {:?}", options.rules),
                    true,
                );
                return;
            }

            if options.raw_output && options.output_type == 1 && options.output.is_some() {
                p(
                    None,
                    "--raw-output option is only available in JSON formats. Please specify an output type of 2-5.",
                    true,
                );
                return;
            }

            if !validate_min_level(&options.min_level) {
                return;
            }

            // Execute appropriate timeline function
            match cmd {
                AzureTimeline { .. } => azure_timeline(options, common_opt),
                AwsCtTimeline { .. } => aws_timeline(options, common_opt),
                _ => unreachable!(),
            }
        }
        AwsCtMetrics {
            input_opt,
            output,
            field_name,
            common_opt,
        } => {
            display_logo(common_opt.quiet, no_color, true, false);
            if !check_path_exists(input_opt.filepath.clone(), input_opt.directory.clone()) {
                return;
            }
            aws_metrics(input_opt, field_name.as_ref(), output, no_color);
        }
        AwsCtSummary {
            input_opt,
            include_sts,
            output,
            hide_descriptions,
            geo_ip,
            common_opt,
        } => {
            display_logo(common_opt.quiet, no_color, true, false);
            if !check_path_exists(input_opt.filepath.clone(), input_opt.directory.clone()) {
                return;
            }
            aws_summary(
                input_opt,
                output,
                no_color,
                include_sts,
                hide_descriptions,
                geo_ip,
            );
        }
        UpdateRules { common_opt } => {
            display_logo(common_opt.quiet, no_color, true, false);
            start_update_rules(no_color);
        }
    }

    // Print elapsed time
    let duration = start.elapsed();
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    p(Green.rdg(no_color), "Elapsed time: ", false);
    p(
        None,
        &format!("{hours:02}:{minutes:02}:{seconds:02}\n"),
        true,
    );

    let debug = match cmd {
        AwsCtTimeline { common_opt, .. }
        | AwsCtMetrics { common_opt, .. }
        | AwsCtSummary { common_opt, .. }
        | AzureTimeline { common_opt, .. }
        | UpdateRules { common_opt } => common_opt.debug,
    };

    // Print issue reporting info for timeline commands
    if matches!(cmd, AwsCtTimeline { .. } | AzureTimeline { .. }) && !debug {
        print_issue_reporting_info(no_color);
    }

    if debug {
        println!("Memory usage stats:");
        unsafe {
            mi_stats_print_out(None, null_mut());
        }
    }
}

fn validate_min_level(min_level: &str) -> bool {
    const VALID_LEVELS: &[&str] = &[
        "informational",
        "info",
        "low",
        "medium",
        "med",
        "high",
        "critical",
        "crit",
    ];

    if !VALID_LEVELS.contains(&min_level) {
        p(
            None,
            &format!(
                "Invalid minimum level: {}. Valid levels are: informational, low, medium, high, critical.",
                min_level
            ),
            true,
        );
        return false;
    }
    true
}

fn print_issue_reporting_info(no_color: bool) {
    let messages = [
        (
            "Please report any issues with Suzaku rules to: ",
            "https://github.com/Yamato-Security/suzaku-rules/issues",
        ),
        (
            "Please report any false positives with Sigma rules to: ",
            "https://github.com/SigmaHQ/sigma/issues",
        ),
        (
            "Please submit new Sigma rules with pull requests to: ",
            "https://github.com/SigmaHQ/sigma/pulls",
        ),
    ];

    for (prefix, url) in &messages {
        p(Green.rdg(no_color), prefix, false);
        p(None, url, true);
    }
    println!();
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
        let msg = format!("Version: {VERSION} ({RELEASE_NAME})");
        p(None, msg.as_str(), true);
    } else {
        p(Green.rdg(no_color), "Version: ", false);
        p(None, &format!("{VERSION} ({RELEASE_NAME})\n"), false);
    }
    println!()
}
