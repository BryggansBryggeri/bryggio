use embedded_hal::digital::v2::OutputPin;
use gpio_cdev::errors;

pub fn get_gpio_pin(pin_number: u32, label: &str) -> Result<GpioPin, errors::Error> {
    Ok(GpioPin::new(pin_number, label))
}

pub struct GpioPin {
    pub pin_number: u32,
    pub label: String,
    state: GpioState,
}

impl GpioPin {
    pub fn new(pin_number: u32, label: &str) -> Self {
        GpioPin {
            pin_number,
            label: label.into(),
            state: GpioState::Low,
        }
    }
}

impl OutputPin for GpioPin {
    type Error = gpio_cdev::errors::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state = GpioState::Low;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state = GpioState::High;
        Ok(())
    }
}

enum GpioState {
    Low,
    High,
}
