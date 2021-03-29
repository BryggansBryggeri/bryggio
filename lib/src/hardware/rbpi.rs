use crate::hardware::HardwareError;
use gpio_cdev::{errors::Error, Chip, LineRequestFlags};
use linux_embedded_hal::CdevPin;

pub type GpioPin = CdevPin;

pub fn get_gpio_pin(pin_number: u32, label: &str) -> Result<CdevPin, HardwareError> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let line = chip.get_line(pin_number)?;
    let handle = line.request(LineRequestFlags::OUTPUT, 0, label)?;
    CdevPin::new(handle).map_err(HardwareError::from)
}
