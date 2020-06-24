#![forbid(unsafe_code)]
use bryggio_lib::brewery::Brewery;
use bryggio_lib::config;
use bryggio_lib::pub_sub::{nats_client::run_nats_server, PubSubClient};

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
    let mut nats_server_child = run_nats_server(&config.nats);
    let mut brewery = Brewery::init_from_config(&config);
    brewery.client_loop();
    let res = nats_server_child.kill();
    println!("{:?}", res);
}
