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
            if let Some(dir) = directory {
                println!("Directory: {}", dir);
            }
            if let Some(file) = file {
                println!("File: {}", file);
            }
            if let Some(output) = output {
                println!("Output: {}", output);
            }
        }
    }
}
