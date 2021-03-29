use crate::{
    actor::{Actor, ActorError},
    hardware::HardwareError,
};
use embedded_hal::digital::OutputPin;

pub struct SimpleGpioActor<T: OutputPin + Send> {
    pub id: String,
    handle: T,
}

impl<T: OutputPin + Send> SimpleGpioActor<T> {
    pub fn try_new(id: &str, handle: T) -> Result<SimpleGpioActor<T>, ActorError> {
        Ok(SimpleGpioActor {
            id: id.into(),
            handle,
        })
    }
}

impl<T: OutputPin + Send> Actor for SimpleGpioActor<T> {
    type Signal = f32;
    fn validate_signal(&self, signal: Self::Signal) -> Result<(), ActorError> {
        if signal >= 0.0 {
            Ok(())
        } else {
            Err(ActorError::InvalidSignal {
                signal,
                lower_bound: 0.0,
                upper_bound: 1.0,
            })
        }
    }

    fn set_signal(&mut self, signal: Self::Signal) -> Result<(), ActorError> {
        self.validate_signal(signal)?;
        if signal > 0.0 {
            self.handle.try_set_high().map_err(|_err| {
                ActorError::Hardware(HardwareError::GenericGpio(String::from(
                    "Failed setting high",
                )))
            })
        } else {
            self.handle.try_set_low().map_err(|_err| {
                ActorError::Hardware(HardwareError::GenericGpio(String::from(
                    "Failed setting low",
                )))
            })
        }
    }
}
