use crate::cloudtrail::{load_json_from_dir, load_json_from_file};
use crate::cmd::{Cli, Commands};
use clap::Parser;
use std::fs;

mod cloudtrail;
mod cmd;
mod rules;
mod scan;

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
            let mut events = Vec::new();
            if let Some(d) = directory {
                events = load_json_from_dir(d).unwrap();
            } else if let Some(f) = file {
                events = load_json_from_file(f).unwrap();
            }

            for event in events {
                for rule in &rules {
                    if rule.is_match(&event) {
                        println!("Matched rule: {:?}", rule.title);
                        //println!("Matched event: {:?}", event);
                    }
                }
            }
            println!("Output: {:?}", output);
        }
    }
}
