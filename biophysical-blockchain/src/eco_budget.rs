use crate::types::{
    BioTokenState,
    HostEnvelope,
    SystemAdjustment,
    EcoBandProfile,
    LifeforceBandSeries,
    SafetyCurveWave,
};
use crate::lifeforce::{LifeforceError, apply_lifeforce_guarded_adjustment};
use crate::sealed::inner::Sealed;

/// Dimensionless eco impact score per host and epoch.
#[derive(Clone, Debug)]
pub struct EcoImpactScore {
    pub host_id: String,
    /// Average FLOPs per task in this epoch.
    pub avg_flops_per_task: f64,
    /// Average nJ per task (estimated energy).
    pub avg_nj_per_task: f64,
    /// Normalized score in [0, 1].
    pub score_01: f32,
}

/// Non-financial evolution points derived from eco-net rewards.
/// Per-host, non-transferable, system-only.
#[derive(Clone, Debug)]
pub struct EvolutionPoints {
    pub host_id: String,
    /// Accumulated, un-applied points (dimensionless).
    pub accrued: f32,
}

/// Per-host eco-budget envelope: static + dynamic fields.
#[derive(Clone, Debug)]
pub struct EcoBudgetEnvelope {
    pub host_id: String,
    /// Hard cap on eco FLOPs per window (from HostEnvelope.ecoflopslimit).
    pub eco_flops_limit: f64,
    /// Eco band profile (Low/Medium/High + avg FLOPs/nJ).
    pub eco_profile: EcoBandProfile,
    /// Optional max evolution points that may be applied per turn.
    pub max_points_per_turn: f32,
}

impl Sealed for EcoImpactScore {}
impl Sealed for EvolutionPoints {}
impl Sealed for EcoBudgetEnvelope {}

impl EcoImpactScore {
    /// Compute a normalized EcoImpactScore from FLOPs + nJ.
    pub fn from_telemetry(host_id: String, avg_flops: f64, avg_nj: f64) -> Self {
        let flops_norm = (avg_flops / 1.0e9_f64).min(1.0).max(0.0);
        let nj_norm = (avg_nj / 1.0e3_f64).min(1.0).max(0.0);
        // Lower FLOPs/energy → lower score → better eco behavior.
        let score = 1.0_f32 - ((0.5 * flops_norm + 0.5 * nj_norm) as f32);
        EcoImpactScore {
            host_id,
            avg_flops_per_task: avg_flops,
            avg_nj_per_task: avg_nj,
            score_01: score.clamp(0.0, 1.0),
        }
    }
}

impl EvolutionPoints {
    pub fn new(host_id: String) -> Self {
        EvolutionPoints { host_id, accrued: 0.0 }
    }

    /// Add eco-net reward as small, bounded increment.
    pub fn add_reward(&mut self, impact: &EcoImpactScore) {
        // Reward is proportional to eco improvement; bounded to avoid jumps.
        let delta = (impact.score_01 * 0.05_f32).clamp(0.0, 0.05);
        self.accrued = (self.accrued + delta).clamp(0.0, 1.0);
    }

    /// Consume at most max_per_turn for a single micro-step.
    pub fn consume_for_turn(&mut self, max_per_turn: f32) -> f32 {
        let allowed = max_per_turn.min(self.accrued);
        self.accrued -= allowed;
        allowed
    }
}

/// Deterministic mapping from eco-net rewards to small SystemAdjustment deltas.
/// This is a helper; final enforcement still goes through lifeforce guards.
pub fn evolution_delta_from_points(
    points_used: f32,
) -> SystemAdjustment {
    // Very small deltas; do not touch BLOOD/OXYGEN directly.
    let scale = points_used as f64;
    SystemAdjustment {
        deltabrain: 0.5_f64 * scale,
        deltawave: 0.3_f64 * scale,
        deltablood: 0.0,
        deltaoxygen: 0.0,
        deltanano: 0.2_f64 * scale,
        deltasmart: 0.1_f64 * scale,
        ecocost: 0.0,
        reason: "eco-net-evolution-microstep".to_string(),
    }
}

/// Apply an eco-budgeted evolution micro-step:
/// - uses points within EcoBudgetEnvelope
/// - always passes through lifeforce/eco guards
/// - never overrides HostEnvelope limits.
pub fn apply_eco_evolution_microstep(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    lifeforce_series: LifeforceBandSeries,
    eco_profile: EcoBandProfile,
    wave_curve: SafetyCurveWave,
    eco_budget: &EcoBudgetEnvelope,
    points: &mut EvolutionPoints,
) -> Result<(), LifeforceError> {
    let max_per_turn = eco_budget.max_points_per_turn;
    if max_per_turn <= 0.0 {
        return Ok(()); // path disabled for this host.
    }
    let used = points.consume_for_turn(max_per_turn);
    if used <= 0.0 {
        return Ok(()); // nothing to apply this turn.
    }
    let delta = evolution_delta_from_points(used);
    apply_lifeforce_guarded_adjustment(
        state,
        env,
        delta,
        lifeforce_series,
        eco_profile,
        wave_curve,
    )
}
