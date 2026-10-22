// Unit Test Suite for Consecutive Switch Cooldown Penalty & Velocity Throttle (Imp-488 & Imp-518)
// Validates escalating cooldown penalties, reset idle durations, and velocity throttling.

#[cfg(test)]
mod tests {
    use super::super::consecutive_switch_cooldown::{
        ConsecutiveSwitchCooldown, CooldownLadderConfig,
    };
    use super::super::consecutive_switch_throttle::{
        ConsecutiveSwitchThrottle, ThrottleConfig,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn test_consecutive_switch_cooldown_escalation() {
        let config = CooldownLadderConfig {
            base_penalty: Duration::from_secs(10),
            max_penalty: Duration::from_secs(100),
            penalty_multiplier: 2.0,
            reset_idle_duration: Duration::from_secs(60),
        };
        let cooldown = ConsecutiveSwitchCooldown::new(config);
        let inst = "inst_alpha";
        let now = Instant::now();

        // 1st switch: base penalty (10s)
        let p1 = cooldown.record_and_get_penalty(inst, now);
        assert_eq!(p1, Duration::from_secs(10));

        // 2nd switch: 10 * 2.0^1 = 20s
        let p2 = cooldown.record_and_get_penalty(inst, now + Duration::from_secs(1));
        assert_eq!(p2, Duration::from_secs(20));

        // 3rd switch: 10 * 2.0^2 = 40s
        let p3 = cooldown.record_and_get_penalty(inst, now + Duration::from_secs(2));
        assert_eq!(p3, Duration::from_secs(40));

        // 4th switch: 10 * 2.0^3 = 80s
        let p4 = cooldown.record_and_get_penalty(inst, now + Duration::from_secs(3));
        assert_eq!(p4, Duration::from_secs(80));

        // 5th switch: 10 * 2.0^4 = 160s, clamped to max_penalty (100s)
        let p5 = cooldown.record_and_get_penalty(inst, now + Duration::from_secs(4));
        assert_eq!(p5, Duration::from_secs(100));
    }

    #[test]
    fn test_consecutive_switch_cooldown_resets_after_idle_duration() {
        let config = CooldownLadderConfig {
            base_penalty: Duration::from_secs(10),
            max_penalty: Duration::from_secs(100),
            penalty_multiplier: 2.0,
            reset_idle_duration: Duration::from_secs(30),
        };
        let cooldown = ConsecutiveSwitchCooldown::new(config);
        let inst = "inst_beta";
        let now = Instant::now();

        cooldown.record_and_get_penalty(inst, now);
        cooldown.record_and_get_penalty(inst, now + Duration::from_secs(1));

        // Wait 35s (> 30s reset_idle_duration)
        let p_reset = cooldown.record_and_get_penalty(inst, now + Duration::from_secs(35));
        assert_eq!(p_reset, Duration::from_secs(10));
    }

    #[test]
    fn test_velocity_throttle_blocks_rapid_switches() {
        let config = ThrottleConfig {
            max_switches_per_window: 2,
            window_duration: Duration::from_secs(10),
            cooloff_penalty: Duration::from_secs(15),
        };
        let throttle = ConsecutiveSwitchThrottle::new(config);
        let inst = "inst_gamma";
        let now = Instant::now();

        assert!(throttle.check_and_record(inst, now).is_ok());
        assert!(throttle.check_and_record(inst, now + Duration::from_secs(1)).is_ok());

        // 3rd switch in 2s exceeds max of 2: blocked
        let res = throttle.check_and_record(inst, now + Duration::from_secs(2));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Duration::from_secs(15));
    }
}
