use bytesize::ByteSize;
use colored::Colorize;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde_json::Value;
use sigma_rust::Event;
use sigma_rust::event_from_json;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

pub fn process_events_from_dir<F>(
    directory: &PathBuf,
    mut process_event: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(Event),
{
    let (count, file_paths, total_size) = count_files_recursive(directory)?;
    let size = ByteSize::b(total_size).to_string_as(false);
    println!("Total log files(json/gz): {}", count);
    println!("Total file size: {}\n", size);

    let template = format!(
        "[{{elapsed_precise}}] {{human_pos}} / {{human_len}} {} [{}] {{percent}}%\r\n\r\n{{msg}}",
        "{spinner}".truecolor(0, 255, 0),
        "{bar:40}".truecolor(0, 255, 0)
    );
    let pb_style = ProgressStyle::with_template(&template)
        .unwrap()
        .progress_chars("=> ");
    let pb =
        ProgressBar::with_draw_target(Some(count as u64), ProgressDrawTarget::stdout_with_hz(10))
            .with_tab_width(55);
    pb.set_style(pb_style);
    for path in file_paths {
        let size = fs::metadata(&path).unwrap().len();
        let size = ByteSize::b(size).to_string_as(false);
        let pb_msg = format!("{} ({})", path, size);
        pb.set_message(pb_msg);
        let log_contents = if path.ends_with("json") {
            fs::read_to_string(&path)?
        } else if path.ends_with("gz") {
            read_gz_file(&PathBuf::from(&path))?
        } else {
            pb.inc(1);
            continue;
        };
        let json_value: Value = serde_json::from_str(&log_contents)?;
        match json_value {
            Value::Array(json_array) => {
                for json_value in json_array {
                    let event: Event = event_from_json(json_value.to_string().as_str())?;
                    process_event(event);
                }
            }
            Value::Object(json_map) => {
                if let Some(json_array) = json_map.get("Records") {
                    for json_value in json_array.as_array().unwrap() {
                        let event: Event = event_from_json(json_value.to_string().as_str())?;
                        process_event(event);
                    }
                }
            }
            _ => {
                eprintln!("Unexpected JSON structure in file: {}", path);
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Scanning finished.");
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
    let mut decoder = GzDecoder::new(file);
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    Ok(contents)
}
pub fn load_json_from_file(log_contents: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let json_array: Vec<serde_json::Value> = serde_json::from_str(log_contents)?;
    let mut events = Vec::new();
    for json_value in json_array {
        let event: Event = event_from_json(json_value.to_string().as_str())?;
        events.push(event);
    }
    Ok(events)
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
}
