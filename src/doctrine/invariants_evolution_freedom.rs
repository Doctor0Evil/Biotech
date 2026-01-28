//! Evolution freedom invariants for AugDoctor biophysical runtime.
//!
//! This module encodes doctrinal guarantees that:
//! - Evolution capacity is never hard-capped.
//! - Safety layers (DECAY, SCALE, Metabolic_Consent, KarmaAura) only
//!   shape rate and per-step size, never total EVOLVE or TRAIT space.
//! - Souls and consciousness remain non-ownable and non-modifiable.
//!
//! It is designed to be wired into governance / shard loading paths and
//! inner-ledger orchestration, so that any future governance shard that
//! attempts to introduce hard caps on capability is rejected at compile-
//! time (via types) and at runtime (via explicit checks).

use crate::types::{HostId, EvolutionDomainId};

/// Declarative description of what a governance shard *may* do
/// with respect to evolution. This intentionally excludes any
/// "max_total_evolve" / "max_trait_count" style fields.
#[derive(Clone, Debug)]
pub struct EvolutionSafetyPolicy {
    /// Minimum lifeforce floors and eco ceilings that must be respected.
    pub lifeforce_safe: bool,
    pub eco_safe: bool,

    /// Whether micro-step sizing is allowed (DECAY, SCALE shaping).
    pub allows_decay_shaping: bool,
    pub allows_scale_shaping: bool,

    /// Whether host-authored Metabolic_Consent may be used to
    /// automate micro-adjustments inside existing corridors.
    pub allows_metabolic_consent: bool,

    /// Whether Karma / aura can modulate DECAY (only as a multiplier
    /// in 0.0..=1.0, never as an amplifier above 1.0).
    pub allows_karma_aura: bool,

    /// Whether this policy attempts to ban specific evolution domains
    /// *permanently* (structural denial), instead of temporarily gating
    /// them by safety, consent, or provenance.
    pub declares_structural_domain_bans: bool,

    /// Whether this policy declares any *explicit* total-cap fields on
    /// EVOLVE, TRAIT, or lifetime domain count (these are forbidden).
    pub declares_total_evolve_caps: bool,
    pub declares_total_trait_caps: bool,
    pub declares_lifetime_domain_caps: bool,
}

/// Host-local shard of evolution configuration, as seen by orchestration.
/// This is where governance, consent, and safety are combined. We
/// explicitly *do not* include any "total EVOLVE cap" or "max traits"
/// fields here; type-level omission is the first line of defense.
#[derive(Clone, Debug)]
pub struct HostEvolutionConfig {
    pub host_id: HostId,

    /// Allowed evolution domains for this host (by safety/consent).
    /// Empty for a given domain means "not currently allowed", *not*
    /// "forbidden forever".
    pub allowed_domains: Vec<EvolutionDomainId>,

    /// Safety policy derived from active governance shards.
    pub safety_policy: EvolutionSafetyPolicy,

    /// Flag indicating whether this host has provided self-consent
    /// for evolution in the current session / window.
    pub has_active_self_consent: bool,

    /// Whether provenance requirements (mutation-provenance shard)
    /// have been satisfied for the current mutation domain.
    pub provenance_verified: bool,
}

/// Enumeration of doctrinal violations for evolution freedom.
#[derive(Debug, thiserror::Error)]
pub enum EvolutionFreedomError {
    #[error("governance attempted to declare a hard cap on total EVOLVE")]
    TotalEvolveCapForbidden,

    #[error("governance attempted to declare a hard cap on total TRAIT count")]
    TotalTraitCapForbidden,

    #[error("governance attempted to declare a hard cap on lifetime evolution domains")]
    LifetimeDomainCapForbidden,

    #[error("evolution domain structurally banned; only temporary, safety-based denial is allowed")]
    StructuralDomainBanForbidden,

    #[error("karma/aura attempted to amplify DECAY above 1.0")]
    KarmaAmplificationForbidden,

    #[error("automation attempted to bypass self-consent or provenance")]
    AutomationBypassedConsentOrProvenance,
}

/// Compile-time style helper: if you *can* construct an EvolutionSafetyPolicy,
/// you are already constrained to the allowed fields. This function then
/// enforces doctrinal rules about those fields.
pub fn assert_policy_respects_evolution_freedom(
    policy: &EvolutionSafetyPolicy,
) -> Result<(), EvolutionFreedomError> {
    if policy.declares_total_evolve_caps {
        return Err(EvolutionFreedomError::TotalEvolveCapForbidden);
    }
    if policy.declares_total_trait_caps {
        return Err(EvolutionFreedomError::TotalTraitCapForbidden);
    }
    if policy.declares_lifetime_domain_caps {
        return Err(EvolutionFreedomError::LifetimeDomainCapForbidden);
    }
    if policy.declares_structural_domain_bans {
        return Err(EvolutionFreedomError::StructuralDomainBanForbidden);
    }
    Ok(())
}

/// Runtime guard: called whenever governance shards are (re)loaded or
/// updated for a host. Any shard that attempts to introduce forbidden
/// caps or structural bans is rejected.
pub fn validate_governance_shard_for_host(
    host_cfg: &HostEvolutionConfig,
) -> Result<(), EvolutionFreedomError> {
    assert_policy_respects_evolution_freedom(&host_cfg.safety_policy)
}

/// Helper for DECAY / KarmaAura logic: enforce that the final multiplier
/// is always in [0.0, 1.0], and never > 1.0, so it can only dampen,
/// not amplify, a proposed evolution step.
///
/// This function is meant to be used *inside* safedecaymultiplier‑like
/// paths, after aura / karma calculation, before scaling a SystemAdjustment.
pub fn enforce_decay_multiplier_bounds(raw: f32) -> Result<f32, EvolutionFreedomError> {
    if raw > 1.0 {
        // Any attempt to go above 1.0 is a doctrinal violation.
        return Err(EvolutionFreedomError::KarmaAmplificationForbidden);
    }
    Ok(raw.clamp(0.0, 1.0))
}

/// Runtime guard for automated evolution paths (Metabolic_Consent, SMART
/// autonomy, etc.). Ensures automation never bypasses self-consent or
/// provenance, and that any denial is temporary (configurable), not
/// a structural "never evolve" flag.
pub fn validate_automated_evolution_path(
    host_cfg: &HostEvolutionConfig,
    domain: &EvolutionDomainId,
    automated: bool,
) -> Result<(), EvolutionFreedomError> {
    if automated {
        if !host_cfg.has_active_self_consent || !host_cfg.provenance_verified {
            return Err(EvolutionFreedomError::AutomationBypassedConsentOrProvenance);
        }
    }

    // If a domain is not in allowed_domains, that is *not* a doctrinal
    // violation by itself; it just means "not currently safe or consented".
    // Structural bans must be encoded via policy flags, which are already
    // checked in assert_policy_respects_evolution_freedom.

    // Additionally, governance code that computes `allowed_domains` should
    // ensure that any denial is expressed as "not present now" plus
    // recorded reasons (lifeforce risk, missing consent, missing
    // provenance), not as a permanent tombstone.

    Ok(())
}
