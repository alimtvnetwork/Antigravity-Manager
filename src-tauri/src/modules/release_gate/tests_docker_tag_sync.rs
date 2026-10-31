#[cfg(test)]
mod tests {
    use super::super::docker_sync_verifier::{evaluate_docker_tag_sync, DockerSyncEvaluation};

    #[test]
    fn test_valid_production_release_main() {
        let eval = evaluate_docker_tag_sync("main", "v4.250.0", true, false);
        assert!(eval.is_valid);
        assert!(eval.is_production_channel);
        assert!(eval.latest_tag_allowed);
    }

    #[test]
    fn test_invalid_production_release_missing_make_latest() {
        let eval = evaluate_docker_tag_sync("main", "v4.250.0", false, false);
        assert!(!eval.is_valid);
        assert!(eval.reason.contains("must update the 'latest' tag"));
    }

    #[test]
    fn test_invalid_production_release_marked_prerelease() {
        let eval = evaluate_docker_tag_sync("main", "v4.250.0", true, true);
        assert!(!eval.is_valid);
        assert!(eval.reason.contains("cannot be marked as a prerelease"));
    }

    #[test]
    fn test_invalid_prerelease_tag_on_main() {
        let eval = evaluate_docker_tag_sync("main", "v4.250.0-beta.1", true, false);
        assert!(!eval.is_valid);
        assert!(eval.reason.contains("strictly forbidden on 'main' branch"));
    }

    #[test]
    fn test_valid_preview_release_beta() {
        let eval = evaluate_docker_tag_sync("beta", "v4.250.0-beta.1", false, true);
        assert!(eval.is_valid);
        assert!(!eval.is_production_channel);
        assert!(!eval.latest_tag_allowed);
    }

    #[test]
    fn test_invalid_preview_release_touching_latest() {
        let eval = evaluate_docker_tag_sync("beta", "v4.250.0-beta.1", true, true);
        assert!(!eval.is_valid);
        assert!(eval
            .reason
            .contains("must NEVER update production 'latest' tag"));
    }

    #[test]
    fn test_invalid_preview_release_without_prerelease_flag() {
        let eval = evaluate_docker_tag_sync("beta", "v4.250.0-beta.1", false, false);
        assert!(!eval.is_valid);
        assert!(eval.reason.contains("must have prerelease: true"));
    }

    #[test]
    fn test_invalid_unauthorized_branch() {
        let eval = evaluate_docker_tag_sync("feature/my-feat", "v4.250.0", true, false);
        assert!(!eval.is_valid);
        assert!(eval.reason.contains("Unauthorized release branch"));
    }
}
