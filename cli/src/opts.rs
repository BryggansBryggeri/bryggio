use std::net;
use std::path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub struct Opt {
    #[structopt(long)]
    pub config: Option<path::PathBuf>,
    #[structopt(long)]
    pub ip: net::IpAddr,
    #[structopt(long)]
    pub port: u16,
    #[structopt(long)]
    pub command: String,
    #[structopt(long)]
    pub verbose: bool,
}
