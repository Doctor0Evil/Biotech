use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RankAxis {
    Safety,
    Legal,
    Biomech,
    Psych,
    Rollback,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RankVector {
    pub safety: f32,
    pub legal: f32,
    pub biomech: f32,
    pub psych: f32,
    pub rollback: f32,
}

impl RankVector {
    pub fn dominates(&self, other: &RankVector) -> bool {
        let self_coords = [self.safety, self.legal, self.biomech, self.psych, self.rollback];
        let other_coords = [other.safety, other.legal, other.biomech, other.psych, other.rollback];

        let mut strictly_better = false;
        for (a, b) in self_coords.iter().zip(other_coords.iter()) {
            if a < b {
                return false;
            }
            if a > b {
                strictly_better = true;
            }
        }
        strictly_better
    }

    pub fn weighted_score(&self, w: &RankWeights) -> f32 {
        self.safety * w.safety
            + self.legal * w.legal
            + self.biomech * w.biomech
            + self.psych * w.psych
            + self.rollback * w.rollback
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RankWeights {
    pub safety: f32,
    pub legal: f32,
    pub biomech: f32,
    pub psych: f32,
    pub rollback: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateScore {
    pub id: CandidateId,
    pub rank: RankVector,
    pub scalar_score: f32,
}

#[derive(Debug, Error)]
pub enum CyberRankError {
    #[error("no viable candidate")]
    NoViableCandidate,
}
