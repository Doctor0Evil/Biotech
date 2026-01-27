use bci_ledger_service::civic_audit::CivicAuditEntry;
use bci_ledger_service::security::CivicClass;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CivicTagStats {
    pub tag: String,
    pub count: u64,
    pub heroic_count: u64,
    pub good_count: u64,
    pub neutral_count: u64,
    pub disallowed_count: u64,
    pub lifeforce_ok_count: u64,
    pub low_eco_count: u64,
    pub medium_eco_count: u64,
    pub high_eco_count: u64,
    pub avg_reward_multiplier: f64,
}

impl CivicTagStats {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            count: 0,
            heroic_count: 0,
            good_count: 0,
            neutral_count: 0,
            disallowed_count: 0,
            lifeforce_ok_count: 0,
            low_eco_count: 0,
            medium_eco_count: 0,
            high_eco_count: 0,
            avg_reward_multiplier: 0.0,
        }
    }

    pub fn register(&mut self, entry: &CivicAuditEntry) {
        self.count += 1;
        match entry.civic_class {
            CivicClass::CivicHeroic => self.heroic_count += 1,
            CivicClass::CivicGood => self.good_count += 1,
            CivicClass::Neutral => self.neutral_count += 1,
            CivicClass::Disallowed => self.disallowed_count += 1,
        }
        if entry.lifeforce_ok {
            self.lifeforce_ok_count += 1;
        }
        match entry.eco_band.as_str() {
            "low" => self.low_eco_count += 1,
            "medium" => self.medium_eco_count += 1,
            "high" => self.high_eco_count += 1,
            _ => {}
        }

        // Incremental average of reward_multiplier.
        let n = self.count as f64;
        self.avg_reward_multiplier =
            ((n - 1.0) * self.avg_reward_multiplier + entry.reward_multiplier) / n;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub multiplier_min: f64,
    pub multiplier_max: f64,
    pub heroic_base: f64,
    pub good_base: f64,
    pub neutral_base: f64,
    pub eco_low_bonus: f64,
    pub min_samples_for_adjust: u64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            multiplier_min: 0.0,
            multiplier_max: 4.0,
            heroic_base: 3.0,
            good_base: 1.5,
            neutral_base: 1.0,
            eco_low_bonus: 1.25,
            min_samples_for_adjust: 16,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestedProfile {
    pub heroic_tags: Vec<String>,
    pub heroic_multiplier: f64,
    pub good_tags: Vec<String>,
    pub good_multiplier: f64,
    pub neutral_multiplier: f64,
    pub disallowed_tags: Vec<String>,
    pub eco_low_threshold: f64,
    pub eco_low_bonus: f64,
    pub hex_proofs: Vec<String>,
}

/// Analyze aggregated stats and produce a conservative suggested profile.
pub fn suggest_profile(
    stats: &HashMap<String, CivicTagStats>,
    cfg: &AnalysisConfig,
) -> SuggestedProfile {
    let mut heroic_tags = Vec::new();
    let mut good_tags = Vec::new();
    let mut disallowed_tags = Vec::new();

    for (tag, s) in stats.iter() {
        if s.count < cfg.min_samples_for_adjust {
            continue;
        }

        let lifeforce_ok_ratio = s.lifeforce_ok_count as f64 / s.count as f64;
        let low_eco_ratio = s.low_eco_count as f64 / s.count as f64;
        let disallowed_ratio = s.disallowed_count as f64 / s.count as f64;

        if disallowed_ratio > 0.1 {
            disallowed_tags.push(tag.clone());
            continue;
        }

        if lifeforce_ok_ratio > 0.9 && low_eco_ratio > 0.6 {
            heroic_tags.push(tag.clone());
        } else if lifeforce_ok_ratio > 0.8 && low_eco_ratio > 0.4 {
            good_tags.push(tag.clone());
        }
    }

    let heroic_multiplier = cfg
        .heroic_base
        .min(cfg.multiplier_max)
        .max(cfg.multiplier_min);
    let good_multiplier = cfg
        .good_base
        .min(cfg.multiplier_max)
        .max(cfg.multiplier_min);
    let neutral_multiplier = cfg
        .neutral_base
        .min(cfg.multiplier_max)
        .max(cfg.multiplier_min);

    let eco_low_threshold = 1_000.0;
    let eco_low_bonus = cfg.eco_low_bonus.min(cfg.multiplier_max / heroic_multiplier);

    let hex_proofs = vec![
        "0xC1V1C-ANL-01".to_string(),
        "0xC1V1C-ANL-02".to_string(),
        "0xC1V1C-ANL-03".to_string(),
        "0xC1V1C-ANL-04".to_string(),
        "0xC1V1C-ANL-05".to_string(),
        "0xC1V1C-ANL-06".to_string(),
        "0xC1V1C-ANL-07".to_string(),
        "0xC1V1C-ANL-08".to_string(),
        "0xC1V1C-ANL-09".to_string(),
        "0xC1V1C-ANL-10".to_string(),
    ];

    SuggestedProfile {
        heroic_tags,
        heroic_multiplier,
        good_tags,
        good_multiplier,
        neutral_multiplier,
        disallowed_tags,
        eco_low_threshold,
        eco_low_bonus,
        hex_proofs,
    }
}
