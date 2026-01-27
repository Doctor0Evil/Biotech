mod model;
mod aln_emit;

use model::{AnalysisConfig, CivicTagStats, SuggestedProfile, suggest_profile};
use aln_emit::emit_civic_reward_profile_aln;
use bci_ledger_service::civic_audit::CivicAuditEntry;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <hostid> <input_audit_jsonl> <output_profile_aln>",
            args[0]
        );
        std::process::exit(1);
    }

    let hostid = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut stats: HashMap<String, CivicTagStats> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: CivicAuditEntry = serde_json::from_str(&line)?;
        for tag in &entry.civic_tags {
            let key = tag.to_lowercase();
            let stat = stats
                .entry(key.clone())
                .or_insert_with(|| CivicTagStats::new(&key));
            stat.register(&entry);
        }
    }

    let cfg = AnalysisConfig::default();
    let suggested: SuggestedProfile = suggest_profile(&stats, &cfg);

    let aln = emit_civic_reward_profile_aln(hostid, &suggested);
    let mut out = File::create(output_path)?;
    out.write_all(aln.as_bytes())?;

    Ok(())
}
