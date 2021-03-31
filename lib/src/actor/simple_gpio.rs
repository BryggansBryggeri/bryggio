use crate::{
    actor::{Actor, ActorError},
    hardware::{dummy::GpioState, HardwareError},
};
use embedded_hal::digital::OutputPin;

use super::ActorSignal;

pub struct SimpleGpioActor<T: OutputPin + Send> {
    pub id: String,
    handle: T,
    state: GpioState,
}

impl<T: OutputPin + Send> SimpleGpioActor<T> {
    pub fn try_new(id: &str, handle: T) -> Result<SimpleGpioActor<T>, ActorError> {
        Ok(SimpleGpioActor {
            id: id.into(),
            handle,
            state: GpioState::Low,
        })
    }

    pub fn state(&self) -> GpioState {
        self.state
    }
}

impl<T: OutputPin + Send> Actor for SimpleGpioActor<T> {
    fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError> {
        if signal.signal >= 0.0 {
            Ok(())
        } else {
            Err(ActorError::InvalidSignal {
                signal: signal.signal,
                lower_bound: 0.0,
                upper_bound: 1.0,
            })
        }
    }

    fn set_signal(&mut self, signal: &ActorSignal) -> Result<(), ActorError> {
        self.validate_signal(signal)?;
        if signal.signal > 0.0 {
            self.handle.try_set_high().map_err(|_err| {
                ActorError::Hardware(HardwareError::GenericGpio(String::from(
                    "Failed setting high",
                )))
            })?;
            self.state = GpioState::High;
            Ok(())
        } else {
            self.handle.try_set_low().map_err(|_err| {
                ActorError::Hardware(HardwareError::GenericGpio(String::from(
                    "Failed setting low",
                )))
            })?;
            self.state = GpioState::Low;
            Ok(())
        }
    }
}
