use crate::pub_sub::ClientId;
use serde::{Deserialize, Serialize};
use std::f32;
use thiserror::Error;

pub mod hysteresis;
pub mod manual;
pub mod pid;
pub mod pub_sub;
pub use pub_sub::ControllerClient;

pub trait Control: Send {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn get_state(&self) -> State;
    fn set_state(&mut self, new_state: State);
    fn get_control_signal(&self) -> f32;
    fn get_target(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
    fn validate_target(&self, new_target: f32) -> Result<f32, ControllerError>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Inactive,
    Active,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerType {
    #[serde(rename = "hysteresis")]
    Hysteresis { offset_on: f32, offset_off: f32 },
    #[serde(rename = "pid")]
    Pid { kp: f32, ki: f32, kd: f32 },
    #[serde(rename = "manual")]
    Manual,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControllerConfig {
    pub controller_id: ClientId,
    pub(crate) actor_id: ClientId,
    pub(crate) sensor_id: ClientId,
    #[serde(rename = "type")]
    pub type_: ControllerType,
}

impl ControllerConfig {
    pub fn dummy() -> Self {
        ControllerConfig {
            controller_id: ClientId("controller".into()),
            actor_id: ClientId("dummy_actor".into()),
            sensor_id: ClientId("dummy_sensor".into()),
            type_: ControllerType::Manual,
        }
    }
    pub fn client_ids(&self) -> impl Iterator<Item = &ClientId> {
        std::iter::once(&self.actor_id).chain(std::iter::once(&self.sensor_id))
    }

    pub fn get_controller(&self, target: f32) -> Result<Box<dyn Control>, ControllerError> {
        match self.type_ {
            ControllerType::Hysteresis {
                offset_on,
                offset_off,
            } => {
                let control = hysteresis::Controller::try_new(target, offset_on, offset_off)?;
                Ok(Box::new(control))
            }
            ControllerType::Pid { kp, ki, kd } => {
                let control = pid::Controller::new(target, kp, ki, kd, None, None, None);
                Ok(Box::new(control))
            }
            ControllerType::Manual { .. } => Ok(Box::new(manual::ManualController::new(target))),
            //_ => unimplemented!(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ControllerError {
    #[error("Param. error: {0}")]
    ParamError(String),
    #[error("Invalid target '{0}': {1}")]
    InvalidTarget(f32, String),
    #[error("Unknown type: {0}")]
    Type(String),
}
