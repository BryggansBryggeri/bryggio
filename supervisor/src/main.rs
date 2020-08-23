#![forbid(unsafe_code)]
use bryggio_lib::brewery::Brewery;
use bryggio_lib::config;
use bryggio_lib::pub_sub::{nats_client::run_nats_server, PubSubClient, PubSubError};

fn main() -> Result<(), PubSubError> {
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
    let mut nats_server_child = run_nats_server(&config.nats)?;
    let brewery = Brewery::init_from_config(&config);
    brewery.client_loop()?;
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()))
}
