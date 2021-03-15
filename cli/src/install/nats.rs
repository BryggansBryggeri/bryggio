use crate::install::github_api::latest_github_release;
use crate::install::{
    download_file, get_local_version, make_executable, semver_from_text_output, should_download,
};
use log::info;
use semver::Version;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use url::Url;

pub(crate) fn setup_nats_server(nats_path: &Path, update: bool) {
    println!("nats path: {}", nats_path.to_string_lossy());
    let local_version = get_local_version(nats_path);
    if !update && local_version.is_some() {
        info!("Keeping existing NATS server.");
        return;
    }
    let (latest_nats_version, nats_url) = github_meta_data();
    if should_download(update, local_version, latest_nats_version) {
        let zip_path = nats_path.with_extension("zip");
        download_file(&zip_path, &nats_url);
        extract_server(&zip_path);
        make_executable(nats_path);
    }
}

fn extract_server(zip_path: &Path) {
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut nats_server_name = PathBuf::new();
    for i in 0..archive.len() {
        let file_ = archive.by_index(i).unwrap();
        let file_name = file_.enclosed_name().unwrap().to_owned();
        if file_name.file_stem().unwrap() == "nats-server" {
            nats_server_name = file_name;
        }
    }
    println!("FILE: {:?}", nats_server_name);
    let mut extracted_file = archive.by_name(nats_server_name.to_str().unwrap()).unwrap();
    let mut outfile = fs::File::create(&zip_path.with_file_name("nats-server")).unwrap();
    println!("OUT: {:?}", outfile);
    io::copy(&mut extracted_file, &mut outfile).unwrap();
}

fn github_meta_data() -> (Version, Url) {
    let latest = latest_github_release(NATS_GITHUB_LATEST);

    #[cfg(target_os = "linux")]
    let os = "linux";
    #[cfg(target_os = "macos")]
    let os = "darwin-amd64";
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
        //.collect::<Result<Vec<Url>, url::ParseError>>()
        //.unwrap();
        .last()
        .unwrap()
        .unwrap();
    let version = semver_from_text_output(&latest.tag_name);
    (version, url)
}

const NATS_GITHUB_LATEST: &str = "https://api.github.com/repos/nats-io/nats-server/releases/latest";
