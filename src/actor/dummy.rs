use crate::actor;

pub struct Actor {
    pub id: &'static str,
}

impl Actor {
    pub fn new(id: &'static str) -> Actor {
        Actor { id: id }
    }
}

impl actor::Actor for Actor {
    fn validate_signal(&self, _signal: &f32) -> Result<(), actor::Error> {
        Ok(())
    }
    fn set_signal(&self, _signal: &f32) -> Result<(), actor::Error> {
        Ok(())
    }
}
