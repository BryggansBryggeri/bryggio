pub mod hysteresis;
use crate::actor;
use crate::sensor;
use std::f32;
use std::sync;
use std::{thread, time};

pub type ControllerHandle = sync::Arc<sync::Mutex<Box<dyn Control>>>;

#[derive(Clone, Debug)]
pub enum State {
    Inactive,
    Automatic,
    Manual,
}

pub fn run_controller<C: ?Sized, A: ?Sized>(
    controller_lock: sync::Arc<sync::Mutex<C>>,
    actor_lock: sync::Arc<sync::Mutex<A>>,
    sensor: sensor::SensorHandle,
) where
    C: Control,
    A: actor::Actor,
{
    let start_time = time::SystemTime::now();
    let sleep_time = 1000;
    let actor = match actor_lock.lock() {
        Ok(actor) => actor,
        Err(err) => panic!("Could not acquire actor lock: {}", err),
    };
    loop {
        let mut controller = match controller_lock.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock {}", err),
        };
        match controller.get_state() {
            State::Inactive => {
                println!("Inactivating controller, stopping");
                return;
            }
            State::Automatic => {
                let measurement = match sensor::get_measurement(&sensor) {
                    Ok(measurement) => Some(measurement),
                    Err(err) => {
                        println!(
                            "Error getting measurment from sensor: {}. Error: {}",
                            "some_id", //sensor.get_id(),
                            err
                        );
                        None
                    }
                };
                let signal = controller.calculate_signal(measurement);
                drop(controller);
                match actor.set_signal(signal) {
                    Ok(()) => {}
                    Err(err) => println!("Error setting signal: {}", err),
                };
                println!(
                    "{}, {}, {}.",
                    start_time.elapsed().unwrap().as_secs(),
                    measurement.unwrap_or(f32::NAN),
                    signal
                );
            }
            State::Manual => {
                let signal = controller.get_signal();
                drop(controller);
                match actor.set_signal(signal) {
                    Ok(()) => {}
                    Err(err) => println!("Error setting signal: {}", err),
                };
                println!("{}, {}.", start_time.elapsed().unwrap().as_secs(), signal);
            }
        }
        thread::sleep(time::Duration::from_millis(sleep_time));
    }
}

pub trait Control {
    fn run<A>(
        &mut self,
        sleep_time: u64,
        actor: sync::Arc<sync::Mutex<A>>,
        sensor: sensor::SensorHandle,
    ) where
        A: actor::Actor;
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn update_state(&self);
    fn get_state(&self) -> State;
    fn get_signal(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
}
