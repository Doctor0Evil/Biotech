use crate::types::{BioTokenState, HostEnvelope, SystemAdjustment};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RadsError {
    #[error("RADS pressure exceeds EVOLVE capacity or SCALE ceiling")]
    RadsOverEvolutionBudget,
}

/// Pure helper: enforce RADS ≤ EVOLVE, RADS ≤ SCALE, with no lifeforce drain.
pub fn enforce_rads_evolve_scale_guards(
    state: &BioTokenState,
    env: &HostEnvelope,
    adj: &SystemAdjustment,
    rads_pressure: f64,   // 0.0–1.0 derived from telemetry
    evolve_capacity: f64, // 0.0–1.0 host-local, system-only
    scale_ceiling: f64,   // 0.0–1.0 per-turn SCALE ceiling
) -> Result<(), RadsError> {
    let ev_cap = evolve_capacity.clamp(0.0, 1.0);
    let sc_cap = scale_ceiling.clamp(0.0, 1.0);
    let hard_cap = ev_cap.min(sc_cap);

    if rads_pressure > hard_cap {
        return Err(RadsError::RadsOverEvolutionBudget);
    }

    // Important: we do NOT touch state.brain, state.blood, state.oxygen here.
    // Lifeforce is only adjusted by existing uncontrollable-event paths.

    // We also do NOT alter env.brainmin/bloodmin/oxygenmin or SCALE itself.
    // RADS can only be used upstream to shrink adj.deltanano or to refuse the op.

    Ok(())
}
