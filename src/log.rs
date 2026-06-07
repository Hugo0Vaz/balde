// use std::path::PathBuf;

use serde::{Deserialize};

// fn configure_logging(path: PathBuf, level: LogLevel) -> Result<(), Box<dyn std::error::Error>> {
//     todo!();
// }

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}
