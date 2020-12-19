pub mod duty_cycle;
pub mod hysteresis;
pub mod manual;
pub mod pid;
pub mod pub_sub;

use crate::pub_sub::ClientId;
use serde::Deserialize;
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

#[non_exhaustive]
#[derive(Deserialize)]
pub enum ControllerType {
    #[serde(rename = "hysteresis")]
    Hysteresis { offset_on: f32, offset_off: f32 },
    #[serde(rename = "manual")]
    Manual,
}

#[derive(Deserialize)]
pub struct ControllerConfig {
    pub(crate) controller_id: ClientId,
    pub(crate) actor_id: ClientId,
    pub(crate) sensor_id: ClientId,
    type_: ControllerType,
}

impl ControllerConfig {
    pub fn client_ids(&self) -> impl Iterator<Item = &ClientId> {
        std::iter::once(&self.actor_id).chain(std::iter::once(&self.sensor_id))
    }

    pub fn get_controller(&self) -> Result<Box<dyn Control>, Error> {
        match self.type_ {
            ControllerType::Hysteresis {
                offset_on,
                offset_off,
            } => {
                let control = hysteresis::Controller::try_new(offset_on, offset_off)?;
                Ok(Box::new(control))
            }
            ControllerType::Manual { .. } => Ok(Box::new(manual::Controller::new())),
            _ => unimplemented!(),
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
