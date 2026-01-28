use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::{EcoBand, EcoBandProfile, LifeforceBand, LifeforceBandSeries};
use crate::lifeforce_rads::enforce_rads_evolve_scale_guards;

impl InnerLedger {
    pub fn system_apply_with_rads(
        &mut self,
        idheader: IdentityHeader,
        requiredk: f32,
        mut adj: SystemAdjustment,
        timestamputc: &str,
        rads_pressure: f64,
        evolve_capacity: f64,
        scale_ceiling: f64,
    ) -> Result<LedgerEvent, InnerLedgerError> {
        // 1. Identity guard (unchanged).
        validate_identity_for_inner_ledger(idheader.clone(), requiredk)?;

        // 2. RADS guard — cannot exceed EVOLVE or SCALE, cannot drain lifeforce.
        enforce_rads_evolve_scale_guards(
            &self.state,
            &self.env,
            &adj,
            rads_pressure,
            evolve_capacity,
            scale_ceiling,
        ).map_err(InnerLedgerError::Rads)?;

        // 3. Lifeforce + eco invariants (existing logic).
        apply_lifeforce_guarded_adjustment(&mut self.state, self.env.clone(), adj.clone())?;

        // 4. Hash + event commit (existing).
        let newhash = hash_state(self.env.hostid.clone(), self.env.clone(), self.state.clone());
        let event = LedgerEvent {
            hostid: self.env.hostid.clone(),
            prevstatehash: self.laststatehash.clone(),
            newstatehash: newhash.clone(),
            adjustment: adj,
            timestamputc: timestamputc.to_string(),
            attestedby: idheader.issuerdid.clone(),
        };
        self.laststatehash = newhash;
        Ok(event)
    }
}

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
