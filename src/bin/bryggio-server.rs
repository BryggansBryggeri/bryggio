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
    let mut brewery = brewery::Brewery::new(brew_endpoint);
    brewery.init_from_config(&config);
    let brewery_thread = thread::spawn(move || brewery.run());

    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::backend::start_controller,
                routes::backend::stop_controller,
                routes::backend::get_measurement,
                routes::backend::set_target_signal,
                routes::backend::get_target_signal,
                routes::backend::get_control_signal,
                routes::backend::add_sensor,
                routes::backend::get_full_state,
                routes::backend::list_available_sensors,
                routes::backend::get_config,
                routes::backend::get_brewery_name,
                routes::backend::get_bryggio_version,
            ],
        )
        .register(catchers![routes::backend::not_found])
        .manage(web_endpoint)
        .manage(config)
        .launch();

    brewery_thread.join().expect("Brewery thread panicked.");
}
