// use std::path::PathBuf;

use serde::{Deserialize};

// fn configure_logging(path: PathBuf, level: LogLevel) -> Result<(), Box<dyn std::error::Error>> {
//     todo!();
// }

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}
