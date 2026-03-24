//! Direct model of a GPIO pin — binary on/off.
use crate::{
    actor::{ActorError, ActorSignal},
    hardware::{GpioState, HardwareError},
    time::TimeStamp,
};
use embedded_hal::digital::OutputPin;

pub struct BinaryGpioActor<T: OutputPin + Send> {
    pub(crate) handle: T,
    pub(crate) state: GpioState,
    pub(crate) current_signal: ActorSignal,
    time_out: Option<TimeStamp>,
    internal_clock: TimeStamp,
}

impl<T: OutputPin + Send> BinaryGpioActor<T> {
    pub fn try_new(
        handle: T,
        time_out: Option<TimeStamp>,
    ) -> Result<BinaryGpioActor<T>, ActorError> {
        Ok(BinaryGpioActor {
            handle,
            state: GpioState::Low,
            current_signal: ActorSignal::new(0.0),
            time_out,
            internal_clock: TimeStamp(0),
        })
    }

    pub fn state(&self) -> GpioState {
        self.state
    }

    pub fn time_out_check(&self) -> Result<(), ActorError> {
        let timeout_time = TimeStamp::now() - self.internal_clock;
        if timeout_time < self.time_out.unwrap_or(TimeStamp(0)) {
            Err(ActorError::TimeOut(
                self.internal_clock + self.time_out.unwrap_or(TimeStamp(0)) - TimeStamp::now(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError> {
        if signal.gpio_state() == self.state {
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

    pub fn update_signal(&mut self, signal: &ActorSignal) -> Result<(), ActorError> {
        self.validate_signal(signal)?;
        self.current_signal = signal.clone();
        Ok(())
    }

    pub fn set_signal(&mut self) -> Result<(), ActorError> {
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

    pub fn turn_off(&mut self) -> Result<(), ActorError> {
        self.update_signal(&ActorSignal::new(0.0))?;
        self.set_signal()
    }
}
