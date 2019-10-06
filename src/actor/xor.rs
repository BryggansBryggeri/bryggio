use crate::actor;
use crate::hardware;
use gpio_cdev;

pub struct XOr {
    pub own_id: String,
    pub paired_id: String,
    handle: gpio_cdev::LineHandle,
}

impl XOr {
    pub fn new(own_id: &str, paired_id: &str, pin_number: Option<u32>) -> XOr {
        let pin_number = pin_number.unwrap();
        let handle = match hardware::get_gpio_handle("/dev/gpiochip0", pin_number, &own_id) {
            Ok(handle) => handle,
            Err(err) => {
                panic!("Could not get handle, {}.", err);
            }
        };
        XOr {
            own_id: own_id.into(),
            paired_id: paired_id.into(),
            handle,
        }
    }
}

impl actor::Actor for XOr {
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
