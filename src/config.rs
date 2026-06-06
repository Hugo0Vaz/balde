use std::path::{Path, PathBuf};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Config {
    bucket: BucketConfig,
    baldes: HashMap<String, Balde>,
}

#[derive(Debug, Deserialize)]
struct BucketConfig {
    name: String,
    region: String,
    url: String,
    key_id: String,
    key: String,
}

#[derive(Debug, Deserialize)]
struct Balde {
    name: String,
    path: PathBuf,
    #[serde(default)]
    filter: Vec<String>,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
