//! Emulates a slow pseudo-pwm GPIO pin.
//!
//! This actor can set a power-level for a GPIO pin.
//! It wraps a [`super::bin_gpio::BinaryGpioActor`] and emulates a power-level via a duty-cycle.
//! For instance a power-level 70% is created with a 10s duty-cycle by turning on the GPIO for 7s
//! and off for 3s and then looping this cycle.
//! On average the power output will be 70%.

use super::{bin_gpio::BinaryGpioActor, ActorSignal};
use crate::{
    actor::{Actor, ActorError},
    logger::debug,
    time::TimeStamp,
};
use embedded_hal::digital::blocking::OutputPin;

pub struct SimpleGpioActor<T: OutputPin + Send> {
    pub id: String,
    bin_gpio: BinaryGpioActor<T>,
    current_signal: ActorSignal,
    cycle_duration: TimeStamp,
    start_time: TimeStamp,
}

impl<T: OutputPin + Send> SimpleGpioActor<T> {
    pub fn try_new(
        id: &str,
        handle: T,
        time_out: Option<TimeStamp>,
    ) -> Result<SimpleGpioActor<T>, ActorError> {
        let bin_id = format!("{}_bin_gpio", id);
        let bin_gpio = BinaryGpioActor::try_new(&bin_id, handle, time_out)?;
        Ok(SimpleGpioActor {
            id: id.into(),
            bin_gpio,
            current_signal: ActorSignal::new(id.into(), 0.0),
            cycle_duration: CYCLE_DURATION,
            start_time: TimeStamp::now(),
        })
    }

    fn pct_to_bin(&self, signal: f32, cycle_duration: TimeStamp) -> f32 {
        let delta = TimeStamp::now() - self.start_time;
        if calculate_cycle_ratio(delta.0 as f32, cycle_duration.0 as f32) > signal {
            0.0
        } else {
            1.0
        }
    }
}

const CYCLE_DURATION: TimeStamp = TimeStamp(10000);

impl<T: OutputPin + Send> Actor for SimpleGpioActor<T> {
    fn update_signal(&mut self, signal: &ActorSignal) -> Result<(), ActorError> {
        self.validate_signal(signal)?;
        self.current_signal = signal.clone();
        Ok(())
    }

    fn set_signal(&mut self) -> Result<(), ActorError> {
        let bin_signal = self.pct_to_bin(self.current_signal.signal, self.cycle_duration);
        let bin_signal = ActorSignal {
            id: self.id.clone().into(),
            signal: bin_signal,
        };
        println!("Bin signal: {:?}", bin_signal);
        self.bin_gpio.update_signal(&bin_signal)?;
        self.bin_gpio.set_signal()?;
        Ok(())
    }

    fn turn_off(&mut self) -> Result<(), ActorError> {
        self.update_signal(&ActorSignal::new(self.id.clone().into(), 0.0))?;
        self.set_signal()
    }

    fn validate_signal(&self, signal: &ActorSignal) -> Result<(), ActorError> {
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
}

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
