#![forbid(unsafe_code)]
use bryggio_core::pub_sub::PubSubClient;
use bryggio_sensor_box::{SensorBox, SensorBoxConfig, SensorBoxError};
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    if let Err(err) = run_sensor_box() {
        println!("{}", err)
    }
}

fn run_sensor_box() -> Result<(), SensorBoxError> {
    let opt = Opt::from_args();
    match opt {
        Opt::Run { config_file } => {
            let config = SensorBoxConfig::try_new(config_file.as_path())?;
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
