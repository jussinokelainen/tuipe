mod tuipe;
use crate::tuipe::db_path;
use chrono::Local;
use chrono::SubsecRound;
use color_eyre::Result;
use simplelog::{Config, LevelFilter, WriteLogger};
use sqlite::State;
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

// Function to check for missing columns in the database, and add the missing ones
fn fix_database() -> Result<()> {
    let (db_path, success) = db_path();
    if !success {
        return Ok(());
    }

    let connection = sqlite::open(db_path).ok();
    match connection {
        Some(conn) => {
            let required_columns = [
                ("wpm", "REAl"),
                ("raw_wpm", "REAL"),
                ("accuracy", "REAL"),
                ("test_type", "TEXT"),
                ("language", "TEXT"),
                ("capital_letters", "BOOLEAN"),
                ("characters_typed", "INTEGER"),
                ("time", "INTEGER"),
            ];

            let mut statement = conn.prepare("PRAGMA table_info(results)")?;

            let mut existing_columns = Vec::new();

            while let State::Row = statement.next()? {
                let name: String = statement.read(1)?;
                existing_columns.push(name);
            }

            for (name, data_type) in required_columns {
                if !existing_columns.iter().any(|column| column == name) {
                    conn.execute(format!(
                        "ALTER TABLE results ADD COLUMN {} {}",
                        name, data_type
                    ))?;
                    println!("Added missing column '{}'", name);
                }
            }

            Ok(())
        }
        None => Ok(()),
    }
}

fn main() -> Result<()> {
    let log_file_path = get_log_file_path()?;
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(&log_file_path)?,
    )?;

    // Get and parse arguments
    let args = std::env::args();
    let mut args_as_vec = vec![];
    for arg in args.enumerate() {
        args_as_vec.push(arg.1)
    }
    args_as_vec.remove(0);
    log::info!("{:?}", args_as_vec);
    if args_as_vec.len() > 0 {
        // Some arguments were given

        // Currently fixing the database is the only operation
        // that can be done with the arguments so only check for it
        if args_as_vec[0] == "--fix-database" {
            fix_database()?;
            return Ok(());
        }
    }

    log::info!("Application started");
    color_eyre::install()?;
    ratatui::run(|terminal| Tuipe::new().run(terminal))
}
