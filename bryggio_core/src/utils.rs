use semver::Version;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub const fn bryggio_version_str() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn bryggio_version() -> Option<Version> {
    Version::parse(bryggio_version_str()).ok()
}

/// File read helper function.
pub fn read_file_to_string<P: AsRef<Path>>(file_name: P) -> std::io::Result<String> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
