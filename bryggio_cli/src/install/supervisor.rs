use crate::install::{
    download_file, get_local_version, github_api::latest_github_release, make_executable,
    semver_from_text_output, should_download,
};
use crate::opts::SupervisorOpt;
use log::info;
use semver::Version;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::str;
use std::{fs::create_dir, path::PathBuf};
use url::Url;

use crate::install::nats;

use super::InstallError;

pub fn install_supervisor(opt: &SupervisorOpt) -> Result<(), InstallError> {
    let bryggio_root =
        PathBuf::from(
            shellexpand::tilde(opt.dir.to_str().ok_or_else(|| {
                InstallError::InvalidPath(String::from(opt.dir.to_string_lossy()))
            })?)
            .to_string(),
        );
    info!(
        "Installing `bryggio-supervisor` in '{}'",
        bryggio_root.to_string_lossy()
    );
    setup_directory(&bryggio_root)?;
    nats::setup_nats_server(&bryggio_root.join(opt.nats_path.as_path()), opt.update)?;
    setup_supervisor(
        &bryggio_root.join(opt.supervisor_path.as_path()),
        opt.update,
    )?;
    enable_one_wire();
    setup_gpio_user()
}

fn setup_directory(bryggio_root: &Path) -> Result<(), InstallError> {
    if !bryggio_root.exists() {
        create_dir(bryggio_root)?;
    }
    Ok(())
}

pub(crate) fn setup_supervisor(supervisor_path: &Path, update: bool) -> Result<(), InstallError> {
    let local_version = get_local_version(supervisor_path);
    if !update && local_version.is_some() {
        info!("Keeping existing bryggio-supervisor.");
        return Ok(());
    }

    let (latest_supervisor_version, supervisor_url) = github_meta_data()?;
    if should_download(update, local_version, latest_supervisor_version) {
        download_file(&supervisor_path, &supervisor_url);
        make_executable(supervisor_path);
        let link = Path::new("/usr/local/bin/bryggio-supervisor");
        symlink(supervisor_path, link).map_err(|err| {
            InstallError::PermissionDenied(format!(
                "symlinking '{} -> {}'. {}",
                supervisor_path.to_string_lossy(),
                link.to_string_lossy(),
                err,
            ))
        })?;
    };
    Ok(())
}

fn enable_one_wire() {}
fn setup_gpio_user() -> Result<(), InstallError> {
    Ok(())
}

fn github_meta_data() -> Result<(Version, Url), InstallError> {
    let latest = latest_github_release(SUPERVISOR_GITHUB_LATEST)?;
    let url = latest
        .urls()
        .map(|x| Url::parse(&x.url))
        .last()
        .unwrap()
        .unwrap();
    let version = semver_from_text_output(&latest.tag_name)?;
    Ok((version, url))
}

const SUPERVISOR_GITHUB_LATEST: &str =
    "https://api.github.com/repos/bryggansbryggeri/bryggio/releases/latest";
