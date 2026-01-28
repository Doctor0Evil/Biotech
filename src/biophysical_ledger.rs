use core::fmt;

/// Core biophysical tokens: strictly per-host, non-financial, non-transferable.
/// All values are normalized to [0.0, 1.0] where possible, or to bounded ranges.
#[derive(Clone, Copy, Debug)]
pub struct BioTokenState {
    // Cognitive / learning capacity and history.
    pub brain: f32,        // 0.0..1.0

    // Workload safety and neurorights envelope.
    pub wave: f32,         // 0.0..1.0

    // Lifeforce guardrails (no cross-host depletion).
    pub blood: f32,        // 0.0..1.0, hard floor enforced
    pub oxygen: f32,       // 0.0..1.0, hard floor enforced

    // Nanotech / augmentation envelope (strictly local).
    pub nano: f32,         // 0.0..1.0

    // Autonomy / SMART control band (0 = off, 1 = full auto).
    pub smart: f32,        // 0.0..1.0

    // Evolution metric: non-mintable, non-transferable, per-host only.
    pub evolve: f32,       // 0.0..1.0, governed by brain + wave + eco participation
}

/// Pain / intensity preferences: strictly per-host, never exported as defaults.
#[derive(Clone, Copy, Debug)]
pub struct PainProfile {
    // Preferred upper bound for pain; still constrained by safety floors.
    pub preferred_pain_upper_band: f32, // 0.0..1.0

    // Desired stress band (e.g., 0.0..1.0); allows “high challenge” for this host.
    pub desired_stress_band: f32,       // 0.0..1.0

    // Whether this host explicitly allows high-pain training regimes.
    pub allow_high_pain_training: bool,
}

/// Lifeforce and safety bands for this host.
#[derive(Clone, Copy, Debug)]
pub enum LifeforceBand {
    Safe,
    SoftWarn,
    HardStop,
}

