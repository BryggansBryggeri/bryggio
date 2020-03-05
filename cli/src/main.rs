use bryggio_cli;
use bryggio_cli::opts::Opt;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let api = bryggio_cli::Api::new(opt.ip, opt.port);
    api.send(&opt.command);
}
