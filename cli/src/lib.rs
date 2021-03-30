use crate::opts::Opt;
use bryggio_lib::{pub_sub::PubSubError, supervisor::config::SupervisorConfigError};
use install::InstallError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;
use thiserror::Error;
use url::Url;

pub mod brewery;
pub mod install;
pub mod opts;
pub mod rbpi;
pub mod wifi_settings;

pub fn send<T>(request: &Url) -> Result<T, serde_json::error::Error>
where
    T: Serialize + DeserializeOwned,
{
    println!("Sending request: '{}'.", request);
    let response_raw = ureq::get(request.as_str())
        .call()
        .expect("ureq called failed")
        .into_string()
        .unwrap();
    serde_json::from_str(&response_raw)
}

pub fn init_logging(opt: &Opt) {
    let mut builder = env_logger::Builder::from_default_env();
    if opt.verbose() {
        builder.filter(None, log::LevelFilter::Debug);
    } else {
        builder.filter(None, log::LevelFilter::Info);
    }
    builder
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Install error: {0}")]
    Install(#[from] InstallError),
    #[error("Supervisor config error: {0}")]
    SupervisorConfig(#[from] SupervisorConfigError),
    #[error("Pubsub error: {0}")]
    PubSub(#[from] PubSubError),
    #[error("Feature '{0}' not implemented yet")]
    UnimplementedFeature(&'static str),
}