/// DID + role + context envelope for any mutating call.
#[derive(Clone, Debug)]
pub struct IdentityHeader {
    pub issuer_did: String,
    pub subject_role: SubjectRole,
    pub network_tier: NetworkTier,
    pub knowledge_factor: f32, // 0.0..1.0
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubjectRole {
    AugmentedCitizen,
    AuthorizedResearcher,
    SystemDaemon,
    ExternalService,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkTier {
    Trusted,
    LabSandbox,
    External,
}

/// Consent shard: explicit proof that the host approved a given mutation.
#[derive(Clone, Debug)]
pub struct DemonstratedConsentShard {
    pub host_did: String,
    pub action: ConsentAction,
    pub magnitude: f32,          // requested magnitude, normalized
    pub valid_from_epoch: i64,
    pub valid_until_epoch: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsentAction {
    AdjustPainProfile,
    AdjustSmartAutonomy,
    ApplyEvolutionUpgrade,
}

/// Lightweight proof that the consent shard is valid (e.g., signed off-chain).
#[derive(Clone, Debug)]
pub struct ConsentProof {
    pub shard_id: String,
    pub verified: bool,
}

/// System-level adjustment proposal, always per-host, never cross-host.
#[derive(Clone, Debug)]
pub struct SystemAdjustment {
    pub target_host_did: String,

    // Optional deltas; absence means "no change".
    pub delta_brain: Option<f32>,
    pub delta_wave: Option<f32>,
    pub delta_blood: Option<f32>,
    pub delta_oxygen: Option<f32>,
    pub delta_nano: Option<f32>,
    pub delta_smart: Option<f32>,
    pub delta_evolve: Option<f32>,

    // Optional pain/intensity patch.
    pub new_pain_profile: Option<PainProfile>,
}

/// Governance invariants baked into the ledger: no transfer, no stake,
/// no cross-host mutation, mandatory identity + consent for evolution.
#[derive(Clone, Debug)]
pub struct BiophysicalLedger {
    pub host_did: String,
    pub tokens: BioTokenState,
    pub pain: PainProfile,

    // Hard floors for BLOOD and OXYGEN (lifeforce, non-negotiable).
    pub blood_floor: f32,
    pub oxygen_floor: f32,
}

#[derive(Debug)]
pub enum LedgerError {
    InvalidIdentity(&'static str),
    ConsentRequired(&'static str),
    ConsentMismatch(&'static str),
    SafetyViolation(&'static str),
    CrossHostMutationForbidden,
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::InvalidIdentity(msg) => write!(f, "InvalidIdentity: {}", msg),
            LedgerError::ConsentRequired(msg) => write!(f, "ConsentRequired: {}", msg),
            LedgerError::ConsentMismatch(msg) => write!(f, "ConsentMismatch: {}", msg),
            LedgerError::SafetyViolation(msg) => write!(f, "SafetyViolation: {}", msg),
            LedgerError::CrossHostMutationForbidden => {
                write!(f, "CrossHostMutationForbidden")
            }
        }
    }
}

impl BiophysicalLedger {
    pub fn new(host_did: String, tokens: BioTokenState, pain: PainProfile) -> Self {
        // Conservative default floors; can be tuned but never removed.
        let blood_floor = 0.20;
        let oxygen_floor = 0.20;

        Self {
            host_did,
            tokens,
            pain,
            blood_floor,
            oxygen_floor,
        }
    }

    /// Identity check: only AugmentedCitizen / AuthorizedResearcher / SystemDaemon
    /// on Trusted tier may mutate inner ledger.
    pub fn validate_identity_for_inner_ledger(
        &self,
        id: &IdentityHeader,
    ) -> Result<(), LedgerError> {
        match id.subject_role {
            SubjectRole::AugmentedCitizen
            | SubjectRole::AuthorizedResearcher
            | SubjectRole::SystemDaemon => {}
            _ => return Err(LedgerError::InvalidIdentity("role not allowed")),
        }

        if id.network_tier != NetworkTier::Trusted {
            return Err(LedgerError::InvalidIdentity("network_tier not trusted"));
        }

        if id.knowledge_factor < 0.3 {
            return Err(LedgerError::InvalidIdentity("knowledge_factor too low"));
        }

        Ok(())
    }

    /// Compute LifeforceBand from current BLOOD/OXYGEN levels.
    pub fn lifeforce_band(&self) -> LifeforceBand {
        let b = self.tokens.blood;
        let o = self.tokens.oxygen;

        if b <= self.blood_floor || o <= self.oxygen_floor {
            LifeforceBand::HardStop
        } else if b < 0.35 || o < 0.35 {
            LifeforceBand::SoftWarn
        } else {
            LifeforceBand::Safe
        }
    }

    /// Apply a per-host SystemAdjustment under full governance:
    /// - identity must be valid,
    /// - host must match,
    /// - floors must not be violated,
    /// - evolution/pain/smart changes require consent.
    pub fn apply_adjustment(
        &mut self,
        id: &IdentityHeader,
        adj: &SystemAdjustment,
        consent: Option<(&DemonstratedConsentShard, &ConsentProof)>,
        current_epoch: i64,
    ) -> Result<(), LedgerError> {
        // 1. Identity guard.
        self.validate_identity_for_inner_ledger(id)?;

        // 2. Cross-host protection: adjustments must target this host only.
        if adj.target_host_did != self.host_did {
            return Err(LedgerError::CrossHostMutationForbidden);
        }

        // Clone current state for checks.
        let mut next = self.tokens;
        let mut next_pain = self.pain;

        // 3. Apply deltas locally (no transfer semantics).
        if let Some(d) = adj.delta_brain {
            next.brain = clamp01(next.brain + d);
        }
        if let Some(d) = adj.delta_wave {
            next.wave = clamp01(next.wave + d);
        }
        if let Some(d) = adj.delta_blood {
            next.blood = clamp01(next.blood + d);
        }
        if let Some(d) = adj.delta_oxygen {
            next.oxygen = clamp01(next.oxygen + d);
        }
        if let Some(d) = adj.delta_nano {
            next.nano = clamp01(next.nano + d);
        }
        if let Some(d) = adj.delta_smart {
            next.smart = clamp01(next.smart + d);
        }
        if let Some(d) = adj.delta_evolve {
            // EVOLVE can only be increased and is non-mintable externally:
            // require positive delta and consent.
            if d < 0.0 {
                return Err(LedgerError::SafetyViolation(
                    "negative evolve delta not allowed",
                ));
            }
            next.evolve = clamp01(next.evolve + d);
        }

        // Pain profile override, if provided.
        if let Some(p) = adj.new_pain_profile {
            next_pain = PainProfile {
                preferred_pain_upper_band: clamp01(p.preferred_pain_upper_band),
                desired_stress_band: clamp01(p.desired_stress_band),
                allow_high_pain_training: p.allow_high_pain_training,
            };
        }

        // 4. Enforce lifeforce floors.
        if next.blood < self.blood_floor || next.oxygen < self.oxygen_floor {
            return Err(LedgerError::SafetyViolation(
                "lifeforce floors would be violated",
            ));
        }

        // 5. Consent requirements for sensitive changes.
        let evolve_changed = adj.delta_evolve.is_some();
        let pain_changed = adj.new_pain_profile.is_some();
        let smart_changed = adj.delta_smart.is_some();

        if evolve_changed || pain_changed || smart_changed {
            let (shard, proof) = consent.ok_or(LedgerError::ConsentRequired(
                "evolution, pain, or smart changes require consent",
            ))?;

            if !proof.verified {
                return Err(LedgerError::ConsentMismatch("unverified consent proof"));
            }

            if shard.host_did != self.host_did {
                return Err(LedgerError::ConsentMismatch(
                    "consent shard host mismatch",
                ));
            }

            if current_epoch < shard.valid_from_epoch
                || current_epoch > shard.valid_until_epoch
            {
                return Err(LedgerError::ConsentMismatch("consent shard expired"));
            }

            // Simple action consistency check: in a full implementation, you would
            // match magnitudes more precisely. Here we enforce action type only.
            if evolve_changed && shard.action != ConsentAction::ApplyEvolutionUpgrade {
                return Err(LedgerError::ConsentMismatch(
                    "consent shard not for evolution",
                ));
            }
            if pain_changed && shard.action != ConsentAction::AdjustPainProfile {
                return Err(LedgerError::ConsentMismatch(
                    "consent shard not for pain profile",
                ));
            }
            if smart_changed && shard.action != ConsentAction::AdjustSmartAutonomy {
                return Err(LedgerError::ConsentMismatch(
                    "consent shard not for smart autonomy",
                ));
            }
        }

        // 6. Commit new state.
        self.tokens = next;
        self.pain = next_pain;

        Ok(())
    }
}

/// Boundary API types: external systems see only redacted summaries and may
/// submit SystemAdjustment proposals; they never see raw tokens or inner state.

/// Summary of host state safe to expose to AI chats / vendors.
#[derive(Clone, Debug)]
pub struct HostSummary {
    pub host_did: String,
    pub stress_band: f32,         // redacted, derived
    pub lifeforce_band: LifeforceBand,
    pub can_increase_challenge: bool,
}

/// Boundary service: wraps BiophysicalLedger and enforces that external callers
/// only interact via redacted summaries and adjustment proposals.
pub struct BoundaryService<'a> {
    pub ledger: &'a mut BiophysicalLedger,
}

impl<'a> BoundaryService<'a> {
    pub fn summarize_for_external(&self) -> HostSummary {
        let lf = self.ledger.lifeforce_band();
        let can_increase_challenge = matches!(lf, LifeforceBand::Safe)
            && self.ledger.pain.allow_high_pain_training;

        HostSummary {
            host_did: self.ledger.host_did.clone(),
            stress_band: clamp01(self.ledger.pain.desired_stress_band),
            lifeforce_band: lf,
            can_increase_challenge,
        }
    }

    /// External systems can only submit proposals; the ledger decides.
    pub fn submit_adjustment(
        &mut self,
        id: &IdentityHeader,
        adj: &SystemAdjustment,
        consent: Option<(&DemonstratedConsentShard, &ConsentProof)>,
        current_epoch: i64,
    ) -> Result<(), LedgerError> {
        self.ledger.apply_adjustment(id, adj, consent, current_epoch)
    }
}

fn clamp01(x: f32) -> f32 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}
