use csv::Writer;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};
use termcolor::{BufferWriter, Color, ColorSpec, WriteColor};

pub fn get_writer(output: &Option<PathBuf>) -> Writer<Box<dyn Write>> {
    let wtr: Writer<Box<dyn io::Write>> = if let Some(output) = output {
        Writer::from_writer(Box::new(fs::File::create(output).unwrap()))
    } else {
        Writer::from_writer(Box::new(io::stdout()))
    };
    wtr
}

pub fn s(input: String) -> String {
    input.replace(r#"Value(String(""#, "").replace(r#""))"#, "")
}

pub fn check_path_exists(filepath: Option<PathBuf>, dirpath: Option<PathBuf>) -> bool {
    if let Some(file) = filepath {
        if !file.exists() {
            println!("File {:?} does not exist.", file);
            return false;
        }
    }

    if let Some(dir) = dirpath {
        if !dir.exists() {
            println!("Directory {:?} does not exist.", dir);
            return false;
        }
    }
    true
}

pub fn write_color_buffer(
    wtr: &BufferWriter,
    color: Option<Color>,
    output_str: &str,
    newline_flag: bool,
) -> io::Result<()> {
    let mut buf = wtr.buffer();
    buf.set_color(ColorSpec::new().set_fg(color)).ok();
    if newline_flag {
        writeln!(buf, "{output_str}").ok();
    } else {
        write!(buf, "{output_str}").ok();
    }
    wtr.print(&buf)
}
