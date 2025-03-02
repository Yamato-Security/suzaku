use sigma_rust::Event;
use sigma_rust::event_from_json;
use std::error::Error;
use std::fs;

pub fn load_json_from_dir(directory: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let mut events = Vec::new();
    load_json_recursive(directory, &mut events)?;
    Ok(events)
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

fn load_json_recursive(directory: &str, all_events: &mut Vec<Event>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let events = load_json_from_file(path.to_str().unwrap())?;
            all_events.extend(events);
        } else if path.is_dir() {
            load_json_recursive(path.to_str().unwrap(), all_events)?;
        }
    }
    Ok(())
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

    #[test]
    fn test_load_event_from_dir() {
        let test_dir = "test_files";
        let result = load_json_from_dir(test_dir);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
    }
}
