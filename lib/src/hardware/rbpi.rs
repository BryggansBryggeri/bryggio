use gpio_cdev::{errors::Error, Chip, LineRequestFlags};
use linux_embedded_hal::CdevPin;

pub fn get_gpio_pin(pin_number: u32, label: &str) -> Result<CdevPin, Error> {
    let mut chip = match Chip::new("/dev/gpiochip0") {
        Ok(chip) => chip,
        Err(e) => return Err(e),
    };
    let line = match chip.get_line(pin_number) {
        Ok(line) => line,
        Err(e) => return Err(e),
    };
    let handle = line.request(LineRequestFlags::OUTPUT, 0, label)?;
    CdevPin::new(handle)
}
