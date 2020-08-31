pub mod duty_cycle;
pub mod hysteresis;
pub mod manual;
pub mod pid;

use crate::pub_sub::{
    nats_client::NatsClient, nats_client::NatsConfig, ClientId, Message, PubSubClient, PubSubError,
    Subject,
};
use std::convert::TryFrom;
use std::error as std_error;
use std::f32;

pub trait Control: Send {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn get_state(&self) -> State;
    fn set_state(&mut self, new_state: State);
    fn get_control_signal(&self) -> f32;
    fn get_target(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
}

pub struct ControllerClient<C>
where
    C: Control,
{
    id: ClientId,
    controller: C,
    client: NatsClient,
}

impl<C> ControllerClient<C>
where
    C: Control,
{
    pub fn new(id: ClientId, controller: C, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).unwrap();
        ControllerClient {
            id,
            controller,
            client,
        }
    }
}

// impl<C> PubSubClient for ControllerClient<C>
// where
//     C: Control,
// {
//     fn client_loop(self) -> Result<(), PubSubError> {
//         let sensor_subject = Subject(format!("sensor.{}", self.id));
//         let sensor = self.subscribe(&sensor_subject)?;
//         let actor_subject = Subject(format!("actor.{}", self.id));
//         loop {
//             for meas in sensor.try_iter() {
//                 self.controller.calculate_signal(meas);
//                 self.publish(&self.gen_meas_subject(), &self.gen_meas_msg(meas))?;
//                 sleep(Duration::from_millis(500));
//             }
//         }
//     }
//
//     fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
//         self.client.subscribe(subject)
//     }
//
//     fn publish(&self, subject: &Subject, msg: &Message) -> Result<(), PubSubError> {
//         self.client.publish(subject, msg)
//     }
// }

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Inactive,
    Active,
}

pub enum ControllerType {
    Hysteresis,
    Manual,
}

impl TryFrom<String> for ControllerType {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Error> {
        match string.to_ascii_lowercase().as_ref() {
            "hysteresis" => Ok(ControllerType::Hysteresis),
            "manual" => Ok(ControllerType::Manual),
            _ => Err(Error::ConversionError(string.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ParamError(String),
    ConcurrencyError(String),
    ConversionError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ParamError(param) => write!(f, "Invalid param: {}", param),
            Error::ConcurrencyError(err) => write!(f, "Concurrency error: {}", err),
            Error::ConversionError(type_string) => {
                write!(f, "Unable to parse '{}' to ControllerType", type_string)
            }
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParamError(_) => "Invalid param",
            Error::ConcurrencyError(_) => "Concurrency error",
            Error::ConversionError(_) => "Conversion error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
