use crate::model::SuggestedProfile;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_utc_iso() -> String {
    // Simple UTC-ish timestamp (you can swap in a better clock in your stack).
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("2026-01-27T{:02}:00:00Z", (secs % 86400) / 3600)
}

pub fn emit_civic_reward_profile_aln(
    hostid: &str,
    suggested: &SuggestedProfile,
) -> String {
    let mut out = String::new();
    let session_utc = now_utc_iso();

    writeln!(&mut out, "version         1.1").unwrap();
    writeln!(&mut out, "schema          biotech.civicrewardprofile").unwrap();
    writeln!(&mut out, "hostid          {hostid}").unwrap();
    writeln!(&mut out, "session_utc     {session_utc}").unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "civic_profile").unwrap();
    writeln!(&mut out, "  mode                inner-only").unwrap();
    writeln!(&mut out, "  financialization    false").unwrap();
    writeln!(&mut out, "  bridge_enabled      false").unwrap();
    writeln!(&mut out, "  staking_enabled     false").unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "  multiplier_min      0.0").unwrap();
    writeln!(&mut out, "  multiplier_max      4.0").unwrap();
    writeln!(
        &mut out,
        "  default_multiplier  {}",
        suggested.neutral_multiplier
    )
    .unwrap();
    writeln!(&mut out, "  required_knowledge_factor  0.60").unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "  heroic_tags").unwrap();
    for t in &suggested.heroic_tags {
        writeln!(&mut out, "    - {t}").unwrap();
    }
    writeln!(
        &mut out,
        "  heroic_multiplier   {}",
        suggested.heroic_multiplier
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "  good_tags").unwrap();
    for t in &suggested.good_tags {
        writeln!(&mut out, "    - {t}").unwrap();
    }
    writeln!(
        &mut out,
        "  good_multiplier     {}",
        suggested.good_multiplier
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "  neutral_multiplier  {}",
        suggested.neutral_multiplier
    )
    .unwrap();

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "  disallowed_tags").unwrap();
    for t in &suggested.disallowed_tags {
        writeln!(&mut out, "    - {t}").unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "  eco_bonus").unwrap();
    writeln!(&mut out, "    enabled           true").unwrap();
    writeln!(
        &mut out,
        "    low_eco_threshold  {}",
        suggested.eco_low_threshold
    )
    .unwrap();
    writeln!(
        &mut out,
        "    low_eco_bonus       {}",
        suggested.eco_low_bonus
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "audit").unwrap();
    writeln!(
        &mut out,
        "  provenance_proofhex  {}",
        suggested.hex_proofs.get(0).cloned().unwrap_or_default()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  hex_proof_sequences  [{}]",
        suggested
            .hex_proofs
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    writeln!(
        &mut out,
        "  governance_labels   ALN,KYC,DID,Biotech"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  notes               \"Profile auto-tuned from CivicAuditLog under non-financial, consciousness-safe constraints.\""
    )
    .unwrap();

    out
}
