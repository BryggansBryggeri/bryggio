use reqwest;
use std::net;

#[derive(Debug)]
struct Api {
    socket: net::SocketAddr,
}

impl Api {
    fn new(ip: net::IpAddr, port: u16) -> Self {
        Api {
            socket: net::SocketAddr::new(ip, port),
        }
    }
    fn send(&self, command: &str) {
        println!("Sending command {}.", command);
        let url = &format!("http://{}/{}", self.socket, command);
        let response = reqwest::get(url).unwrap().text().unwrap();
        println!("{}", response);
    }
}

fn main() {
    let ip = net::IpAddr::V4(net::Ipv4Addr::LOCALHOST);
    let port = 8001;
    let api = Api::new(ip, port);
    api.send("list_available_sensors");
}
