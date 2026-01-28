//! NANO Donation Router (non-financial, per-host)
//!
//! This module turns surplus NANO capacity into scheduled
//! "donation jobs" (compute or tooling for approved charities / labs)
//! without ever transferring tokens between hosts.
//!
//! Invariants:
//! - NANO never leaves this host's BioTokenState.
//! - Lifeforce + metabolic floors must be respected before any job.
//! - Only DID/ALN/Bostrom identities with allowed roles can register
//!   or receive donation jobs.
//! - All donations are logged as civic, eco-positive events.

use std::collections::HashMap;

use crate::biophysicalruntime::{BioTokenState, RuntimeError};
use crate::lifeforcesafety::LifeforceState;
use crate::alndid::{ALNDID, AccessEnvelope, DIDDirectory, RoleClass};

/// High-level classification of donation beneficiaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DonationKind {
    ScientificNonProfit,
    PublicHealthLab,
    OpenSourceTooling,
    ClimateModeling,
}

/// Static profile for an approved beneficiary.
#[derive(Debug, Clone)]
pub struct DonationBeneficiary {
    pub org_id: String,
    pub did: ALNDID,
    pub kind: DonationKind,
    /// Hard cap of NANO *fraction* this org may consume per night (0–1).
    pub max_nano_fraction_per_night: f64,
    /// Whether this beneficiary is currently active.
    pub active: bool,
}

/// A single scheduled donation job on this host.
#[derive(Debug, Clone)]
pub struct DonationJob {
    pub job_id: String,
    pub beneficiary_org: String,
    pub beneficiary_did: ALNDID,
    /// Fraction of current NANO earmarked for this job (0–1).
    pub nano_fraction: f64,
    /// Estimated eco cost in FLOPs-equivalent (host-local units).
    pub eco_cost_flops: u64,
    /// High-level description for audit.
    pub label: String,
}

/// Nightly context for routing donations.
#[derive(Debug, Clone)]
pub struct DonationContext {
    /// 0–1, surplus NANO capacity after all host-local needs.
    pub surplus_nano_fraction01: f64,
    /// 0–1, eco band inverted (1 = very eco-positive night).
    pub eco_alignment01: f64,
    /// Whether host explicitly opted in for donation this window.
    pub host_opt_in: bool,
}

/// Lightweight audit record for a donation decision.
#[derive(Debug, Clone)]
pub struct DonationAudit {
    pub applied_jobs: Vec<DonationJob>,
    pub total_nano_spent: u64,
    pub total_eco_cost_flops: u64,
    pub reason: String,
}

/// Access policy for using the router.
#[derive(Debug, Clone)]
pub struct DonationPolicy {
    /// Minimum biophysics knowledge score required to *configure* router.
    pub min_admin_knowledge: f64,
    /// Minimum eco-alignment score before any donation is allowed.
    pub min_eco_alignment: f64,
    /// Maximum combined NANO fraction per night that may be donated.
    pub max_total_nano_fraction_per_night: f64,
}

impl Default for DonationPolicy {
    fn default() -> Self {
        Self {
            min_admin_knowledge: 0.7,
            min_eco_alignment: 0.6,
            max_total_nano_fraction_per_night: 0.25,
        }
    }
}

/// Router that schedules NANO-backed donation jobs on a single host.
pub struct NanoDonationRouter<D>
where
    D: DIDDirectory,
{
    policy: DonationPolicy,
    lifeforce: LifeforceState,
    did_directory: D,
    beneficiaries: HashMap<String, DonationBeneficiary>,
}

