use crate::actor::Actor;
use crate::logger::{error, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    ClientState, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::time::TimeStamp;
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
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
        Subject(format!("actor.{}.current_signal", self.id))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignalMsg {
    pub(crate) id: ClientId,
    pub(crate) timestamp: TimeStamp,
    pub(crate) signal: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorSubMsg {
    #[serde(rename = "set_signal")]
    SetSignal(SignalMsg),
    // #[serde(rename = "stop")]
    // Stop,
}

impl TryFrom<Message> for ActorSubMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        decode_nats_data(&msg.data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorPubMsg {
    CurrentSignal(SignalMsg),
}

impl Into<PubSubMsg> for ActorPubMsg {
    fn into(self) -> PubSubMsg {
        match &self {
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
        let sub = match self.subscribe(&Subject(format!("actor.{}.set_signal", self.id))) {
            Ok(sub) => sub,
            Err(err) => {
                error(&self, err.to_string(), &format!("actor.{}", self.id));
                return Err(err);
            }
        };
        // let mut state = ClientState::Active;
        loop {
            if let Some(contr_message) = sub.next() {
                let res: Result<(), PubSubError> = match ActorSubMsg::try_from(contr_message) {
                    Ok(msg) => match msg {
                        ActorSubMsg::SetSignal(msg) => self
                            .actor
                            .set_signal(msg.signal)
                            .map_err(|err| err.into())
                            .and_then(|()| {
                                self.publish(
                                    &self.gen_signal_subject(),
                                    &ActorPubMsg::CurrentSignal(msg).into(),
                                )
                            }),
                    },
                    Err(err) => Err(err),
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
