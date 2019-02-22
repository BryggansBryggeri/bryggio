use gpio_cdev::{Chip, LineRequestFlags};
use std::{thread, time};

use crate::hardware::set_gpio;

pub struct HysteresisControl {
    pub state: bool,
    pub offset: f32
}

impl HysteresisControl {
    pub fn run(&self) {
        let pin_number = 21;
        let label = "rustbeer";
        let time_wait = time::Duration::from_millis(1000);
        let now = time::Instant::now();
        let mut chip = Chip::new("/dev/gpiochip0").unwrap();
        let handle = chip.get_line(pin_number).unwrap();
        for i in 1..10 {
            println!("Iter: {}", i);
            handle.request(LineRequestFlags::OUTPUT, 1, label)
                .unwrap();
            thread::sleep(time_wait);
        }
    }
}

pub trait Control {
}
