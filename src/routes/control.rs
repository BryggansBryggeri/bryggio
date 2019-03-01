use rocket::http::RawStr;
use rocket::response::Redirect;
use std::thread;

use rustbeer::control;
use rustbeer::control::Control;

#[get("/measure")]
pub fn measure() -> Redirect {
    let offset_on = 5.0;
    let offset_off = 3.0;
    let mut control = control::HysteresisControl::new(offset_on, offset_off).unwrap();
    thread::spawn(move || control.run());
    Redirect::to("/")
}

#[get("/set_target_temp?<temp>")]
pub fn set_target_temp(temp: Option<&RawStr>) -> String {
    temp.map(|temp| format!("Target: {} C", temp))
        .unwrap_or_else(|| "Invalid target".into())
}

#[get("/get_temp")]
pub fn get_temp() -> String {
    let temp = Some("63");
    temp.map(|temp| format!("Current temp: {} C", temp))
        .unwrap_or_else(|| "Could not read temp".into())
}
