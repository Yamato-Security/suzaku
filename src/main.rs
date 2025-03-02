use crate::cloudtrail::{load_json_from_file, process_events_from_dir};
use crate::cmd::{Cli, Commands};
use clap::Parser;
use std::fs;

mod cloudtrail;
mod cmd;
mod rules;

fn main() {
    let logo = fs::read_to_string("art/logo.txt").unwrap();
    println!("\x1b[38;2;255;175;0m{}\x1b[0m", logo);
    println!();

    let rules = rules::load_rules_from_dir("rules");
    println!("Loaded {} rules", rules.len());

    let cli = Cli::parse();
    match &cli.cmd {
        Commands::AwsDetect {
            directory,
            file,
            output,
        } => {
            let scan_by_all_rules = |event| {
                for rule in &rules {
                    if rule.is_match(&event) {
                        println!("Matched rule: {:?}", rule.title);
                        println!("Matched event: {:?}", event);
                    }
                }
            };
            if let Some(d) = directory {
                process_events_from_dir(d, scan_by_all_rules).unwrap();
            } else if let Some(f) = file {
                let events = load_json_from_file(f).unwrap();
                events.into_iter().for_each(scan_by_all_rules);
            }
            println!("Output: {:?}", output);
        }
    }
}
