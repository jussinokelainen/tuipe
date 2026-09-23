mod tuipe;
use crate::tuipe::Language;
use crate::tuipe::TestType;
use chrono::Local;
use chrono::SubsecRound;
use color_eyre::Result;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{env, fs::File, fs::create_dir_all, path::PathBuf};
use tuipe::Tuipe;

// Get state directory (~/.local/state/tuipe/)
fn get_state_dir() -> Result<PathBuf> {
    let path = PathBuf::from(env::var("HOME").expect("$HOME not set")).join(".local/state/tuipe/");
    create_dir_all(&path)?;
    Ok(path)
}

fn get_logs_dir() -> Result<PathBuf> {
    let path = get_state_dir()?.join("logs/");
    create_dir_all(&path)?;
    Ok(path)
}

fn get_log_file_path() -> Result<PathBuf> {
    // Get timestamp
    let mut time = Local::now()
        .trunc_subsecs(0)
        .to_rfc3339()
        .replace(":", "-")
        .replace("T", "_");

    // Remove timezone from timestamp
    if let Some(pos) = time.find('+') {
        time.truncate(pos);
    }

    // Add '.log' to timestamp and join it to the directory
    Ok(get_logs_dir()?.join(time + ".log"))
}

fn main() -> Result<()> {
    let log_file_path = get_log_file_path()?;
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(&log_file_path)?,
    )?;
    log::info!("Application started");
    color_eyre::install()?;
    ratatui::run(|terminal| Tuipe::new().run(terminal))
}
