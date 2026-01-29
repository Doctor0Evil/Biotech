use crate::evolution_turns::MAX_DAILY_TURNS_INNER;
use crate::eco_budget::EcoBudgetEnvelope;
use crate::types::EcoBandProfile;

/// Parsed subset of neuromorph-evolution-budget.aln and neuromorph-eco-profile.aln
/// from your ALN parser / qpudatashards layer.
pub struct ParsedEvolutionBudgetShard {
    pub host_id: String,
    pub max_daily_turns: u8,
    pub max_points_per_turn: f32,
}

pub fn clamp_daily_turns(shard_value: u8) -> u8 {
    shard_value.min(MAX_DAILY_TURNS_INNER)
}

pub fn envelope_from_shards(
    eco_profile: EcoBandProfile,
    budget: &ParsedEvolutionBudgetShard,
    eco_flops_limit: f64,
) -> EcoBudgetEnvelope {
    EcoBudgetEnvelope {
        host_id: budget.host_id.clone(),
        eco_flops_limit,
        eco_profile,
        max_points_per_turn: budget.max_points_per_turn.max(0.0),
    }
}
