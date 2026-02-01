use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AxisSpec {
    pub name: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViabilityKernelSpec {
    pub id: String,
    pub description: String,
    /// Matrix A: rows of length 8 (for 8D state).
    pub A: Vec<[f32; 8]>,
    /// Vector b: length matches A.len().
    pub b: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModeKernelId {
    pub mode_name: String,
    pub kernel_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelSet {
    pub subject_id: String,
    pub version: String,
    pub axes: [AxisSpec; 8],
    pub kernels: Vec<ViabilityKernelSpec>,
    pub mode_map: Vec<ModeKernelId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateVector8(pub [f32; 8]);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlVector(pub [f32; 8]);

/// Simple disturbance bounding box.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisturbanceSet {
    pub max_delta: [f32; 8],
}

#[derive(Debug, Error)]
pub enum ViabilityError {
    #[error("Kernel not found: {0}")]
    KernelNotFound(String),
    #[error("Mode not mapped: {0}")]
    ModeNotMapped(String),
    #[error("Dimension mismatch in Ax<=b data")]
    DimensionMismatch,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
}
