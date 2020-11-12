pub mod duty_cycle;
pub mod hysteresis;
pub mod manual;
pub mod pid;
pub mod pub_sub;

use std::convert::TryFrom;
use std::error as std_error;
use std::f32;

pub trait Control: Send {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn get_state(&self) -> State;
    fn set_state(&mut self, new_state: State);
    fn get_control_signal(&self) -> f32;
    fn get_target(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Inactive,
    Active,
}

pub enum ControllerType {
    Hysteresis,
    Manual,
}

impl TryFrom<String> for ControllerType {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Error> {
        match string.to_ascii_lowercase().as_ref() {
            "hysteresis" => Ok(ControllerType::Hysteresis),
            "manual" => Ok(ControllerType::Manual),
            _ => Err(Error::ConversionError(string.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ParamError(String),
    ConversionError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ParamError(param) => write!(f, "Invalid param: {}", param),
            Error::ConversionError(type_string) => {
                write!(f, "Unable to parse '{}' to ControllerType", type_string)
            }
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParamError(_) => "Invalid param",
            Error::ConversionError(_) => "Conversion error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
