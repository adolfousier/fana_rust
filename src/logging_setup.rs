use std::fs;
use std::io::Write;
use std::path::Path;
use chrono::Local;
use log::{LevelFilter, SetLoggerError};

const LOG_DIR: &str = "./logs/";
const LOG_FILE: &str = "debug.log";

pub fn setup_logging() -> Result<(), SetLoggerError> {
    // Ensure the log directory exists
    if !Path::new(LOG_DIR).exists() {
        fs::create_dir_all(LOG_DIR).unwrap();
    }

    // Set up the log file path
    let log_file_path = format!("{}{}", LOG_DIR, LOG_FILE);
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file_path)
        .unwrap();

    // Configure the logger
    env_logger::builder()
        .format(move |buf, record| {
            writeln!(
                buf,
                "{} - {} {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    Ok(())
}

