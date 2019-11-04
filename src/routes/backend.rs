use crate::api;
use crate::brewery;
use crate::config;
use crate::sensor;
use crate::utils;
use rocket;
use rocket_contrib::json;

#[get("/start_controller?<controller_id>&<sensor_id>&<actor_id>")]
pub fn start_controller(
    controller_id: String,
    sensor_id: String,
    actor_id: String,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::StartController {
        controller_id,
        sensor_id,
        actor_id,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/stop_controller?<controller_id>")]
pub fn stop_controller(
    controller_id: String,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::StopController { controller_id };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/set_target_signal?<controller_id>&<new_target_signal>")]
pub fn set_target_signal(
    controller_id: String,
    new_target_signal: f32,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::SetTarget {
        controller_id,
        new_target_signal,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/get_target_signal?<controller_id>")]
pub fn get_target_signal(
    controller_id: String,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::GetTarget { controller_id };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/get_control_signal?<controller_id>")]
pub fn get_control_signal(
    controller_id: String,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::GetControlSignal { controller_id };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}
#[get("/get_measurement?<sensor_id>")]
pub fn get_measurement(
    sensor_id: String,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::GetMeasurement { sensor_id };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/add_sensor?<sensor_id>&<sensor_type>")]
pub fn add_sensor(
    sensor_id: String,
    sensor_type: String,
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let sensor_type = sensor::SensorType::from_str(sensor_type);
    let request = brewery::Command::AddSensor {
        sensor_id,
        sensor_type,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/get_full_state")]
pub fn get_full_state(
    api_endpoint: rocket::State<api::WebEndpoint<f32>>,
) -> json::Json<api::Response<f32>> {
    let request = brewery::Command::GetFullState;
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_api_response(api_response)
}

#[get("/list_available_sensors")]
pub fn list_available_sensors() -> json::Json<api::Response<Vec<sensor::dsb1820::DSB1820Address>>> {
    let response = match sensor::dsb1820::list_available() {
        Ok(available_sensors) => api::Response {
            result: Some(available_sensors),
            message: None,
            success: true,
        },
        Err(err) => api::Response {
            result: None,
            message: Some(err.to_string()),
            success: false,
        },
    };
    api::generate_api_response(Ok(response))
}

#[get("/get_config")]
pub fn get_config(
    config: rocket::State<config::Config>,
) -> json::Json<api::Response<config::Config>> {
    let config = (*config.inner()).clone();
    let response = api::Response {
        result: Some(config),
        message: None,
        success: true,
    };
    api::generate_api_response(Ok(response))
}

#[get("/get_brewery_name")]
pub fn get_brewery_name(
    config: rocket::State<config::Config>,
) -> json::Json<api::Response<String>> {
    let brewery_name = config.general.brewery_name.clone();
    let response = api::Response {
        result: Some(brewery_name),
        message: None,
        success: true,
    };
    api::generate_api_response(Ok(response))
}

#[get("/get_bryggio_version")]
pub fn get_bryggio_version() -> json::Json<api::Response<String>> {
    let version = utils::get_bryggio_version().into();
    let response = api::Response {
        result: Some(version),
        message: None,
        success: true,
    };
    api::generate_api_response(Ok(response))
}

#[catch(404)]
pub fn not_found(req: &rocket::Request) -> json::Json<api::Response<f32>> {
    let error_response = api::Response {
        success: false,
        result: None,
        message: Some(format!(
            "Error 404: '{}' is not a valid API call",
            req.uri()
        )),
    };
    json::Json(error_response)
}
