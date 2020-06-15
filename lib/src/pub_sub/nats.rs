use nats;
use std::error as std_error;

pub(crate) struct Client {
    client: nats::Connection,
}

impl Client {
    pub fn try_new(server: &str, user: &str, pass: &str) -> Result<Client, PubSubError> {
        let opts = nats::ConnectionOptions::with_user_pass(&args.user, &args.pass);
        match opts.connect(&args.server) {
            Ok(nc) => Ok(),
        }
    }
}

fn main() -> CliResult {
    let args = Cli::from_args();

    match args.cmd {
        Command::Pub { subject, msg } => {
            nc.publish(&subject, &msg)?;
            println!("Published to '{}': '{}'", subject, msg);
        }
        Command::Sub { subject } => {
            let sub = nc.subscribe(&subject)?;
            println!("Listening on '{}'", subject);
            for msg in sub.messages() {
                println!("Received a {}", msg);
            }
        }
        Command::Request { subject, msg } => {
            println!("Waiting on response for '{}'", subject);
            let resp = nc.request(&subject, &msg)?;
            println!("Response is {}", resp);
        }
        Command::Reply { subject, resp } => {
            let sub = nc.queue_subscribe(&subject, "rust-box")?;
            println!("Listening for requests on '{}'", subject);
            for msg in sub.messages() {
                println!("Received a request {}", msg);
                msg.respond(&resp)?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum PubSubError {
    Generic(String),
}

impl std::fmt::Display for PubSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PubSubError::Generic(err) => write!(f, "Can you be more specfic?: {}", err),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ConcurrencyError(_) => "Can you be more specfic?",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
