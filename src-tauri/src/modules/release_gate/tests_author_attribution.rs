#[cfg(test)]
mod tests {
    use super::super::attribution_verifier::verify_author_attribution;

    #[test]
    fn test_author_attribution_only_aukgit_is_compliant() {
        let content = "## Release Notes\nImplemented feature X and resolved Y. (Thanks to @aukgit)";
        let audit = verify_author_attribution(content);

        assert!(audit.is_compliant);
        assert_eq!(audit.authorized_handles, vec!["@aukgit"]);
        assert!(audit.disallowed_handles.is_empty());
    }

    #[test]
    fn test_author_attribution_other_handle_is_disallowed() {
        let content = "Implemented by @randomuser with review from @aukgit.";
        let audit = verify_author_attribution(content);

        assert!(!audit.is_compliant);
        assert_eq!(audit.authorized_handles, vec!["@aukgit"]);
        assert_eq!(audit.disallowed_handles, vec!["@randomuser"]);
    }

    #[test]
    fn test_author_attribution_empty_text() {
        let audit = verify_author_attribution("");
        assert!(audit.is_compliant);
        assert!(audit.authorized_handles.is_empty());
        assert!(audit.disallowed_handles.is_empty());
    }

    #[test]
    fn test_author_attribution_case_insensitive() {
        let content = "Thanks to @AUKGIT!";
        let audit = verify_author_attribution(content);

        assert!(audit.is_compliant);
        assert_eq!(audit.authorized_handles, vec!["@aukgit"]);
    }
}
