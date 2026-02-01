//! cybernano-vector-cyberrank
//!
//! Multi-axis CyberRank vectors and Tsafe-compatible selection logic.

mod model;
mod tsafe_select;

pub use model::{
    RankAxis,
    RankVector,
    RankWeights,
    CandidateId,
    CandidateScore,
    CyberRankError,
};
pub use tsafe_select::tsafe_select;
