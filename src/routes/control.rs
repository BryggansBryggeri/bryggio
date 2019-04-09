use crate::api;
use crate::brewery;
use rocket::http::RawStr;
use rocket::State;
use rocket_contrib::json;
use std::collections::HashMap;

#[get("/start_measure")]
pub fn start_measure(api_endpoint: State<api::WebEndpoint>) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();

    let request = api::Request {
        command: brewery::Command::StartController,
        id: None,
        parameter: None,
    };
    match api_endpoint.send_and_wait_for_response(request) {
        Ok(api_response) => {
            response.insert("success".to_string(), api_response.success.to_string());
        }
        Err(e) => {
            response.insert("success".to_string(), "false".to_string());
            response.insert("response".to_string(), "error".to_string());
        }
    }
    json::Json(response)
}

#[get("/stop_measure")]
pub fn stop_measure(api_endpoint: State<api::WebEndpoint>) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    let request = api::Request {
        command: brewery::Command::StopController,
        id: None,
        parameter: None,
    };
    match api_endpoint.send_and_wait_for_response(request) {
        Ok(api_response) => {
            response.insert("success".to_string(), api_response.success.to_string());
        }
        Err(e) => {
            response.insert("success".to_string(), "false".to_string());
            response.insert("response".to_string(), "error".to_string());
        }
    }
    json::Json(response)
}

#[get("/set_target_temp?<temp>")]
pub fn set_target_temp(
    temp: Option<&RawStr>,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}

#[get("/get_temp")]
pub fn get_temp(api_endpoint: State<api::WebEndpoint>) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}

#[get("/get_full_state")]
pub fn get_full_state(
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}
