use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Deserialize)]
pub struct Config {
    path: &'static str,
}

impl Config {
    pub fn new(config_file: &'static str) -> Config {
        //let mut file = match File::open(config_file) {
        //    Ok(file) => file,
        //    Err(_) => panic!("Could not find config file"),
        //};

        //let mut config_toml = String::new();
        //file.read_to_string(&mut config_toml).expect("Wrong");

        //let config: Config = toml::from_str(&config_toml).unwrap();
        //println!("config_toml");
        //config
        let mut f = File::open("/etc/hosts").expect("Unable to open file");
        let mut data = String::new();
        f.read_to_string(&mut data).expect("Unable to read string");
        let config: Config = toml::from_str(&data).unwrap();
        println!("{}", data);
        config
    }
}
