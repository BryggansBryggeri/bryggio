use crate::config;
use crate::control::Control;
use crate::control::HysteresisControl;
use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Brewery {
    pub mash_tun: MashTun,
    kettle: Kettle,
    state: Arc<Mutex<bool>>,
}

impl Brewery {
    pub fn new(config: &config::Config, state: Arc<Mutex<bool>>) -> Brewery {
        Brewery {
            mash_tun: MashTun::new(state.clone()),
            kettle: Kettle { controller: 7 },
            state: state,
        }
    }

    pub fn run(&mut self) {
        self.mash_tun.run();
    }
}

pub struct MashTun {
    pub controller: HysteresisControl,
}

impl MashTun {
    pub fn new(state: Arc<Mutex<bool>>) -> MashTun {
        MashTun {
            controller: HysteresisControl::new(2.0, 1.0, state).unwrap(),
        }
    }

    pub fn run(&mut self) {
        println!("Starting controller");
        self.controller.run(1000);
    }
}

struct Kettle {
    controller: u8,
}

pub trait Vessel {
    fn set_target_temp(&self) -> Result<(), io::Error> {
        Ok(())
    }

    fn get_target_temp(&self) -> Result<(f32), io::Error> {
        Ok(63.0)
    }

    fn update_control_signal(&self) -> Result<(), io::Error> {
        Ok(())
    }
}
