//! GPIO drivers — PWM emulation and binary on/off.
//!
//! All public methods take `&self` (interior mutability via `Mutex`)
//! so these types can be used from the `Hal` trait without outer locking.
//!
//! Pin I/O uses the Linux GPIO character device (`/dev/gpiochipN`)
//! via the `gpio_cdev` crate.

use bryggio_core::types::Power;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::sync::Mutex;
use std::time::Instant;

/// Duty-cycle period in milliseconds.
const CYCLE_DURATION_MS: u128 = 10_000;

/// Default GPIO chip device path on Raspberry Pi.
const DEFAULT_CHIP: &str = "/dev/gpiochip0";

pub use gpio_cdev::Error as GpioError;

/// Request a single output line, driven low initially.
fn request_output(pin: u32, label: &str) -> Result<LineHandle, GpioError> {
    let mut chip = Chip::new(DEFAULT_CHIP)?;
    let line = chip.get_line(pin)?;
    let handle = line.request(LineRequestFlags::OUTPUT, 0, label)?;
    Ok(handle)
}

// ---------------------------------------------------------------------------
// PwmGpio
// ---------------------------------------------------------------------------

struct PwmInner {
    state: bool,
    current_power: Power,
    start_time: Instant,
}

/// PWM-emulating GPIO actor.
///
/// Emulates continuous power output on a binary GPIO pin by cycling on/off
/// within a fixed period (e.g. 70% power = 7 s on, 3 s off in a 10 s cycle).
///
/// The line is released automatically when this struct is dropped.
pub struct PwmGpio {
    handle: LineHandle,
    inner: Mutex<PwmInner>,
}

impl PwmGpio {
    pub fn new(pin: u32, label: &str) -> Result<Self, GpioError> {
        let handle = request_output(pin, label)?;
        Ok(PwmGpio {
            handle,
            inner: Mutex::new(PwmInner {
                state: false,
                current_power: Power::off(),
                start_time: Instant::now(),
            }),
        })
    }

    /// Set the desired power level [0, 1].
    pub fn set_power(&self, power: Power) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.current_power = power;
    }

    /// Call this on each tick to update the GPIO state based on duty cycle.
    /// Returns whether the pin is high.
    pub fn tick(&self) -> Result<bool, GpioError> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let position = inner.start_time.elapsed().as_millis() % CYCLE_DURATION_MS;
        // power is [0.0, 1.0], CYCLE_DURATION_MS fits in f64 — safe truncation.
        #[allow(clippy::as_conversions)]
        let threshold = (f64::from(inner.current_power.value()) * CYCLE_DURATION_MS as f64) as u128;
        let high = position < threshold;
        if high != inner.state {
            self.handle.set_value(u8::from(high))?;
            inner.state = high;
        }
        Ok(high)
    }
}

impl Drop for PwmGpio {
    fn drop(&mut self) {
        let _ = self.handle.set_value(0);
    }
}

// ---------------------------------------------------------------------------
// BinaryGpio
// ---------------------------------------------------------------------------

/// Simple on/off GPIO pin (e.g. pump relay).
///
/// The line is released automatically when this struct is dropped.
pub struct BinaryGpio {
    handle: LineHandle,
    state: Mutex<bool>,
}

impl BinaryGpio {
    pub fn new(pin: u32, label: &str) -> Result<Self, GpioError> {
        let handle = request_output(pin, label)?;
        Ok(BinaryGpio {
            handle,
            state: Mutex::new(false),
        })
    }

    pub fn set(&self, on: bool) -> Result<(), GpioError> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if on != *state {
            self.handle.set_value(u8::from(on))?;
            *state = on;
        }
        Ok(())
    }
}

impl Drop for BinaryGpio {
    fn drop(&mut self) {
        let _ = self.handle.set_value(0);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
}
