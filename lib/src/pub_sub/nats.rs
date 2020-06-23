use crate::pub_sub::PubSubError;
use nats;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NatsConfig {
    server: String,
    user: String,
    pass: String,
}

pub struct NatsClient {
    client: nats::Connection,
}

impl NatsClient {
    pub fn try_new(config: &NatsConfig) -> Result<NatsClient, PubSubError> {
        let opts = nats::ConnectionOptions::with_user_pass(&config.user, &config.pass);
        match opts.connect(&config.server) {
            Ok(nc) => Ok(NatsClient { client: nc }),
            Err(err) => Err(PubSubError::Generic(err.to_string())),
        }
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
