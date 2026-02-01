use crate::kernel_spec::ViabilityKernelSpec;
use serde_yaml::from_reader;
use std::fs::File;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("spec invalid: {0}")]
    InvalidSpec(String),

    #[error("Tsafe error: {0}")]
    Tsafe(String),
}

pub struct HarnessConfig<'a> {
    pub kernel_path: &'a Path,
    pub fake_state: Vec<f64>,
}

pub fn run_once(cfg: HarnessConfig<'_>) -> Result<bool, HarnessError> {
    let file = File::open(cfg.kernel_path)?;
    let spec: ViabilityKernelSpec = from_reader(file)?;
    spec.validate().map_err(HarnessError::InvalidSpec)?;

    if cfg.fake_state.len() != spec.state_dim {
        return Err(HarnessError::InvalidSpec(format!(
            "fake_state dim {} != state_dim {}",
            cfg.fake_state.len(),
            spec.state_dim
        )));
    }

    // --- Tsafe wiring: adapt this to your actual API ---
    // Example assuming tsafe exposes a struct ViabilityKernel { A, b }
    // and a method `is_safe(x: &[f64]) -> bool`.

    let kernel = tsafe::ViabilityKernel::new(spec.A.clone(), spec.b.clone())
        .map_err(|e| HarnessError::Tsafe(e.to_string()))?;

    let ok = kernel
        .is_safe(&cfg.fake_state)
        .map_err(|e| HarnessError::Tsafe(e.to_string()))?;

    Ok(ok)
}
