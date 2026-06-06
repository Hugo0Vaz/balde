use std::{fs::{canonicalize, read_to_string}, path::{Path, PathBuf}};
use serde::Deserialize;
use toml::from_str;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    bucket: BucketConfig,
    baldes: HashMap<String, Balde>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BucketConfig {
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

pub fn load_config(path: impl AsRef<Path>) -> Result<Config, Box<dyn std::error::Error>> {
    let abs_path: PathBuf = canonicalize(path)?;
    let content = read_to_string(abs_path)?;
    let config: Config = from_str(&content)?;
    Ok(config)
}
