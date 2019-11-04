#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use bryggio::api;
use bryggio::brewery;
use bryggio::config;
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
                api::routes::start_controller,
                api::routes::stop_controller,
                api::routes::get_measurement,
                api::routes::set_target_signal,
                api::routes::get_target_signal,
                api::routes::get_control_signal,
                api::routes::add_sensor,
                api::routes::get_full_state,
                api::routes::list_available_sensors,
                api::routes::get_config,
                api::routes::get_brewery_name,
                api::routes::get_bryggio_version,
            ],
        )
        .register(catchers![api::routes::not_found])
        .manage(web_endpoint)
        .manage(config)
        .launch();

    brewery_thread.join().expect("Brewery thread panicked.");
}
