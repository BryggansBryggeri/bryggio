use crate::actor;

pub struct Actor {
    pub id: String,
}

impl Actor {
    pub fn new(id: &str) -> Actor {
        Actor { id: id.into() }
    }
}

impl actor::Actor for Actor {
    fn validate_signal(&self, _signal: f32) -> Result<(), actor::Error> {
        Ok(())
    }
    fn set_signal(&self, _signal: f32) -> Result<(), actor::Error> {
        Ok(())
    }
}
