use lazy_static::lazy_static;
use log::info;
use semver::Version;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use url::Url;
mod github_api;
mod nats;
pub mod supervisor;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::str;
use thiserror::Error;

pub(crate) fn download_file<P: AsRef<Path>>(dest_path: P, source_url: &Url) {
    let dest_path = dest_path.as_ref();
    info!("Downloading to '{}'", dest_path.to_string_lossy());
    if dest_path.exists() {
        fs::remove_file(dest_path).expect("Could not remove file.");
    }
    let mut file = fs::File::create(dest_path).expect("Could not create file");
    info!(
        "Downloading executable from '{}{}' to '{}'",
        source_url.host().unwrap(),
        source_url.path(),
        dest_path.to_string_lossy(),
    );
    io::copy(
        &mut ureq::get(source_url.as_str())
            .call()
            .expect("ureq called failed")
            .into_reader(),
        &mut file,
    )
    .expect("Could not write to file");
}

pub(crate) fn make_executable<P: AsRef<Path>>(file_path: P) {
    let file_path = file_path.as_ref();
    info!(
        "Making '{}' executable",
        file_path.file_name().unwrap().to_string_lossy()
    );
    let f = File::open(file_path).unwrap();
    let metadata = f.metadata().expect("Could not get metadata");
    let mut permissions = metadata.permissions();
    permissions.set_mode(EXECUTABLE_PERMISSIONS); // Read/write for owner and read for others.
    fs::set_permissions(file_path, permissions).expect("Could not set permissions");
}

fn get_local_version(path: &Path) -> Option<Version> {
    if path.exists() {
        let version = semver_from_executable(path).ok()?;
        info!(
            "Found existing {} v{}.",
            path.file_name().unwrap().to_string_lossy(),
            version
        );
        Some(version)
    } else {
        None
    }
}

fn semver_from_executable(path: &Path) -> Result<Version, InstallError> {
    let output: Vec<u8> = Command::new(path)
        .arg("--version")
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to get version from '{}'. Err: '{}'",
                path.to_str().unwrap(),
                err.to_string()
            )
        })
        .stdout;
    semver_from_text_output(&str::from_utf8(&output).unwrap())
}

pub(crate) fn semver_from_text_output<S: AsRef<str>>(output: &S) -> Result<Version, InstallError> {
    let output = output.as_ref();
    Version::parse(
        SEMVER_VERSION_PATTERN
            .captures(output.as_ref())
            .ok_or_else(|| InstallError::SemVer(output.to_string()))?
            .get(1)
            .unwrap()
            .as_str(),
    )
    .map_err(|_err| InstallError::SemVer(output.to_string()))
}

// TODO: Better logging of all possible outcomes.
fn should_download(update: bool, local_version: Option<Version>, latest_version: Version) -> bool {
    if let Some(local_version) = local_version {
        info!("Keeping existing version");
        if update && latest_version > local_version {
            info!("Newer version released ({}), downloading", latest_version);
            true
        } else {
            false
        }
    } else {
        info!("No local version found, downloading");
        true
    }
}

lazy_static! {
    static ref SEMVER_VERSION_PATTERN: regex::Regex = regex::Regex::new(
        r"(\d+.\d+.\d+)"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
}

// TODO: Consider these permissions.
// Currently:
// User: rwx
// Group: rx
// Other: rx
const EXECUTABLE_PERMISSIONS: u32 = 0o0755;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Failed to parse semver version from '{0}'.")]
    SemVer(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Permission denied: {0}")]
    OsError(#[from] std::io::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
