use crate::sealed::inner::Sealed;
use crate::types::{
    BioTokenState,
    HostEnvelope,
    IdentityHeader,
    SystemAdjustment,
    LifeforceBandSeries,
    EcoBandProfile,
    SafetyCurveWave,
};
use crate::innerledger::{InnerLedger, InnerLedgerError};
use crate::lifeforce::{LifeforceError, apply_lifeforce_guarded_adjustment};
use crate::access::validate_identity_for_inner_ledger;
use crate::consensus::{LedgerEvent, hashstate};

/// Trait for components that can apply a guarded biophysical mutation
/// to a token state under a host envelope, lifeforce bands, and eco profile.
pub trait LifeforceMutator: Sealed {
    fn apply_guarded(
        state: &mut BioTokenState,
        env: &HostEnvelope,
        adj: &SystemAdjustment,
        lifeforce_series: &LifeforceBandSeries,
        eco_profile: &EcoBandProfile,
        wave_curve: &SafetyCurveWave,
    ) -> Result<(), LifeforceError>;
}

/// Trait for components that can perform a full inner-ledger mutation:
/// identity + lifeforce/eco/WAVE validation plus hash-attested commit.
pub trait LedgerMutator: Sealed {
    fn system_apply_guarded(
        ledger: &mut InnerLedger,
        idheader: &IdentityHeader,
        required_knowledge_factor: f32,
        adj: SystemAdjustment,
        timestamputc: &str,
        lifeforce_series: &LifeforceBandSeries,
        eco_profile: &EcoBandProfile,
        wave_curve: &SafetyCurveWave,
    ) -> Result<LedgerEvent, InnerLedgerError>;
}

// --- Sealed implementations ---

impl LifeforceMutator for BioTokenState {
    fn apply_guarded(
        state: &mut BioTokenState,
        env: &HostEnvelope,
        adj: &SystemAdjustment,
        lifeforce_series: &LifeforceBandSeries,
        eco_profile: &EcoBandProfile,
        wave_curve: &SafetyCurveWave,
    ) -> Result<(), LifeforceError> {
        apply_lifeforce_guarded_adjustment(
            state,
            env.clone(),
            adj.clone(),
            lifeforce_series,
            eco_profile,
            wave_curve,
        )
    }
}

impl LedgerMutator for InnerLedger {
    fn system_apply_guarded(
        ledger: &mut InnerLedger,
        idheader: &IdentityHeader,
        required_knowledge_factor: f32,
        adj: SystemAdjustment,
        timestamputc: &str,
        lifeforce_series: &LifeforceBandSeries,
        eco_profile: &EcoBandProfile,
        wave_curve: &SafetyCurveWave,
    ) -> Result<LedgerEvent, InnerLedgerError> {
        // 1. Identity / tier / role guard.
        validate_identity_for_inner_ledger(idheader.clone(), required_knowledge_factor)?;

        // 2. Biophysical lifeforce + eco + WAVE invariants.
        BioTokenState::apply_guarded(
            &mut ledger.state,
            &ledger.env,
            &adj,
            lifeforce_series,
            eco_profile,
            wave_curve,
        )?;

        // 3. Hash & event commit.
        let newhash = hashstate(
            ledger.env.hostid.as_str(),
            ledger.env.clone(),
            ledger.state.clone(),
        );

        let event = LedgerEvent {
            hostid: ledger.env.hostid.clone(),
            prevstatehash: ledger.laststatehash.clone(),
            newstatehash: newhash.clone(),
            adjustment: adj.clone(),
            timestamputc: timestamputc.to_string(),
            attestedby: idheader.issuerdid.clone(),
        };

        ledger.laststatehash = newhash;
        Ok(event)
    }
}
