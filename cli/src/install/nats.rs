use lazy_static::lazy_static;
use log::{debug, info};
use semver::Version;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str;
use url::Url;

pub(crate) fn generate_config() {}

pub(crate) fn download_server(nats_path: &Path, update: bool) {
    let (local_exists, local_version) = local_meta_data(nats_path);
    if !update && local_exists {
        info!("Keeping existing NATS server.");
        return;
    }
    let (latest_nats_version, nats_url) = github_meta_data();
    if should_download(local_exists, update, local_version, latest_nats_version) {
        if local_exists {
            fs::remove_file(nats_path).expect("Could not remove file.");
        }
        let mut file = fs::File::create(nats_path).expect("Could not create file");
        info!("Downloading NATS server from '{}'", nats_url);
        io::copy(
            &mut ureq::get(nats_url.as_str()).call().into_reader(),
            &mut file,
        )
        .expect("Could not write to file");
    }
}

fn should_download(
    local_nats_exists: bool,
    update: bool,
    local_version: Option<Version>,
    latest_version: Version,
) -> bool {
    if !local_nats_exists {
        info!("No local NATS server found, downloading");
        true
    } else {
        info!(
            "Newer NATS server released ({}), downloading",
            latest_version
        );
        update && local_version.unwrap() < latest_version
    }
}

fn local_meta_data(nats_path: &Path) -> (bool, Option<Version>) {
    let local_exists = local_nats_present(nats_path);
    if local_exists {
        let version = local_nats_version(nats_path);
        info!("NATS server v{} already installed.", version);
        (local_exists, Some(version))
    } else {
        (local_exists, None)
    }
}

fn local_nats_present(nats_path: &Path) -> bool {
    debug!(
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
    let raw_string = NATS_SERVER_VERSION_PATTERN
        .captures(output)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    Version::parse(raw_string).unwrap()
}

fn github_meta_data() -> (Version, Url) {
    (latest_nats_release(), latest_nats_url())
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

#[derive(Deserialize)]
struct NatsServerRelease {
    tag_name: String,
    assets: Vec<ReleaseArchitecture>,
}

impl NatsServerRelease {
    fn url(&self) -> Url {
        #[cfg(target_os = "linux")]
        let os = "linux-amd64";
        #[cfg(target_os = "macos")]
        let os = "darwin-amd64";
        #[cfg(target_os = "windows")]
        let os = "windows-amd64";
        #[cfg(target_arch = "x86_64")]
        let arch = "amd64";
        #[cfg(target_arch = "arm")]
        let arch = "arm7";
        self.assets
            .iter()
            .filter(|x| x.name.contains(os))
            .filter(|x| x.name.contains(arch))
            .filter(|x| x.name.contains(".zip"))
            .map(|x| Url::parse(&x.url))
            .last()
            .unwrap()
            .unwrap()
    }
}

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
struct ReleaseArchitecture {
    #[serde(rename = "browser_download_url")]
    url: String,
    name: String,
}
