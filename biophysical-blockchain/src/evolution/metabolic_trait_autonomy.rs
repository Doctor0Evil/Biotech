//! SMART + Metabolic_Consent + TRAIT interplay:
//! SMART may only auto-adjust traits when:
//! - Metabolic_Consent mode allows micro-mutations.
//! - Trait.consent.smart_autonomy_allowed == true.
//! - TraitGuardDecision != Block/ReversalOnly for forward steps.

use crate::trait_schema::{TraitDomain, TraitPanel};
use crate::evolution::trait_guard::{EvolutionProposal, TraitGuard, TraitGuardDecision};

#[derive(Clone, Copy, Debug)]
pub enum MetabolicConsentMode {
    ManualOnly,
    AutoMicro,
    AutoMicroPlusWave,
}

/// Decide whether SMART is allowed to apply a tiny, automatic evolution step
/// for a given trait in metabolic housekeeping.
pub fn smart_can_autoadjust_trait(
    mode: MetabolicConsentMode,
    traits: &TraitPanel,
    trait_id: &str,
    domain: TraitDomain,
) -> bool {
    if !matches!(mode, MetabolicConsentMode::AutoMicro | MetabolicConsentMode::AutoMicroPlusWave) {
        return false;
    }

    let Some(t) = traits.traits.iter().find(|t| t.consent.trait_id == trait_id) else {
        return false;
    };

    if !t.consent.smart_autonomy_allowed {
        return false;
    }

    let proposal = EvolutionProposal {
        domain,
        trait_id: Some(trait_id.to_string()),
        requested_scale_fraction: 0.05, // micro-step, subject to SCALE/DECAY anyway
        direction: +1,
    };

    match TraitGuard::evaluate(traits, &proposal) {
        TraitGuardDecision::Allow | TraitGuardDecision::AllowWithDownscale => true,
        TraitGuardDecision::ReversalOnly | TraitGuardDecision::Block => false,
    }
}
