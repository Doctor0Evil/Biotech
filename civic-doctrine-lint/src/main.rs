use anyhow::{anyhow, Result};
use regex::Regex;
use std::{env, fs, path::Path};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let root = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let root_path = Path::new(&root).parent().unwrap_or(Path::new("."));
    run_lints(root_path)
}

fn run_lints(root: &Path) -> Result<()> {
    let forbidden = vec![
        Regex::new(r"\btransfer\b").unwrap(),
        Regex::new(r"\bstake\b").unwrap(),
        Regex::new(r"\bbridge\b").unwrap(),
        Regex::new(r"\bwallet\b").unwrap(),
        Regex::new(r"Balance\s*<").unwrap(),
        Regex::new(r"CrossHost").unwrap(),
    ];

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if !is_rust_source(path) {
            continue;
        }
        let contents = fs::read_to_string(path)?;
        for re in &forbidden {
            if re.is_match(&contents) {
                return Err(anyhow!(
                    "civic-doctrine-lint: forbidden pattern {:?} in {}",
                    re.as_str(),
                    path.display()
                ));
            }
        }
    }

    Ok(())
}

fn is_rust_source(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => true,
        _ => false,
    }
}
