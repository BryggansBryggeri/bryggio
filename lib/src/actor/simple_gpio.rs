use crate::actor;
use crate::hardware;
use gpio_cdev;

pub struct Actor {
    pub id: String,
    handle: gpio_cdev::LineHandle,
}

impl Actor {
    pub fn new(id: &str, pin_number: u32) -> Result<Actor, actor::Error> {
        let pin_number = pin_number;
        match hardware::get_gpio_handle("/dev/gpiochip0", pin_number, &id) {
            Ok(handle) => Ok(Actor {
                id: id.into(),
                handle,
            }),
            Err(err) => Err(actor::Error::ActorError(err.to_string())),
        }
    }
}

impl actor::Actor for Actor {
    fn validate_signal(&self, _signal: f32) -> Result<(), actor::Error> {
        Ok(())
    }

    fn set_signal(&self, signal: f32) -> Result<(), actor::Error> {
        self.validate_signal(signal)?;
        let gpio_state = match signal {
            signal if signal > 0.0 => 1,
            signal if signal <= 0.0 => 0,
            _ => 0,
        };
        match self.handle.set_value(gpio_state) {
            Ok(()) => Ok(()),
            Err(err) => Err(actor::Error::ActorError(err.to_string())),
        }
    }
}
