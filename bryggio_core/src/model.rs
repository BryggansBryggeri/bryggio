//! The physical brewery model — concrete, not generic.
use crate::sensor::TempSensor;

/// Top-level brewery definition.
pub struct Brewery {
    pub vessel: Vessel,
}

/// The single vessel (mash tun + boil kettle with basket insert).
pub struct Vessel {
    pub top_temp_sensor: TempSensor,
    pub bottom_temp_sensor: TempSensor,
}
