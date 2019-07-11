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
        SimpleGpio {
            id: id,
            handle: handle,
        }
    }
}

impl actor::Actor for SimpleGpio {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error> {
        let gpio_state = match signal {
            signal if signal > 0.0 => 1,
            signal if signal <= 0.0 => 0,
            _ => 0,
        };
        self.handle.set_value(gpio_state)
    }
}
