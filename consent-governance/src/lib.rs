use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemonstratedConsentShard {
    pub version: String,
    pub subject_did: String,             // ALN/DID/Bostrom
    pub biosample_id: String,
    pub allowed_uses: Vec<String>,       // e.g. "nonprofit-research"
    pub forbidden_domains: Vec<String>,  // e.g. "commercial", "military"
    pub not_before: DateTime<Utc>,
    pub not_after: Option<DateTime<Utc>>,
    pub revocation_rules: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustodyActionTrail {
    pub biosample_id: String,
    pub actions: Vec<CustodyAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustodyAction {
    pub ts_utc: DateTime<Utc>,
    pub actor_did: String,
    pub role: String,            // donor, custodian, researcher, regulator, community
    pub protocol_id: String,
    pub outcome_hash: String,    // hash only, never raw data
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleRightsMatrix {
    pub version: String,
    pub role: String,
    pub allowed_actions: Vec<String>,
}

impl RoleRightsMatrix {
    pub fn allows(&self, action: &str) -> bool {
        self.allowed_actions.iter().any(|a| a == action)
    }
}
