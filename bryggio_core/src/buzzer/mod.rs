use crate::actor::{bin_gpio::BinaryGpioActor, Actor, ActorError, ActorSignal};
#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;
use crate::hardware::HardwareError;
use crate::pub_sub::PubSubError;
use thiserror::Error;

pub mod pub_sub;

pub struct Buzzer {
    constant: BinaryGpioActor<hardware_impl::GpioPin>,
    pulse: BinaryGpioActor<hardware_impl::GpioPin>,
}

impl Buzzer {
    pub fn try_new(id: &str, constant: u32, pulse: u32) -> Result<Self, BuzzerError> {
        let constant_pin = hardware_impl::get_gpio_pin(constant, id)?;
        let pulse_pin = hardware_impl::get_gpio_pin(pulse, id)?;
        let constant_actor = BinaryGpioActor::try_new(id, constant_pin, None)?;
        let pulse_actor = BinaryGpioActor::try_new(id, pulse_pin, None)?;
        Ok(Buzzer {
            constant: constant_actor,
            pulse: pulse_actor,
        })
    }

    pub fn constant(&mut self) -> Result<(), BuzzerError> {
        self.stop()?;
        let signal = ActorSignal::new(self.constant.id.clone(), 1.0);
        Ok(self.constant.set_signal(&signal)?)
    }
    pub fn pulse(&mut self) -> Result<(), BuzzerError> {
        self.stop()?;
        let signal = ActorSignal::new(self.pulse.id.clone(), 1.0);
        Ok(self.pulse.set_signal(&signal)?)
    }
    pub fn stop(&mut self) -> Result<(), BuzzerError> {
        let signal = ActorSignal::new(self.constant.id.clone(), 1.0);
        self.constant.set_signal(&signal)?;
        let signal = ActorSignal::new(self.pulse.id.clone(), 1.0);
        Ok(self.pulse.set_signal(&signal)?)
    }
}

#[derive(Error, Debug)]
pub enum BuzzerError {
    #[error("Generic")]
    Generic,
    #[error("Hardware error {0}")]
    Hardware(#[from] HardwareError),
    #[error("Actor error {0}")]
    Actor(#[from] ActorError),
}

impl From<BuzzerError> for PubSubError {
    fn from(err: BuzzerError) -> Self {
        PubSubError::Client(format!("Buzzer error: '{}'", err))
    }
}
