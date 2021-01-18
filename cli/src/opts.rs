use crate::wifi_settings::{Password, Ssid};
use std::path::PathBuf;
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
    pub config: PathBuf,
    #[structopt(long)]
    pub(crate) topic: String,
    #[structopt(long)]
    pub(crate) msg: String,
    #[structopt(flatten)]
    pub(crate) common: Common,
}

#[derive(Debug, StructOpt)]
pub enum InstallTarget {
    /// Install `bryggio-supervisor`
    #[structopt(name = "bryggio-supervisor")]
    Supervisor(SupervisorOpt),
    /// Install `bryggio-cli`
    #[structopt(name = "bryggio-cli")]
    Cli(CliOpt),
}

impl InstallTarget {
    fn verbose(&self) -> bool {
        match self {
            Self::Supervisor(opt) => opt.common.verbose,
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
pub struct SupervisorOpt {
    #[structopt(flatten)]
    common: Common,
    #[structopt(long)]
    pub update: bool,
    #[structopt(default_value = "target/nats-server", long)]
    pub nats_path: PathBuf,
    #[structopt(default_value = "target/gh_supervisor", long)]
    pub supervisor_path: PathBuf,
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
