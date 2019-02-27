use crate::hardware;
use gpio_cdev;

pub struct DummyActor {
    pub id: &'static str,
}

// TODO: String to something like a'<static>&str
impl Actor for DummyActor {
    fn new(id: &'static str, _: Option<u32>) -> DummyActor {
        DummyActor { id: id }
    }
    fn set_power(&self, power: f32) -> Result<(), gpio_cdev::errors::Error> {
        Ok(())
    }
}

pub struct SimpleGpio {
    pub id: &'static str,
    handle: gpio_cdev::LineHandle,
}

impl Actor for SimpleGpio {
    fn new(id: &'static str, pin_number: Option<u32>) -> SimpleGpio {
        let pin_number = pin_number.unwrap();
        let handle = match hardware::get_gpio_handle("/dev/gpiochip0", pin_number, &id) {
            Ok(handle) => handle,
            Err(e) => {
                panic!("Could not get handle, {}.", e);
            }
        };
        SimpleGpio {
            id: id,
            handle: handle,
        }
    }
    fn set_power(&self, power: f32) -> Result<(), gpio_cdev::errors::Error> {
        let gpio_state = match power {
            power if power > 0.0 => 1,
            power if power <= 0.0 => 0,
            _ => 0,
        };
        self.handle.set_value(gpio_state)
    }
}

pub trait Actor {
    fn new(id: &'static str, pin_number: Option<u32>) -> Self;
    fn set_power(&self, power: f32) -> Result<(), gpio_cdev::errors::Error>;
}
