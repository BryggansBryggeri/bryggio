use crate::opts::ServerOpt;
use log::info;
pub fn install_server(_opt: &ServerOpt) {
    info!("Installing `bryggio-server`");
    download_binary();
    generate_config();
    enable_one_wire();
}

fn download_binary() {}
fn generate_config() {}
fn enable_one_wire() {}
