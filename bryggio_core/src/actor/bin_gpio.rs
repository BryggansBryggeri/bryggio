//! Direct model of a GPIO pin.
//!
//! Like the actual hardware, this actor has only two states, on and off.
//! This model is useful in itself, for simple hardware like indicators,
//! but also for more complex abstractions.

use crate::{
    actor::{Actor, ActorError},
    hardware::{GpioState, HardwareError},
    time::TimeStamp,
};
use embedded_hal::digital::OutputPin;

use super::ActorSignal;

pub struct BinaryGpioActor<T: OutputPin + Send> {
    pub id: String,
    pub(crate) handle: T,
    pub(crate) state: GpioState,
    pub(crate) current_signal: ActorSignal,
    time_out: Option<TimeStamp>,
    internal_clock: TimeStamp,
}

impl<T: OutputPin + Send> BinaryGpioActor<T> {
    pub fn try_new(
        id: &str,
        handle: T,
        time_out: Option<TimeStamp>,
    ) -> Result<BinaryGpioActor<T>, ActorError> {
        Ok(BinaryGpioActor {
            id: id.into(),
            handle,
            state: GpioState::Low,
            current_signal: ActorSignal::new(id.into(), 0.0),
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

impl<T: OutputPin + Send> Actor for BinaryGpioActor<T> {
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

    fn update_signal(&mut self, signal: &ActorSignal) -> Result<(), ActorError> {
        self.validate_signal(signal)?;
        self.current_signal = signal.clone();
        Ok(())
    }

    fn set_signal(&mut self) -> Result<(), ActorError> {
        if self.current_signal.signal > 0.0 {
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
        self.update_signal(&ActorSignal::new(self.id.clone().into(), 0.0))?;
        self.set_signal()
    }
}
