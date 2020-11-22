#![forbid(unsafe_code)]
use bryggio_lib::config;
use bryggio_lib::pub_sub::{nats_client::run_nats_server, PubSubClient, PubSubError};
use bryggio_lib::supervisor::Supervisor;

fn main() -> Result<(), PubSubError> {
    let config_file = "./bryggio.toml";
    let config = match config::Config::try_new(&config_file) {
        Ok(config) => config,
        Err(err) => {
            return Err(PubSubError::Configuration(format!(
                "Invalid config file '{}'. Error: {}.",
                config_file,
                err.to_string()
            )));
        }
    };
    let mut nats_server_child = run_nats_server(&config.nats)?;
    let supervisor = Supervisor::init_from_config(&config);
    supervisor.client_loop()?;
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()))
}
