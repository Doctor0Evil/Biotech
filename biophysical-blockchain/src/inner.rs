mod sealed {
    pub trait Sealed {}
}

use crate::{types::*, turns::DailyTurnState};
use sealed::Sealed;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LifeforceError {
    #[error("lifeforce invariant violated")]
    Invariant,
    #[error("eco budget exceeded")]
    Eco,
    #[error("daily evolution turn limit reached")]
    TurnLimit,
}

pub trait LifeforceMutator: Sealed {
    fn apply_guarded(
        &self,
        state: &mut BioTokenState,
        env: &HostEnvelope,
        adj: SystemAdjustment,
    ) -> Result<(), LifeforceError>;
}

pub struct InnerLedger {
    pub env: HostEnvelope,
    pub state: BioTokenState,
    daily_turns: DailyTurnState,
}

impl Sealed for InnerLedger {}

impl LifeforceMutator for InnerLedger {
    fn apply_guarded(
        &self,
        state: &mut BioTokenState,
        env: &HostEnvelope,
        adj: SystemAdjustment,
    ) -> Result<(), LifeforceError> {
        let mut next = state.clone();

next.brain += adj.deltabrain;
next.wave += adj.deltawave;
next.nano += adj.deltanano;
next.smart += adj.deltasmart;

// Hard lifeforce floors
if next.brain < self.env.brainmin
    || next.blood < self.env.bloodmin
    || next.oxygen < self.env.oxygenmin
{
    return Err(LifeforceError::Invariant);
}

// Nano / SMART envelopes
if next.nano > self.env.nanomaxfraction
    || next.smart > self.env.smartmax
{
    return Err(LifeforceError::Invariant);
}

// Eco budget
if adj.ecocost > self.env.ecoflopslimit {
    return Err(LifeforceError::Eco);
}


        if next.nano > env.nanomaxfraction
            || next.smart > env.smartmax
        {
            return Err(LifeforceError::Invariant);
        }

        if adj.ecocost > env.ecoflopslimit {
            return Err(LifeforceError::Eco);
        }

        *state = next;
        Ok(())
    }
}

impl InnerLedger {
    pub fn new(env: HostEnvelope, state: BioTokenState) -> Self {
        Self { env, state, daily_turns: DailyTurnState::new_today() }
    }

    pub fn system_apply_evolution(
        &mut self,
        adj: SystemAdjustment,
        max_daily_turns: u8,
    ) -> Result<(), LifeforceError> {
        const HARD_MAX_TURNS: u8 = 10;
        let allowed = max_daily_turns.min(HARD_MAX_TURNS);
        if !self.daily_turns.can_consume_turn(allowed) {
            return Err(LifeforceError::TurnLimit);
        }
        self.apply_guarded(&mut self.state, &self.env, adj)
    }
}
