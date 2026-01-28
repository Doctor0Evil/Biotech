use serde::{Serialize, Deserialize};

/// Host-local psycho-environmental snapshot (non-financial).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PsychoEnvSnapshot {
    pub sanity: f32,        // 0.0–1.0
    pub decay: f32,         // 0.0–1.0
    pub indoor_bias: f32,   // 0.0–1.0
    pub social_score: f32,  // 0.0–1.0
}

/// Host-local NANO / eco / RADS safety snapshot (non-financial).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoRadsSnapshot {
    /// Normalized NANO duty 0.0–1.0 vs. host nanomaxfraction envelope.
    pub nano_duty: f32,
    /// Eco band [0.0, 1.0], usually mapped from EcoBand::Low/Medium/High.
    pub eco_band_scalar: f32,
    /// Cumulative radiation-equivalent budget consumed this epoch 0.0–1.0.
    pub rads_budget_used: f32,
    /// Instantaneous device telemetry risk score 0.0–1.0 (shielding, temp, etc.).
    pub device_risk: f32,
}
