//! Emulates a slow pseudo-PWM GPIO pin via duty-cycle.
//!
//! For instance 70% power on a 10s cycle = 7s on, 3s off.

use super::bin_gpio::BinaryGpioActor;
use crate::{actor::ActorError, actor::ActorSignal, time::TimeStamp};
use embedded_hal::digital::OutputPin;

pub struct SimpleGpioActor<T: OutputPin + Send> {
    bin_gpio: BinaryGpioActor<T>,
    current_signal: ActorSignal,
    cycle_duration: TimeStamp,
    start_time: TimeStamp,
}

impl<T: OutputPin + Send> SimpleGpioActor<T> {
    pub fn try_new(
        handle: T,
        time_out: Option<TimeStamp>,
    ) -> Result<SimpleGpioActor<T>, ActorError> {
        let bin_gpio = BinaryGpioActor::try_new(handle, time_out)?;
        Ok(SimpleGpioActor {
            bin_gpio,
            current_signal: ActorSignal::new(0.0),
            cycle_duration: CYCLE_DURATION,
            start_time: TimeStamp::now(),
        })
    }

    /// Map percentage to binary signal based on location in period
    fn pct_to_bin(&self, signal: f32, cycle_duration: TimeStamp) -> f32 {
        let delta = TimeStamp::now() - self.start_time;
        if calculate_cycle_ratio(delta.0 as f32, cycle_duration.0 as f32) > signal {
            0.0
        } else {
            1.0
        }
    }

    pub fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError> {
        if signal.signal >= 0.0 && signal.signal <= 1.0 {
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
        let bin_signal = self.pct_to_bin(self.current_signal.signal, self.cycle_duration);
        let bin_signal = ActorSignal::new(bin_signal);
        if self.bin_gpio.current_signal != bin_signal {
            self.bin_gpio.update_signal(&bin_signal)?;
            self.bin_gpio.set_signal()?;
        }
        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<(), ActorError> {
        self.update_signal(&ActorSignal::new(0.0))?;
        self.set_signal()
    }
}

const CYCLE_DURATION: TimeStamp = TimeStamp(10000);

fn calculate_cycle_ratio(delta: f32, cycle_length: f32) -> f32 {
    (delta % cycle_length) / cycle_length
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn duty_cycle() {
        assert_approx_eq!(calculate_cycle_ratio(17.0, 10.0), 0.7);
        assert_approx_eq!(calculate_cycle_ratio(27.0, 10.0), 0.7);
    }
}
