use std::fs;
use std::io;
use std::path::Path;
mod nats;
use log::info;
pub mod server;
use url::Url;

pub(crate) fn download_file<P: AsRef<Path>>(dest_path: P, source_url: &Url) {
    let dest_path = dest_path.as_ref();
    if dest_path.exists() {
        fs::remove_file(dest_path).expect("Could not remove file.");
    }
    let mut file = fs::File::create(dest_path).expect("Could not create file");
    info!("Downloading NATS server from '{}'", source_url);
    io::copy(
        &mut ureq::get(source_url.as_str()).call().into_reader(),
        &mut file,
    )
    .expect("Could not write to file");
}
