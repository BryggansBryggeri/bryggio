use crate::opts::Opt;
use bryggio_lib::api;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::io::Write;
use ureq;
use url::Url;
pub mod opts;

/// TODO: Generic response
pub fn send<T>(request: &Url) -> Result<api::Response<T>, serde_json::error::Error>
where
    T: Serialize + DeserializeOwned,
{
    println!("Sending request: '{}'.", request);
    let response_raw = ureq::get(request.as_str()).call().into_string().unwrap();
    serde_json::from_str(&response_raw)
}

pub fn init_logging(opt: &Opt) {
    let mut builder = env_logger::Builder::from_default_env();
    if opt.verbose() {
        builder.filter(None, log::LevelFilter::Debug);
    }
    builder
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();
}
