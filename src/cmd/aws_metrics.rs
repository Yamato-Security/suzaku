use crate::core::scan::{get_content, load_json_from_file, process_events_from_dir};
use crate::core::util::{get_writer, output_path_info, p};
use comfy_table::{Cell, CellAlignment, Table};
use csv::Writer;
use serde_json::Value;
use sigma_rust::{Event, event_from_json};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use termcolor::Color;

pub async fn aws_metrics(
    directory: &Option<PathBuf>,
    file: &Option<PathBuf>,
    field: &str,
    output: &Option<PathBuf>,
    no_color: bool,
) {
    let mut wtr = get_writer(output);
    let csv_header = vec!["EventName", "Percent", "Total"];
    if output.is_some() {
        wtr.write_record(&csv_header).unwrap();
    }

    let mut count_map = HashMap::new();
    let mut stats_func = async |json_value: &Value| {
        let event: Event = match event_from_json(json_value.to_string().as_str()) {
            Ok(event) => event,
            Err(_) => return,
        };
        let value = event.get(field);
        if let Some(value) = value {
            let event_name = value.value_to_string();
            let count = count_map.entry(event_name).or_insert(0);
            *count += 1;
        }
    };

    if let Some(d) = directory {
        let _ = process_events_from_dir(stats_func, d, true, no_color).await;
        print_count_map_desc(csv_header, &count_map, wtr, output, no_color);
    } else if let Some(f) = file {
        let log_contents = get_content(f);
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            for event in events {
                let _ = stats_func(&event).await;
            }
            print_count_map_desc(csv_header, &count_map, wtr, output, no_color);
        }
    }
}

fn print_count_map_desc(
    csv_header: Vec<&str>,
    total_map: &HashMap<String, i32>,
    mut wrt: Writer<Box<dyn Write>>,
    output: &Option<PathBuf>,
    no_color: bool,
) {
    let header_cells: Vec<Cell> = csv_header
        .iter()
        .map(|s| Cell::new(s).set_alignment(CellAlignment::Center))
        .collect();
    let mut table = Table::new();
    table.set_header(header_cells);

    let mut total_vec: Vec<(&String, &i32)> = total_map.iter().collect();
    total_vec.sort_by(|a, b| b.1.cmp(a.1));
    let total: i32 = total_map.values().sum();

    if total == 0 {
        p(Some(Color::Rgb(255, 0, 0)), "No events found.", true);
        return;
    }

    for (event_name, count) in total_vec {
        let count = count.to_string();
        let rate = (count.parse::<f64>().unwrap() / total as f64) * 100.0;
        let rate = format!("{:.2}%", rate);
        let record = vec![event_name, rate.as_str(), count.as_str()];
        if output.is_none() {
            table.add_row(record.iter().map(Cell::new));
        } else {
            wrt.write_record(record).unwrap();
        }
    }
    wrt.flush().ok();
    match output {
        Some(csv) => output_path_info(no_color, [csv.clone()].as_slice()),
        None => println!("{}", table),
    }
}
