#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;
use crate::{
    hardware::GpioState,
    pub_sub::{ClientId, PubSubError},
};
use crate::{hardware::HardwareError, time::TimeStamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod pub_sub;
pub mod simple_gpio;
pub mod xor_gpio;
pub use pub_sub::ActorClient;

pub trait Actor: Send {
    fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError>;
    fn set_signal(&mut self, signal: &ActorSignal) -> Result<(), ActorError>;
    fn turn_off(&mut self) -> Result<(), ActorError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActorSignal {
    // TODO: Deserialization
    id: String,
    signal: f32,
}

impl ActorSignal {
    pub fn new<T: Into<String>>(id: T, signal: f32) -> Self {
        ActorSignal {
            id: id.into(),
            signal,
        }
    }

    pub fn gpio_state(&self) -> GpioState {
        if self.signal > 0.0 {
            GpioState::High
        } else {
            GpioState::Low
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum ActorType {
    #[serde(rename = "simple_gpio")]
    SimpleGpio {
        pin_number: u32,
        time_out: Option<TimeStamp>,
    },
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
            ActorType::SimpleGpio {
                pin_number,
                time_out,
            } => {
                let gpio_pin = hardware_impl::get_gpio_pin(*pin_number, &self.id.as_ref())
                    .map_err(HardwareError::from)?;
                let actor =
                    simple_gpio::SimpleGpioActor::try_new(self.id.as_ref(), gpio_pin, *time_out)?;
                Ok(Box::new(actor))
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Invalid signal: {signal}, must be in ({lower_bound}, {upper_bound})")]
    InvalidSignal {
        signal: f32,
        lower_bound: f32,
        upper_bound: f32,
    },
    #[error("No state change in signal")]
    ChangingToAlreadyActiveState,
    #[error("Remaining cool down time: {0}")]
    TimeOut(TimeStamp),
    #[error("Generic: {0}")]
    Generic(String),
    #[error("Hardware: {0}")]
    Hardware(#[from] HardwareError),
}

impl From<ActorError> for PubSubError {
    fn from(err: ActorError) -> PubSubError {
        PubSubError::Client(format!("Actor error: '{}'", err))
    }
}
