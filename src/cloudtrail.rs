use colored::Colorize;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use sigma_rust::Event;
use sigma_rust::event_from_json;
use std::error::Error;
use std::fs;

pub fn process_events_from_dir<F>(
    directory: &str,
    mut process_event: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(Event),
{
    let (count, file_paths) = count_files_recursive(directory)?;
    println!("JSON files: {}", count);

    let spinner = "{spinner}".truecolor(0, 255, 0).to_string();
    let bar = "{bar:40}".truecolor(0, 255, 0).to_string();
    let template = format!(
        "[{{elapsed_precise}}] {{human_pos}} / {{human_len}} {} [{}] {{percent}}%\r\n\r\n{{msg}}",
        spinner, bar
    );
    let progress_style = ProgressStyle::with_template(&template) // Pass `&template` here
        .unwrap()
        .progress_chars("=> ");
    let pb =
        ProgressBar::with_draw_target(Some(count as u64), ProgressDrawTarget::stdout_with_hz(10))
            .with_tab_width(55);
    pb.set_style(progress_style);
    for path in file_paths {
        let pb_msg = format!("{} ({})", path, 0);
        pb.set_message(pb_msg);
        let log_contents = fs::read_to_string(path)?;
        let json_array: Vec<serde_json::Value> = serde_json::from_str(&log_contents)?;
        for json_value in json_array {
            let event: Event = event_from_json(json_value.to_string().as_str())?;
            process_event(event);
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done.");
    Ok(())
}

fn count_files_recursive(directory: &str) -> Result<(usize, Vec<String>), Box<dyn Error>> {
    let mut count = 0;
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            count += 1;
            paths.push(path.to_str().unwrap().to_string());
        } else if path.is_dir() {
            let (sub_count, sub_paths) = count_files_recursive(path.to_str().unwrap())?;
            count += sub_count;
            paths.extend(sub_paths);
        }
    }
    Ok((count, paths))
}

pub fn load_json_from_file(file_path: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let log_contents = fs::read_to_string(file_path)?;
    let json_array: Vec<serde_json::Value> = serde_json::from_str(&log_contents)?;

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
        let result = load_json_from_file(test_file);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.len(), 1);
    }
}
