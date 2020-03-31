use crate::actor;
use embedded_hal::digital::v2::OutputPin;

pub struct Actor<T: OutputPin + Send> {
    pub id: String,
    handle: T,
}

impl<T: OutputPin + Send> Actor<T> {
    pub fn new(id: &str, handle: T) -> Result<Actor<T>, actor::Error> {
        Ok(Actor {
            id: id.into(),
            handle,
        })
    }
}

impl<T: OutputPin + Send> actor::Actor for Actor<T> {
    fn validate_signal(&self, _signal: f32) -> Result<(), actor::Error> {
        Ok(())
    }

    fn set_signal(&mut self, signal: f32) -> Result<(), actor::Error> {
        self.validate_signal(signal)?;
        let outcome = if signal > 0.0 {
            self.handle.set_high()
        } else {
            self.handle.set_low()
        };
        match outcome {
            Ok(()) => Ok(()),
            Err(_err) => Err(actor::Error::ActorError("TODO: GPIO error".into())),
        }
    }
}
