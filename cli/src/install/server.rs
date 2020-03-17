use crate::opts::ServerOpt;
use log::info;
use std::path::Path;

use crate::install::nats;

pub fn install_server(opt: &ServerOpt) {
    //let nats_path: &Path = Path::new("/usr/local/bin/nats-server");
    let nats_path: &Path = Path::new("nats-server.zip");
    info!("Installing `bryggio-server`");
    update_os();
    nats::download_server(nats_path, opt.update);
    download_bryggio_binary();
    generate_config();
    enable_one_wire();
}

fn update_os() {}
fn download_bryggio_binary() {}
fn generate_config() {}
fn enable_one_wire() {}
