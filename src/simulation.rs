use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::domain::DeviceName;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum DeviceType {
    SolarPanel,
    StableDevice {
        produces: i32,
    },
    Store {
        max_charge_per_tick: u32,
        max_capacity: u32,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    name: DeviceName,
    #[serde(flatten)]
    device_type: DeviceType,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    start_time: Timestamp,
    end_time: Timestamp,
    devices: Vec<Device>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_sample_config_works() {
        let _: Simulation =
            serde_json::from_str(include_str!("../res/example_simulation.json")).unwrap();
    }
}
