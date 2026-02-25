#![cfg(kani)]

use kani::any;

use bioscale_upgrade_store::{HostBudget, UpgradeDescriptor};
use cyberswarm_neurostack::evolution_window::{
    BioscaleEvolutionWindow, DailyBioscaleEvolutionState,
};
use cyberswarm_neurostack::safety::check_upgrade_safe_in_window; // your existing guard.

#[kani::proof]
fn check_evolution_window_safety() {
    // Symbolic host and window; ceilings are conservative.
    let host = HostBudget {
        daily_energy_joules: 30_000.0,
        remaining_energy_joules: 30_000.0,
        daily_protein_grams: 80.0,
        remaining_protein_grams: 80.0,
        ..HostBudget::default()
    };

    let window = BioscaleEvolutionWindow {
        valid_from: std::time::UNIX_EPOCH,
        valid_until: std::time::UNIX_EPOCH, // time not relevant here
        max_upgrades_in_window: 3,
        max_total_energy_joules: 9_000.0,
        max_total_duty_fraction: 0.4,
    };

    let mut state = DailyBioscaleEvolutionState::default();

    // Bounded 3-step window; Kani chooses arbitrary descriptors.
    for _ in 0..3 {
        let desc: UpgradeDescriptor = any();
        let decision = check_upgrade_safe_in_window(&host, &window, &state, &desc);

        // If approved, register and assert that cumulative ceilings hold.
        if decision.is_approved() {
            state.register_approval(&desc);
            assert!(state.total_energy_joules <= window.max_total_energy_joules + 1e-6);
            assert!(state.total_duty_fraction <= window.max_total_duty_fraction + 1e-6);
        }
    }
}
