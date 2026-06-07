use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct CliConfig {
    #[arg(long)]
    pub balde_config_file: Option<PathBuf>,
    #[arg(long)]
    pub log_level: Option<String>,
    #[arg(long)]
    pub log_path: Option<PathBuf>,
}
