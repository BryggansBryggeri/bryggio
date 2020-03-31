use gpio_cdev::{errors, Chip, LineHandle, LineRequestFlags};

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
