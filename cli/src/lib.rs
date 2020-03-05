pub mod opts;
use reqwest;
use std::net;

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
    pub fn send(&self, command: &str) {
        println!("Sending command {}.", command);
        let url = &format!("http://{}/{}", self.socket, command);
        let response = reqwest::get(url).unwrap().text().unwrap();
        println!("{}", response);
    }
}
