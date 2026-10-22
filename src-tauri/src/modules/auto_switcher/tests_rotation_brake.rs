#[cfg(test)]
mod tests {
    use super::super::rotation_brake::{
        check_rotation_frequency, DEFAULT_MAX_SWITCHES_PER_HOUR,
    };

    #[test]
    fn test_rotation_frequency_empty_history() {
        let now = 10_000;
        let check = check_rotation_frequency(&[], now, None);

        assert!(check.can_rotate);
        assert_eq!(check.switches_in_past_hour, 0);
        assert_eq!(check.max_allowed_per_hour, DEFAULT_MAX_SWITCHES_PER_HOUR);
    }

    #[test]
    fn test_rotation_frequency_under_limit() {
        let now = 10_000;
        let timestamps = vec![now - 300, now - 120, now - 30]; // 3 recent switches
        let check = check_rotation_frequency(&timestamps, now, None);

        assert!(check.can_rotate);
        assert_eq!(check.switches_in_past_hour, 3);
        assert_eq!(check.max_allowed_per_hour, 5);
    }

    #[test]
    fn test_rotation_frequency_at_limit_blocks() {
        let now = 10_000;
        let timestamps = vec![
            now - 1000,
            now - 800,
            now - 600,
            now - 400,
            now - 200,
        ]; // Exactly 5 switches
        let check = check_rotation_frequency(&timestamps, now, None);

        assert!(!check.can_rotate);
        assert_eq!(check.switches_in_past_hour, 5);
        assert_eq!(check.max_allowed_per_hour, 5);
    }

    #[test]
    fn test_rotation_frequency_exceeds_limit_blocks() {
        let now = 10_000;
        let timestamps = vec![
            now - 3000,
            now - 2500,
            now - 2000,
            now - 1500,
            now - 1000,
            now - 500,
            now - 100,
        ]; // 7 switches in the past hour
        let check = check_rotation_frequency(&timestamps, now, None);

        assert!(!check.can_rotate);
        assert_eq!(check.switches_in_past_hour, 7);
    }

    #[test]
    fn test_rotation_frequency_filters_stale_switches() {
        let now = 10_000;
        // one_hour_ago is 10_000 - 3600 = 6400.
        // Timestamps < 6400 are outside the 1-hour window.
        let timestamps = vec![
            1000,
            2000,
            5000,
            6399, // stale (< 6400)
            6400, // exactly 1 hour ago (included)
            8000, // recent
            9500, // recent
        ];
        let check = check_rotation_frequency(&timestamps, now, None);

        assert!(check.can_rotate);
        assert_eq!(check.switches_in_past_hour, 3);
    }

    #[test]
    fn test_rotation_frequency_custom_limit_override() {
        let now = 10_000;
        let timestamps = vec![now - 200, now - 100];
        // Custom limit of 2: count is 2 >= 2 -> blocked
        let check = check_rotation_frequency(&timestamps, now, Some(2));

        assert!(!check.can_rotate);
        assert_eq!(check.switches_in_past_hour, 2);
        assert_eq!(check.max_allowed_per_hour, 2);

        // Custom limit of 3: count is 2 < 3 -> allowed
        let check_lenient = check_rotation_frequency(&timestamps, now, Some(3));
        assert!(check_lenient.can_rotate);
        assert_eq!(check_lenient.switches_in_past_hour, 2);
        assert_eq!(check_lenient.max_allowed_per_hour, 3);
    }
}
