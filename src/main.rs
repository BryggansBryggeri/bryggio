#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::RawStr;
use rustbeer::run;
use rustbeer::control;
use rustbeer::hardware;
use rustbeer::config::Config;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/set_target_temp?<temp>")]
fn set_target_temp(temp: Option<&RawStr>) -> String {
        temp.map(|temp| format!("Target: {} C", temp))
            .unwrap_or_else(|| "Invalid target".into())
}

#[get("/get_temp")]
fn get_temp() -> String {
        let temp = Some("63");
        temp.map(|temp| format!("Current temp: {} C", temp))
            .unwrap_or_else(|| "Could not read temp".into())
}

fn main() {
    //rocket::ignite().mount("/", routes![index, get_temp, set_target_temp]).launch();
    let control = control::HysteresisControl{
        state: true,
        offset: 5.0
    };
    hardware::set_gpio(21, 1, "aba");
    //control.run();
}
