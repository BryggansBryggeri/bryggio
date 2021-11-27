use super::ActorSignal;
use crate::actor::ActorError;
use crate::logger::{error, info};
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsClientConfig,
    ClientId, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::time::{TimeStamp, LOOP_PAUSE_TIME};
use crate::{actor::Actor, pub_sub::MessageParseError};
use nats::{Message, Subscription};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::thread::sleep;

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
    fn client_loop(mut self) -> Result<(), PubSubError> {
        info(
            &self,
            format!("Starting actor with id '{}'", self.id),
            &format!("actor.{}", self.id),
        );
        let sub_set_signal = self.subscribe(&actor_set_signal_subject(&self.id))?;
        let sub_turn_off = self.subscribe(&actor_turn_off_subject(&self.id))?;
        loop {
            if let Some(contr_message) = sub_set_signal.try_next() {
                let res: Result<(), PubSubError> =
                    match ActorSubMsg::try_from(contr_message.clone()) {
                        Ok(msg) => match msg {
                            ActorSubMsg::SetSignal(new_signal) => {
                                let sign_res = self.actor.update_signal(&new_signal.signal);
                                match sign_res {
                                    Ok(()) => self.publish(
                                        &actor_current_signal_subject(&self.id),
                                        &ActorPubMsg::CurrentSignal(new_signal).into(),
                                    ),
                                    Err(err) => match err {
                                        ActorError::ChangingToAlreadyActiveState => self.publish(
                                            &actor_current_signal_subject(&self.id),
                                            &ActorPubMsg::CurrentSignal(new_signal).into(),
                                        ),
                                        _ => Err(err.into()),
                                    },
                                }
                            }
                            ActorSubMsg::TurnOff => match self.actor.turn_off() {
                                Ok(()) => {
                                    let shut_off_signal = SignalMsg::new(
                                        self.id.clone(),
                                        ActorSignal::new(self.id.clone(), 0.0),
                                    );
                                    if let Err(err) = self.publish(
                                        &actor_turn_off_subject(&self.id),
                                        &ActorPubMsg::CurrentSignal(shut_off_signal).into(),
                                    ) {
                                        error(
                                            &self,
                                            format!("Failed switching off client '{}'", err),
                                            &format!("actor.{}", self.id),
                                        )
                                    };
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
            match self.actor.set_signal() {
                Err(err) => match err {
                    ActorError::ChangingToAlreadyActiveState => {}
                    _ => error(
                        &self,
                        format!("Failed setting signal: '{}'", err.to_string()),
                        &format!("actor.{}", self.id),
                    ),
                },
                _ => {}
            };
            sleep(LOOP_PAUSE_TIME);
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}

impl ActorClient {
    pub fn new(id: ClientId, actor: Box<dyn Actor>, config: &NatsClientConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ActorClient { id, actor, client }
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
        println!("sub subject {}", sub_subject);
        match sub_subject {
            "set_signal" => decode_nats_data(&msg.data),
            "turn_off" => Ok(Self::TurnOff),
            _ => Err(MessageParseError::InvalidSubject(Subject(msg.subject))),
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
