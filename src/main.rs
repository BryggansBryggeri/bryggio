#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use bryggio::brewery;
use bryggio::config;
use bryggio::control::Control;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

mod routes;

fn main() {
    let config_file = "./Bryggio.toml";
    let config = config::Config::new(&config_file);
    let brew_state = brewery::Brewery::generate_state(&config);

    let mut brewery = brewery::Brewery::new(&config, brew_state.clone());
    thread::spawn(move || brewery.run());

    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::serve_static::general_files,
                routes::serve_static::javascript,
                routes::index::index,
                routes::control::start_measure,
                routes::control::stop_measure,
                routes::control::get_temp,
                routes::control::set_target_temp
            ],
        )
        .manage(brew_state.clone())
        .attach(Template::fairing())
        .launch();
}
