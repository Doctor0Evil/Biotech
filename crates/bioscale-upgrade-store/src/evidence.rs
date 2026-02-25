#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceTag {
    pub short_hex: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Fixed length – enforced by macros and manifest tooling.
    pub sequences: Vec<EvidenceTag>,
}

impl EvidenceBundle {
    pub fn require_10(&self, upgrade_id: &str) {
        assert!(
            self.sequences.len() == 10,
            "EvidenceBundle must contain exactly 10 tags for upgrade {}",
            upgrade_id
        );
    }
}

/// Canonical host biophysics chain (RMR, ATP, thermo, neurovascular, ML, pain).
pub const DEFAULT_BCI_EVIDENCE: EvidenceBundle = EvidenceBundle {
    sequences: vec![
        EvidenceTag {
            short_hex: "a1f3c9b2".into(),
            description: "RMR & ATP turnover vs. kWh per evolution cycle",
        },
        EvidenceTag {
            short_hex: "4be79d01".into(),
            description: "Oxphos efficiency & joule coupling",
        },
        EvidenceTag {
            short_hex: "9cd4a7e8".into(),
            description: "Protein synthesis cost per amino acid",
        },
        EvidenceTag {
            short_hex: "2f8c6b44".into(),
            description: "Thermoregulatory core & brain temperature limits",
        },
        EvidenceTag {
            short_hex: "7e1da2ff".into(),
            description: "Peripheral circulatory adaptation under load",
        },
        EvidenceTag {
            short_hex: "5b93e0c3".into(),
            description: "Neurovascular coupling constraints",
        },
        EvidenceTag {
            short_hex: "d0174aac".into(),
            description: "Safe BCI duty-cycles & cortical tolerances",
        },
        EvidenceTag {
            short_hex: "6ac2f9d9".into(),
            description: "Neuromorphic ML energy vs. thermal/CO envelopes",
        },
        EvidenceTag {
            short_hex: "c4e61b20".into(),
            description: "Protein turnover kinetics under adaptation",
        },
        EvidenceTag {
            short_hex: "8f09d5ee".into(),
            description: "Inflammation/pain rollback thresholds",
        },
    ],
};
