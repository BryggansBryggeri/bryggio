#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;
use crate::pub_sub::{ClientId, PubSubError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod pub_sub;
pub mod simple_gpio;
pub use pub_sub::ActorClient;

pub trait Actor: Send {
    fn validate_signal(&self, signal: f32) -> Result<(), ActorError>;
    fn set_signal(&mut self, signal: f32) -> Result<(), ActorError>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum ActorType {
    #[serde(rename = "simple_gpio")]
    SimpleGpio(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActorConfig {
    pub id: ClientId,
    #[serde(rename = "type")]
    pub(crate) type_: ActorType,
}

impl ActorConfig {
    pub fn get_actor(&self) -> Result<Box<dyn Actor>, ActorError> {
        match &self.type_ {
            ActorType::SimpleGpio(pin_number) => {
                let gpio_pin = hardware_impl::get_gpio_pin(*pin_number, &self.id.as_ref()).unwrap();
                let actor = simple_gpio::SimpleGpioActor::try_new(self.id.as_ref(), gpio_pin)?;
                Ok(Box::new(actor))
            }
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum ActorError {
    #[error("Invalid signal: {signal}, must be in ({lower_bound}, {upper_bound})")]
    InvalidSignal {
        signal: f32,
        lower_bound: f32,
        upper_bound: f32,
    },
    #[error("Generic: {0}")]
    Generic(String),
}

impl From<ActorError> for PubSubError {
    fn from(err: ActorError) -> PubSubError {
        PubSubError::Client(format!("Actor error: '{}'", err))
    }
}
