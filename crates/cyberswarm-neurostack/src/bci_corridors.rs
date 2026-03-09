#![forbid(unsafe_code)]

use std::{fs, path::PathBuf};
use chrono::NaiveDate;
use serde::Serialize;
use bioscale_upgrade_store::{UpgradeDescriptor, EvidenceBundle};
use bioscale_evolution_registry::EvolutionRegistry;

#[derive(Serialize)]
struct ManifestUpgrade {
    upgrade_id: String,
    energy_joules: f64,
    protein_aa: f64,
    thermo_envelope: String,
    ml_schedule: String,
    reversal_policy: String,
    evidence_hex_tags: Vec<String>,
}

#[derive(Serialize)]
struct ResearchManifest {
    date: String,
    host_did: String,
    bostrom_address: String,
    git_commit: String,
    upgrades: Vec<ManifestUpgrade>,
}

fn main() {
    let date_str = std::env::args()
        .skip_while(|a| a != "--date")
        .nth(1)
        .expect("--date required");

    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .expect("invalid date");

    let registry = EvolutionRegistry::load()
        .expect("failed to load evolution registry");

    let git_commit = std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".into());

    let upgrades: Vec<ManifestUpgrade> = registry
        .all_upgrades()
        .map(|desc: &UpgradeDescriptor| ManifestUpgrade {
            upgrade_id: desc.id.to_string(),
            energy_joules: desc.energy_joules,
            protein_aa: desc.protein_aa,
            thermo_envelope: desc.thermo_envelope.short_label(),
            ml_schedule: desc.ml_schedule.short_label(),
            reversal_policy: desc.reversal.policy_label(),
            evidence_hex_tags: desc.evidence.sequences
                .iter()
                .map(|t| t.short_hex.clone())
                .collect(),
        })
        .collect();

    let manifest = ResearchManifest {
        date: date.format("%Y-%m-%d").to_string(),
        host_did: "didaln-organic-cpuhost".into(),
        bostrom_address: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
        git_commit,
        upgrades,
    };

    let mut path = PathBuf::from("research");
    fs::create_dir_all(&path).expect("failed to create research directory");
    path.push(format!("{}-manifest.json", date_str));

    let json = serde_json::to_string_pretty(&manifest)
        .expect("serialize manifest");
    fs::write(&path, json).expect("write manifest");
}
