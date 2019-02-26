use gpio_cdev::{errors, Chip, LineHandle, LineRequestFlags};
use std::{thread, time};

pub fn set_gpio(pin_number: u32, state: u8, label: &str) {
    let time_wait = time::Duration::from_millis(1000);
    let now = time::Instant::now();
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let handle = chip
        .get_line(pin_number)
        .unwrap()
        .request(LineRequestFlags::OUTPUT, 1, label)
        .unwrap();
    thread::sleep(time_wait);
}

pub fn get_gpio_handle(
    chip_id: &str,
    pin_number: u32,
    label: &str,
) -> Result<LineHandle, errors::Error> {
    let mut chip = match Chip::new(chip_id) {
        Ok(chip) => chip,
        Err(e) => return Err(e),
    };
    let line = match chip.get_line(pin_number) {
        Ok(line) => line,
        Err(e) => return Err(e),
    };
    line.request(LineRequestFlags::OUTPUT, 0, label)
}
