#![forbid(unsafe_code)]
use bryggio_lib::brewery::Brewery;
use bryggio_lib::config;
use std::process::Command;

fn main() {
    let config_file = "./Bryggio.toml";
    let config = match config::Config::new(&config_file) {
        Ok(config) => config,
        Err(err) => {
            println!(
                "Invalid config file '{}'. Error: {}. Using default.",
                config_file,
                err.to_string()
            );
            config::Config::default()
        }
    };
    let mut brewery = Brewery::init_from_config(&config);
    brewery.run();
}
