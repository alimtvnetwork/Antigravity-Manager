#[cfg(test)]
mod tests {
    use super::super::semver_monotonic_validator::{
        validate_semver_transition, BumpType, ParsedSemVer,
    };

    #[test]
    fn test_parse_valid_semver() {
        let v = ParsedSemVer::parse("v4.250.1").unwrap();
        assert_eq!(v.major, 4);
        assert_eq!(v.minor, 250);
        assert_eq!(v.patch, 1);
        assert_eq!(v.beta, None);

        let v_beta = ParsedSemVer::parse("4.251.0-beta.3").unwrap();
        assert_eq!(v_beta.major, 4);
        assert_eq!(v_beta.minor, 251);
        assert_eq!(v_beta.patch, 0);
        assert_eq!(v_beta.beta, Some(3));
    }

    #[test]
    fn test_valid_patch_bump() {
        let eval = validate_semver_transition("v4.250.0", "v4.250.1");
        assert!(eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::Patch);
    }

    #[test]
    fn test_valid_minor_bump() {
        let eval = validate_semver_transition("v4.250.9", "v4.251.0");
        assert!(eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::Minor);
    }

    #[test]
    fn test_valid_major_bump() {
        let eval = validate_semver_transition("v4.250.0", "v5.0.0");
        assert!(eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::Major);
    }

    #[test]
    fn test_valid_beta_increment() {
        let eval = validate_semver_transition("v4.251.0-beta.1", "v4.251.0-beta.2");
        assert!(eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::BetaIncrement);
    }

    #[test]
    fn test_valid_prerelease_to_stable() {
        let eval = validate_semver_transition("v4.251.0-beta.4", "v4.251.0");
        assert!(eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::PrereleaseToStable);
    }

    #[test]
    fn test_invalid_identical_version() {
        let eval = validate_semver_transition("v4.250.0", "v4.250.0");
        assert!(!eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::Identical);
    }

    #[test]
    fn test_invalid_downgrade() {
        let eval = validate_semver_transition("v4.250.1", "v4.250.0");
        assert!(!eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::Downgrade);
    }

    #[test]
    fn test_invalid_beta_downgrade() {
        let eval = validate_semver_transition("v4.251.0-beta.5", "v4.251.0-beta.3");
        assert!(!eval.is_valid);
        assert_eq!(eval.bump_type, BumpType::Downgrade);
    }
}
