//! Candidate Selection Tie-Breaker Randomizer.
//! Introduces deterministic pseudo-random jitter for candidate accounts with identical composite scores.

/// Computes a tiny deterministic tie-breaker delta [0.0001, 0.0099] based on account ID and time seed.
pub fn compute_deterministic_tie_breaker(account_id: &str, salt_epoch_min: i64) -> f64 {
    // 64-bit FNV-1a hash
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in account_id.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for byte in salt_epoch_min.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    // Map to 1..99 range then divide by 10,000 to get 0.0001 to 0.0099
    let jitter_int = (hash % 99) + 1;
    jitter_int as f64 / 10000.0
}

/// Applies deterministic tie-breaker jitter to a score.
pub fn apply_score_tie_breaker(base_score: f64, account_id: &str, salt_epoch_min: i64) -> f64 {
    let jitter = compute_deterministic_tie_breaker(account_id, salt_epoch_min);
    (base_score + jitter * 10000.0).round() / 10000.0
}
