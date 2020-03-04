#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use bryggio_lib::api;
use bryggio_lib::brewery;
use bryggio_lib::config;
use rocket::http::Method; // 1.
use rocket_cors::{
    AllowedHeaders,
    AllowedOrigins,
    Cors,
    CorsOptions, // 3.
    Error,       // 2.
};
use std::thread;

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        // 4.
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "http://localhost:8000",
        "http://0.0.0.0:8000",
    ]);

    CorsOptions {
        // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(), // 1.
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin", // 6.
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

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
        .attach(make_cors())
        .manage(web_endpoint)
        .manage(config)
        .launch();

    brewery_thread.join().expect("Brewery thread panicked.");
}
