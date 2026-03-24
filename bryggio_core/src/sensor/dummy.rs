//! Dummy sensor for debugging and prototyping.
use rand::rng;
use rand_distr::{Distribution, Normal};

/// Generic dummy sensor providing mock measurements through a random walk.
pub struct DummySensor {
    latest_value: f32,
    rng: Normal<f32>,
}

impl DummySensor {
    pub fn new(initial_value: f32) -> DummySensor {
        let normal_distr = Normal::new(0.0, 10.0).expect("valid distribution params");
        DummySensor {
            latest_value: initial_value,
            rng: normal_distr,
        }
    }

    pub fn get_measurement(&mut self) -> f32 {
        let measurement = self.latest_value + self.rng.sample(&mut rng()) / 10.0;
        self.latest_value = measurement;
        measurement
    }
}
