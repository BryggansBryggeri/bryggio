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
    let brewery = Brewery::init_from_config(&config);
    match brewery.client_loop() {
        Ok(()) => {}
        Err(err) => println!("Supervisor loop ended with err: {}", err.to_string()),
    };
    let res = nats_server_child.kill();
    println!("{:?}", res);
}
