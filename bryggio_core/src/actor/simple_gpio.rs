//! Simple model of GPIO pin.
//!
//! This actor has only two states, on and off.

use crate::{
    actor::{Actor, ActorError},
    hardware::{GpioState, HardwareError},
    time::TimeStamp,
};
use embedded_hal::digital::blocking::OutputPin;

use super::ActorSignal;

pub struct SimpleGpioActor<T: OutputPin + Send> {
    pub id: String,
    handle: T,
    state: GpioState,
    time_out: Option<TimeStamp>,
    internal_clock: TimeStamp,
}

impl<T: OutputPin + Send> SimpleGpioActor<T> {
    pub fn try_new(
        id: &str,
        handle: T,
        time_out: Option<TimeStamp>,
    ) -> Result<SimpleGpioActor<T>, ActorError> {
        Ok(SimpleGpioActor {
            id: id.into(),
            handle,
            state: GpioState::Low,
            time_out,
            internal_clock: TimeStamp(0),
        })
    }

    pub fn state(&self) -> GpioState {
        self.state
    }

    pub fn time_out_check(&self) -> Result<(), ActorError> {
        // Always positive since internal_clock is a previous init with ::now()
        let timeout_time = TimeStamp::now() - self.internal_clock;
        if timeout_time < self.time_out.unwrap_or(TimeStamp(0)) {
            Err(ActorError::TimeOut(
                self.internal_clock + self.time_out.unwrap_or(TimeStamp(0)) - TimeStamp::now(),
            ))
        } else {
            Ok(())
        }
    }
}

impl<T: OutputPin + Send> Actor for SimpleGpioActor<T> {
    fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError> {
        if ActorSignal::gpio_state(signal) == self.state {
            return Err(ActorError::ChangingToAlreadyActiveState);
        }
        self.time_out_check()?;
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
            self.handle.set_high().map_err(|_err| {
                ActorError::Hardware(HardwareError::GenericGpio(String::from(
                    "Failed setting high",
                )))
            })?;
            self.state = GpioState::High;
        } else {
            self.handle.set_low().map_err(|_err| {
                ActorError::Hardware(HardwareError::GenericGpio(String::from(
                    "Failed setting low",
                )))
            })?;
            self.state = GpioState::Low;
        }
        self.internal_clock = TimeStamp::now();
        Ok(())
    }

    fn turn_off(&mut self) -> Result<(), ActorError> {
        self.set_signal(&ActorSignal::new(self.id.clone(), 0.0))
    }
}
