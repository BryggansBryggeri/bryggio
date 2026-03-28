//! The physical brewery model — concrete, not generic.
use crate::{hal::ActorOutputs, sensor::TempSensor};
use std::f64::consts::PI;

/// Top-level brewery definition.
pub struct Brewery {
    pub vessel: Vessel,
}

impl Default for Brewery {
    fn default() -> Self {
        Self::new()
    }
}

impl Brewery {
    pub fn new() -> Self {
        let vessel = Vessel {
            top_temp_sensor: TempSensor::Ds18b20 {
                address: String::from("test_top"),
            },
            bottom_temp_sensor: TempSensor::Ds18b20 {
                address: String::from("test_bottom"),
            },
        };
        Self { vessel }
    }
}

/// The single vessel (mash tun + boil kettle with basket insert).
pub struct Vessel {
    pub top_temp_sensor: TempSensor,
    pub bottom_temp_sensor: TempSensor,
}

#[derive(Debug, Clone, Copy)]
pub struct BrewerySimulation {
    // Two-zone model: bottom (near heater) and top.
    // Each zone holds half the volume.
    temp_top: f32,
    temp_bottom: f32,
    ambient_temp: f32,

    heater_power: f32, // [0, 1]
    pump_on: bool,

    water_mass: f32,
    vessel: VesselParams,
}

impl Default for BrewerySimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl BrewerySimulation {
    pub fn new() -> Self {
        let ambient = 20.0;
        let vessel = VesselParams::default();
        Self {
            temp_top: ambient,
            temp_bottom: ambient,
            heater_power: 0.0,
            pump_on: false,
            ambient_temp: ambient,
            water_mass: vessel.volume_liters_by_height(vessel.height * 0.7) * WATER_DENSITY,
            vessel,
        }
    }

    /// Advance the physics simulation by `dt` seconds.
    pub fn update_sensors(self, dt: f32) -> Self {
        if dt <= 0.0 {
            return self;
        }
        let zone_mass = self.water_mass / 2.0;
        let zone_mcp = zone_mass * CP_WATER;

        // --- Heating: all heater energy goes into bottom zone ---
        let heating = self.vessel.heater_watts * self.heater_power / zone_mcp;

        // Ambient losses per zone
        // Note: that the rate is constant if the zones are divided symmetrically,
        // i.e., if you halve both the water mass and surface area, then those effects cancel.
        let loss_bottom = self.thermal_relaxation_rate() * (self.temp_bottom - self.ambient_temp);
        let loss_top = self.thermal_relaxation_rate() * (self.temp_top - self.ambient_temp);

        // --- Inter-zone heat transfer ---
        let coupling = if self.pump_on {
            ZONE_CONDUCTION + ZONE_PUMP_MIXING
        } else {
            ZONE_CONDUCTION
        };
        // Heat flow from bottom to top (positive when bottom is hotter).
        let zone_transfer = coupling * (self.temp_bottom - self.temp_top) / zone_mcp;

        // --- Integrate (forward Euler) ---
        let temp_top = self.temp_top + (-loss_top + zone_transfer) * dt;
        let temp_bottom = self.temp_bottom + (heating - loss_bottom - zone_transfer) * dt;
        Self {
            temp_top,
            temp_bottom,
            ..self
        }
    }

    pub fn update_actors(self, outputs: &ActorOutputs) -> Self {
        Self {
            heater_power: outputs.heater_power.value(),
            pump_on: outputs.pump_on,
            ..self
        }
    }

    pub fn temp(&self) -> (f32, f32) {
        (self.temp_top, self.temp_bottom)
    }

    /// Estimate thermal relaxation rate
    fn thermal_relaxation_rate(&self) -> f32 {
        self.vessel.heat_loss_coeff * self.vessel.total_surface_area_m2()
            / (self.water_mass * CP_WATER)
    }
}

/// Vessel geometry and insulation parameters.
#[derive(Debug, Clone, Copy)]
pub struct VesselParams {
    /// radius in m
    radius: f32,
    /// height in m
    height: f32,
    heater_watts: f32,
    /// Overall heat loss coefficient in W/(m²·K).
    /// Typical: ~5 for well-insulated, ~10-15 for bare stainless.
    heat_loss_coeff: f32,
}

impl VesselParams {
    /// Compute volume as a function of liquid height
    pub fn volume_liters_by_height(&self, height: f32) -> f32 {
        self.total_volume_liters() * height / self.height
    }

    /// Compute total volume of the vessel
    pub fn total_volume_liters(&self) -> f32 {
        let volume_m3 = self.surface_area_circle() * self.height;
        volume_m3 * 1000.0
    }

    pub fn total_surface_area_m2(&self) -> f32 {
        self.surface_area_cylinder_mantle() + 2.0 * self.surface_area_circle()
    }

    /// Surface area of circular parts
    ///
    /// The vessel is roughly cylindrical, this computes the area of either the top or the bottom.
    /// I.e., it is _not_ the joint area of all circular parts.
    fn surface_area_circle(&self) -> f32 {
        #![allow(clippy::as_conversions)] // PI fits into an f32 with acceptable precision.
        self.radius * self.radius * PI as f32
    }

    /// Surface area of cylinder mantle
    ///
    /// The vessel is roughly cylindrical, this computes the area of either the top or the bottom.
    /// I.e., it is _not_ the joint area of all circular parts.
    fn surface_area_cylinder_mantle(&self) -> f32 {
        #![allow(clippy::as_conversions)] // PI fits into an f32 with acceptable precision.
        let circumference = 2.0 * self.radius * PI as f32;
        circumference * self.height
    }
}

impl Default for VesselParams {
    /// Reasonable defaults for a ~50L cylindrical homebrew vessel.
    /// Roughly 40cm diameter, 50cm tall, modest insulation.
    fn default() -> Self {
        VesselParams {
            height: 0.5,
            radius: 0.2,
            heater_watts: 3000.0,
            heat_loss_coeff: 8.0,
        }
    }
}

/// Physical constants for water.
const CP_WATER: f32 = 4186.0; // J/(kg·K)
const WATER_DENSITY: f32 = 1.0; // kg/L  (good enough for brewing temps)

/// Inter-zone conduction coefficient in W/K.
/// Models passive heat transfer between bottom and top halves.
const ZONE_CONDUCTION: f32 = 50.0;

/// When the pump is on, convective mixing is much stronger.
/// This is an effective conductance in W/K for the pump-driven mixing.
const ZONE_PUMP_MIXING: f32 = 2000.0;
