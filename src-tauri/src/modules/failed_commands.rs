//! Levenshtein distance suggester and failed command telemetry helpers.
use crate::modules::repo_db;

pub use repo_db::failed_commands::{
    clear_failed_commands, count_failed_commands, list_failed_commands, log_failed_command,
};

/// Imp-28: Classical dynamic programming Levenshtein distance
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (m, n) = (a_chars.len(), b_chars.len());

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

/// Suggest up to 4 top matched commands based on Levenshtein edit distance <= 2 or substring match
pub fn suggest_commands(input: &str, candidates: &[&str]) -> Vec<String> {
    let clean = input.trim().to_lowercase();
    if clean.is_empty() {
        return Vec::new();
    }

    let mut matches: Vec<(usize, &str)> = candidates
        .iter()
        .filter_map(|&cand| {
            let c_clean = cand.to_lowercase();
            if c_clean == clean {
                return Some((0, cand));
            }
            if c_clean.starts_with(&clean) || clean.starts_with(&c_clean) {
                return Some((1, cand));
            }
            if c_clean.contains(&clean) {
                return Some((2, cand));
            }
            let dist = levenshtein_distance(&clean, &c_clean);
            if dist <= 2 {
                Some((dist + 2, cand))
            } else {
                None
            }
        })
        .collect();

    matches.sort_by_key(|&(score, name)| (score, name.len(), name));
    matches
        .into_iter()
        .take(4)
        .map(|(_, cand)| cand.to_string())
        .collect()
}
