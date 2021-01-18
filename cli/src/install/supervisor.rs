use crate::opts::ServerOpt;
use log::info;
use std::path::Path;

use crate::install::nats;

pub fn install_supervisor(opt: &ServerOpt) {
    //let nats_path: &Path = Path::new("/usr/local/bin/nats-server");
    info!("Installing `bryggio-supervisor`");
    setup_directory();
    nats::download_server(opt.nats_path, opt.update);
    nats::generate_config();
    download_bryggio_binary();
    generate_config();
    enable_one_wire();
    setup_gpio_user();
}

fn setup_directory() {}
pub(crate) fn download_supervisor(nats_path: &Path, update: bool) {
    let (local_exists, local_version) = local_meta_data(nats_path);
    if !update && local_exists {
        info!("Keeping existing NATS server.");
        return;
    }
    let (latest_nats_version, nats_url) = github_meta_data();
    if should_download(local_exists, update, local_version, latest_nats_version) {}
    install::download_file(&nats_path, &nats_url);
}

    info!(
        "Downloading bryggio v{}",
        bryggio_lib::utils::get_bryggio_version()
    );
}
fn generate_config() {}
fn enable_one_wire() {}
fn setup_gpio_user() {}
