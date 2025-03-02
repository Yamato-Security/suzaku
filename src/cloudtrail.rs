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
    process_events_recursive(directory, &mut process_event)?;
    Ok(())
}

fn process_events_recursive<F>(directory: &str, process_event: &mut F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(Event),
{
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let log_contents = fs::read_to_string(path)?;
            let json_array: Vec<serde_json::Value> = serde_json::from_str(&log_contents)?;
            for json_value in json_array {
                let event: Event = event_from_json(json_value.to_string().as_str())?;
                process_event(event);
            }
        } else if path.is_dir() {
            process_events_recursive(path.to_str().unwrap(), process_event)?;
        }
    }
    Ok(())
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
