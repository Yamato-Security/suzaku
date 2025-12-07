use crate::core::color::SuzakuColor::Green;
use crate::core::log_source::LogSource;
use crate::option::geoip::GeoIPSearch;
use bytesize::ByteSize;
use csv::Writer;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
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
        if !file.is_file() {
            println!("Path {file:?} is not a file (it may be a directory or special file type).");
            return false;
        }
    }

    if let Some(dir) = dirpath {
        if !dir.exists() {
            println!("Directory {dir:?} does not exist.");
            return false;
        }
        if !dir.is_dir() {
            println!("Path {dir:?} is not a directory (it may be a file or special file type).");
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

pub fn output_path_info(no_color: bool, output_paths: &[PathBuf], has_detect: bool) {
    p(Green.rdg(no_color), "Results saved: ", false);
    if !has_detect {
        p(None, "None", true);
        return;
    }
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

/**
 * Set the global thread number for rayon.
 * @param val The number of threads.
 */
pub fn set_rayon_threat_number(val: usize) {
    if val == 0 {
        return;
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(val)
        .build_global()
        .unwrap();
}

pub fn load_profile(
    log: &LogSource,
    geo_search: &Option<GeoIPSearch>,
    skip_sigma: bool,
) -> Vec<(String, String)> {
    let file = File::open(log.profile_path()).expect("Unable to open profile file");
    let reader = BufReader::new(file);
    let mut profile = vec![];

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let val = parts[1].trim().trim_matches('\'');
            if skip_sigma && val.contains("sigma") {
                continue;
            }
            profile.push((key.to_string(), val.to_string()));
            if key == "SrcIP" && geo_search.is_some() {
                profile.push(("SrcASN".to_string(), "SrcASN".to_string()));
                profile.push(("SrcCity".to_string(), "SrcCity".to_string()));
                profile.push(("SrcCountry".to_string(), "SrcCountry".to_string()));
            }
        }
    }
    profile
}
