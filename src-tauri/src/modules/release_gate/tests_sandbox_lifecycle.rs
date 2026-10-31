#[cfg(test)]
mod tests {
    use super::super::test_sandbox_lifecycle_verifier::{
        generate_transient_sandbox_id, is_transient_sandbox_id, validate_sandbox_cleanup,
        TRANSIENT_SANDBOX_PREFIX,
    };

    #[test]
    fn test_generate_and_detect_transient_sandbox() {
        let sandbox_id = generate_transient_sandbox_id(1792652100);
        assert!(sandbox_id.starts_with(TRANSIENT_SANDBOX_PREFIX));
        assert!(is_transient_sandbox_id(&sandbox_id));
        assert!(!is_transient_sandbox_id("agm-production-instance"));
        assert!(!is_transient_sandbox_id("default"));
    }

    #[test]
    fn test_validate_sandbox_cleanup_clean() {
        let report = validate_sandbox_cleanup(&[], &[]);
        assert!(report.is_clean);
        assert!(report.orphan_paths.is_empty());
        assert!(report.orphan_pids.is_empty());
        assert!(report.summary.contains("Zero-orphan"));
    }

    #[test]
    fn test_validate_sandbox_cleanup_with_remnant_paths() {
        let paths = &[
            "data/test-sandbox-1/workspaceStorage",
            "data/test-sandbox-1/lock",
        ];
        let report = validate_sandbox_cleanup(paths, &[]);
        assert!(!report.is_clean);
        assert_eq!(report.orphan_paths.len(), 2);
    }

    #[test]
    fn test_validate_sandbox_cleanup_with_orphan_pids() {
        let pids = &[98234, 98235];
        let report = validate_sandbox_cleanup(&[], pids);
        assert!(!report.is_clean);
        assert_eq!(report.orphan_pids.len(), 2);
    }
}
