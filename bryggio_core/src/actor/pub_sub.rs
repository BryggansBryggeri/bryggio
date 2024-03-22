use super::ActorSignal;
use crate::actor::ActorError;
use crate::logger::{error, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    ClientId, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::time::TimeStamp;
use crate::{actor::Actor, pub_sub::MessageParseError};
use async_nats::{Message, Subscriber};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub fn actor_current_signal_subject(id: &ClientId) -> Subject {
    Subject(format!("actor.{}.current_signal", id))
}

pub fn actor_set_signal_subject(id: &ClientId) -> Subject {
    Subject(format!("actor.{}.set_signal", id))
}

pub fn actor_turn_off_subject(id: &ClientId) -> Subject {
    Subject(format!("actor.{}.turn_off", id))
}

pub struct ActorClient {
    id: ClientId,
    actor: Box<dyn Actor>,
    client: NatsClient,
}

impl PubSubClient for ActorClient {
    async fn client_loop(mut self) -> Result<(), PubSubError> {
        info(
            &self,
            format!("Starting actor with id '{}'", self.id),
            &format!("actor.{}", self.id),
        )
        .await;
        let sub_set_signal = self.subscribe(&actor_set_signal_subject(&self.id)).await?;
        let sub_turn_off = self.subscribe(&actor_turn_off_subject(&self.id)).await?;

        // This will efficiently wait for any of the above subscriptions to trigger and then handle
        // the incoming message.
        // The other possible construct is perhaps to spawn a looped task for every subscription.
        let mut messages = futures::stream::select(sub_turn_off, sub_set_signal);
        loop {
            let msg = messages.select_next_some().await;
            if let Err(err) = self.handle_msg(msg).await {
                error(
                    &self,
                    format!("Failed handling actor msg: '{}'", err),
                    &format!("actor.{}", self.id),
                )
                .await
            };

            // Always set the signal, if the signal has not changed, then it is a no-op.
            if let Err(err) = self.actor.set_signal() {
                match err {
                    ActorError::ChangingToAlreadyActiveState => {}
                    _ => {
                        error(
                            &self,
                            format!("Failed setting signal: '{}'", err),
                            &format!("actor.{}", self.id),
                        )
                        .await
                    }
                }
            }
        }
    }

    async fn subscribe(&self, subject: &Subject) -> Result<Subscriber, PubSubError> {
        self.client.subscribe(subject).await
    }

    async fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg).await
    }
}

impl ActorClient {
    pub async fn new(id: ClientId, actor: Box<dyn Actor>, config: &NatsClientConfig) -> Self {
        let client = NatsClient::try_new(config).await.unwrap();
        ActorClient { id, actor, client }
    }

    async fn handle_msg(&mut self, msg: Message) -> Result<(), PubSubError> {
        match ActorSubMsg::try_from(msg.clone())? {
            ActorSubMsg::TurnOff => {
                if let Err(err) = self.turn_off(msg).await {
                    error(self, err.to_string(), &format!("actor.{}", self.id)).await;
                    Err(err)
                } else {
                    Ok(())
                }
            }
            ActorSubMsg::SetSignal(new_signal) => {
                if let Err(err) = self.update_signal(new_signal).await {
                    error(self, err.to_string(), &format!("actor.{}", self.id)).await;
                    Err(err)
                } else {
                    Ok(())
                }
            }
        }
    }

    async fn update_signal(&mut self, new_signal: SignalMsg) -> Result<(), PubSubError> {
        let sign_res = self.actor.update_signal(&new_signal.signal);
        match sign_res {
            Ok(()) => {
                self.publish(
                    &actor_current_signal_subject(&self.id),
                    &ActorPubMsg::CurrentSignal(new_signal).into(),
                )
                .await
            }
            Err(err) => match err {
                ActorError::ChangingToAlreadyActiveState => {
                    self.publish(
                        &actor_current_signal_subject(&self.id),
                        &ActorPubMsg::CurrentSignal(new_signal).into(),
                    )
                    .await
                }
                _ => Err(err.into()),
            },
        }
    }

    async fn turn_off(&mut self, contr_message: Message) -> Result<(), PubSubError> {
        match self.actor.turn_off() {
            Ok(()) => {
                if let Some(reply_subj) = contr_message.reply {
                    self.publish(&reply_subj.into(), &"Actor output set to zero".into())
                        .await?
                } else {
                    // TODO: Log error missing reply subj
                };
                let shut_off_signal =
                    SignalMsg::new(self.id.clone(), ActorSignal::new(self.id.clone(), 0.0));
                self.publish(
                    &actor_current_signal_subject(&self.id),
                    &ActorPubMsg::CurrentSignal(shut_off_signal).into(),
                )
                .await
            }
            Err(_err) => {
                if let Some(reply_subj) = contr_message.reply {
                    self.publish(&reply_subj.into(), &"Error turning off actor".into())
                        .await
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignalMsg {
    pub(crate) id: ClientId,
    pub(crate) timestamp: TimeStamp,
    pub(crate) signal: ActorSignal,
}

impl SignalMsg {
    pub fn new(id: ClientId, signal: ActorSignal) -> Self {
        Self {
            id,
            timestamp: TimeStamp::now(),
            signal,
        }
    }
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
        let sub_subject = tmp.next().unwrap();
        match sub_subject {
            "set_signal" => decode_nats_data(&msg.payload),
            "turn_off" => Ok(Self::TurnOff),
            _ => Err(MessageParseError::InvalidSubject(msg.subject.into())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ActorPubMsg {
    CurrentSignal(SignalMsg),
}

impl From<ActorPubMsg> for PubSubMsg {
    fn from(msg: ActorPubMsg) -> Self {
        match &msg {
            ActorPubMsg::CurrentSignal(signal_msg) => {
                PubSubMsg(serde_json::to_string(&signal_msg).expect("Pub sub serialization error"))
            }
        }
    }
}
