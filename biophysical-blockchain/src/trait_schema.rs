//! TRAIT schema: host-bound, consent-given trait descriptors for evolution paths.
//! - Non-financial, non-transferable.
//! - Governed by BRAIN, bounded by SMART and Metabolic_Consent.
//! - Used by evolution eligibility filters and DECAY/safety logic.

use serde::{Deserialize, Serialize};

/// Domains where traits can be applied. Pure routing labels, not assets.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraitDomain {
    /// Default cognitive / sensory tuning traits.
    Generic,
    /// Somatic, defensive micro-morphology (e.g., teeth/claws micro-steps).
    TeethClawsDefense,
    /// Metabolic efficiency traits (energy usage, thermal comfort, etc.).
    Metabolic,
    /// Purely software-only traits (tooling, interface preferences).
    SoftwareOnly,
}

/// Runtime view of pain / discomfort corridors relevant to a TRAIT.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraitDiscomfortEvidence {
    /// 0.0–1.0 summary of sustained nociception / aversion from BCI.
    pub pain_index: f32,
    /// Count of recent HardStop episodes while this trait was active.
    pub hard_stop_count: u32,
    /// Rolling average of BLOOD/OXYGEN soft-floor proximity 0.0–1.0.
    pub biocompatibility_stress: f32,
}

/// Static consent envelope for a single trait on a host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraitConsent {
    /// Human-readable trait ID (e.g., "night-vision-micro", "teeth-micro-v1").
    pub trait_id: String,
    /// Domain routing label.
    pub domain: TraitDomain,
    /// Whether the host currently consents to this trait being *active*.
    pub active: bool,
    /// Whether SMART is allowed to auto-maintain / auto-trim this trait
    /// under Metabolic_Consent corridors.
    pub smart_autonomy_allowed: bool,
    /// Optional, host-defined maximum evolution rate for this trait,
    /// expressed as a fraction of daily SCALE (0.0–1.0).
    pub max_daily_scale_fraction: f32,
}

/// Dynamic runtime state of a trait, including discomfort evidence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraitRuntimeState {
    pub consent: TraitConsent,
    /// Evidence accumulated from PainCorridorSignal, LifeforceBandSeries, etc.
    pub discomfort: TraitDiscomfortEvidence,
    /// Normalized 0.0–1.0 "activation level" along the mutation path.
    /// This is not a token balance; it is a configuration intensity.
    pub activation_level: f32,
    /// Whether the system has marked this trait as cooling-down or revoked
    /// due to sustained discomfort.
    pub cooling_down: bool,
}

/// Per-host TRAIT panel (host-bound, non-transferable).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TraitPanel {
    pub traits: Vec<TraitRuntimeState>,
}

impl TraitPanel {
    /// Find a trait by ID.
    pub fn find_mut(&mut self, trait_id: &str) -> Option<&mut TraitRuntimeState> {
        self.traits
            .iter_mut()
            .find(|t| t.consent.trait_id == trait_id)
    }

    /// Host-initiated consent toggle. This is the only place where `active`
    /// may change; callers must already have verified ALN/DID consent shards.
    pub fn set_active(&mut self, trait_id: &str, active: bool) {
        if let Some(t) = self.find_mut(trait_id) {
            t.consent.active = active;
            if !active {
                t.cooling_down = true;
            }
        }
    }

    /// Update discomfort evidence from BCI / lifeforce observations.
    pub fn update_discomfort(
        &mut self,
        trait_id: &str,
        pain_index: f32,
        hard_stop_delta: u32,
        biocompatibility_stress: f32,
    ) {
        if let Some(t) = self.find_mut(trait_id) {
            t.discomfort.pain_index = pain_index.clamp(0.0, 1.0);
            t.discomfort.hard_stop_count =
                t.discomfort.hard_stop_count.saturating_add(hard_stop_delta);
            t.discomfort.biocompatibility_stress =
                biocompatibility_stress.clamp(0.0, 1.0);
        }
    }
}
