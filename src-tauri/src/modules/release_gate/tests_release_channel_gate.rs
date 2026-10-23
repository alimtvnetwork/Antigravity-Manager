#[cfg(test)]
mod tests {
    use super::super::channel_gate::{validate_release_channel, ReleaseChannel};

    #[test]
    fn test_stable_release_on_main_valid() {
        let result = validate_release_channel("v4.250.0", "main");
        assert!(result.is_valid);
        assert_eq!(result.channel, ReleaseChannel::Stable);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_stable_release_on_beta_forbidden() {
        let result = validate_release_channel("v4.250.0", "beta");
        assert!(!result.is_valid);
        assert_eq!(result.channel, ReleaseChannel::Stable);
        assert!(result
            .error_message
            .unwrap()
            .contains("forbidden on branch 'beta'"));
    }

    #[test]
    fn test_stable_release_on_feature_branch_forbidden() {
        let result = validate_release_channel("v4.250.0", "feature/my-feat");
        assert!(!result.is_valid);
        assert!(result
            .error_message
            .unwrap()
            .contains("Must be tagged on 'main'"));
    }

    #[test]
    fn test_preview_release_on_beta_valid() {
        let result = validate_release_channel("v4.250.0-beta.1", "beta");
        assert!(result.is_valid);
        assert_eq!(result.channel, ReleaseChannel::Preview);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_preview_release_on_main_forbidden() {
        let result = validate_release_channel("v4.250.0-beta.1", "main");
        assert!(!result.is_valid);
        assert_eq!(result.channel, ReleaseChannel::Preview);
        assert!(result
            .error_message
            .unwrap()
            .contains("forbidden on branch 'main'"));
    }
}
