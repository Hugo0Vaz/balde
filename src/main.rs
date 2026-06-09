use std::process::exit;

mod config;
mod log;

fn main() {
    let config = match config::load_config() {
        Ok(config) => config,
        Err(_e) => {
            println!("It was not possible to load the config with error: {}", _e);
            exit(127);
        },
    };

    println!("Debug mode is set to {}", config.debug);
    println!("We're logging to {:?}", config.log_path);
    println!("LogLevel is set to: {:?}", config.log_level);
    println!("The bucket url is: {}", config.bucket_url);
    println!("With bucket key: {}", config.bucket_key);
    println!("Bucket on region: {}", config.bucket_region);
    println!("The bucket id is: {}", config.bucket_key_id);
    println!("Bucket name is: {}", config.bucket_name);

    for balde in config.baldes {
        let id = balde.0;
        let b = balde.1;

        let name = b.name;
        let filter = b.filter;
        let path = b.path;
        println!("  Pretty Name: {}\n  ID: {}\n  Path: {:?}\n  Filter: {:?}", name, id,path, filter);
    }
}
