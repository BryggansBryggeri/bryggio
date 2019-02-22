use gpio_cdev::{Chip, LineRequestFlags};
use std::{thread, time};

pub enum Mode {
    Automatic,
    Manual,
    Boil,
    Inactive,
}

enum HysteresisState {
    Below,
    Within,
    Over,
    Unmeasured,
}

pub struct HysteresisControl {
    pub offset: f32,
    pub target: f32,
    pub current_power: f32,
    pub mode: Mode,
    actor: Actor,
    sensor: Sensor,
    current_state: HysteresisState,
    previous_state: HysteresisState,
}

impl HysteresisControl {
    pub fn new(offset: f32) -> HysteresisControl {
        HysteresisControl {
            offset: offset,
            target: 68.0,
            current_power: 0.0,
            mode: Mode::Inactive,
            current_state: HysteresisState::Unmeasured,
            previous_state: HysteresisState::Unmeasured,
        }
    }

    fn get_state(&self, value: &f32) -> HysteresisState {
        if value - self.target < -self.offset {
            return HysteresisState::Below;
        } else if value - self.target > self.offset {
            return HysteresisState::Over;
        } else {
            return HysteresisState::Within;
        }
    }
}

impl Control for HysteresisControl {
    fn run(&self) {
        let pin_number = 21;
        let label = "rustbeer";
        let mut chip = Chip::new("/dev/gpiochip0").unwrap();
        let handle = chip.get_line(pin_number).unwrap();
        handle.request(LineRequestFlags::OUTPUT, 1, label).unwrap();

        match &self.mode {
            Mode::Inactive => {}
            _ => {
                let measurement = self.get_measurement();
                let power = self.calculate_power(&measurement);
                self.hardware_control(power, self.get_period());
                thread::sleep(self.get_sleep_time());
            }
        }
    }

    fn get_measurement(&self) -> f32 {
        65.0
    }

    fn calculate_power(&self, measurement: &f32) -> f32 {
        match self.get_state(measurement) {
            HysteresisState::Below => {
                return 100.0;
            }
            HysteresisState::Over => {
                return 0.0;
            }
            HysteresisState::Within => {
                return 50.0;
            }
            HysteresisState::Unmeasured => {
                return 0.0;
            }
        }
    }

    fn get_sleep_time(&self) -> time::Duration {
        time::Duration::from_secs(3)
    }

    fn get_period(&self) -> time::Duration {
        std::time::Duration::from_millis(10000)
    }

    fn is_active(&self) -> bool {
        true
    }

    fn hardware_control(&self, power: f32, period: time::Duration) {}
}

pub trait Control {
    fn run(&self);
    fn get_measurement(&self) -> f32;
    fn calculate_power(&self, measurement: &f32) -> f32;
    fn get_sleep_time(&self) -> time::Duration;
    fn get_period(&self) -> time::Duration;
    fn is_active(&self) -> bool;
    fn hardware_control(&self, power: f32, period: time::Duration);
}
