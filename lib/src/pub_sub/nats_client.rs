use crate::pub_sub::{Message, PubSubError, Subject};
use nats::{Connection, Options, Subscription};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NatsConfig {
    bin_path: String,
    server: String,
    user: String,
    pass: String,
}

#[derive(Clone)]
pub struct NatsClient(Connection);

impl NatsClient {
    pub fn try_new(config: &NatsConfig) -> Result<NatsClient, PubSubError> {
        let opts = Options::with_user_pass(&config.user, &config.pass);
        match opts.connect(&config.server) {
            Ok(nc) => Ok(NatsClient(nc)),
            Err(err) => Err(PubSubError::Generic(err.to_string())),
        }
    }
    pub fn subscribe(&self, subject: &Subject) -> Subscription {
        self.0.subscribe(&subject.0).expect("Subscribe failed")
    }

    pub fn publish(&self, subject: &Subject, msg: &Message) {
        self.0
            .publish(&subject.0, &msg.0)
            .expect("Subscribe failed");
    }
}

pub fn run_nats_server(config: &NatsConfig) -> Child {
    let child = Command::new(&config.bin_path)
        .arg("-c")
        .arg("config.yaml")
        .spawn()
        .expect("failed to execute child");

    // Sleeps for a short while to ensure that the server is up and running before
    // the first connection comes.
    sleep(Duration::from_millis(10));
    child
}

// fn main() -> CliResult {
//     let args = Cli::from_args();
//
//     match args.cmd {
//         Command::Pub { subject, msg } => {
//             nc.publish(&subject, &msg)?;
//             println!("Published to '{}': '{}'", subject, msg);
//         }
//         Command::Sub { subject } => {
//             let sub = nc.subscribe(&subject)?;
//             println!("Listening on '{}'", subject);
//             for msg in sub.messages() {
//                 println!("Received a {}", msg);
//             }
//         }
//         Command::Request { subject, msg } => {
//             println!("Waiting on response for '{}'", subject);
//             let resp = nc.request(&subject, &msg)?;
//             println!("Response is {}", resp);
//         }
//         Command::Reply { subject, resp } => {
//             let sub = nc.queue_subscribe(&subject, "rust-box")?;
//             println!("Listening for requests on '{}'", subject);
//             for msg in sub.messages() {
//                 println!("Received a request {}", msg);
//                 msg.respond(&resp)?;
//             }
//         }
//     }
//
//     Ok(())
// }
