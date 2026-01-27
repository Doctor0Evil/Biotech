use prometheus::{register_counter_vec, register_gauge_vec, CounterVec, GaugeVec};

pub struct GuardMetrics {
    pub guards_scans_total: CounterVec,
    pub guard_blocks_total: CounterVec,
    pub bci_duty_cycle_current_session: GaugeVec,
    pub bci_duty_ceiling_violations_total: CounterVec,
    pub bci_thermal_delta_celsius: GaugeVec,
    pub bci_thermal_ceiling_violations_total: CounterVec,
    pub bci_envelope_breach_total: CounterVec,
}

impl GuardMetrics {
    pub fn new() -> Self {
        let labels = &["domain", "policy_id", "upgrade_id", "host_id"];

        Self {
            guards_scans_total: register_counter_vec!(
                "guards_scans_total",
                "Total guard evaluations by domain and policy",
                labels
            ).unwrap(),
            guard_blocks_total: register_counter_vec!(
                "guard_blocks_total",
                "Total guard blocks by domain, policy and reason",
                &["domain", "policy_id", "reason", "upgrade_id", "host_id"]
            ).unwrap(),
            bci_duty_cycle_current_session: register_gauge_vec!(
                "bci_duty_cycle_current_session",
                "Current BCI duty cycle fraction",
                &["host_id"]
            ).unwrap(),
            bci_duty_ceiling_violations_total: register_counter_vec!(
                "bci_duty_ceiling_violations_total",
                "BCI duty cycle ceiling violations",
                &["policy_id", "host_id"]
            ).unwrap(),
            bci_thermal_delta_celsius: register_gauge_vec!(
                "bci_thermal_delta_celsius",
                "Current BCI thermal delta vs baseline",
                &["host_id"]
            ).unwrap(),
            bci_thermal_ceiling_violations_total: register_counter_vec!(
                "bci_thermal_ceiling_violations_total",
                "Thermal ceiling violations for BCI",
                &["policy_id", "host_id"]
            ).unwrap(),
            bci_envelope_breach_total: register_counter_vec!(
                "bci_envelope_breach_total",
                "Any breach of BCI envelopes (energy, protein, thermal, HRV, EEG)",
                &["dimension", "policy_id", "host_id"]
            ).unwrap(),
        }
    }
}
