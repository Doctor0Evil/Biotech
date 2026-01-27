use std::{fs, path::PathBuf};
use chrono::NaiveDate;
use serde::Serialize;
use bioscale_upgrade_store::{UpgradeDescriptor, EvidenceBundle};
use bioscale_upgrade_store::registry::UpgradeRegistry;
use cyberswarm_neurostack::BciHostSnapshot;

#[derive(Serialize)]
struct ResearchUpgrade {
    upgrade_id: String,
    energy_joules: f64,
    protein_aa: u64,
    thermo_envelope: ThermoExport,
    ml_schedule: MlExport,
    reversal: ReversalExport,
    evidence_hex_tags: Vec<String>,
}

#[derive(Serialize)]
struct BciSnapshotsExport {
    eeg_load_min: f32,
    eeg_load_max: f32,
    hrv_ms_min: f32,
    hrv_ms_max: f32,
    core_temp_c_min: f32,
    core_temp_c_max: f32,
    duty_cycle_min: f32,
    duty_cycle_max: f32,
}

#[derive(Serialize)]
struct AlnParticleExport {
    particle_hash: String,
    is_compliant: bool,
}

#[derive(Serialize)]
struct MetricsSchemaExport {
    version: String,
    metric_families: Vec<String>,
}

#[derive(Serialize)]
struct ResearchManifest {
    date: NaiveDate,
    host_did: String,
    bostrom_address: String,
    git_commit: String,
    crate_versions: Vec<(String, String)>,
    upgrades: Vec<ResearchUpgrade>,
    bci_snapshots: BciSnapshotsExport,
    aln_particles: Vec<AlnParticleExport>,
    metrics_schema: MetricsSchemaExport,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let date = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d").expect("invalid date");

    let registry = UpgradeRegistry::load_default().expect("registry");
    let upgrades = registry
        .all_descriptors()
        .into_iter()
        .map(export_upgrade)
        .collect();

    let bci_snapshots = aggregate_bci_snapshots(date);
    let aln_particles = export_aln_particles(&registry);
    let metrics_schema = export_metrics_schema();

    let manifest = ResearchManifest {
        date,
        host_did: std::env::var("HOST_DID").unwrap(),
        bostrom_address: std::env::var("BOSTROM_ADDRESS").unwrap(),
        git_commit: current_git_commit(),
        crate_versions: collect_crate_versions(),
        upgrades,
        bci_snapshots,
        aln_particles,
        metrics_schema,
    };

    let out_path = PathBuf::from(format!("research/{}-manifest.json", date));
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    fs::write(out_path, json).unwrap();
}
