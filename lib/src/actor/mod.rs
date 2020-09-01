pub mod simple_gpio;
use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, ClientId, Message, PubSubClient, PubSubError,
    Subject,
};
use nats::Subscription;
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
    controller_id: ClientId,
    actor: A,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl<A> ActorClient<A>
where
    A: Actor,
{
    pub fn new(id: ClientId, controller_id: ClientId, actor: A, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ActorClient {
            id,
            controller_id,
            actor,
            client,
        }
    }

    fn gen_signal_msg(&self, signal: f32) -> Message {
        Message(format!("{}", signal))
    }

    fn gen_signal_subject(&self) -> Subject {
        Subject(format!("actor.{}.current_signal", self.id))
    }
}

impl<A> PubSubClient for ActorClient<A>
where
    A: Actor,
{
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let supervisor = self.subscribe(&Subject(format!("command.actor.{}.*", self.id)))?;
        let controller = self.subscribe(&Subject(format!("actor.{}.set_signal", self.id)))?;
        loop {
            for _msg in supervisor.try_iter() {
                // TODO: Deal with supervisor command
            }
            // TODO: abstract the parsing of messages.
            if let Some(contr_message) = controller.next() {
                if let Ok(signal) = String::from_utf8(contr_message.data) {
                    if let Ok(signal) = signal.parse() {
                        self.actor.set_signal(signal);
                        self.publish(&self.gen_signal_subject(), &self.gen_signal_msg(signal))?;
                    }
                }
            }
        }
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, subject: &Subject, msg: &Message) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidSignal(f32),
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
