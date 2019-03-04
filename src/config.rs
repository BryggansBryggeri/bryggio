use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Deserialize)]
pub struct Config {
    name: String,
}

impl Config {
    pub fn new(config_file: &'static str) -> Config {
        let mut f = File::open(config_file).expect("Unable to open file");
        let mut data = String::new();
        f.read_to_string(&mut data).expect("Unable to read string");
        let config: Config = toml::de::from_str(&data).unwrap();
        config
    }
}
