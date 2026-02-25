#![forbid(unsafe_code)]

use std::time::SystemTime;

use bioscale_upgrade_store::{HostBudget, UpgradeDecision, UpgradeDescriptor};
use cybernet_metrics::BiophysicalFlowsSnapshot;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponsibilityScalar(pub f64);

impl ResponsibilityScalar {
    pub fn neutral() -> Self { ResponsibilityScalar(0.0) }
    pub fn add_delta(self, delta: f64) -> Self { ResponsibilityScalar(self.0 + delta) }
    pub fn value(self) -> f64 { self.0 }
}

#[derive(Clone, Debug)]
pub struct OuterFreedomEnvelope {
    pub max_outer_radius: f64,
    pub bandwidth_cap: f64,
    pub device_control_cap: f64,
}

pub trait OuterFreedomModel {
    fn outer_envelope(&self, r: ResponsibilityScalar) -> OuterFreedomEnvelope;

    fn monotone_check(&self, r1: ResponsibilityScalar, r2: ResponsibilityScalar) -> bool {
        if r2.value() <= r1.value() {
            return true;
        }
        let e1 = self.outer_envelope(r1);
        let e2 = self.outer_envelope(r2);
        e2.max_outer_radius + 1e-9 >= e1.max_outer_radius
            && e2.bandwidth_cap + 1e-9 >= e1.bandwidth_cap
            && e2.device_control_cap + 1e-9 >= e1.device_control_cap
    }
}

#[derive(Clone, Debug)]
pub struct ResponsibilityEvidence {
    pub flows: BiophysicalFlowsSnapshot,
    pub delta_r: f64,
    pub evidence_bundle: bioscale_upgrade_store::EvidenceBundle,
}

#[derive(Clone, Debug)]
pub struct KarmaVerdict {
    pub is_karma_admissible: bool,
    pub pre_r: ResponsibilityScalar,
    pub post_r: ResponsibilityScalar,
    pub delta_r: f64,
}

pub trait KarmaConstraint {
    fn responsibility_delta(
        &self,
        now: SystemTime,
        flows: &BiophysicalFlowsSnapshot,
    ) -> ResponsibilityEvidence;

    fn karma_admissible(
        &self,
        now: SystemTime,
        pre_r: ResponsibilityScalar,
        flows: &BiophysicalFlowsSnapshot,
        model: &dyn OuterFreedomModel,
    ) -> KarmaVerdict {
        let re = self.responsibility_delta(now, flows);
        let post_r = pre_r.add_delta(re.delta_r);
        let pre_env = model.outer_envelope(pre_r);
        let post_env = model.outer_envelope(post_r);

        let greed_pattern = re.delta_r < 0.0
            && (post_env.max_outer_radius > pre_env.max_outer_radius + 1e-9
                || post_env.bandwidth_cap > pre_env.bandwidth_cap + 1e-9
                || post_env.device_control_cap > pre_env.device_control_cap + 1e-9);

        KarmaVerdict {
            is_karma_admissible: !greed_pattern && re.delta_r >= 0.0,
            pre_r,
            post_r,
            delta_r: re.delta_r,
        }
    }
}

/// Any upgrade controlling outer actions must go through this predicate.
pub trait FairnessConstrainedUpgrade: KarmaConstraint {
    fn descriptor(&self) -> UpgradeDescriptor;

    fn evaluate_with_fairness(
        &self,
        host: &HostBudget,
        now: SystemTime,
        current_r: ResponsibilityScalar,
        flows: &BiophysicalFlowsSnapshot,
        model: &dyn OuterFreedomModel,
    ) -> UpgradeDecision {
        let verdict = self.karma_admissible(now, current_r, flows, model);
        if !verdict.is_karma_admissible {
            return UpgradeDecision::Denied {
                reason: "FairnessConstraint: Δr < 0 with outer expansion".into(),
            };
        }
        bioscale_upgrade_store::evaluate_upgrade(self.descriptor(), host, now)
    }
}
