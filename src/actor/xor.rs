use crate::actor;
use crate::hardware;
use gpio_cdev;

pub struct XOr {
    pub own_id: &'static str,
    pub paired_id: &'static str,
    handle: gpio_cdev::LineHandle,
}

impl XOr {
    pub fn new(own_id: &'static str, paired_id: &'static str, pin_number: Option<u32>) -> XOr {
        let pin_number = pin_number.unwrap();
        let handle = match hardware::get_gpio_handle("/dev/gpiochip0", pin_number, &own_id) {
            Ok(handle) => handle,
            Err(err) => {
                panic!("Could not get handle, {}.", err);
            }
        };
        XOr {
            own_id: own_id,
            paired_id: paired_id,
            handle: handle,
        }
    }
}

impl actor::Actor for XOr {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error> {
        let gpio_state = match signal {
            signal if signal > 0.0 => 1,
            signal if signal <= 0.0 => 0,
            _ => 0,
        };
        self.handle.set_value(gpio_state)
    }
}
