use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::identity::IdentityHeader; // from aln-did-access

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EEGFeatureSummaryV1 {
    pub schemaversion: String,          // "eegfeaturesummary.v1"
    pub header: IdentityHeader,         // issuer_did, subject_role, network_tier, knowledge_factor
    pub session_id: String,
    pub environment_id: String,
    pub intent_label: String,
    pub channel_count: u16,
    pub fs_hz: u16,
    pub band_alpha_power: f32,
    pub band_beta_power: f32,
    pub band_gamma_power: Option<f32>,
    pub csp_component: f32,
    pub erp_latency_ms: u16,
    pub classifier_confidence: f32,
    pub reward_score: f32,
    pub safety_decision: String,        // "Allow", "AllowWithRedaction", "Deny"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralRopeSegment {
    pub segment_id: String,
    pub ts_utc: DateTime<Utc>,
    pub plane_label: String,       // "bci-hci-eeg", etc.
    pub text: String,              // tokenized EEGFEATURE / BCI trace
    pub reward_score: f32,
    pub safety_decision: String,
    pub risk_score: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShotLevelPolicySignal {
    pub task_id: String,
    pub plane_label: String,
    pub risk_score: f32,
    pub latency_budget_ms: u32,
    pub token_budget: u32,
    pub historical_error_rate: f32,
}
