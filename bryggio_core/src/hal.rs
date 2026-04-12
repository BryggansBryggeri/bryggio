//! Hardware abstraction layer trait.
//!
//! The tick loop is generic over this trait. Implementations live in bryggio_server.
use crate::{sensor::SensorReadings, types::Power};
use thiserror::Error;

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
    fn read_sensors(&self) -> impl Future<Output = SensorReadings> + Send;

    fn apply_outputs(
        &self,
        outputs: &ActorOutputs,
    ) -> impl Future<Output = Result<(), HalError>> + Send;

    /// Current time as unix epoch seconds.
    ///
    /// Real HALs return wall-clock time. The mock HAL returns virtual time,
    /// allowing the simulation to run faster than real time.
    fn now(&self) -> u64;

    /// How long the tick loop should sleep between ticks.
    ///
    /// Real HALs return 1s. The mock HAL returns a shorter duration
    /// to speed up simulation without changing the physics dt.
    fn tick_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }
}
