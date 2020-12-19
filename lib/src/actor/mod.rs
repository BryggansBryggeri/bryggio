pub mod simple_gpio;
use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, ClientId, ClientState, PubSubClient,
    PubSubError, PubSubMsg, Subject,
};
use nats::{Message, Subscription};
use std::convert::TryFrom;
use std::error as std_error;

pub trait Actor: Send {
    fn validate_signal(&self, signal: f32) -> Result<(), Error>;
    fn set_signal(&mut self, signal: f32) -> Result<(), Error>;
}

pub struct ActorClient<A>
where
    A: Actor,
{
    id: ClientId,
    actor: A,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl<A> ActorClient<A>
where
    A: Actor,
{
    pub fn new(id: ClientId, actor: A, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ActorClient { id, actor, client }
    }

    fn gen_signal_subject(&self) -> Subject {
        Subject(format!("actor.{}.current_signal", self.id))
    }
}

#[derive(Debug)]
pub enum ActorSubMsg {
    SetSignal(f32),
    Stop,
}

impl TryFrom<Message> for ActorSubMsg {
    type Error = PubSubError;
    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let signal = String::from_utf8(value.data)
            .map_err(|err| PubSubError::MessageParse(err.to_string()))?
            .parse::<f32>()
            .map_err(|err| PubSubError::MessageParse(err.to_string()))?;
        Ok(ActorSubMsg::SetSignal(signal))
    }
}

#[derive(Debug)]
pub enum ActorPubMsg {
    CurrentSignal(f32),
}

impl ActorPubMsg {
    fn into_msg(&self) -> PubSubMsg {
        match self {
            ActorPubMsg::CurrentSignal(signal) => PubSubMsg(format!("{}", signal)),
        }
    }
}

impl<A> PubSubClient for ActorClient<A>
where
    A: Actor,
{
    type Return = ();
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let sub = self.subscribe(&Subject(format!("actor.{}.set_signal", self.id)))?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(contr_message) = sub.next() {
                if let Ok(msg) = ActorSubMsg::try_from(contr_message) {
                    match msg {
                        ActorSubMsg::SetSignal(signal) => {
                            if let Ok(()) = self.actor.set_signal(signal) {
                                self.publish(
                                    &self.gen_signal_subject(),
                                    &ActorPubMsg::CurrentSignal(signal).into_msg(),
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

#[derive(Debug, Clone)]
pub enum Error {
    InvalidSignal(String),
    ActorError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidSignal(signal) => write!(f, "Invalid signal: {}", signal),
            Error::ActorError(error) => write!(f, "Actor error: {}", error),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidSignal(_) => "Invalid signal",
            Error::ActorError(_) => "Actor error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
