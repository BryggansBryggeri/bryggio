#![forbid(unsafe_code)]
use bryggio_cli::opts::{InstallTarget, Opt};
use bryggio_cli::{brewery, install};
use log::info;
use structopt::StructOpt;

fn run_subcommand(opt: Opt) {
    match opt {
        Opt::Brewery(command) => {
            brewery::process_command(&command);
        }
        Opt::Install(target) => match target {
            InstallTarget::Server(opt) => install::server::install_server(&opt),
            InstallTarget::Cli(_opt) => info!("Installing `bryggio-cli`"),
        },
    }
}

fn main() {
    let opt = Opt::from_args();
    bryggio_cli::init_logging(&opt);
    run_subcommand(opt)
}
