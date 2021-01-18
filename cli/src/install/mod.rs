use lazy_static::lazy_static;
use log::info;
use std::fs;
use std::io;
use std::path::Path;
use url::Url;
mod github_api;
mod nats;
pub mod server;

pub(crate) fn download_file<P: AsRef<Path>>(dest_path: P, source_url: &Url) {
    let dest_path = dest_path.as_ref();
    if dest_path.exists() {
        fs::remove_file(dest_path).expect("Could not remove file.");
    }
    let mut file = fs::File::create(dest_path).expect("Could not create file");
    info!("Downloading NATS server from '{}'", source_url);
    io::copy(
        &mut ureq::get(source_url.as_str())
            .call()
            .expect("ureq called failed")
            .into_reader(),
        &mut file,
    )
    .expect("Could not write to file");
}

lazy_static! {
    static ref SEMVER_VERSION_PATTERN: regex::Regex = regex::Regex::new(
        r"v([0-9].[0-9].[0-9])"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
}
