use crate::config;
use crate::control::Control;
use crate::control::HysteresisControl;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct BrewState {
    pub name: Arc<Mutex<String>>,
    pub controller: Arc<Mutex<bool>>,
}

impl Serialize for BrewState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BrewState", 1)?;
        state.serialize_field("name", &self.name.lock().unwrap().clone())?;
        state.end()
    }
}

pub struct Brewery {
    pub mash_tun: MashTun,
    kettle: Kettle,
    state: BrewState,
}

impl Brewery {
    pub fn new(config: &config::Config, state: BrewState) -> Brewery {
        Brewery {
            mash_tun: MashTun::new(state.clone()),
            kettle: Kettle { controller: 7 },
            state: state,
        }
    }

    pub fn run(&mut self) {
        self.mash_tun.run();
    }

    pub fn generate_state(config: &config::Config) -> BrewState {
        BrewState {
            name: Arc::new(Mutex::new(config.name.clone())),
            controller: Arc::new(Mutex::new(false)),
        }
    }
}

pub struct MashTun {
    pub controller: HysteresisControl,
}

impl MashTun {
    pub fn new(state: BrewState) -> MashTun {
        MashTun {
            controller: HysteresisControl::new(2.0, 1.0, state).unwrap(),
        }
    }

    pub fn run(&mut self) {
        self.controller.run(1000);
    }
}

struct Kettle {
    controller: u8,
}
