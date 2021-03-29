#![forbid(unsafe_code)]
use bryggio_lib::pub_sub::{PubSubClient, PubSubError};
use bryggio_sensor_box::{SensorBox, SensorBoxConfig, SensorBoxError};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

fn config_file_from_args(config_file: &Path) -> Result<SensorBoxConfig, SensorBoxError> {
    match SensorBoxConfig::try_new(&config_file) {
        Ok(config) => Ok(config),
        Err(err) => Err(PubSubError::Configuration(format!(
            "Invalid config file '{}'. Error: {}.",
            config_file.to_string_lossy(),
            err.to_string()
        ))
        .into()),
    }
}

fn main() -> Result<(), SensorBoxError> {
    let opt = Opt::from_args();
    match opt {
        Opt::Run { config_file } => {
            let config = config_file_from_args(config_file.as_path())?;
            println!("Starting sensor box");
            let sensor_box = SensorBox::init_from_config(config)?;
            sensor_box.client_loop()?;
        }
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-sensor-box", about = "Sensor box client for BryggIO")]
pub enum Opt {
    ///Run supervisor
    #[structopt(name = "run")]
    Run { config_file: PathBuf },
}
