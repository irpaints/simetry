pub use racing_flags::RacingFlags;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::select;
use uom::si::f64::{AngularVelocity, Velocity};

pub mod assetto_corsa;
pub mod assetto_corsa_competizione;
pub mod dirt_rally_2;
pub mod generic_http;
pub mod iracing;
mod racing_flags;
pub mod rfactor_2;
pub mod truck_simulator;
mod windows_util;

/// Explicitly marks each hardcoded value in the code that is not handled by the specific sim.
#[inline]
fn unhandled<T>(value: T) -> T {
    value
}

/// Sim that we can connect to via the common [`connect`] function.
#[async_trait::async_trait]
pub trait Simetry {
    /// Name of the sim we are connected to.
    fn name(&self) -> &str;

    /// Waits for the next reading of data from the sim and returns it.
    ///
    /// A `None` value means that the connection is done, similar to an iterator.
    async fn next_moment(&mut self) -> Option<Box<dyn Moment>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimetryConnectionBuilder {
    pub generic_http_uri: String,
    pub truck_simulator_uri: String,
    pub dirt_rally_2_uri: String,
    pub retry_delay: Duration,
}

impl Default for SimetryConnectionBuilder {
    fn default() -> Self {
        Self {
            generic_http_uri: generic_http::DEFAULT_URI.to_string(),
            truck_simulator_uri: truck_simulator::DEFAULT_URI.to_string(),
            dirt_rally_2_uri: dirt_rally_2::Client::DEFAULT_URI.to_string(),
            retry_delay: Duration::from_secs(5),
        }
    }
}

impl SimetryConnectionBuilder {
    pub async fn connect(self) -> Box<dyn Simetry> {
        let retry_delay = self.retry_delay;
        let iracing_future = iracing::Client::connect(retry_delay);
        let assetto_corsa_future = assetto_corsa::Client::connect(retry_delay);
        let assetto_corsa_competizione_future =
            assetto_corsa_competizione::Client::connect(retry_delay);
        let rfactor_2_future = rfactor_2::Client::connect();
        let dirt_rally_2_future =
            dirt_rally_2::Client::connect(&self.dirt_rally_2_uri, retry_delay);
        let generic_http_future =
            generic_http::GenericHttpClient::connect(&self.generic_http_uri, retry_delay);
        let truck_simulator_future =
            truck_simulator::TruckSimulatorClient::connect(&self.truck_simulator_uri, retry_delay);

        select! {
            x = iracing_future => Box::new(x),
            x = assetto_corsa_future => Box::new(x),
            x = assetto_corsa_competizione_future => Box::new(x),
            x = rfactor_2_future => Box::new(x),
            x = dirt_rally_2_future => Box::new(x),
            x = generic_http_future => Box::new(x),
            x = truck_simulator_future => Box::new(x),
        }
    }
}

/// Connect to any running sim that is supported.
#[inline]
pub async fn connect() -> Box<dyn Simetry> {
    SimetryConnectionBuilder::default().connect().await
}

// TODO: make interface where every value is an option
/// Generic support for any sim by providing processed data for most common data-points.
///
/// If a sim does not support certain data, a suitable default value is used.
/// The documentation of every method explains why certain defaults are chosen.
pub trait Moment {
    /// Check if there is a vehicle to the left of the driver.
    ///
    /// If not supported by the sim, always returns `false`.
    fn vehicle_left(&self) -> bool {
        false
    }

    /// Check if there is a vehicle to the right of the driver.
    ///
    /// If not supported by the sim, always returns `false`.
    fn vehicle_right(&self) -> bool {
        false
    }

    fn basic_telemetry(&self) -> Option<BasicTelemetry> {
        None
    }

    fn shift_point(&self) -> Option<AngularVelocity> {
        None
    }

    fn flags(&self) -> RacingFlags {
        RacingFlags::default()
    }

    /// ID that uniquely identifies the current vehicle make and model.
    ///
    /// If you want to provide behavior for a specific vehicle make and model,
    /// this property is the right choice.
    fn vehicle_unique_id(&self) -> Option<String> {
        None
    }

    /// Check if the ignition is on.
    ///
    /// If not supported by the sim, always returns `true`.
    fn ignition_on(&self) -> bool {
        true
    }

    /// Check if the starter motor is engaged.
    ///
    /// If not supported by the sim, always returns `false`.
    fn starter_on(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BasicTelemetry {
    pub gear: i8,
    pub speed: Velocity,
    pub engine_rotation_speed: AngularVelocity,
    pub max_engine_rotation_speed: AngularVelocity,
    pub pit_limiter_engaged: bool,
    pub in_pit_lane: bool,
}
