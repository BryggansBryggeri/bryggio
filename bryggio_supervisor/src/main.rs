#![forbid(unsafe_code)]
use bryggio_core::pub_sub::{
    nats_client::{run_nats_server, NatsServerConfig},
    PubSubClient, PubSubError,
};
use bryggio_core::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};
use std::path::PathBuf;
use structopt::StructOpt;

// Note: I have started some trials into converting the code base to be async.
// At present, none of it is async, only the main function which have no practical meaning.
#[tokio::main]
async fn main() {
    if let Err(err) = run_supervisor().await {
        println!("{}", err)
    }
}

/// Supervisor main loop
///
/// A config (sample-bryggio.json) is parsed, with settings for both the NATS server and the
/// bryggio process.
/// The NATS server is started in a separate process then,
/// a Supervisor client is started which runs indefinetly.
async fn run_supervisor() -> Result<(), SupervisorError> {
    let opt = Opt::from_args();
    match opt {
        Opt::Run { config_file } => {
            let config = SupervisorConfig::try_new(config_file.as_path())?;
            let mut nats_server_child = run_nats_server(
                &NatsServerConfig::from(config.clone()),
                &config.nats.bin_path,
            )?;
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
