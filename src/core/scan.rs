use crate::core::color::SuzakuColor::{Green, Orange};
use crate::core::util::p;
use bytesize::ByteSize;
use colored::Colorize;
use console::style;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

pub async fn process_events_from_dir<F>(
    mut process_event: F,
    directory: &PathBuf,
    show_progress: bool,
    no_color: bool,
) -> Result<(), Box<dyn Error>>
where
    F: AsyncFnMut(&Value),
{
    let (count, file_paths, total_size) = count_files_recursive(directory)?;
    let size = ByteSize::b(total_size).display().to_string();
    p(Green.rdg(no_color), "Total log files: ", false);
    p(None, count.to_string().as_str(), true);
    p(Green.rdg(no_color), "Total file size: ", false);
    p(None, size.to_string().as_str(), true);
    println!();

    p(Orange.rdg(no_color), "Scanning now. Please wait.", true);
    println!();

    let template = if no_color {
        "[{elapsed_precise}] {human_pos} / {human_len} {spinner} [{bar:40}] {percent}%\r\n\r\n{msg}"
            .to_string()
    } else {
        format!(
            "[{{elapsed_precise}}] {{human_pos}} / {{human_len}} {} [{}] {{percent}}%\r\n\r\n{{msg}}",
            "{spinner}".truecolor(0, 255, 0),
            "{bar:40}".truecolor(0, 255, 0)
        )
    };
    let pb_style = ProgressStyle::with_template(&template)
        .unwrap()
        .progress_chars("=> ");
    let pb =
        ProgressBar::with_draw_target(Some(count as u64), ProgressDrawTarget::stdout_with_hz(10))
            .with_tab_width(55);
    pb.set_style(pb_style);
    if show_progress {
        pb.enable_steady_tick(Duration::from_millis(300));
    }
    for path in file_paths {
        if show_progress {
            let size = fs::metadata(&path).unwrap().len();
            let size = ByteSize::b(size).display().to_string();
            let pb_msg = format!("{} ({})", path, size);
            pb.set_message(pb_msg);
        }
        let log_contents = if path.ends_with("json") {
            fs::read_to_string(&path)?
        } else if path.ends_with("gz") {
            read_gz_file(&PathBuf::from(&path))?
        } else {
            pb.inc(1);
            continue;
        };
        let json_value: Result<Value, _> = serde_json::from_str(&log_contents);
        match json_value {
            Ok(json_value) => {
                match json_value {
                    Value::Array(json_array) => {
                        for json_value in json_array {
                            let _ = process_event(&json_value).await;
                        }
                    }
                    Value::Object(json_map) => {
                        if let Some(json_array) = json_map.get("Records") {
                            for json_value in json_array.as_array().unwrap() {
                                let _ = process_event(json_value).await;
                            }
                        }
                    }
                    _ => {
                        // TODO: Handle unexpected JSON structure
                    }
                }
            }
            Err(_) => {
                // TODO: Handle unexpected JSON structure
            }
        }

        if show_progress {
            pb.inc(1);
        }
    }
    if show_progress {
        if no_color {
            pb.finish_with_message("Scanning finished.\n");
        } else {
            pb.finish_with_message(style("Scanning finished.\n").color256(214).to_string());
        }
    }
    Ok(())
}

fn count_files_recursive(directory: &PathBuf) -> Result<(usize, Vec<String>, u64), Box<dyn Error>> {
    let mut count = 0;
    let mut paths = Vec::new();
    let mut total_size = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "json" || ext == "gz" {
                    count += 1;
                    total_size += fs::metadata(&path)?.len();
                    paths.push(path.to_str().unwrap().to_string());
                }
            }
        } else if path.is_dir() {
            let (sub_count, sub_paths, sub_size) = count_files_recursive(&path)?;
            count += sub_count;
            total_size += sub_size;
            paths.extend(sub_paths);
        }
    }
    Ok((count, paths, total_size))
}

pub fn read_gz_file(file_path: &PathBuf) -> io::Result<String> {
    let file = File::open(file_path)?;
    let mut decoder = GzDecoder::new(BufReader::new(file));
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    Ok(contents)
}
pub fn load_json_from_file(log_contents: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut events = Vec::new();
    let json_value: Value = serde_json::from_str(log_contents)?;
    match json_value {
        Value::Array(json_array) => {
            for json_value in json_array {
                events.push(json_value);
            }
        }
        Value::Object(json_map) => {
            if let Some(json_array) = json_map.get("Records") {
                for json_value in json_array.as_array().unwrap() {
                    events.push(json_value.clone());
                }
            }
        }
        _ => {
            eprintln!("Unexpected JSON structure in file:");
        }
    }
    Ok(events)
}

pub fn get_content(f: &PathBuf) -> String {
    let path = f.display().to_string();
    if path.ends_with(".json") {
        fs::read_to_string(f).unwrap_or_default()
    } else if path.ends_with(".gz") {
        read_gz_file(f).unwrap_or_default()
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_event_from_file() {
        let test_file = "test_files/json/DeleteTrail.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 1);
    }

    #[test]
    fn test_load_event_from_file_record() {
        let test_file = "test_files/json/test.json";
        let log_contents = fs::read_to_string(test_file).unwrap();
        let result = load_json_from_file(&log_contents);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 29);
    }
}
