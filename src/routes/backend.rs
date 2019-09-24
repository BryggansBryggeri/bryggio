use crate::api;
use crate::brewery;
use rocket::State;
use rocket_contrib::json;
use std::collections::HashMap;

#[get("/start_controller?<id>")]
pub fn start_controller(
    id: Option<String>,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = api::Request {
        command: brewery::Command::StartController,
        id,
        parameter: None,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/stop_controller?<id>")]
pub fn stop_controller(
    id: Option<String>,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = api::Request {
        command: brewery::Command::StopController,
        id,
        parameter: None,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/set_target_signal?<id>&<new_target>")]
pub fn set_target_signal(
    id: Option<String>,
    new_target: Option<f32>,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = api::Request {
        command: brewery::Command::SetTarget,
        id,
        parameter: new_target,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/get_measurement?<sensor_id>")]
pub fn get_measurement(
    sensor_id: Option<String>,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = api::Request {
        command: brewery::Command::GetMeasurement,
        id: sensor_id,
        parameter: None,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/get_full_state")]
pub fn get_full_state(
    _api_endpoint: State<api::WebEndpoint>,
) -> json::Json<HashMap<String, String>> {
    let mut response = HashMap::new();
    response.insert("success".to_string(), "false".to_string());
    response.insert("message".to_string(), "Not implemented yet".to_string());
    json::Json(response)
}
