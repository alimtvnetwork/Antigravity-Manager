//! Unit tests for Stepped Cooldown Decay (Imp-571)

use std::time::{Duration, Instant};

use super::cooldown_decay_stepper::{
    calculate_stepped_cooldown_decay, COOLDOWN_CLEAR_FLOOR_SECS, DEFAULT_DECAY_HALF_LIFE,
};

#[test]
fn test_expired_cooldown_returns_zero() {
    let now = Instant::now();
    let initial = Duration::from_secs(300);
    let started = now - Duration::from_secs(400); // 400s elapsed > 300s
    let last_error = started;

    let remaining = calculate_stepped_cooldown_decay(initial, started, last_error, now);
    assert_eq!(remaining, Duration::ZERO);
}

#[test]
fn test_zero_half_lives_returns_linear_remaining() {
    let now = Instant::now();
    let initial = Duration::from_secs(600);
    let started = now - Duration::from_secs(100);
    let last_error = started; // 100s idle < 300s half-life

    let remaining = calculate_stepped_cooldown_decay(initial, started, last_error, now);
    assert_eq!(remaining, Duration::from_secs(500));
}

#[test]
fn test_one_half_life_applies_half_decay() {
    let now = Instant::now();
    let initial = Duration::from_secs(1000);
    let started = now - DEFAULT_DECAY_HALF_LIFE; // 300s elapsed
    let last_error = started; // 300s idle = exactly 1 half-life

    // Remaining linear: 1000 - 300 = 700s. Decay factor: 0.5 -> 350s.
    let remaining = calculate_stepped_cooldown_decay(initial, started, last_error, now);
    assert_eq!(remaining, Duration::from_secs(350));
}

#[test]
fn test_two_half_lives_applies_quarter_decay() {
    let now = Instant::now();
    let initial = Duration::from_secs(1200);
    let started = now - (DEFAULT_DECAY_HALF_LIFE * 2); // 600s elapsed
    let last_error = started; // 600s idle = 2 half-lives

    // Remaining linear: 1200 - 600 = 600s. Decay factor: 0.25 -> 150s.
    let remaining = calculate_stepped_cooldown_decay(initial, started, last_error, now);
    assert_eq!(remaining, Duration::from_secs(150));
}

#[test]
fn test_decay_below_floor_clears_to_zero() {
    let now = Instant::now();
    let initial = Duration::from_secs(320);
    let started = now - (DEFAULT_DECAY_HALF_LIFE * 3); // 900s idle would expire anyway, test non-expired edge
    let started_recently = now - Duration::from_secs(315); // 5s linear left
    let last_error = now - DEFAULT_DECAY_HALF_LIFE; // 1 half life: 5s * 0.5 = 2.5s <= 5s floor

    let remaining = calculate_stepped_cooldown_decay(initial, started_recently, last_error, now);
    assert_eq!(remaining, Duration::ZERO);
}
