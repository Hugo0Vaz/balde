use std::{path::PathBuf, process::exit};

use clap::Parser;

mod config;
mod log;
mod cli;

use cli::CliConfig;

fn main() {
    let cli_config = CliConfig::parse();

    let config_path = match cli_config.balde_config_file {
        Some(path) => path,
        None => PathBuf::from("~/.config/balde/balde.toml"),
    };

    let config = match config::load_config(config_path) {
        Ok(config) => config,
        Err(_e) => {
            println!("It was not possible to load the config with error: {}", _e);
            exit(127);
        },
    };

    println!("Config loaded: {:?}", config);
    //
    // 3. Inicia o loop de sincronização dos arquivos
    println!("Baldeee");
}
