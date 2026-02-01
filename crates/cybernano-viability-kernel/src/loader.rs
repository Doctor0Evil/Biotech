use crate::{KernelSet, ViabilityError};
use serde_yaml;
use std::fs;

pub trait ViabilityKernelLoader {
    fn load_kernel_set(&self, path: &str) -> Result<KernelSet, ViabilityError>;
}

pub struct FileSystemLoader;

impl ViabilityKernelLoader for FileSystemLoader {
    fn load_kernel_set(&self, path: &str) -> Result<KernelSet, ViabilityError> {
        let text = fs::read_to_string(path)
            .map_err(|e| ViabilityError::Io(e.to_string()))?;
        serde_yaml::from_str::<KernelSet>(&text)
            .map_err(|e| ViabilityError::Parse(e.to_string()))
    }
}
