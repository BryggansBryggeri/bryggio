#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rustbeer::brewery;
use rustbeer::config;

mod routes;

fn main() {
    let config_file = "./config.toml";
    let config = config::Config::new(&config_file);
    let brewery = brewery::Brewery::new(&config);

    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::index::index,
                routes::control::measure,
                routes::control::get_temp,
                routes::control::set_target_temp
            ],
        )
        .launch();
}
