use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    ClientId, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::{logger::error, pub_sub::MessageParseError};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::Buzzer;

pub struct BuzzerClient {
    id: ClientId,
    buzzer: Buzzer,
    client: NatsClient,
}

impl BuzzerClient {
    pub fn try_new(
        id: ClientId,
        buzzer: Buzzer,
        config: &NatsClientConfig,
    ) -> Result<Self, PubSubError> {
        let client = NatsClient::try_new(config)?;
        Ok(BuzzerClient { id, buzzer, client })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignalPattern {
    Constant,
    Pulse,
    Stop,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BuzzerSubMsg {
    #[serde(rename = "buzz")]
    Buzz(SignalPattern),
    // #[serde(rename = "stop")]
    // Stop,
}

impl TryFrom<Message> for BuzzerSubMsg {
    type Error = MessageParseError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        decode_nats_data(&msg.data)
    }
}

impl PubSubClient for BuzzerClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let sub = match self.subscribe(&Subject(format!("buzzer.{}.buzz", self.id))) {
            Ok(sub) => sub,
            Err(err) => {
                error(&self, err.to_string(), &format!("buzzer.{}", self.id));
                return Err(err);
            }
        };
        loop {
            if let Some(contr_message) = sub.next() {
                let res: Result<(), PubSubError> = match BuzzerSubMsg::try_from(contr_message) {
                    Ok(msg) => match msg {
                        BuzzerSubMsg::Buzz(pat) => {
                            let res = match pat {
                                SignalPattern::Constant => self.buzzer.constant(),
                                SignalPattern::Pulse => self.buzzer.pulse(),
                                SignalPattern::Stop => self.buzzer.stop(),
                            };
                            res.map_err(PubSubError::from)
                        }
                    },
                    Err(err) => Err(err).map_err(PubSubError::from),
                };
                if let Err(err) = res {
                    error(&self, err.to_string(), &format!("actor.{}", self.id));
                }
            }
        }
        // TODO: Exit gracefully
        // Ok(())
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}
