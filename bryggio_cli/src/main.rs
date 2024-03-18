#![forbid(unsafe_code)]
use bryggio_cli::{brewery, install};
use bryggio_cli::{
    opts::{InstallTarget, Opt},
    CliError,
};
use structopt::StructOpt;

fn run_subcommand(opt: Opt) -> Result<(), CliError> {
    match opt {
        Opt::Publish(opts) => brewery::publish_command(&opts).map_err(CliError::from),
        Opt::Request(opts) => brewery::request(&opts).map_err(CliError::from),
        Opt::Install(target) => match target {
            InstallTarget::Supervisor(opt) => {
                install::supervisor::install_supervisor(&opt).map_err(CliError::from)
            }
            InstallTarget::Cli(_opt) => Err(CliError::UnimplementedFeature("CLI update")),
        },
        Opt::RbPiSetup(_opt) => Err(CliError::UnimplementedFeature("Rbpi setup")),
        Opt::ListSensors(_opt) => Ok(brewery::list_available_sensors()),
    }
}

fn main() {
    let opt = Opt::from_args();
    bryggio_cli::init_logging(&opt);
    if let Err(err) = run_subcommand(opt) {
        println!("{}", err);
    }
}
