//! Microspace sovereignty guard: enforces that all biophysically-active
//! operations remain self-only, host-local, and consent-bound, and that
//! no proposal can externalize negative energy onto others.

use serde::{Deserialize, Serialize};

use crate::biophysical_chain_neuroautomationpipeline::{
    BiophysicalConstraints,
    EvolutionProposal,
};

/// Logical identifier for the sovereign host (e.g., Bostrom / ALN DID).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HostId(pub String);

/// Classification of whether an operation is allowed to affect others.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExternalEffectPolicy {
    /// Absolutely no cross-host effects are permitted.
    ForbidExternalNegative,
    /// Future extension: allow neutral/positive shared eco-effects only.
    StrictSelfOnly,
}

/// Sovereign microspace configuration, per host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicrospaceSovereigntyProfile {
    /// The sovereign owner of this microspace.
    pub host_id: HostId,
    /// Doctrine for external effects.
    pub external_policy: ExternalEffectPolicy,
    /// Require explicit consent token for any irreversible biophysical change.
    pub require_irreversible_consent: bool,
    /// Require that all proposals declare themselves as self-only.
    pub require_self_only_flag: bool,
}

/// Lightweight projection of a proposal's self-only declaration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfOnlyDeclaration {
    /// Proposal claims it affects only this host's microspace.
    pub self_only: bool,
    /// HostId the proposal claims to be bound to.
    pub host_id: HostId,
}

/// Result of a microspace sovereignty guard check.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MicrospaceGuardDecision {
    Allowed,
    Rejected(String),
}

/// Extracts self-only declaration from an EvolutionProposal's metadata.
/// Assumes you attach ALN- or JSON-encoded metadata into the proposal.
fn extract_self_only_declaration(
    proposal: &EvolutionProposal,
) -> Option<SelfOnlyDeclaration> {
    // This is intentionally schematic: adapt to your actual metadata field.
    proposal
        .metadata
        .get("self_only_declaration")
        .and_then(|val| serde_json::from_value(val.clone()).ok())
}

/// Checks whether a proposal is compatible with the sovereign microspace profile
/// and the global biophysical constraints.
///
/// This function should be called inside your DefaultProposalValidator
/// *before* any actuation is scheduled, and before consensus signs the block.
pub fn guard_microspace_sovereignty(
    profile: &MicrospaceSovereigntyProfile,
    constraints: &BiophysicalConstraints,
    proposal: &EvolutionProposal,
) -> MicrospaceGuardDecision {
    // 1. Enforce self-only declaration when required.
    if profile.require_self_only_flag {
        let decl_opt = extract_self_only_declaration(proposal);
        let decl = match decl_opt {
            Some(d) => d,
            None => {
                return MicrospaceGuardDecision::Rejected(
                    "missing self_only_declaration for biophysically-active proposal".to_string(),
                );
            }
        };
        if !decl.self_only {
            return MicrospaceGuardDecision::Rejected(
                "proposal not marked self_only for sovereign microspace".to_string(),
            );
        }
        if decl.host_id != profile.host_id {
            return MicrospaceGuardDecision::Rejected(
                "proposal host_id does not match sovereign microspace host_id".to_string(),
            );
        }
    }

    // 2. Forbid external negative effects by domain/target analysis.
    match profile.external_policy {
        ExternalEffectPolicy::ForbidExternalNegative
        | ExternalEffectPolicy::StrictSelfOnly => {
            // Reject any proposal that declares cross-host or shared targets.
            for pattern in &proposal.patterns {
                if pattern
                    .targets
                    .iter()
                    .any(|t| t.contains("external-host") || t.contains("shared-space"))
                {
                    return MicrospaceGuardDecision::Rejected(
                        "cross-host or shared-space biophysical target not permitted".to_string(),
                    );
                }
            }
        }
    }

    // 3. Irreversible changes must carry explicit consent token when configured.
    if profile.require_irreversible_consent {
        let mut needs_consent = false;
        for pattern in &proposal.patterns {
            if pattern.reversibility.is_irreversible() {
                needs_consent = true;
                break;
            }
        }
        if needs_consent {
            if proposal.irreversible_token.is_none() {
                return MicrospaceGuardDecision::Rejected(
                    "irreversible biophysical change requires explicit irreversible_token".to_string(),
                );
            }
        }
    }

    // 4. Respect global biophysical constraints (pain, BLOOD, etc.) as you configured.
    //    This assumes BiophysicalConstraints already encode your self-chosen pain/blood envelopes.
    if let Err(err) = constraints.check_proposal(proposal) {
        return MicrospaceGuardDecision::Rejected(format!(
            "biophysical constraints rejected proposal: {err}"
        ));
    }

    MicrospaceGuardDecision::Allowed
}
