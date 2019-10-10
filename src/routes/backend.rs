use crate::api;
use crate::brewery;
use rocket::State;
use rocket_contrib::json;
use std::collections::HashMap;

#[get("/start_controller?<controller_id>&<sensor_id>&<actor_id>")]
pub fn start_controller(
    controller_id: String,
    sensor_id: String,
    actor_id: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let mut id = HashMap::new();
    id.insert(String::from("controller"), controller_id);
    id.insert(String::from("sensor"), sensor_id);
    id.insert(String::from("actor"), actor_id);
    let request = api::Request {
        command: brewery::Command::StartController,
        id,
        parameter: None,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/stop_controller?<controller_id>")]
pub fn stop_controller(
    controller_id: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let mut id = HashMap::new();
    id.insert(String::from("controller"), controller_id);
    let request = api::Request {
        command: brewery::Command::StopController,
        id,
        parameter: None,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/set_target_signal?<controller_id>&<new_target>")]
pub fn set_target_signal(
    controller_id: String,
    new_target: Option<f32>,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let mut id = HashMap::new();
    id.insert(String::from("controller"), controller_id);
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
    sensor_id: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let mut id = HashMap::new();
    id.insert(String::from("sensor"), sensor_id);
    let request = api::Request {
        command: brewery::Command::GetMeasurement,
        id,
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
