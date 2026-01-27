use crate::security::CivicClass;
use augdoctorpolicies::shotlevelpolicy::ShotLevel;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CivicAuditEntry {
    pub timestamp_utc: String,
    pub civic_tags: Vec<String>,
    pub civic_class: CivicClass,
    pub reward_multiplier: f64,
    pub eco_cost: f64,
    pub eco_band: String,
    pub lifeforce_ok: bool,
    pub brain_delta_abs: f64,
    pub wave_delta_abs: f64,
    pub nano_delta_abs: f64,
    pub smart_delta_abs: f64,
    pub shot_level: ShotLevel,
    pub handshake_phase: String,
}

fn eco_band_label(eco_cost: f64) -> String {
    if eco_cost <= 1_000.0 {
        "low".to_string()
    } else if eco_cost <= 5_000.0 {
        "medium".to_string()
    } else {
        "high".to_string()
    }
}

/// Append one entry to a host-local JSONL audit file, enforcing max_entries.
/// No host IDs or token balances are stored, only deltas and civic metadata.[file:1]
pub fn append_civic_audit_entry<P: AsRef<Path>>(
    path: P,
    entry: &CivicAuditEntry,
    max_entries: usize,
) -> std::io::Result<()> {
    let path = path.as_ref();

    // Count existing lines if file exists, trimming from the start if over limit.
    if path.exists() {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().flatten().collect();
        if lines.len() >= max_entries {
            let drop_count = lines.len() + 1 - max_entries;
            lines.drain(0..drop_count);
        }
        // Rewrite truncated file.
        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        for l in &lines {
            writeln!(f, "{l}")?;
        }
    }

    // Append new line.
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(entry)?;
    writeln!(f, "{line}")?;

    Ok(())
}
