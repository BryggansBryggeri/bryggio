use crate::hardware::HardwareError;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorKind, ErrorType};
use embedded_hal::digital::{InputPin, OutputPin};

pub fn get_gpio_pin(pin_number: u32, label: &str) -> Result<GpioPin, HardwareError> {
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

impl InputPin for GpioPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state.into())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let bool_state: bool = self.state.into();
        Ok(!bool_state)
    }
}

#[derive(Debug)]
pub struct GpioPinError {}
impl ErrorType for GpioPin {
    type Error = GpioPinError;
}

impl embedded_hal::digital::Error for GpioPinError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl OutputPin for GpioPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state = GpioState::Low;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state = GpioState::High;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpioState {
    Low,
    High,
}

impl From<GpioState> for bool {
    fn from(state: GpioState) -> Self {
        match state {
            GpioState::High => true,
            GpioState::Low => false,
        }
    }
}

pub struct Delay {}

impl DelayNs for Delay {
    fn delay_ns(&mut self, _ns: u32) {}
}
