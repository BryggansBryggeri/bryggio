use crate::actor::Actor;
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
    id: ClientId,
    timestamp: TimeStamp,
    signal: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorSubMsg {
    #[serde(rename = "set_signal")]
    SetSignal(SignalMsg),
    #[serde(rename = "stop")]
    Stop,
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
        PubSubMsg(serde_json::to_string(&self).expect("Pub sub serialization error"))
    }
}

impl PubSubClient for ActorClient {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let sub = self.subscribe(&Subject(format!("actor.{}.set_signal", self.id)))?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(contr_message) = sub.next() {
                //println!("Actor msg: {}", &contr_message);
                //println!(
                //    "Ser same msg: {}",
                //    serde_json::to_string(&ActorSubMsg::SetSignal(SignalMsg {
                //        id: self.id.clone(),
                //        timestamp: TimeStamp::now(),
                //        signal: 2.0
                //    }))
                //    .unwrap()
                //);
                if let Ok(msg) = ActorSubMsg::try_from(contr_message) {
                    match msg {
                        ActorSubMsg::SetSignal(msg) => {
                            if let Ok(()) = self.actor.set_signal(msg.signal) {
                                println!("publish");
                                self.publish(
                                    &self.gen_signal_subject(),
                                    &ActorPubMsg::CurrentSignal(msg).into(),
                                )?;
                            }
                        }
                        ActorSubMsg::Stop => state = ClientState::Inactive,
                    }
                }
            }
        }
        // TODO: Exit gracefully
        Ok(())
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}
