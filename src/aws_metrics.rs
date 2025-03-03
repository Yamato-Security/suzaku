use crate::scan::{load_json_from_file, process_events_from_dir, read_gz_file};
use crate::util::{get_writer, s};
use comfy_table::{Cell, CellAlignment, Table};
use csv::Writer;
use sigma_rust::Event;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn aws_metrics(directory: &Option<PathBuf>, file: &Option<PathBuf>, output: &Option<PathBuf>) {
    let mut wtr = get_writer(output);
    let csv_header = vec!["Total", "%", "eventName"];
    if output.is_some() {
        wtr.write_record(&csv_header).unwrap();
    }

    let mut count_map = HashMap::new();
    let stats_func = |event: Event| {
        let event_name = s(format!("{:?}", event.get("eventName").unwrap()));
        let count = count_map.entry(event_name).or_insert(0);
        *count += 1;
    };

    if let Some(d) = directory {
        process_events_from_dir(d, true, stats_func).unwrap();
        print_count_map_desc(csv_header, &count_map, wtr, output.is_none());
    } else if let Some(f) = file {
        let path = f.display().to_string();
        let log_contents = if path.ends_with(".json") {
            fs::read_to_string(f).unwrap_or_default()
        } else if path.ends_with(".gz") {
            read_gz_file(f).unwrap_or_default()
        } else {
            "".to_string()
        };
        let events = load_json_from_file(&log_contents);
        if let Ok(events) = events {
            events.into_iter().for_each(stats_func);
            print_count_map_desc(csv_header, &count_map, wtr, output.is_none());
        }
    }
}

fn print_count_map_desc(
    csv_header: Vec<&str>,
    total_map: &HashMap<String, i32>,
    mut wrt: Writer<Box<dyn Write>>,
    show_table: bool,
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
        println!("No events found.");
        return;
    }

    for (event_name, count) in total_vec {
        let count = count.to_string();
        let rate = (count.parse::<f64>().unwrap() / total as f64) * 100.0;
        let rate = format!("{:.2}%", rate);
        let record = vec![event_name, rate.as_str(), count.as_str()];
        if show_table {
            table.add_row(record.iter().map(Cell::new));
        } else {
            wrt.write_record(record).unwrap();
        }
    }
    wrt.flush().ok();
    if show_table {
        println!("{}", table);
    }
}
