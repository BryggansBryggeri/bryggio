use bryggio_cli;
use bryggio_cli::opts::Opt;
use env_logger;
use log::{debug, error, info};
use structopt::StructOpt;

fn run_subcommand(opt: Opt) {
    match opt {
        Opt::Brewery(options) => {
            match bryggio_cli::send::<f32>(&options.url().unwrap()) {
                Ok(response) => {
                    if response.success {
                        match response.result {
                            Some(result) => info!("Result: {}", result),
                            None => info!("Successful"),
                        }
                    } else {
                        debug!("Message: {}", response.message.unwrap());
                    }
                }
                Err(err) => {
                    error!("Error sending command: {}", err);
                }
            };
        }
        Opt::Install(opt) => {
            info!("Installing");
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    bryggio_cli::init_logging(&opt);
    run_subcommand(opt)
}
