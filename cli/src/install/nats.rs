use lazy_static::lazy_static;
use log::info;
use semver::Version;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str;

const GITHUB_META_DATA: &str = " https://api.github.com/repos/nats-io/nats-server/releases/latest";

#[cfg(target_arch = "x86_64")]
const NATS: &str = "https://github.com/nats-io/nats-server/releases/download/v2.1.4/nats-server-v2.1.4-linux-amd64.zip";

#[cfg(target_arch = "arm")]
const NATS: &str = "https://github.com/nats-io/nats-server/releases/download/v2.1.4/nats-server-v2.1.4-linux-arm7.zip";

lazy_static! {
    static ref NATS_SERVER_VERSION_PATTERN: regex::Regex = regex::Regex::new(
        r"v([0-9].[0-9].[0-9])"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
    static ref GITHUB_VERSION_PATTERN: regex::Regex = regex::Regex::new(
        r"v([0-9].[0-9].[0-9])"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
}

#[derive(Deserialize)]
struct NatsServerRelease {
    tag_name: String,
}

pub(crate) fn download_server(nats_path: &Path, update: bool) {
    if should_download(nats_path, update) {
        if local_nats_present(nats_path) {
            fs::remove_file(nats_path).expect("Could not remove file.");
        }
        let mut file = fs::File::create(nats_path).expect("Could not create file");
        info!("Downloading NATS server");
        io::copy(&mut ureq::get(NATS).call().into_reader(), &mut file)
            .expect("Could not write to file");
    }
}

fn should_download(nats_path: &Path, update: bool) -> bool {
    if !local_nats_present(nats_path) {
        true
    } else {
        update && local_nats_version(nats_path) < latest_nats_release()
    }
}

fn local_nats_present(nats_path: &Path) -> bool {
    info!(
        "Checking for existing NATS server binary at '{}'",
        nats_path.to_str().unwrap()
    );
    nats_path.exists()
}

fn local_nats_version(nats_path: &Path) -> Version {
    let output: Vec<u8> = Command::new(nats_path)
        .arg("--version")
        .output()
        .expect(&format!(
            "Failed to get version from '{}'",
            nats_path.to_str().unwrap()
        ))
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    let raw_string = GITHUB_VERSION_PATTERN
        .captures(output)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    Version::parse(raw_string).unwrap()
}

fn latest_nats_release() -> Version {
    let response_raw = ureq::get(GITHUB_META_DATA).call().into_string().unwrap();
    let meta_data: NatsServerRelease = serde_json::from_str(&response_raw).unwrap();
    let raw_string = GITHUB_VERSION_PATTERN
        .captures(&meta_data.tag_name)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    Version::parse(raw_string).unwrap()
}
