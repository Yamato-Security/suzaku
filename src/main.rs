use crate::cmd::{Cli, Commands};
use clap::Parser;
use std::fs;

mod cmd;

fn main() {
    let logo = fs::read_to_string("art/logo.txt").expect("Failed to read logo file");
    println!("\x1b[38;2;255;175;0m{}\x1b[0m", logo);
    println!();

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
