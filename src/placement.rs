//! Boltzmann (logit) sampling: CHOOSE_BOLTZMANN operator.
//!
//! Given a set of candidate positions with utility scores, sample one
//! with probability P(p) ∝ exp(β · utility(p)).
//!
//! Uses the log-sum-exp trick to avoid numerical overflow at high β.

use rand::Rng;

use crate::types::Pos;

/// A candidate position with its evaluated utility.
pub struct Candidate {
    pub pos: Pos,
    pub utility: f32,
}

/// Sample a position from candidates using Boltzmann distribution.
///
/// Returns `None` if `candidates` is empty.
pub fn boltzmann_sample<R: Rng>(
    candidates: &[Candidate],
    beta: f32,
    rng: &mut R,
) -> Option<Pos> {
    if candidates.is_empty() {
        return None;
    }

    // Log-sum-exp trick: subtract max before exp to prevent overflow.
    let max_u = candidates
        .iter()
        .map(|c| c.utility)
        .fold(f32::NEG_INFINITY, f32::max);

    let weights: Vec<f32> = candidates
        .iter()
        .map(|c| ((c.utility - max_u) * beta).exp())
        .collect();

    let total: f32 = weights.iter().sum();

    // Inverse CDF sampling.
    let mut r = rng.random::<f32>() * total;
    for (i, &w) in weights.iter().enumerate() {
        r -= w;
        if r <= 0.0 {
            return Some(candidates[i].pos);
        }
    }

    // Floating-point edge case fallback.
    Some(candidates.last().unwrap().pos)
}
