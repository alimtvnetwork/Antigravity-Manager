#[cfg(test)]
mod tests {
    use super::super::caller_ide_preservation_guard::{
        validate_termination_safety, TerminationSafetyViolation,
    };

    #[test]
    fn test_termination_safety_caller_blocked_without_force() {
        let report = validate_termination_safety("inst-1", Some("inst-1"), false);
        assert!(!report.is_permitted);
        assert!(report.is_caller_instance);

        match report.violation {
            Some(TerminationSafetyViolation::CallerSelfPreservationBlocked {
                exit_code,
                target_id,
                ..
            }) => {
                assert_eq!(exit_code, 2);
                assert_eq!(target_id, "inst-1");
            }
            _ => panic!("Expected CallerSelfPreservationBlocked violation"),
        }
    }

    #[test]
    fn test_termination_safety_caller_permitted_with_force() {
        let report = validate_termination_safety("inst-1", Some("inst-1"), true);
        assert!(report.is_permitted);
        assert!(report.is_caller_instance);
        assert!(report.violation.is_none());
    }

    #[test]
    fn test_termination_safety_different_instance_permitted() {
        let report = validate_termination_safety("inst-sandbox-99", Some("inst-1"), false);
        assert!(report.is_permitted);
        assert!(!report.is_caller_instance);
        assert!(report.violation.is_none());
    }

    #[test]
    fn test_termination_safety_no_caller_detected_permitted() {
        let report = validate_termination_safety("inst-sandbox-99", None, false);
        assert!(report.is_permitted);
        assert!(!report.is_caller_instance);
        assert!(report.violation.is_none());
    }
}
