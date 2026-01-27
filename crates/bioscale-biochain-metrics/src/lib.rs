use prometheus::{Counter, CounterVec, register_counter, register_counter_vec};

pub struct ValidatorBioMetrics {
    blocks_bio_safe_total: Counter,
    aln_non_compliant_total: CounterVec,
    evidence_invalid_total: CounterVec,
    corridor_mismatch_total: CounterVec,
}

impl ValidatorBioMetrics {
    pub fn new() -> Self {
        Self {
            blocks_bio_safe_total: register_counter!(
                "biochain_blocks_bio_safe_total",
                "Blocks passing biophysical and ALN checks"
            ).unwrap(),
            aln_non_compliant_total: register_counter_vec!(
                "biochain_aln_non_compliant_total",
                "Blocks/tx rejected due to ALN neurorights violations",
                &["validator_id"]
            ).unwrap(),
            evidence_invalid_total: register_counter_vec!(
                "biochain_evidence_invalid_total",
                "Blocks/tx with invalid biophysical evidence bundles",
                &["validator_id"]
            ).unwrap(),
            corridor_mismatch_total: register_counter_vec!(
                "biochain_corridor_mismatch_total",
                "Blocks/tx where corridor hash did not match registry",
                &["validator_id"]
            ).unwrap(),
        }
    }

    pub fn inc_blocks_bio_safe(&self) {
        self.blocks_bio_safe_total.inc();
    }

    pub fn inc_aln_non_compliant(&self, _block_hash: &[u8]) {
        self.aln_non_compliant_total
            .with_label_values(&["validator"])
            .inc();
    }

    pub fn inc_evidence_invalid(&self, _block_hash: &[u8]) {
        self.evidence_invalid_total
            .with_label_values(&["validator"])
            .inc();
    }

    pub fn inc_corridor_mismatch(&self, _block_hash: &[u8]) {
        self.corridor_mismatch_total
            .with_label_values(&["validator"])
            .inc();
    }
}
