mod security;
mod civic_profile;
mod civic_audit;

use civic_audit::{CivicAuditEntry, append_civic_audit_entry};
use civic_profile::CivicRewardProfile;
// ... existing imports ...

#[derive(Clone)]
struct AppState {
    ledger: Arc<Mutex<InnerLedger>>,
    rope: Arc<Mutex<NeuralRope>>,
    civic_profile: Arc<CivicRewardProfile>,
}

fn init_ledger() -> InnerLedger { /* unchanged */ }

fn init_civic_profile() -> CivicRewardProfile {
    // In production, point this to a host-specific path derived from hostid.
    CivicRewardProfile::load_from_json(
        "qpu/data/shards/profiles/civic-reward-profile.json",
    )
    .unwrap_or_else(|_| {
        // Fallback: safe defaults if shard is missing.
        CivicRewardProfile {
            multiplier_min: 0.0,
            multiplier_max: 4.0,
            default_multiplier: 1.0,
            required_knowledge_factor: 0.60,
            heroic_tags: Default::default(),
            heroic_multiplier: 3.0,
            good_tags: Default::default(),
            good_multiplier: 1.5,
            neutral_multiplier: 1.0,
            disallowed_tags: Default::default(),
            eco_bonus_enabled: false,
            eco_low_threshold: 0.0,
            eco_low_bonus: 1.0,
        }
    })
}
