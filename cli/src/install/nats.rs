use crate::install;
use crate::install::github_api::Release;
use crate::install::SEMVER_VERSION_PATTERN;
use log::{debug, info};
use semver::Version;
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
    if should_download(local_exists, update, local_version, latest_nats_version) {}
    install::download_file(&nats_path, &nats_url);
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
        // TODO: This should not print if update == false.
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
        .unwrap_or_else(|err| {
            panic!(
                "Failed to get version from '{}'. Err: '{}'",
                nats_path.to_str().unwrap(),
                err.to_string()
            )
        })
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    let raw_string = SEMVER_VERSION_PATTERN
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
    let response_raw = ureq::get(GITHUB_META_DATA)
        .call()
        .expect("ureq call failed")
        .into_string()
        .unwrap();
    let meta_data: Release = serde_json::from_str(&response_raw).unwrap();
    meta_data.url(|x| x.name.contains(".zip"))
}

fn latest_nats_release() -> Version {
    let response_raw = ureq::get(GITHUB_META_DATA)
        .call()
        .expect("ureq called failed.")
        .into_string()
        .unwrap();
    let meta_data: Release = serde_json::from_str(&response_raw).unwrap();
    let raw_string = SEMVER_VERSION_PATTERN
        .captures(&meta_data.tag_name)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();
    Version::parse(raw_string).unwrap()
}

const GITHUB_META_DATA: &str = "https://api.github.com/repos/nats-io/nats-server/releases/latest";
