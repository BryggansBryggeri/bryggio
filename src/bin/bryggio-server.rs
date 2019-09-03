#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use bryggio::api;
use bryggio::brewery;
use bryggio::config;
use bryggio::routes;
use std::thread;

fn main() {
    let config_file = "./Bryggio.toml";
    let config = match config::Config::new(&config_file) {
        Ok(config) => config,
        Err(err) => {
            println!(
                "Invalid config file '{}'. Error: {}",
                config_file,
                err.to_string()
            );
            return;
        }
    };
    let (web_endpoint, brew_endpoint) = api::create_api_endpoints();
    let mut brewery = brewery::Brewery::new(&config, brew_endpoint);
    let brewery_thread = thread::spawn(move || brewery.run());

    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::backend::start_controller,
                routes::backend::stop_controller,
                routes::backend::get_measurement,
                routes::backend::set_target_signal,
                routes::backend::get_full_state
            ],
        )
        .manage(web_endpoint)
        .launch();

    brewery_thread.join().expect("Brewery thread panicked.");
}
