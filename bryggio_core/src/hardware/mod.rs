use thiserror::Error;

#[cfg(target_arch = "x86_64")]
pub(crate) mod dummy;
#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
pub(crate) mod rbpi;
#[cfg(all(target_arch = "arm", target_os = "linux"))]
pub(crate) mod rbpi;

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("Generic GPIO error {0}")]
    GenericGpio(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
