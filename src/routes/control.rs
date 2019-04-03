use rocket::http::RawStr;
use rocket::response::Redirect;
use rocket::State;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

use bryggio::brewery::BrewState;
use bryggio::control;
use bryggio::control::Control;

// TODO: Return JSON objects instead of templates?
// {success: true, other_key, val}
// Easier to split backend into backend frontend

#[get("/start_measure")]
pub fn start_measure(brew_state: State<BrewState>) -> Redirect {
    println!("Starting measurement");
    let mut state = brew_state.controller.write().unwrap();
    *state = control::Mode::Automatic;
    Redirect::to("/")
}

#[get("/stop_measure")]
pub fn stop_measure(brew_state: State<BrewState>) -> Redirect {
    println!("Stopping measurement");
    let mut state = brew_state.controller.write().unwrap();
    *state = control::Mode::Inactive;
    Redirect::to("/")
}

#[get("/set_target_temp?<temp>")]
pub fn set_target_temp(temp: Option<&RawStr>, brew_state: State<BrewState>) -> String {
    temp.map(|temp| format!("Target: {} C", temp))
        .unwrap_or_else(|| "Invalid target".into())
}

#[get("/get_temp")]
pub fn get_temp() -> String {
    let temp = Some("63");
    temp.map(|temp| format!("Current temp: {} C", temp))
        .unwrap_or_else(|| "Could not read temp".into())
}
