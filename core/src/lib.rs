#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

//! CableThermal-DLR Core (Community Edition)
//!
//! High-Precision Deterministic Thermal Rating Engine based on IEEE 738 / CIGRE TB 207.
//! This crate contains the AGPLv3 open-source implementation for steady-state
//! thermal heat balance equations. It is designed to be `#![no_std]` compatible
//! (with `libm`) for embedded usage, though currently uses `std` for simplicity.

use std::f64::consts::PI;
use thiserror::Error;
use tracing::{debug, instrument};

/// Core Error types for the Thermal Engine
#[derive(Error, Debug, PartialEq)]
pub enum ThermalError {
    /// Returned when physical parameters are impossible (e.g. negative diameter)
    #[error("Invalid conductor physical parameters: {0}")]
    InvalidParameters(&'static str),
    /// Returned when environmental factors exceed reasonable physical limits
    #[error("Extreme weather bounds exceeded: {0}")]
    ExtremeWeatherBounds(&'static str),
    /// Calculation resulted in an unphysical state (e.g. ambient temp > max conductor temp)
    #[error("Thermal equilibrium impossible: {0}")]
    EquilibriumImpossible(&'static str),
}

/// Physical properties of the overhead conductor
#[derive(Debug, Clone, Copy)]
pub struct Conductor {
    /// Diameter in meters (Must be > 0)
    pub diameter: f64,
    /// Electrical resistance in Ohms per meter at the evaluation temperature (Must be > 0)
    pub resistance_per_m: f64,
    /// Emissivity coefficient (0.0 to 1.0)
    pub emissivity: f64,
    /// Solar absorptivity coefficient (0.0 to 1.0)
    pub solar_absorptivity: f64,
}

impl Conductor {
    /// Validates the conductor parameters
    pub fn validate(&self) -> Result<(), ThermalError> {
        if self.diameter <= 0.0 {
            return Err(ThermalError::InvalidParameters("Diameter must be positive"));
        }
        if self.resistance_per_m <= 0.0 {
            return Err(ThermalError::InvalidParameters(
                "Resistance must be positive",
            ));
        }
        if !(0.0..=1.0).contains(&self.emissivity) {
            return Err(ThermalError::InvalidParameters(
                "Emissivity must be between 0.0 and 1.0",
            ));
        }
        if !(0.0..=1.0).contains(&self.solar_absorptivity) {
            return Err(ThermalError::InvalidParameters(
                "Absorptivity must be between 0.0 and 1.0",
            ));
        }
        Ok(())
    }
}

/// Real-time weather telemetry from SCADA or weather stations
#[derive(Debug, Clone, Copy)]
pub struct WeatherTelemetry {
    /// Ambient temperature in Celsius
    pub ambient_temp: f64,
    /// Wind speed in meters per second
    pub wind_speed: f64,
    /// Solar irradiance in Watts per square meter
    pub solar_irradiance: f64,
}

impl WeatherTelemetry {
    /// Validates weather telemetry inputs against Earth-normal bounds
    pub fn validate(&self) -> Result<(), ThermalError> {
        if self.ambient_temp < -100.0 || self.ambient_temp > 100.0 {
            return Err(ThermalError::ExtremeWeatherBounds(
                "Ambient temperature out of physical bounds",
            ));
        }
        if self.wind_speed < 0.0 || self.wind_speed > 150.0 {
            return Err(ThermalError::ExtremeWeatherBounds(
                "Wind speed out of physical bounds",
            ));
        }
        if self.solar_irradiance < 0.0 || self.solar_irradiance > 1500.0 {
            return Err(ThermalError::ExtremeWeatherBounds(
                "Solar irradiance out of physical bounds",
            ));
        }
        Ok(())
    }
}

/// Solves the dynamic heat balance equation (qc + qr = qs + I^2R)
/// to find the maximum allowable ampacity (I_max).
///
/// # Arguments
/// * `conductor` - The physical profile of the transmission line.
/// * `weather` - Instantaneous weather telemetry.
/// * `max_conductor_temp_c` - The safety boundary temperature in Celsius.
///
/// # Returns
/// The safe Ampacity in Amperes, or a `ThermalError` if parameters are invalid.
#[instrument(level = "debug", skip_all)]
pub fn calculate_steady_state_ampacity(
    conductor: &Conductor,
    weather: &WeatherTelemetry,
    max_conductor_temp_c: f64,
) -> Result<f64, ThermalError> {
    conductor.validate()?;
    weather.validate()?;

    if max_conductor_temp_c <= weather.ambient_temp {
        return Err(ThermalError::EquilibriumImpossible(
            "Max temp is below or equal to ambient temp.",
        ));
    }

    // 1. Solar Heat Gain (q_s) [W/m]
    let q_s = conductor.solar_absorptivity * conductor.diameter * weather.solar_irradiance;
    debug!("Solar Heat Gain (q_s): {} W/m", q_s);

    // 2. Radiated Heat Loss (q_r) [W/m]
    let stefan_boltzmann = 5.6697e-8;
    let t_c_kelvin = max_conductor_temp_c + 273.15;
    let t_a_kelvin = weather.ambient_temp + 273.15;
    let q_r = stefan_boltzmann
        * PI
        * conductor.diameter
        * conductor.emissivity
        * (t_c_kelvin.powi(4) - t_a_kelvin.powi(4));
    debug!("Radiated Heat Loss (q_r): {} W/m", q_r);

    // 3. Convected Heat Loss (q_c) [W/m]
    let density_air = 1.225;
    let dynamic_viscosity = 1.81e-5;
    let thermal_conductivity_air = 0.025;

    let reynolds = (density_air * weather.wind_speed * conductor.diameter) / dynamic_viscosity;
    let nusselt = 0.64 * reynolds.powf(0.2) + 0.2 * reynolds.powf(0.61);
    let q_c =
        nusselt * thermal_conductivity_air * (max_conductor_temp_c - weather.ambient_temp) * PI;
    debug!("Convected Heat Loss (q_c): {} W/m", q_c);

    // 4. Heat Balance: I^2 * R = q_c + q_r - q_s
    let i_squared_r = q_c + q_r - q_s;

    if i_squared_r <= 0.0 {
        return Ok(0.0); // Ambient heating already exceeds max_temp limits
    }

    let ampacity = (i_squared_r / conductor.resistance_per_m).sqrt();
    debug!("Calculated Ampacity: {} A", ampacity);

    Ok(ampacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ampacity() {
        let conductor = Conductor {
            diameter: 0.02814,
            resistance_per_m: 7.28e-5,
            emissivity: 0.5,
            solar_absorptivity: 0.5,
        };

        let weather = WeatherTelemetry {
            ambient_temp: 40.0,
            wind_speed: 0.61,
            solar_irradiance: 1000.0,
        };

        let ampacity = calculate_steady_state_ampacity(&conductor, &weather, 100.0).unwrap();
        assert!(
            ampacity > 500.0 && ampacity < 1500.0,
            "Ampacity out of bounds: {}",
            ampacity
        );
    }

    #[test]
    fn test_invalid_equilibrium() {
        let conductor = Conductor {
            diameter: 0.02814,
            resistance_per_m: 7.28e-5,
            emissivity: 0.5,
            solar_absorptivity: 0.5,
        };
        let weather = WeatherTelemetry {
            ambient_temp: 40.0,
            wind_speed: 1.0,
            solar_irradiance: 500.0,
        };

        let result = calculate_steady_state_ampacity(&conductor, &weather, 35.0);
        assert_eq!(
            result,
            Err(ThermalError::EquilibriumImpossible(
                "Max temp is below or equal to ambient temp."
            ))
        );
    }
}
