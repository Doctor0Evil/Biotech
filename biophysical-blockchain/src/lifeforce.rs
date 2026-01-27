use crate::types::{
    BioTokenState,
    HostEnvelope,
    SystemAdjustment,
    LifeforceBandSeries,
    LifeforceBand,
    EcoBandProfile,
    SafetyCurveWave,
};
use thiserror::Error;

/// Errors raised by lifeforce and eco guards.
#[derive(Debug, Error)]
pub enum LifeforceError {
    #[error("forbidden: BRAIN would go below host minimum (death condition)")]
    BrainNegative,
    #[error("forbidden: BLOOD would reach or cross zero (unsafe depletion)")]
    BloodDepletion,
    #[error("forbidden: OXYGEN would reach or cross zero (unsafe depletion)")]
    OxygenDepletion,
    #[error("forbidden: SMART autonomy would exceed host smartmax or BRAIN")]
    SmartOverMax,
    #[error("forbidden: NANO exceeds host nanomaxfraction envelope")]
    NanoOverEnvelope,
    #[error("forbidden: eco-cost {0} exceeds host ecoflopslimit")]
    EcoOverLimit(f64),
    #[error("forbidden: lifeforce band is in HardStop")]
    LifeforceHardStop,
    #[error("forbidden: WAVE would exceed safe ceiling under current fatigue")]
    WaveOverSafeCeiling,
    #[error("forbidden: BRAIN below eco-neutral reserve for current eco band")]
    BrainBelowEcoNeutral,
}

/// Compute a scalar fatigue index in [0.0, 1.0] from the latest lifeforce sample.
///
/// - Safe band => fatigue near 0.0.
/// - SoftWarn => moderate fatigue.
/// - HardStop => maximal fatigue (1.0).
fn compute_fatigue_from_lifeforce(series: &LifeforceBandSeries) -> (f64, LifeforceBand) {
    if let Some(last) = series.samples.last() {
        let band = last.band.clone();
        let base = match band {
            LifeforceBand::Safe => 0.1_f64,
            LifeforceBand::SoftWarn => 0.5_f64,
            LifeforceBand::HardStop => 1.0_f64,
        };
        // Add a small term from (1 - lifeforce_l) so low lifeforce implies higher fatigue.
        let lf = last.lifeforce_l.clamp(0.0, 1.0) as f64;
        let fatigue = (base + (1.0 - lf) * 0.4).clamp(0.0, 1.0);
        (fatigue, band)
    } else {
        // No history yet: assume low fatigue but be conservative.
        (0.2_f64, LifeforceBand::Safe)
    }
}

/// Compute eco-neutral BRAIN reserve required for the host, given the eco band profile.
fn econeutral_brain_required(
    eco_profile: &EcoBandProfile,
    state_brain: f64,
) -> f64 {
    eco_profile.econeutral_brain_required(state_brain)
}

/// Apply a system adjustment under lifeforce, WAVE, eco, and SMART/NANO invariants.
///
/// This is the canonical place where token-like balances are changed.
pub fn apply_lifeforce_guarded_adjustment(
    state: &mut BioTokenState,
    env: HostEnvelope,
    adj: SystemAdjustment,
    lifeforce_series: &LifeforceBandSeries,
    eco_profile: &EcoBandProfile,
    wave_curve: &SafetyCurveWave,
) -> Result<(), LifeforceError> {
    // Pre-compute fatigue and lifeforce band.
    let (fatigue, band) = compute_fatigue_from_lifeforce(lifeforce_series);

    // Hard-stop band forbids any mutating operation.
    if matches!(band, LifeforceBand::HardStop) {
        return Err(LifeforceError::LifeforceHardStop);
    }

    // Propose new state.
    let newbrain  = state.brain  + adj.deltabrain;
    let newwave   = state.wave   + adj.deltawave;
    let newblood  = state.blood  + adj.deltablood;
    let newoxygen = state.oxygen + adj.deltaoxygen;
    let newnano   = state.nano   + adj.deltanano;
    let newsmart  = state.smart  + adj.deltasmart;

    // BRAIN floor (death condition).
    if newbrain < env.brainmin {
        return Err(LifeforceError::BrainNegative);
    }

    // BLOOD / OXYGEN hard floors.
    if newblood <= env.bloodmin {
        return Err(LifeforceError::BloodDepletion);
    }
    if newoxygen <= env.oxygenmin {
        return Err(LifeforceError::OxygenDepletion);
    }

    // Eco-neutral BRAIN reserve: require that newbrain stays above this threshold.
    let eco_required = econeutral_brain_required(eco_profile, newbrain);
    if newbrain < eco_required {
        return Err(LifeforceError::BrainBelowEcoNeutral);
    }

    // WAVE ceiling from BRAIN and fatigue (dracula_wave-style protection).
    let safe_wave_ceiling = wave_curve.safe_wave_ceiling(newbrain, fatigue);
    if newwave > safe_wave_ceiling {
        return Err(LifeforceError::WaveOverSafeCeiling);
    }

    // SMART: bounded by host smartmax and by BRAIN.
    if newsmart > env.smartmax || newsmart > newbrain {
        return Err(LifeforceError::SmartOverMax);
    }

    // NANO envelope: fraction of 1.0, capped by host nanomaxfraction.
    if newnano > env.nanomaxfraction {
        return Err(LifeforceError::NanoOverEnvelope);
    }

    // Eco envelope: forbid operations that exceed eco budget.
    if adj.ecocost > env.ecoflopslimit {
        return Err(LifeforceError::EcoOverLimit(adj.ecocost));
    }

    // All checks passed; commit state.
    state.brain  = newbrain;
    state.wave   = newwave;
    state.blood  = newblood;
    state.oxygen = newoxygen;
    state.nano   = newnano;
    state.smart  = newsmart;

    Ok(())
}
