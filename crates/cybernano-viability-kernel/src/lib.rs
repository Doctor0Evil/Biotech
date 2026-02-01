//! cybernano-viability-kernel
//!
//! Universal Rust crate for loading ALN-style viability kernels
//! and enforcing Ax <= b over an 8D biophysical microspace.
//!
//! Designed to be called from:
//! - OrganicCPU orchestrators,
//! - CyberNano / cyberswarm guests,
//! - AI-chat automations that need a "Tsafe-oracle".

mod model;
mod loader;
mod check;

pub use model::{
    AxisSpec,
    ViabilityKernelSpec,
    KernelSet,
    ModeKernelId,
    DisturbanceSet,
    StateVector8,
    ControlVector,
    ViabilityError,
};
pub use loader::{ViabilityKernelLoader, FileSystemLoader};
pub use check::{is_viable, safe_filter};
