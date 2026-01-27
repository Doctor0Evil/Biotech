use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::EcoBand;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CivicAuditLogEntry {
    pub ts_utc: DateTime<Utc>,
    pub civic_tags: Vec<String>,
    pub civic_class: String,   // "CivicHeroic", "CivicGood", "Neutral", "Disallowed"
    pub eco_cost_flops: f64,
    pub eco_band: EcoBand,
    pub lifeforce_ok: bool,
    pub delta_brain_abs: f64,
    pub delta_wave_abs: f64,
    pub delta_nano_abs: f64,
    pub delta_smart_abs: f64,
    pub shot_level: String,    // "ZeroShot", "FewShot"
    pub handshake_phase: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CivicRewardProfile {
    pub version: String,
    pub host_id: String,
    pub multiplier_min: f64,
    pub multiplier_max: f64,
    pub default_multiplier: f64,
    pub required_knowledge_factor: f32,
}
