use std::{fs::{canonicalize, read_to_string}, path::{Path, PathBuf}};
use std::collections::HashMap;

use serde::Deserialize;
use toml::from_str;

use clap::Parser;

use crate::log::LogLevel;

#[derive(Debug, Deserialize)]
struct TomlConfig {
    bucket: TomlBucketConfig,
    baldes: HashMap<String, Balde>,
    log: TomlLogConfig,
}

#[derive(Debug, Deserialize)]
struct TomlLogConfig {
    path: PathBuf,
    level: LogLevel,
}

#[derive(Debug, Deserialize)]
struct TomlBucketConfig {
    name: String,
    region: String,
    url: String,
    key_id: String,
    key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Balde {
    pub name: String,
    #[serde(default)]
    pub filter: Vec<String>,
    pub path: PathBuf,
}


#[derive(Debug, Default)]
pub struct AppConfig {
    pub log_level: LogLevel,
    pub log_path: PathBuf,

    pub debug: bool,

    pub bucket_name: String,
    pub bucket_region: String,
    pub bucket_url: String,
    pub bucket_key_id: String,
    pub bucket_key: String,

    pub baldes: HashMap<String, Balde>,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct CliConfig {
    #[arg(long)]
    pub balde_config_file: Option<PathBuf>,
    #[arg(long)]
    pub log_level: Option<String>,
    #[arg(long)]
    pub log_path: Option<PathBuf>,
    #[arg(short, long)]
    debug: bool,
}

fn load_toml_config(path: impl AsRef<Path>) -> Result<TomlConfig, Box<dyn std::error::Error>> {
    let abs_path: PathBuf = canonicalize(path)?;
    let content: String = read_to_string(abs_path)?;
    let config: TomlConfig = from_str(&content)?;
    Ok(config)
}

fn config_file(cli_config: &CliConfig) -> PathBuf {
    match &cli_config.balde_config_file {
        Some(path) => path.to_path_buf(),
        None => PathBuf::from("~/.config/balde/balde.toml")
    }
}

fn merge_configs(cli_config: &CliConfig, file_config: &TomlConfig) -> AppConfig {
    let log_level = match cli_config.log_level.as_deref() {
        Some("trace") => LogLevel::Trace,
        Some("debug") => LogLevel::Debug,
        Some("info")  => LogLevel::Info,
        Some("warn")  => LogLevel::Warn,
        Some("error") => LogLevel::Error,
        Some("fatal") => LogLevel::Fatal,
        Some(_)        => file_config.log.level.clone(),
        None           => file_config.log.level.clone(),
    };

    let log_path = cli_config
        .log_path
        .clone()
        .unwrap_or_else(|| file_config.log.path.clone());

    AppConfig {
        log_level,
        log_path,
        debug: cli_config.debug,
        bucket_name: file_config.bucket.name.clone(),
        bucket_region: file_config.bucket.region.clone(),
        bucket_url: file_config.bucket.url.clone(),
        bucket_key_id: file_config.bucket.key_id.clone(),
        bucket_key: file_config.bucket.key.clone(),
        baldes: file_config.baldes.clone(),
    }

}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {

    let cli_config = CliConfig::parse();
    let file_config = load_toml_config(config_file(&cli_config))?;

    let config = merge_configs(&cli_config, &file_config);

    Ok(config)
}
