#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use rustbeer::brewery;
use rustbeer::config;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

mod routes;

fn main() {
    let config_file = "./config.toml";
    let config = config::Config::new(&config_file);
    let brew_state = Arc::new(AtomicBool::new(false));
    let brewery = brewery::Brewery::new(&config);

    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::serve_static::files,
                routes::index::index,
                routes::control::measure,
                routes::control::get_temp,
                routes::control::set_target_temp
            ],
        )
        .manage(brew_state.clone())
        .attach(Template::fairing())
        .launch();
}
