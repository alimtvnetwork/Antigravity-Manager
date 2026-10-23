#[cfg(test)]
mod tests {
    use super::super::job_breakaway_fallback_ladder::{
        determine_spawn_strategy, is_job_object_access_denied, SpawnCreationFlagsDecision,
        CREATE_BREAKAWAY_FROM_JOB, ERROR_ACCESS_DENIED,
    };

    #[test]
    fn test_attempt_zero_uses_breakaway() {
        let decision = determine_spawn_strategy(0, None);
        assert_eq!(
            decision,
            SpawnCreationFlagsDecision::WithBreakaway(CREATE_BREAKAWAY_FROM_JOB)
        );
    }

    #[test]
    fn test_attempt_one_with_access_denied_falls_back_to_standard() {
        let decision = determine_spawn_strategy(1, Some(ERROR_ACCESS_DENIED));
        assert_eq!(
            decision,
            SpawnCreationFlagsDecision::StandardWithoutBreakaway
        );
    }

    #[test]
    fn test_attempt_one_with_unrelated_error_terminates() {
        // e.g. error 2 = file not found
        let decision = determine_spawn_strategy(1, Some(2));
        assert_eq!(decision, SpawnCreationFlagsDecision::TerminalFailure);
    }

    #[test]
    fn test_attempt_two_terminates() {
        let decision = determine_spawn_strategy(2, Some(ERROR_ACCESS_DENIED));
        assert_eq!(decision, SpawnCreationFlagsDecision::TerminalFailure);
    }

    #[test]
    fn test_is_job_object_access_denied() {
        assert!(is_job_object_access_denied(5));
        assert!(!is_job_object_access_denied(0));
        assert!(!is_job_object_access_denied(2));
    }
}
