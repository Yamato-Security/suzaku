use chrono::Local;
use clap::{CommandFactory, Parser};
use cmd::aws_detect::aws_detect;
use cmd::aws_metrics::aws_metrics;
use cmd::aws_summary::aws_summary;
use cmd::update::start_update_rules;
use core::color::SuzakuColor::Green;
use core::util::{check_path_exists, p};
use libmimalloc_sys::mi_stats_print_out;
use mimalloc::MiMalloc;
use option::cli::Commands::{AwsCtMetrics, AwsCtSummary, AwsCtTimeline, UpdateRules};
use option::cli::{Cli, RELEASE_NAME, VERSION};
use std::ptr::null_mut;
use std::time::Instant;
use std::{env, fs};

mod cmd;
mod core;
mod option;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
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
                    None,
                    &format!("Rule file or directory does not exist: {:?}", options.rules),
                    true,
                );
                return;
            }
            if options.raw_output && options.output_type == 1 {
                p(
                    None,
                    "--raw-output option is only available in JSON formats. Please specify an output type of 2-5.",
                    true,
                );
                return;
            }
            aws_detect(options, common_opt).await;
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
            aws_metrics(dir, file, field_name, output, no_color).await;
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
                dir,
                file,
                output,
                no_color,
                include_sts,
                hide_descriptions,
                geo_ip,
            )
            .await;
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
        let msg = format!("Version: {} ({})", VERSION, RELEASE_NAME);
        p(None, msg.as_str(), true);
    } else {
        p(Green.rdg(no_color), "Version: ", false);
        p(None, &format!("{} ({})\n", VERSION, RELEASE_NAME), false);
    }
    println!()
}
