use crate::{UpgradeDescriptor, EvidenceBundle};

#[derive(Clone, Debug)]
pub struct ALNComplianceParticle {
    pub host_did: String,
    pub bostrom_address: String,
    pub upgrade_id: String,
    pub consent_ledger_tx: String,
    pub neurorights_flags: u64,
    pub evidence: EvidenceBundle,
    pub budget_fit: bool,
}

impl ALNComplianceParticle {
    pub fn is_compliant(&self) -> bool {
        self.budget_fit
            && self.evidence.sequences.len() == 10
            && self.neurorights_flags & 0b1 != 0 // example: reversibility bit
    }

    pub fn from_descriptor_and_budget(
        host_did: &str,
        bostrom_address: &str,
        consent_ledger_tx: &str,
        neurorights_flags: u64,
        desc: &UpgradeDescriptor,
        budget_fit: bool,
    ) -> Self {
        Self {
            host_did: host_did.to_string(),
            bostrom_address: bostrom_address.to_string(),
            upgrade_id: desc.id.to_string(),
            consent_ledger_tx: consent_ledger_tx.to_string(),
            neurorights_flags,
            evidence: desc.evidence.clone(),
            budget_fit,
        }
    }
}
