use crate::hardware::HardwareError;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::{InputPin, OutputPin};
use gpio_cdev::errors::Error as CdevError;

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
    type Error = CdevError;

    fn try_is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.state.into())
    }

    fn try_is_low(&self) -> Result<bool, Self::Error> {
        let bool_state: bool = self.state.into();
        Ok(!bool_state)
    }
}

impl OutputPin for GpioPin {
    //TODO: better error
    type Error = CdevError;

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        self.state = GpioState::Low;
        Ok(())
    }

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        self.state = GpioState::High;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum GpioState {
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

impl DelayMs<u16> for Delay {
    type Error = CdevError;
    fn try_delay_ms(&mut self, _ms: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl DelayUs<u16> for Delay {
    type Error = CdevError;
    fn try_delay_us(&mut self, _ms: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}
