use crate::core::color::SuzakuColor::Green;
use bytesize::ByteSize;
use csv::Writer;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{fs, io};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub fn get_writer(output: &Option<PathBuf>) -> Writer<Box<dyn Write>> {
    let wtr: Writer<Box<dyn io::Write>> = if let Some(output) = output {
        Writer::from_writer(Box::new(fs::File::create(output).unwrap()))
    } else {
        Writer::from_writer(Box::new(io::stdout()))
    };
    wtr
}

pub fn get_json_writer(output: &Option<PathBuf>) -> BufWriter<Box<dyn Write>> {
    if let Some(output) = output {
        let file = File::create(output).expect("Failed to create file");
        BufWriter::new(Box::new(file))
    } else {
        BufWriter::new(Box::new(std::io::stdout()))
    }
}

pub fn check_path_exists(filepath: Option<PathBuf>, dirpath: Option<PathBuf>) -> bool {
    if let Some(file) = filepath {
        if !file.exists() {
            println!("File {file:?} does not exist.");
            return false;
        }
    }

    if let Some(dir) = dirpath {
        if !dir.exists() {
            println!("Directory {dir:?} does not exist.");
            return false;
        }
    }
    true
}

pub fn p(color: Option<Color>, msg: &str, newline: bool) {
    let wtr = BufferWriter::stdout(ColorChoice::Always);
    let mut buf = wtr.buffer();
    buf.set_color(ColorSpec::new().set_fg(color)).ok();
    if newline {
        writeln!(buf, "{msg}").ok();
    } else {
        write!(buf, "{msg}").ok();
    }
    wtr.print(&buf).ok();
}

pub fn output_path_info(no_color: bool, output_paths: &[PathBuf]) {
    p(Green.rdg(no_color), "Results saved: ", false);
    for (i, path) in output_paths.iter().enumerate() {
        if let Ok(metadata) = path.metadata() {
            let size = ByteSize::b(metadata.len()).display();
            p(None, &format!("{} ({})", path.display(), size), false);
        }
        if i < output_paths.len() - 1 {
            p(None, " and ", false);
        }
    }
    println!();
}
