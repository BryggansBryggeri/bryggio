use bryggio_lib::api;
use reqwest;
use serde_json;
use std::net;
pub mod opts;

#[derive(Debug)]
pub struct Api {
    socket: net::SocketAddr,
}

impl Api {
    pub fn new(ip: net::IpAddr, port: u16) -> Self {
        Api {
            socket: net::SocketAddr::new(ip, port),
        }
    }
    /// TODO: Generic response
    pub fn send(&self, command: &str) -> Result<api::Response<f32>, serde_json::error::Error> {
        println!("Sending command: '{}'.", command);
        let url = &format!("http://{}/{}", self.socket, command);
        let response_raw = reqwest::get(url).unwrap().text().unwrap();
        serde_json::from_str(&response_raw)
    }
}
