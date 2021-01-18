use crate::install::{
    download_file, get_local_version, github_api::latest_github_release, make_executable,
    semver_from_text_output, should_download,
};
use crate::opts::SupervisorOpt;
use log::info;
use semver::Version;
use std::path::Path;
use std::str;
use url::Url;

use crate::install::nats;

pub fn install_supervisor(opt: &SupervisorOpt) {
    //let nats_path: &Path = Path::new("/usr/local/bin/nats-server");
    info!("Installing `bryggio-supervisor`");
    setup_directory();
    nats::download_server(opt.nats_path.as_path(), opt.update);
    nats::generate_config();
    let supervisor_path = opt.supervisor_path.as_path();
    download_supervisor(&supervisor_path, opt.update);
    make_executable(supervisor_path);
    generate_config();
    enable_one_wire();
    setup_gpio_user();
}

fn setup_directory() {}

pub(crate) fn download_supervisor(supervisor_path: &Path, update: bool) {
    let local_version = get_local_version(supervisor_path);
    if !update && local_version.is_some() {
        info!("Keeping existing bryggio-supervisor.");
        return;
    }
    let (latest_supervisor_version, supervisor_url) = github_meta_data();
    if should_download(update, local_version, latest_supervisor_version) {
        download_file(&supervisor_path, &supervisor_url);
    }
}

fn generate_config() {}
fn enable_one_wire() {}
fn setup_gpio_user() {}

fn github_meta_data() -> (Version, Url) {
    let latest = latest_github_release(SUPERVISOR_GITHUB_LATEST);
    let url = latest
        .urls()
        .map(|x| Url::parse(&x.url))
        .last()
        .unwrap()
        .unwrap();
    let version = semver_from_text_output(&latest.tag_name);
    (version, url)
}

const SUPERVISOR_GITHUB_LATEST: &str =
    "https://api.github.com/repos/bryggansbryggeri/bryggio/releases/latest";
