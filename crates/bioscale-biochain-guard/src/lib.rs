use bioscale_biochain_core::{BioProof, BioEnvelopeHash, NeuroRightsFlags};
use bioscale_biochain_aln::AlnParticle;
use bioscale_biochain_metrics::ValidatorBioMetrics;

#[derive(Debug, Clone)]
pub enum BioValidationResult {
    Allowed,
    Denied { reason: String },
}

pub struct BioValidatorGuard {
    metrics: ValidatorBioMetrics,
}

impl BioValidatorGuard {
    pub fn new(metrics: ValidatorBioMetrics) -> Self {
        Self { metrics }
    }

    pub fn validate_block(
        &self,
        block_hash: &[u8],
        bio_proof: &BioProof,
        aln_particle: &AlnParticle,
    ) -> BioValidationResult {
        // 1) Check ALN neurorights flags (consent, reversibility, privacy).
        if !aln_particle.is_compliant() {
            self.metrics.inc_aln_non_compliant(block_hash);
            return BioValidationResult::Denied {
                reason: "ALN neurorights non‑compliant".into(),
            };
        }

        // 2) Check evidence bundle and biophysical corridor hash.
        if bio_proof.evidence_tags.len() != 10 {
            self.metrics.inc_evidence_invalid(block_hash);
            return BioValidationResult::Denied {
                reason: "Invalid evidence bundle (must be 10 tags)".into(),
            };
        }

        if !bio_proof.verify_corridor_hash(&BioEnvelopeHash::from(aln_particle)) {
            self.metrics.inc_corridor_mismatch(block_hash);
            return BioValidationResult::Denied {
                reason: "Biophysical corridor hash mismatch".into(),
            };
        }

        // 3) If everything passes, mark as safe.
        self.metrics.inc_blocks_bio_safe();
        BioValidationResult::Allowed
    }
}
