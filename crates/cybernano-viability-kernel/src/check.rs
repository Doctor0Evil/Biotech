use crate::{KernelSet, StateVector8, ControlVector, DisturbanceSet, ViabilityError};

fn find_kernel<'a>(ks: &'a KernelSet, kernel_id: &str) -> Option<&'a crate::ViabilityKernelSpec> {
    ks.kernels.iter().find(|k| k.id == kernel_id)
}

fn find_kernel_for_mode<'a>(ks: &'a KernelSet, mode: &str)
    -> Result<&'a crate::ViabilityKernelSpec, ViabilityError>
{
    let mk = ks.mode_map
        .iter()
        .find(|m| m.mode_name == mode)
        .ok_or_else(|| ViabilityError::ModeNotMapped(mode.to_string()))?;
    find_kernel(ks, &mk.kernel_id)
        .ok_or_else(|| ViabilityError::KernelNotFound(mk.kernel_id.clone()))
}

/// Check Ax<=b for all constraints, with a conservative disturbance box.
pub fn is_viable(
    kernel_set: &KernelSet,
    mode: &str,
    state: &StateVector8,
    disturbance: &DisturbanceSet,
) -> Result<bool, ViabilityError> {
    let k = find_kernel_for_mode(kernel_set, mode)?;
    if k.A.len() != k.b.len() {
        return Err(ViabilityError::DimensionMismatch);
    }

    let x = &state.0;
    for (row, &b_i) in k.A.iter().zip(k.b.iter()) {
        let mut lhs = 0.0_f32;
        let mut worst_case = 0.0_f32;
        for j in 0..8 {
            lhs += row[j] * x[j];
            // worst-case disturbance sign
            let d = disturbance.max_delta[j];
            let term_plus = row[j] * (x[j] + d);
            let term_minus = row[j] * (x[j] - d);
            worst_case += term_plus.max(term_minus) - row[j] * x[j];
        }
        if lhs + worst_case > b_i {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Naive safe_filter: if nominal control violates constraints,
/// return zero vector (no-op). Replace with smarter projection later.
pub fn safe_filter(
    kernel_set: &KernelSet,
    mode: &str,
    state: &StateVector8,
    disturbance: &DisturbanceSet,
    nominal_control: &ControlVector,
) -> Result<ControlVector, ViabilityError> {
    let next_state = {
        let mut x_next = state.0;
        for j in 0..8 {
            x_next[j] += nominal_control.0[j];
        }
        StateVector8(x_next)
    };

    if is_viable(kernel_set, mode, &next_state, disturbance)? {
        Ok(nominal_control.clone())
    } else {
        Ok(ControlVector([0.0; 8]))
    }
}
