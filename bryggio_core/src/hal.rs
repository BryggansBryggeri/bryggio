//! Hardware abstraction layer trait.
//!
//! The tick loop is generic over this trait. Implementations live in bryggio_server.
use crate::types::{Power, Temperature};
use thiserror::Error;

/// Sensor readings returned by the HAL on each tick.
#[derive(Debug, Clone, Default)]
pub struct SensorReadings {
    pub vessel_temp_top: Option<Temperature>,
    pub vessel_temp_bottom: Option<Temperature>,
}

/// Actor outputs to apply to hardware.
#[derive(Debug, Clone)]
pub struct ActorOutputs {
    pub heater_power: Power,
    pub pump_on: bool,
}

#[derive(Error, Debug)]
pub enum HalError {
    #[error("Sensor read failed: {0}")]
    SensorRead(String),
    #[error("Actor write failed: {0}")]
    ActorWrite(String),
}

/// Hardware abstraction — implemented by RpiHal (real) and MockHal (simulated).
///
/// Uses static dispatch via generics, not trait objects.
pub trait Hal: Send + Sync + 'static {
    fn read_sensors(
        &self,
    ) -> impl std::future::Future<Output = SensorReadings> + Send;

    fn apply_outputs(
        &self,
        outputs: &ActorOutputs,
    ) -> impl std::future::Future<Output = Result<(), HalError>> + Send;
}
