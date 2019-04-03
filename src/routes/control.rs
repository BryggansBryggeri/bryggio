use rocket::http::RawStr;
use rocket::response::Redirect;
use rocket::State;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

use rustbeer::brewery::BrewState;
use rustbeer::control;
use rustbeer::control::Control;

#[get("/start_measure")]
pub fn start_measure(brew_state: State<BrewState>) -> Redirect {
    println!("Starting measurement");
    let mut state = brew_state.controller.lock().unwrap();
    *state = true;
    Redirect::to("/")
}

#[get("/stop_measure")]
pub fn stop_measure(brew_state: State<BrewState>) -> Redirect {
    println!("Stopping measurement");
    let mut state = brew_state.controller.lock().unwrap();
    *state = false;
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
