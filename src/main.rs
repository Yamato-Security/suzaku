use chrono::{DateTime, Local};
use clap::{CommandFactory, Parser};
use cmd::aws_detect::aws_detect;
use cmd::aws_metrics::aws_metrics;
use cmd::aws_summary::aws_summary;
use cmd::update::start_update_rules;
use itertools::Itertools;
use nested::Nested;
use core::color::SuzakuColor::Green;
use core::util::{check_path_exists, p, set_rayon_threat_number};
use libmimalloc_sys::mi_stats_print_out;
use mimalloc::MiMalloc;
use option::cli::Commands::{AwsCtMetrics, AwsCtSummary, AwsCtTimeline, UpdateRules};
use option::cli::{Cli, RELEASE_NAME, VERSION};
use option::htmlreporter;
use std::ptr::null_mut;
use std::time::Instant;
use std::{env, fs};

use crate::option::htmlreporter::HTML_REPORTER;

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
        AwsCtTimeline { common_opt, .. } => common_opt.no_color,
        AwsCtMetrics { common_opt, .. } => common_opt.no_color,
        AwsCtSummary { common_opt, .. } => common_opt.no_color,
        UpdateRules { common_opt } => common_opt.no_color,
    };
    let html_report_path = match cmd {
        AwsCtTimeline { options, .. } => options.html_report,
        _ => None,
    };

    match cmd {
        AwsCtTimeline {
            options,
            common_opt,
        } => {
            if html_report_flag.is_some() {
                add_cmd_and_time_to_html();
            }
            display_logo(common_opt.quiet, no_color, true, false);

            set_rayon_threat_number(options.threat_num);

            let dir = &options.input_opt.directory;
            let file = &options.input_opt.filepath;
            if !check_path_exists(file.clone(), dir.clone()) {
                return;
            }
            if let Some(output) = &options.output
                && !options.clobber
                && output.exists()
            {
                let msg = format!(
                    "The file {} already exists. Please specify a different filename or add the -C, --clobber option to overwrite.",
                    output.display()
                );
                p(None, msg.as_str(), true);
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
            if options.min_level != "informational"
                && options.min_level != "info"
                && options.min_level != "low"
                && options.min_level != "medium"
                && options.min_level != "med"
                && options.min_level != "high"
                && options.min_level != "critical"
                && options.min_level != "crit"
            {
                p(
                    None,
                    &format!(
                        "Invalid minimum level: {}. Valid levels are: informational, low, medium, high, critical.",
                        options.min_level
                    ),
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
            aws_metrics(input_opt, field_name, output, no_color);
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
            let dir = &input_opt.directory;
            let file = &input_opt.filepath;
            if !check_path_exists(file.clone(), dir.clone()) {
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
    if html_report_path.is_some() {
        htmlreporter::add_md_data("## General Overview {#general_overview}", Nested::from_iter(
            vec![format!("- Elapsed time: {hours:02}:{minutes:02}:{seconds:02}\n")],
        ));
        let html_str = HTML_REPORTER.read().unwrap().to_owned().convert_md_to_html();
        htmlreporter::create_html_file(
            html_str,
            html_report_path.unwrap()
            .to_str()
            .unwrap_or(""),
            no_color,
        )
    }
    let debug = match cmd {
        AwsCtTimeline { common_opt, .. } => common_opt.debug,
        AwsCtMetrics { common_opt, .. } => common_opt.debug,
        AwsCtSummary { common_opt, .. } => common_opt.debug,
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
        let msg = format!("Version: {VERSION} ({RELEASE_NAME})");
        p(None, msg.as_str(), true);
    } else {
        p(Green.rdg(no_color), "Version: ", false);
        p(None, &format!("{VERSION} ({RELEASE_NAME})\n"), false);
    }
    println!()
}

fn add_cmd_and_time_to_html() {
    let analysis_start_time: DateTime<Local> = Local::now();
    let mut output_data = Nested::<String>::new();
    output_data.extend(vec![
        format!("- Command line: {}", env::args().join(" ")),
        format!(
            "- Start time: {}",
            analysis_start_time.format("%Y/%m/%d %H:%M")
        ),
    ]);
    htmlreporter::add_md_data("General Overview {#general_overview}", output_data);

}