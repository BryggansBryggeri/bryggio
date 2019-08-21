use crate::actor;
use crate::hardware;
use gpio_cdev;

pub struct SimpleGpio {
    pub id: &'static str,
    handle: gpio_cdev::LineHandle,
}

impl SimpleGpio {
    pub fn new(id: &'static str, pin_number: u32) -> SimpleGpio {
        let pin_number = pin_number;
        let handle = match hardware::get_gpio_handle("/dev/gpiochip0", pin_number, &id) {
            Ok(handle) => handle,
            Err(err) => {
                panic!("Could not get handle, {}.", err);
            }
        };
        SimpleGpio { id, handle }
    }
}

impl actor::Actor for SimpleGpio {
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
