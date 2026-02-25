#![forbid(unsafe_code)]

use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use bioscale_upgrade_store::UpgradeDescriptor;

/// Per-day timebox and ceilings – this is what "git pull + branch" defines.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioscaleEvolutionWindow {
    pub valid_from: SystemTime,
    pub valid_until: SystemTime,
    pub max_upgrades_in_window: u32,
    pub max_total_energy_joules: f64,
    pub max_total_duty_fraction: f64,
}

impl BioscaleEvolutionWindow {
    pub fn for_utc_date(date_utc: &str) -> Self {
        // In practice, compute 24h interval from date_utc; here, 12h centered.
        let now = SystemTime::now();
        let twelve_h = Duration::from_secs(12 * 3600);
        BioscaleEvolutionWindow {
            valid_from: now,
            valid_until: now + twelve_h,
            max_upgrades_in_window: 64,
            max_total_energy_joules: 30_000.0,
            max_total_duty_fraction: 0.80,
        }
    }
}

/// Accumulator used by guards + Kani to prove window invariants.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DailyBioscaleEvolutionState {
    pub approved_ids: Vec<String>,
    pub total_energy_joules: f64,
    pub total_duty_fraction: f64,
}

impl DailyBioscaleEvolutionState {
    pub fn register_approval(&mut self, desc: &UpgradeDescriptor) {
        use bioscale_upgrade_store::{EnergyCost, MlPassSchedule};

        let e: f64 = desc
            .energy_costs
            .iter()
            .map(|c: &EnergyCost| c.joules)
            .sum();
        self.total_energy_joules += e;

        let MlPassSchedule {
            max_continuous_window,
            min_interval,
            ..
        } = desc.ml_schedule;
        let duty = (max_continuous_window.as_secs_f64()
            / min_interval.as_secs_f64())
            .min(1.0);
        self.total_duty_fraction += duty;
        self.approved_ids.push(desc.id.as_str().to_string());
    }
}
