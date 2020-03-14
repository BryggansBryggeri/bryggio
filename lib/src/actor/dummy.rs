use crate::actor;

pub struct Actor {
    pub id: String,
    pub signal: f32,
}

impl Actor {
    pub fn new(id: &str) -> Actor {
        Actor {
            id: id.into(),
            signal: 0.0,
        }
    }
}

impl actor::Actor for Actor {
    fn validate_signal(&self, _signal: f32) -> Result<(), actor::Error> {
        Ok(())
    }
    fn set_signal(&mut self, signal: f32) -> Result<(), actor::Error> {
        self.signal = signal;
        Ok(())
    }
}
