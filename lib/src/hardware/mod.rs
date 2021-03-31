use thiserror::Error;

#[cfg(target_arch = "x86_64")]
pub(crate) mod dummy;
#[cfg(target_arch = "arm")]
pub(crate) mod rbpi;
use gpio_cdev::errors::Error as CdevError;

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("GPIO error {0}")]
    Gpio(#[from] CdevError),
    #[error("Generic GPIO error {0}")]
    GenericGpio(String),
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