impl<D> NanoDonationRouter<D>
where
    D: DIDDirectory,
{
    pub fn new(policy: DonationPolicy, lifeforce: LifeforceState, did_directory: D) -> Self {
        Self {
            policy,
            lifeforce,
            did_directory,
            beneficiaries: HashMap::new(),
        }
    }

    /// Register or update a beneficiary, with ALN/DID + role checks.
    ///
    /// Only augmented-citizen or ethical operator roles may administer.
    pub fn register_beneficiary(
        &mut self,
        admin_did: &ALNDID,
        org_id: String,
        kind: DonationKind,
        max_nano_fraction_per_night: f64,
    ) -> Result<(), RuntimeError> {
        let env = self
            .did_directory
            .resolve_access(admin_did.clone())
            .ok_or(RuntimeError::AccessDenied(
                "Unknown DID for donation admin",
            ))?;

        if env.min_biophysics_knowledge_score < self.policy.min_admin_knowledge {
            return Err(RuntimeError::AccessDenied(
                "Insufficient knowledge to administer donation router",
            ));
        }

        let allowed_role = env
            .roles
            .iter()
            .any(|r| matches!(r, RoleClass::AugmentedCitizen | RoleClass::EthicalOperator));

        if !allowed_role {
            return Err(RuntimeError::AccessDenied(
                "Role not permitted to configure donation router",
            ));
        }

        let capped = max_nano_fraction_per_night.clamp(0.0, 1.0);
        let beneficiary = DonationBeneficiary {
            org_id: org_id.clone(),
            did: admin_did.clone(), // org anchored to this DID namespace
            kind,
            max_nano_fraction_per_night: capped,
            active: true,
        };
        self.beneficiaries.insert(org_id, beneficiary);
        Ok(())
    }

    /// Deactivate a beneficiary without deleting audit history.
    pub fn deactivate_beneficiary(
        &mut self,
        admin_did: &ALNDID,
        org_id: &str,
    ) -> Result<(), RuntimeError> {
        let env = self
            .did_directory
            .resolve_access(admin_did.clone())
            .ok_or(RuntimeError::AccessDenied(
                "Unknown DID for donation admin",
            ))?;

        if env.min_biophysics_knowledge_score < self.policy.min_admin_knowledge {
            return Err(RuntimeError::AccessDenied(
                "Insufficient knowledge to administer donation router",
            ));
        }

        if let Some(b) = self.beneficiaries.get_mut(org_id) {
            b.active = false;
            Ok(())
        } else {
            Err(RuntimeError::AccessDenied(
                "Beneficiary not found on this host",
            ))
        }
    }

    /// Core decision: schedule donation jobs, update local state, and return audit.
    ///
    /// This never transfers NANO between hosts; it only decrements this host's NANO
    /// and emits jobs that higher layers can turn into real workloads for partners.
    pub fn schedule_donations(
        &self,
        state: &mut BioTokenState,
        ctx: &DonationContext,
    ) -> Result<DonationAudit, RuntimeError> {
        // 1. Hard preconditions: host must opt-in, eco alignment must be good.
        if !ctx.host_opt_in {
            return Ok(DonationAudit {
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                reason: "Host did not opt-in for donations this window".to_string(),
            });
        }

        if ctx.eco_alignment01 < self.policy.min_eco_alignment {
            return Ok(DonationAudit {
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                reason: "Eco alignment below threshold; skipping donations".to_string(),
            });
        }

        // 2. Lifeforce + metabolic bands must be green/yellow.
        self.lifeforce
            .validate_bands(state.clone())
            .map_err(RuntimeError::SafetyViolation)?;

        // 3. Compute total available NANO for donation as integer tokens.
        let surplus_fraction = ctx.surplus_nano_fraction01.clamp(0.0, 1.0);
        if surplus_fraction == 0.0 || state.nano <= 0.0 {
            return Ok(DonationAudit {
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                reason: "No surplus NANO available for donation".to_string(),
            });
        }

        let max_total_fraction = self.policy.max_total_nano_fraction_per_night.clamp(0.0, 1.0);
        let effective_fraction = surplus_fraction.min(max_total_fraction);
        let available_nano_tokens = (state.nano * effective_fraction).floor() as u64;
        if available_nano_tokens == 0 {
            return Ok(DonationAudit {
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                reason: "Effective surplus too small for a donation job".to_string(),
            });
        }

        // 4. Simple fair-share allocation across active beneficiaries.
        let active: Vec<_> = self
            .beneficiaries
            .values()
            .filter(|b| b.active)
            .collect();

        if active.is_empty() {
            return Ok(DonationAudit {
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                reason: "No active beneficiaries configured".to_string(),
            });
        }

        let per_beneficiary = (available_nano_tokens / active.len() as u64).max(1);
        let mut jobs = Vec::new();
        let mut nano_spent_total: u64 = 0;
        let mut eco_cost_total: u64 = 0;

        for (idx, b) in active.iter().enumerate() {
            // Per-beneficiary cap in tokens.
            let max_for_b = (state.nano * b.max_nano_fraction_per_night)
                .floor()
                .max(0.0) as u64;
            if max_for_b == 0 {
                continue;
            }

            let planned = per_beneficiary.min(max_for_b);
            if planned == 0 {
                continue;
            }

            // Approximate eco cost: here 1 NANO ≈ 10^6 FLOPs (tunable).
            let eco_cost = planned.saturating_mul(1_000_000);

            let job = DonationJob {
                job_id: format!("nano-donation-{}-{}", b.org_id, idx),
                beneficiary_org: b.org_id.clone(),
                beneficiary_did: b.did.clone(),
                nano_fraction: planned as f64 / state.nano.max(1.0),
                eco_cost_flops: eco_cost,
                label: format!(
                    "Donated NANO-backed compute to {} ({:?})",
                    b.org_id, b.kind
                ),
            };

            jobs.push(job);
            nano_spent_total = nano_spent_total.saturating_add(planned);
            eco_cost_total = eco_cost_total.saturating_add(eco_cost);
        }

        if jobs.is_empty() {
            return Ok(DonationAudit {
                applied_jobs: Vec::new(),
                total_nano_spent: 0,
                total_eco_cost_flops: 0,
                reason: "Beneficiary caps prevented any donation jobs".to_string(),
            });
        }

        // 5. Apply local NANO decrement (no cross-host transfer).
        let nano_delta = -(nano_spent_total as f64);
        state.nano = (state.nano + nano_delta).max(0.0);

        // Re-check invariants after adjustment.
        self.lifeforce
            .validate_bands(state.clone())
            .map_err(RuntimeError::SafetyViolation)?;

        Ok(DonationAudit {
            applied_jobs: jobs,
            total_nano_spent: nano_spent_total,
            total_eco_cost_flops: eco_cost_total,
            reason: "Donations scheduled under surplus NANO and eco-safe bands".to_string(),
        })
    }
}
