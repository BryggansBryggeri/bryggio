use bryggio_cli;
use bryggio_cli::opts::Opt;
use env_logger;
use log::{debug, error, info};
use std::io::Write;
use structopt::StructOpt;

fn run_subcommand(opt: Opt) {
    match opt {
        Opt::Brewery(options) => {
            match bryggio_cli::send(&options.url().unwrap()) {
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

fn init_logging(opt: &Opt) {
    let mut builder = env_logger::Builder::from_default_env();
    if opt.verbose() {
        builder.filter(None, log::LevelFilter::Debug);
    }
    builder
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();
}

fn main() {
    let opt = Opt::from_args();
    init_logging(&opt);
    run_subcommand(opt)
}
