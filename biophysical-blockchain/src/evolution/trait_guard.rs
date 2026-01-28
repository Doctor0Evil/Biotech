//! Evolution-time guard that uses TRAIT panels to:
//! - Gate which EvolutionUpgrade paths are eligible.
//! - Prefer reversal / cooling when discomfort is high.
//! - Enforce per-trait daily SCALE fractions.

use crate::trait_schema::{TraitDomain, TraitPanel, TraitRuntimeState};
use serde::{Deserialize, Serialize};

/// Minimal view of an evolution proposal, as seen by this guard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionProposal {
    /// Domain of the proposed mutation.
    pub domain: TraitDomain,
    /// Optional trait_id this evolution step is trying to adjust.
    pub trait_id: Option<String>,
    /// Proposed fraction of daily SCALE usage for this step (0.0–1.0).
    pub requested_scale_fraction: f32,
    /// Direction of change: +1 for forward evolution, -1 for reversal / rollback.
    pub direction: i8,
}

/// Decision returned to the caller before SystemAdjustment is built.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitGuardDecision {
    /// Allowed as requested.
    Allow,
    /// Allowed but must be downscaled (caller should shrink SCALE/DECAY).
    AllowWithDownscale,
    /// Only reversal is allowed; forward evolution is blocked.
    ReversalOnly,
    /// Trait/domain is in cooling-down or revoked state; block this proposal.
    Block,
}

pub struct TraitGuard;

impl TraitGuard {
    pub fn evaluate(
        panel: &TraitPanel,
        proposal: &EvolutionProposal,
    ) -> TraitGuardDecision {
        // If no trait_id is specified, fall back to domain-only policy.
        if let Some(ref id) = proposal.trait_id {
            if let Some(trait_state) =
                panel.traits.iter().find(|t| t.consent.trait_id == *id)
            {
                return Self::evaluate_for_trait(trait_state, proposal);
            }
        }

        // Domain-only: TeethClawsDefense is always tightly capped.
        match proposal.domain {
            TraitDomain::TeethClawsDefense => {
                if proposal.direction > 0 {
                    TraitGuardDecision::AllowWithDownscale
                } else {
                    TraitGuardDecision::Allow
                }
            }
            _ => TraitGuardDecision::Allow,
        }
    }

    fn evaluate_for_trait(
        trait_state: &TraitRuntimeState,
        proposal: &EvolutionProposal,
    ) -> TraitGuardDecision {
        let c = &trait_state.consent;
        let d = &trait_state.discomfort;

        // If trait is not active, only allow reversal.
        if !c.active {
            return if proposal.direction < 0 {
                TraitGuardDecision::ReversalOnly
            } else {
                TraitGuardDecision::Block
            };
        }

        // Hard veto: high pain or repeated HardStop => only reversal allowed.
        if d.pain_index > 0.7
            || d.hard_stop_count > 0
            || d.biocompatibility_stress > 0.7
        {
            return if proposal.direction < 0 {
                TraitGuardDecision::ReversalOnly
            } else {
                TraitGuardDecision::Block
            };
        }

        // Enforce host-consented daily SCALE fractions for this trait.
        let max_frac = c.max_daily_scale_fraction.clamp(0.0, 1.0);
        if max_frac > 0.0 && proposal.requested_scale_fraction > max_frac {
            return TraitGuardDecision::AllowWithDownscale;
        }

        TraitGuardDecision::Allow
    }
}
