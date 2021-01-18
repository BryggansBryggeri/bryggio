use crate::opts::ServerOpt;
use log::info;
use std::path::Path;

use crate::install::nats;

pub fn install_supervisor(opt: &ServerOpt) {
    //let nats_path: &Path = Path::new("/usr/local/bin/nats-server");
    let nats_path: &Path = Path::new("nats-server.zip");
    info!("Installing `bryggio-supervisor`");
    setup_directory();
    nats::download_server(nats_path, opt.update);
    nats::generate_config();
    download_bryggio_binary();
    generate_config();
    enable_one_wire();
    setup_gpio_user();
}

fn setup_directory() {}
fn download_bryggio_binary() {
    info!(
        "Downloading bryggio v{}",
        bryggio_lib::utils::get_bryggio_version()
    );
}
fn generate_config() {}
fn enable_one_wire() {}
fn setup_gpio_user() {}
