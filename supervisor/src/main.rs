#![forbid(unsafe_code)]
use bryggio_lib::pub_sub::{nats_client::run_nats_server, PubSubClient, PubSubError};
use bryggio_lib::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};
use std::path::PathBuf;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    if let Err(err) = run_supervisor().await {
        println!("{}", err)
    }
}

async fn run_supervisor() -> Result<(), SupervisorError> {
    let opt = Opt::from_args();
    match opt {
        Opt::Run { config_file } => {
            let config = SupervisorConfig::try_new(config_file.as_path())?;
            println!("Starting nats");
            let mut nats_server_child =
                run_nats_server(&config.nats.nats_bin_path, &config.nats.nats_config)?;
            println!("Starting supervisor");
            let supervisor = Supervisor::init_from_config(config)?;
            supervisor.client_loop()?;
            nats_server_child
                .kill()
                .map_err(|err| PubSubError::Server(err.to_string()).into())
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-supervisor", about = "Supervisor client for BryggIO")]
pub enum Opt {
    ///Run supervisor
    #[structopt(name = "run")]
    Run { config_file: PathBuf },
}
