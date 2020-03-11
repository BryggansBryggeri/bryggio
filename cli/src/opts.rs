use std::path;
use structopt::StructOpt;
use url::{ParseError, Url};

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Opt {
    ///Control a `bryggio-server`
    #[structopt(name = "brewery")]
    Brewery(BreweryOpt),
    ///Install bryggio software
    #[structopt(name = "install")]
    Install(InstallTarget),
}

impl Opt {
    pub fn verbose(&self) -> bool {
        match self {
            Self::Brewery(opt) => opt.common.verbose,
            Self::Install(target) => target.verbose(),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct BreweryOpt {
    #[structopt(long)]
    pub config: Option<path::PathBuf>,
    #[structopt(long)]
    pub ip: String,
    #[structopt(long)]
    pub port: u16,
    #[structopt(long)]
    pub command: String,
    #[structopt(flatten)]
    common: Common,
}

impl BreweryOpt {
    pub fn url(&self) -> Result<Url, ParseError> {
        Url::parse(&format!(
            "http://{}:{}/{}",
            self.ip, self.port, self.command
        ))
    }
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
}

#[derive(Debug, StructOpt)]
pub struct Common {
    /// Verbose output
    #[structopt(long)]
    verbose: bool,
    /// Disable `sudo` use. Warning: setup will not function properly
    #[structopt(long)]
    no_sudo: bool,
}
