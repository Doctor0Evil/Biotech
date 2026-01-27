use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::{EcoBand, EcoBandProfile, LifeforceBand, LifeforceBandSeries};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerHostCapacityEnvelope {
    pub host_id: String,
    pub brain_min: f64,
    pub brain_max: f64,
    pub wave_min: f64,
    pub wave_max: f64,
    pub blood_min: f64,
    pub blood_max: f64,
    pub oxygen_min: f64,
    pub oxygen_max: f64,
    pub nano_min: f64,
    pub nano_max: f64,
    pub smart_min: f64,
    pub smart_max: f64,
    /// Safe fractions relative to max, for procedures.
    pub max_wave_fraction: f64,
    pub max_nano_fraction: f64,
    pub max_smart_fraction: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemAdjustment {
    pub ts_utc: DateTime<Utc>,
    pub reason: String,
    pub delta_brain: f64,
    pub delta_wave: f64,
    pub delta_blood: f64,
    pub delta_oxygen: f64,
    pub delta_nano: f64,
    pub delta_smart: f64,
    pub eco_cost_flops: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NanoRouteDecision {
    Safe,
    Defer,
    Deny,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoLifebandRouter {
    pub host_id: String,
}

impl NanoLifebandRouter {
    pub fn classify(
        &self,
        lifeband: &LifeforceBand,
        clarity: f32,
        eco: &EcoBand,
    ) -> NanoRouteDecision {
        match lifeband {
            LifeforceBand::HardStop => NanoRouteDecision::Deny,
            LifeforceBand::SoftWarn => {
                if clarity < 0.5 || matches!(eco, EcoBand::High) {
                    NanoRouteDecision::Defer
                } else {
                    NanoRouteDecision::Safe
                }
            }
            LifeforceBand::Safe => {
                if clarity < 0.3 || matches!(eco, EcoBand::High) {
                    NanoRouteDecision::Defer
                } else {
                    NanoRouteDecision::Safe
                }
            }
        }
    }
}
