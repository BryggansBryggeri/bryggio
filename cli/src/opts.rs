use std::path;
use structopt::StructOpt;
use url::{ParseError, Url};

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Opt {
    ///Control a `bryggio-server`
    #[structopt(name = "brewery")]
    Brewery(BreweryOpt),
    #[structopt(name = "install")]
    ///Install bryggio software
    Install(InstallTarget),
}

impl Opt {
    pub fn verbose(&self) -> bool {
        match self {
            Self::Brewery(opt) => opt.verbose,
            Self::Install(_target) => false,
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
    #[structopt(long)]
    verbose: bool,
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
    #[structopt(name = "bryggio-server")]
    Server,
    #[structopt(name = "bryggio-cli")]
    Cli,
}
