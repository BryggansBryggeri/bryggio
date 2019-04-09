use rocket::http::RawStr;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::json;
use std::collections::HashMap;

use bryggio::brewery;
use bryggio::control;

// TODO: Return JSON objects instead of templates?
// {success: true, other_key, val}
// Easier to split backend into backend frontend

#[get("/start_measure")]
pub fn start_measure(
    api_endpoint: State<brewery::APIWebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut sender = api_endpoint.sender.lock().unwrap();
    sender.send("test".to_string());

    let mut receiver = api_endpoint.receiver.lock().unwrap();
    let answer = receiver.recv().unwrap();
    println!("Brew to web: {}", answer);

    let mut response = HashMap::new();
    response.insert("success".to_string(), "true".to_string());
    json::Json(response)
}

#[get("/stop_measure")]
pub fn stop_measure(
    api_endpoint: State<brewery::APIWebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "true".to_string());
    json::Json(response)
}

#[get("/set_target_temp?<temp>")]
pub fn set_target_temp(
    temp: Option<&RawStr>,
    api_endpoint: State<brewery::APIWebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}

#[get("/get_temp")]
pub fn get_temp(
    api_endpoint: State<brewery::APIWebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}

#[get("/get_full_state")]
pub fn get_full_state(
    api_endpoint: State<brewery::APIWebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}
