use crate::{CandidateId, CandidateScore, CyberRankError, RankVector, RankWeights};

/// Tsafe-style selection:
/// 1) Filter to candidates allowed by a boolean mask (e.g., viability oracle).
/// 2) Compute Pareto front in CyberRank space.
/// 3) Among Pareto-front candidates, pick max weighted_score.
pub fn tsafe_select(
    candidates: &[(CandidateId, RankVector)],
    allowed_mask: &[bool],
    weights: &RankWeights,
) -> Result<CandidateScore, CyberRankError> {
    if candidates.len() != allowed_mask.len() {
        return Err(CyberRankError::NoViableCandidate);
    }

    // Step 1: filter allowed
    let mut allowed = Vec::new();
    for (i, ((id, rank), allowed_flag)) in candidates
        .iter()
        .zip(allowed_mask.iter())
        .enumerate()
    {
        if *allowed_flag {
            allowed.push((i, id.clone(), *rank));
        }
    }

    if allowed.is_empty() {
        return Err(CyberRankError::NoViableCandidate);
    }

    // Step 2: compute Pareto front
    let mut pareto_indices = Vec::new();
    'outer: for (i, _, rank_i) in &allowed {
        for (j, _, rank_j) in &allowed {
            if i == j {
                continue;
            }
            if rank_j.dominates(rank_i) {
                continue 'outer;
            }
        }
        pareto_indices.push(*i);
    }

    // Step 3: pick max weighted score among Pareto front
    let mut best: Option<CandidateScore> = None;
    for idx in pareto_indices {
        let (id, rank) = &candidates[idx];
        let score = rank.weighted_score(weights);
        let cs = CandidateScore {
            id: id.clone(),
            rank: *rank,
            scalar_score: score,
        };
        if let Some(ref mut b) = best {
            if cs.scalar_score > b.scalar_score {
                *b = cs;
            }
        } else {
            best = Some(cs);
        }
    }

    best.ok_or(CyberRankError::NoViableCandidate)
}
