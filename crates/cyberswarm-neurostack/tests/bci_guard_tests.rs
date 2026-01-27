use std::time::{Duration, SystemTime};
use bioscale_upgrade_store::{HostBudget, UpgradeDecision};
use cyberswarm_neurostack::{
    BciHostSnapshot, BciAdapterV2, evaluate_bci_upgrade_with_snapshot,
};

#[test]
fn bci_upgrade_denied_on_high_duty_cycle() {
    let host = HostBudget::with_energy_and_protein(20_000.0, 2_000_000);
    let adapter = BciAdapterV2 {
        energy_joules: None,
        protein_aa: None,
        max_core_celsius: None,
    };
    let desc = adapter.descriptor();

    let snap = BciHostSnapshot {
        eeg_load: 0.5,
        hrv_ms: 40.0,
        core_temp_c: 37.0,
        local_temp_c: 37.2,
        duty_cycle: 0.9, // above 0.4
        pain_vas: 1.0,
        inflammation_score: 0.5,
        observed_at: SystemTime::now(),
    };

    let decision = evaluate_bci_upgrade_with_snapshot(
        &host,
        &desc,
        &snap,
        SystemTime::now() + Duration::from_secs(60),
    );

    matches!(decision, UpgradeDecision::Denied { .. });
}
