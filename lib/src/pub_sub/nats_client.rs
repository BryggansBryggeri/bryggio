use crate::pub_sub::{Message, PubSubError, Subject};
use nats::{Connection, ConnectionOptions, Subscription};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NatsConfig {
    server: String,
    user: String,
    pass: String,
}

#[derive(Clone)]
pub struct NatsClient {
    client: Connection,
}

impl NatsClient {
    pub fn try_new(config: &NatsConfig) -> Result<NatsClient, PubSubError> {
        let opts = ConnectionOptions::with_user_pass(&config.user, &config.pass);
        match opts.connect(&config.server) {
            Ok(nc) => Ok(NatsClient { client: nc }),
            Err(err) => Err(PubSubError::Generic(err.to_string())),
        }
    }
    pub fn subscribe(&self, subject: &Subject) -> Subscription {
        self.client.subscribe(&subject.0).expect("Subscribe failed")
    }

    pub fn publish(&self, subject: &Subject, msg: &Message) {
        self.client
            .publish(&subject.0, &msg.0)
            .expect("Subscribe failed");
    }
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
