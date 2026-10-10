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
        // 显式指定 classic，即便 IDE 正在运行或只有 IDE exe，也必须严格判定为经典版
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
        // 显式指定 ide，必须判定为 ide
        let (is_ide, effective) = resolve_effective_target(Some("ide"), true, false, true, None);
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    pub(crate) fn test_resolve_effective_target_autodetect_classic_running() {
        // target_ide 为 None，经典版正在运行，必须优先保持经典版
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
        // target_ide 为 None，仅 IDE 正在运行，推导为 IDE
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
        // target_ide 为 None，两者均未运行，但经典版 exe 存在，优先经典版
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
        // target_ide 为 None，两者均未运行，无经典版但有 IDE exe，推导为 IDE
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
        // target_ide 为 None，均未运行且均未检测到 exe，默认保底经典版
        let (is_ide, effective) = resolve_effective_target(None, false, false, false, None);
        assert!(!is_ide);
        assert_eq!(effective, None);
    }
}
