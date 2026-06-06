
mod config;

fn main() {
    // 1. Processa os argumentos da linha de comando
    //
    // 2. Lê as configurações do `balde.toml`
    let config = config::load_config("balde.toml").expect("Failed to load balde.toml");
}
