//! GPIO driver — PWM emulation via duty-cycle.
//!
//! Wraps a binary GPIO pin and emulates continuous power output
//! by cycling on/off within a fixed period (e.g. 70% = 7s on, 3s off in a 10s cycle).

use bryggio_core::types::Power;
use std::time::Instant;

/// Duty-cycle period in milliseconds.
const CYCLE_DURATION_MS: u128 = 10_000;
const CYCLE_DURATION_MS_F32: f32 = CYCLE_DURATION_MS as f32;
const _: () = assert!(CYCLE_DURATION_MS_F32 as u128 == CYCLE_DURATION_MS);

/// PWM-emulating GPIO actor.
pub struct PwmGpio {
    state: bool,
    current_power: Power,
    start_time: Instant,
}

impl PwmGpio {
    pub fn new() -> Self {
        PwmGpio {
            state: false,
            current_power: Power::off(),
            start_time: Instant::now(),
        }
    }

    /// Set the desired power level [0, 1].
    pub fn set_power(&mut self, power: Power) {
        self.current_power = power;
    }

    /// Call this on each tick to update the GPIO state based on duty cycle.
    /// Returns whether the pin should be high or low.
    pub fn tick(&mut self) -> bool {
        let position = self.start_time.elapsed().as_millis() % CYCLE_DURATION_MS;
        let threshold = (self.current_power.value() * CYCLE_DURATION_MS_F32) as u128;
        self.state = position < threshold;
        self.state
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
    fn duty_cycle_ratio() {
        assert_approx_eq!(calculate_cycle_ratio(17.0, 10.0), 0.7);
        assert_approx_eq!(calculate_cycle_ratio(27.0, 10.0), 0.7);
    }

    #[test]
    fn integer_tick_matches_tick() {
        let powers = [0.0, 0.1, 0.25, 0.5, 0.7, 1.0];
        for &p in &powers {
            let mut gpio_a = PwmGpio::new();
            let mut gpio_b = PwmGpio::new();
            gpio_a.set_power(Power::new(p));
            gpio_b.set_power(Power::new(p));
            // Synchronize start times
            gpio_b.start_time = gpio_a.start_time;

            for _ in 0..100 {
                let float_result = gpio_a.tick();
                let int_result = gpio_b.integer_tick();
                assert_eq!(
                    float_result,
                    int_result,
                    "Mismatch at power={p}, elapsed={}ms",
                    gpio_a.start_time.elapsed().as_millis()
                );
            }
        }
    }
}
