use crate::types::NanoRadsSnapshot;
use biophysical_blockchain::types::{
    LifeforceBand,
    EcoBand,
    SystemAdjustment,
};
use biophysical_blockchain::nanolifeband::NanoLifebandRouter;
use bci_bioledger_bridge::BciEvent;

/// Classification for NANO routing at the bridge layer.
#[derive(Clone, Debug)]
pub enum NanoRouteDecision {
    Safe,   // Allow full requested NANO delta.
    Defer,  // Time-slice / reschedule, shrink NANO delta.
    Deny,   // Zero-out NANO delta for this shot.
}

/// RADS + NANO policy output for a single EEG/telemetry update.
#[derive(Clone, Debug)]
pub struct RadsNanoPolicy {
    pub decision: NanoRouteDecision,
    /// Scalar 0.0–1.0 applied to requested NANO delta before ledger.
    pub nano_scale: f32,
    /// Additional RADS “cost” to account for in SystemAdjustment.ecocost.
    pub rads_cost: f64,
}

fn eco_band_from_scalar(s: f32) -> EcoBand {
    if s < 0.33 {
        EcoBand::Low
    } else if s < 0.66 {
        EcoBand::Medium
    } else {
        EcoBand::High
    }
}

/// Map EEG + device telemetry into a RADS / NANO routing policy.
pub fn map_eeg_telemetry_to_rads_nano(
    event: &BciEvent,
    nano_snap: &NanoRadsSnapshot,
    lifeforce_band: LifeforceBand,
) -> RadsNanoPolicy {
    // 1. Router classification using existing NanoLifebandRouter.
    let eco_band = eco_band_from_scalar(nano_snap.eco_band_scalar.clamp(0.0, 1.0));
    let clarity = event.clarity_index.clamp(0.0, 1.0);

    let router_decision = NanoLifebandRouter::classify(
        lifeforce_band,
        clarity,
        eco_band,
    );

    // 2. RADS budget pressure from cumulative use + device risk.
    let rads_pressure = (nano_snap.rads_budget_used + nano_snap.device_risk)
        .clamp(0.0, 2.0);

    // 3. Derive NANO scale and RADS eco-cost.
    let (decision, nano_scale) = match router_decision {
        // HardStop lifeforce or router “Deny”.
        crate::nanolifeband::RouterDecision::Deny => {
            (NanoRouteDecision::Deny, 0.0)
        }
        crate::nanolifeband::RouterDecision::Defer => {
            // Defer: cut NANO based on RADS pressure.
            // At rads_pressure >= 1.5 we are near RADS cap.
            let base = 0.5_f32;
            let penalty = (rads_pressure as f32).min(1.5) / 1.5;
            let scale = (base * (1.0 - 0.7 * penalty)).clamp(0.05, 0.5);
            (NanoRouteDecision::Defer, scale)
        }
        crate::nanolifeband::RouterDecision::Safe => {
            // Safe: allow up to 1.0 but taper if RADS is high.
            if rads_pressure < 0.6 {
                (NanoRouteDecision::Safe, 1.0)
            } else {
                let penalty = ((rads_pressure - 0.6) / 0.9).clamp(0.0, 1.0);
                let scale = (1.0 - 0.5 * penalty) as f32;
                (NanoRouteDecision::Safe, scale.clamp(0.4, 1.0))
            }
        }
    };

    // 4. Translate RADS pressure to an ecocost increment (non-financial).
    // This feeds SystemAdjustment.ecocost and is later bounded by ecoflopslimit.
    let rads_cost = (rads_pressure as f64) * 0.05_f64;

    RadsNanoPolicy {
        decision,
        nano_scale,
        rads_cost,
    }
}

/// Apply RADS/NANO policy to a candidate SystemAdjustment before submit.
pub fn apply_rads_nano_policy(
    base: &SystemAdjustment,
    policy: &RadsNanoPolicy,
) -> SystemAdjustment {
    let mut adj = base.clone();

    match policy.decision {
        NanoRouteDecision::Deny => {
            adj.deltanano = 0.0;
        }
        NanoRouteDecision::Defer | NanoRouteDecision::Safe => {
            adj.deltanano *= policy.nano_scale as f64;
        }
    }

    // Add RADS-equivalent eco cost into existing eco budget.
    adj.ecocost += policy.rads_cost;

    adj
}
