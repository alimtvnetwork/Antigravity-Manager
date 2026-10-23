use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_parse_keyring_payload_nested_token() {
        let payload = r#"{
            "token": {
                "access_token": "ya29.test",
                "token_type": "Bearer",
                "refresh_token": "1//test_refresh_token_123",
                "expiry": "2026-09-19T10:00:00.000000Z"
            },
            "auth_method": "consumer"
        }"#;
        let state = parse_keyring_payload(payload).expect("Failed to parse nested keyring payload");
        assert_eq!(state.refresh_token, "1//test_refresh_token_123");
        assert!(state.is_gcp_tos);
    }

    #[test]
    pub(crate) fn test_parse_keyring_payload_flat_token() {
        let payload = r#"{
            "access_token": "ya29.test",
            "refresh_token": "1//test_refresh_token_flat"
        }"#;
        let state = parse_keyring_payload(payload).expect("Failed to parse flat keyring payload");
        assert_eq!(state.refresh_token, "1//test_refresh_token_flat");
    }

    #[test]
    pub(crate) fn test_parse_keyring_payload_missing_token() {
        let payload = r#"{ "auth_method": "consumer" }"#;
        let res = parse_keyring_payload(payload);
        assert!(res.is_err());
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_explicit_classic() {
        // Explicitly specify classic, even if the IDE is running or there is only IDE exe, it must be strictly determined as the classic version
        let (is_ide, effective) = resolve_effective_target(
            Some("classic"),
            false,
            true,
            false,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(!is_ide);
        assert_eq!(effective, Some("classic"));
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_explicit_ide() {
        // Explicitly specify ide, which must be determined as ide
        let (is_ide, effective) = resolve_effective_target(Some("ide"), true, false, true, None);
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_autodetect_classic_running() {
        // target_ide is None, the classic version is running, the classic version must be kept first
        let (is_ide, effective) = resolve_effective_target(
            None,
            true,
            true,
            true,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(!is_ide);
        assert_eq!(effective, None);
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_autodetect_ide_running_only() {
        // target_ide is None, only IDE is running, deduced to IDE
        let (is_ide, effective) = resolve_effective_target(
            None,
            false,
            true,
            true,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_autodetect_classic_exe_exists() {
        // target_ide is None, neither is running, but the classic version exe exists, the classic version takes precedence
        let (is_ide, effective) = resolve_effective_target(
            None,
            false,
            false,
            true,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(!is_ide);
        assert_eq!(effective, None);
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_autodetect_fallback_ide_exe() {
        // target_ide is None, neither is running, there is no classic version but there is IDE exe, it is deduced to be IDE
        let (is_ide, effective) = resolve_effective_target(
            None,
            false,
            false,
            false,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_autodetect_default_fallback() {
        // target_ide is None, none of them are running and none of the exes are detected. The default is the guaranteed classic version.
        let (is_ide, effective) = resolve_effective_target(None, false, false, false, None);
        assert!(!is_ide);
        assert_eq!(effective, None);
    }
}
