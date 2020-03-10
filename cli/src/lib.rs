use bryggio_lib::api;
use serde_json;
use ureq;
use url::Url;
pub mod opts;

/// TODO: Generic response
pub fn send(request: &Url) -> Result<api::Response<f32>, serde_json::error::Error> {
    println!("Sending request: '{}'.", request);
    let response_raw = ureq::get(request.as_str()).call();
    serde_json::from_str(&response_raw.into_string().unwrap())
}
