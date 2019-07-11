use crate::actor;
use gpio_cdev;

pub struct Actor {
    pub id: &'static str,
}

impl Actor {
    pub fn new(id: &'static str) -> Actor {
        Actor { id: id }
    }
}

impl actor::Actor for Actor {
    fn set_signal(&self, _signal: f32) -> Result<(), gpio_cdev::errors::Error> {
        Ok(())
    }
}
