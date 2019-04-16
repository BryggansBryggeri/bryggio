use crate::hardware;
use gpio_cdev;

pub struct DummyActor {
    pub id: &'static str,
}

impl DummyActor {
    pub fn new(id: &'static str) -> DummyActor {
        DummyActor { id: id }
    }
}

impl Actor for DummyActor {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error> {
        Ok(())
    }
}

pub struct SimpleGpio {
    pub id: &'static str,
    handle: gpio_cdev::LineHandle,
}

impl SimpleGpio {
    pub fn new(id: &'static str, pin_number: Option<u32>) -> SimpleGpio {
        let pin_number = pin_number.unwrap();
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

impl Actor for SimpleGpio {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error> {
        let gpio_state = match signal {
            signal if signal > 0.0 => 1,
            signal if signal <= 0.0 => 0,
            _ => 0,
        };
        self.handle.set_value(gpio_state)
    }
}

pub struct XorActor {
    pub own_id: &'static str,
    pub paired_id: &'static str,
    handle: gpio_cdev::LineHandle,
}

impl XorActor {
    pub fn new(own_id: &'static str, paired_id: &'static str, pin_number: Option<u32>) -> XorActor {
        let pin_number = pin_number.unwrap();
        let handle = match hardware::get_gpio_handle("/dev/gpiochip0", pin_number, &own_id) {
            Ok(handle) => handle,
            Err(err) => {
                panic!("Could not get handle, {}.", err);
            }
        };
        XorActor {
            own_id: own_id,
            paired_id: paired_id,
            handle: handle,
        }
    }
}

impl Actor for XorActor {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error> {
        let gpio_state = match signal {
            signal if signal > 0.0 => 1,
            signal if signal <= 0.0 => 0,
            _ => 0,
        };
        self.handle.set_value(gpio_state)
    }
}

pub trait Actor {
    fn set_signal(&self, signal: f32) -> Result<(), gpio_cdev::errors::Error>;
}
