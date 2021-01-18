use crate::install::{
    download_file, get_local_version, github_api::latest_github_release, make_executable,
    semver_from_text_output, should_download,
};
use crate::opts::SupervisorOpt;
use log::info;
use semver::Version;
use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::str;
use url::Url;

use crate::install::nats;

pub fn install_supervisor(opt: &SupervisorOpt) {
    // TODO: Tmp hack before I do proper tilde expansion
    // let mut bryggio_root = dirs::home_dir().expect("Could not locate a home dir.");
    // bryggio_root.push(opt.bryggio_root.as_path());
    let bryggio_root = opt.bryggio_root.as_path();
    info!("Installing `bryggio-supervisor`");
    setup_directory(&bryggio_root);
    nats::setup_nats_server(&bryggio_root.join(opt.nats_path.as_path()), opt.update);
    setup_supervisor(
        &bryggio_root.join(opt.supervisor_path.as_path()),
        opt.update,
    );
    enable_one_wire();
    setup_gpio_user();
}

fn setup_directory(bryggio_root: &Path) {
    if !bryggio_root.exists() {
        create_dir(bryggio_root).unwrap_or_else(|err| {
            panic!(
                "Could not create dir: '{}'. {}",
                bryggio_root.to_string_lossy(),
                err.to_string()
            )
        });
    }
}

pub(crate) fn setup_supervisor(supervisor_path: &Path, update: bool) {
    let local_version = get_local_version(supervisor_path);
    if !update && local_version.is_some() {
        info!("Keeping existing bryggio-supervisor.");
        return;
    }

    let (latest_supervisor_version, supervisor_url) = github_meta_data();
    if should_download(update, local_version, latest_supervisor_version) {
        download_file(&supervisor_path, &supervisor_url);
        make_executable(supervisor_path);
        let link = Path::new("/usr/local/bin/bryggio-supervisor");
        symlink(supervisor_path, link).unwrap_or_else(|err| {
            panic!(
                "Error symlinking '{} -> {}'. {}",
                supervisor_path.to_string_lossy(),
                link.to_string_lossy(),
                err.to_string(),
            )
        });
    }
}

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
