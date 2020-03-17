use lazy_static::lazy_static;
use log::info;
use semver::Version;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str;
use url::{ParseError, Url};

const GITHUB_META_DATA: &str = " https://api.github.com/repos/nats-io/nats-server/releases/latest";

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
    assets: Vec<ReleaseArchitecture>,
}

impl NatsServerRelease {
    fn url(&self) -> Url {
        #[cfg(target_arch = "x86_64")]
        #[cfg(target_os = "linux")]
        let os_arch = "linux-amd64";
        #[cfg(target_os = "macos")]
        let os_arch = "darwin-amd64";
        #[cfg(target_os = "windows")]
        let os_arch = "windows-amd64";
        #[cfg(target_arch = "arm")]
        let os_arch = "arm7";
        self.assets
            .iter()
            .filter(|x| x.name.contains(os_arch))
            .filter(|x| x.name.contains(".zip"))
            .map(|x| Url::parse(&x.url))
            .last()
            .unwrap()
            .unwrap()
    }
}

#[derive(Deserialize)]
struct ReleaseArchitecture {
    #[serde(rename = "browser_download_url")]
    url: String,
    name: String,
}

pub(crate) fn download_server(nats_path: &Path, update: bool) {
    if should_download(nats_path, update) {
        if local_nats_present(nats_path) {
            fs::remove_file(nats_path).expect("Could not remove file.");
        }
        let mut file = fs::File::create(nats_path).expect("Could not create file");
        let nats_url = latest_nats_url();
        info!("Downloading NATS server from '{}'", nats_url);
        io::copy(
            &mut ureq::get(nats_url.as_str()).call().into_reader(),
            &mut file,
        )
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

fn latest_nats_url() -> Url {
    let response_raw = ureq::get(GITHUB_META_DATA).call().into_string().unwrap();
    let meta_data: NatsServerRelease = serde_json::from_str(&response_raw).unwrap();
    meta_data.url()
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
