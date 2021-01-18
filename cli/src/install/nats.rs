use crate::install::github_api::latest_github_release;
use crate::install::{download_file, get_local_version, semver_from_text_output, should_download};
use log::info;
use semver::Version;
use std::path::Path;
use url::Url;

pub(crate) fn generate_config() {}

pub(crate) fn download_server(nats_path: &Path, update: bool) {
    let local_version = get_local_version(nats_path);
    if !update && local_version.is_some() {
        info!("Keeping existing NATS server.");
        return;
    }
    let (latest_nats_version, nats_url) = github_meta_data();
    if should_download(update, local_version, latest_nats_version) {
        download_file(&nats_path, &nats_url);
    }
}

fn github_meta_data() -> (Version, Url) {
    let latest = latest_github_release(NATS_GITHUB_LATEST);

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

    let url = latest
        .urls()
        .filter(|x| x.name.contains(os))
        .filter(|x| x.name.contains(arch))
        .filter(|x| x.name.contains(".zip"))
        .map(|x| Url::parse(&x.url))
        .last()
        .unwrap()
        .unwrap();
    let version = semver_from_text_output(&latest.tag_name);
    (version, url)
}

const NATS_GITHUB_LATEST: &str = "https://api.github.com/repos/nats-io/nats-server/releases/latest";
