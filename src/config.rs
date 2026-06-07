use std::{fs::{canonicalize, read_to_string}, path::{Path, PathBuf}};
use serde::Deserialize;
use toml::from_str;
use std::collections::HashMap;

use crate::log::LogLevel;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TomlConfig {
    bucket: TomlBucketConfig,
    baldes: HashMap<String, Balde>,
    log: TomlLogConfig,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TomlLogConfig {
    path: PathBuf,
    level: LogLevel,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TomlBucketConfig {
    name: String,
    region: String,
    url: String,
    key_id: String,
    key: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Balde {
    name: String,
    #[serde(default)]
    filter: Vec<String>,
    path: PathBuf,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<TomlConfig, Box<dyn std::error::Error>> {
    let abs_path: PathBuf = canonicalize(path)?;
    let content: String = read_to_string(abs_path)?;
    let config: TomlConfig = from_str(&content)?;
    Ok(config)
}
