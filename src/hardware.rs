use gpio_cdev::{Chip, LineRequestFlags};
use std::{thread, time};



// Read the state of GPIO4 on a raspberry pi.  /dev/gpiochip0
// maps to the driver for the SoC (builtin) GPIO controller.
pub fn set_gpio(pin_number: u32, state: u8, label: &str) {
    let time_wait = time::Duration::from_millis(1000);
    let now = time::Instant::now();
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let handle = chip
        .get_line(pin_number).unwrap()
        .request(LineRequestFlags::OUTPUT, 1, label).unwrap();
    thread::sleep(time_wait);
}
