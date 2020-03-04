use std::fs::File;
use std::io::prelude::*;

pub fn get_bryggio_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn read_file_to_string(file_name: &str) -> std::io::Result<String> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
