use crate::actor;
use embedded_hal::digital::v2::OutputPin;

pub struct Actor<T: OutputPin + Send> {
    pub id: String,
    handle: T,
}

impl<T: OutputPin + Send> Actor<T> {
    pub fn try_new(id: &str, handle: T) -> Result<Actor<T>, actor::ActorError> {
        Ok(Actor {
            id: id.into(),
            handle,
        })
    }
}

impl<T: OutputPin + Send> actor::Actor for Actor<T> {
    fn validate_signal(&self, signal: f32) -> Result<(), actor::ActorError> {
        if signal >= 0.0 {
            Ok(())
        } else {
            Err(actor::ActorError::InvalidSignal {
                signal,
                lower_bound: 0.0,
                upper_bound: 1.0,
            })
        }
    }

    fn set_signal(&mut self, signal: f32) -> Result<(), actor::ActorError> {
        self.validate_signal(signal)?;
        let outcome = if signal > 0.0 {
            self.handle.set_high()
        } else {
            self.handle.set_low()
        };
        match outcome {
            Ok(()) => Ok(()),
            Err(_err) => Err(actor::ActorError::ActorError("TODO: GPIO error".into())),
        }
    }
}
