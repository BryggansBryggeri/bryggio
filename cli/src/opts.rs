use crate::wifi_settings::{Password, Ssid};
use std::path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Opt {
    ///Publish a message on a subject.
    #[structopt(name = "publish")]
    Publish(PubSubOpt),
    ///Request a reply on a subject.
    #[structopt(name = "request")]
    Request(PubSubOpt),
    ///Install bryggio software.
    #[structopt(name = "install")]
    Install(InstallTarget),
    ///Automated raspberry pi setup.
    #[structopt(name = "rbpi-setup")]
    RbPiSetup(RbPiOpt),
    ///Test script, switching controllers.
    #[structopt(name = "test")]
    Test(PubSubOpt),
}

impl Opt {
    pub fn verbose(&self) -> bool {
        match self {
            Self::Publish(opt) => opt.common.verbose,
            Self::Request(opt) => opt.common.verbose,
            Self::Install(target) => target.verbose(),
            Self::RbPiSetup(opt) => opt.common.verbose,
            Self::Test(_opt) => true,
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct PubSubOpt {
    #[structopt(long)]
    pub config: path::PathBuf,
    #[structopt(long)]
    pub(crate) topic: String,
    #[structopt(long)]
    pub(crate) msg: String,
    #[structopt(flatten)]
    pub(crate) common: Common,
}

#[derive(Debug, StructOpt)]
pub enum InstallTarget {
    /// Install `bryggio-server`
    #[structopt(name = "bryggio-server")]
    Server(ServerOpt),
    /// Install `bryggio-cli`
    #[structopt(name = "bryggio-cli")]
    Cli(CliOpt),
}

impl InstallTarget {
    fn verbose(&self) -> bool {
        match self {
            Self::Server(opt) => opt.common.verbose,
            Self::Cli(opt) => opt.common.verbose,
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct CliOpt {
    #[structopt(flatten)]
    common: Common,
}

#[derive(Debug, StructOpt)]
pub struct ServerOpt {
    #[structopt(flatten)]
    common: Common,
    #[structopt(long)]
    pub update: bool,
}

#[derive(Debug, StructOpt)]
pub struct RbPiOpt {
    #[structopt(flatten)]
    common: Common,
    #[structopt(long)]
    pub ssid: Ssid,
    #[structopt(long)]
    pub password: Password,
}

#[derive(Debug, StructOpt)]
pub struct Common {
    /// Verbose output
    #[structopt(long)]
    verbose: bool,
}
