use crate::actor::ActorError;
use crate::logger::{error, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::time::TimeStamp;
use crate::{actor::Actor, pub_sub::MessageParseError};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::ActorSignal;
pub struct ActorClient {
    id: ClientId,
    actor: Box<dyn Actor>,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl ActorClient {
    pub fn new(id: ClientId, actor: Box<dyn Actor>, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ActorClient { id, actor, client }
    }

    fn gen_signal_subject(&self) -> Subject {
        Subject(format!("actor_pub.{}.current_signal", self.id))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignalMsg {
    pub(crate) timestamp: TimeStamp,
    pub(crate) signal: ActorSignal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorSubMsg {
    #[serde(rename = "set_signal")]
    SetSignal(SignalMsg),
    #[serde(rename = "turn_off")]
    TurnOff,
    // #[serde(rename = "stop")]
    // Stop,
}

impl TryFrom<Message> for ActorSubMsg {
    type Error = MessageParseError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        let mut tmp = msg.subject.split('.');
        tmp.next();
        tmp.next();
        let sub_sub = tmp.next().unwrap();
        match sub_sub {
            "set_signal" => decode_nats_data(&msg.data),
            "turn_off" => Ok(Self::TurnOff),
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorPubMsg {
    CurrentSignal(SignalMsg),
}

impl From<ActorPubMsg> for PubSubMsg {
    fn from(msg: ActorPubMsg) -> PubSubMsg {
        match &msg {
            ActorPubMsg::CurrentSignal(signal_msg) => {
                PubSubMsg(serde_json::to_string(&signal_msg).expect("Pub sub serialization error"))
            }
        }
    }
}

impl PubSubClient for ActorClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        info(
            &self,
            format!("Starting actor with id '{}'", self.id),
            &format!("actor.{}", self.id),
        );
        let sub_set_signal = match self.subscribe(&Subject(format!("actor.{}.>", self.id))) {
            Ok(sub) => sub,
            Err(err) => {
                error(&self, err.to_string(), &format!("actor.{}", self.id));
                return Err(err);
            }
        };
        // let mut state = ClientState::Active;
        loop {
            if let Some(contr_message) = sub_set_signal.next() {
                let res: Result<(), PubSubError> =
                    match ActorSubMsg::try_from(contr_message.clone()) {
                        Ok(msg) => match msg {
                            ActorSubMsg::SetSignal(new_signal) => {
                                let sign_res = self.actor.set_signal(&new_signal.signal);
                                match sign_res {
                                    Ok(()) => self.publish(
                                        &self.gen_signal_subject(),
                                        &ActorPubMsg::CurrentSignal(new_signal).into(),
                                    ),
                                    Err(err) => match err {
                                        ActorError::ChangingToAlreadyActiveState => self.publish(
                                            &self.gen_signal_subject(),
                                            &ActorPubMsg::CurrentSignal(new_signal).into(),
                                        ),
                                        _ => Err(err.into()),
                                    },
                                }
                            }
                            ActorSubMsg::TurnOff => match self.actor.turn_off() {
                                Ok(()) => {
                                    contr_message
                                        .respond(String::from("Actor output set to zero"))
                                        .map_err(PubSubError::from)?;
                                    Ok(())
                                }
                                Err(_err) => contr_message
                                    .respond(String::from("Error turning off actor"))
                                    .map_err(PubSubError::from),
                            },
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
