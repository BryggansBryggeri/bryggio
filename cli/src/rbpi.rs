use crate::opts::RbPiOpt;
use crate::wifi_settings::WifiSettings;
use log::{debug, error, info};
pub fn setup(opt: &RbPiOpt) {
    info!("Setting up the raspberry pi.");
    let wifi = WifiSettings::new(opt.ssid.clone(), opt.password.clone());
    println!("{}", wifi.to_wpa_supplicant_entry())
}
