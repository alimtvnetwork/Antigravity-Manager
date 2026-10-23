//! Unit tests for Caller IDE Exit Code 2 Enforcer (`Imp-641`).

#[cfg(test)]
mod tests {
    use crate::modules::instance::caller_ide_exit_code_enforcer::{
        evaluate_instance_action, evaluate_instance_action_with_policy, is_caller_target,
        CallerProtectionPolicy, CallerProtectionVerdict, TargetInstanceCommand,
        CALLER_PROTECTION_EXIT_CODE,
    };

    #[test]
    fn test_different_target_returns_allowed_safe_target() {
        let verdict = evaluate_instance_action(
            "inst-worker-01",
            Some("inst-developer-main"),
            false,
            TargetInstanceCommand::Stop,
        );

        assert!(verdict.is_allowed());
        assert!(!verdict.is_blocked());
        assert_eq!(verdict.exit_code(), None);
        assert_eq!(verdict, CallerProtectionVerdict::AllowedSafeTarget);
    }

    #[test]
    fn test_same_target_without_force_blocks_with_exit_code_2() {
        let verdict = evaluate_instance_action(
            "inst-developer-main",
            Some("inst-developer-main"),
            false,
            TargetInstanceCommand::Stop,
        );

        assert!(!verdict.is_allowed());
        assert!(verdict.is_blocked());
        assert_eq!(verdict.exit_code(), Some(2));
        assert_eq!(verdict.exit_code(), Some(CALLER_PROTECTION_EXIT_CODE));

        match verdict {
            CallerProtectionVerdict::BlockedCallerProtectionFailFast {
                caller_id,
                command,
                exit_code,
            } => {
                assert_eq!(caller_id, "inst-developer-main");
                assert_eq!(command, TargetInstanceCommand::Stop);
                assert_eq!(exit_code, 2);
            }
            _ => panic!("Expected BlockedCallerProtectionFailFast"),
        }
    }

    #[test]
    fn test_same_target_with_force_returns_allowed_force_bypass() {
        let verdict = evaluate_instance_action(
            "inst-developer-main",
            Some("inst-developer-main"),
            true,
            TargetInstanceCommand::Remove,
        );

        assert!(verdict.is_allowed());
        assert!(!verdict.is_blocked());
        assert_eq!(verdict.exit_code(), None);

        match verdict {
            CallerProtectionVerdict::AllowedForceBypass { caller_id } => {
                assert_eq!(caller_id, "inst-developer-main");
            }
            _ => panic!("Expected AllowedForceBypass"),
        }
    }

    #[test]
    fn test_none_caller_instance_returns_allowed_safe_target() {
        let verdict = evaluate_instance_action(
            "inst-developer-main",
            None,
            false,
            TargetInstanceCommand::Switch,
        );

        assert!(verdict.is_allowed());
        assert!(!verdict.is_blocked());
        assert_eq!(verdict.exit_code(), None);
        assert_eq!(verdict, CallerProtectionVerdict::AllowedSafeTarget);
    }

    #[test]
    fn test_all_lifecycle_commands_blocked_without_force() {
        let commands = [
            TargetInstanceCommand::Stop,
            TargetInstanceCommand::Remove,
            TargetInstanceCommand::Switch,
            TargetInstanceCommand::Restart,
            TargetInstanceCommand::Kill,
        ];

        for cmd in commands {
            let verdict = evaluate_instance_action("inst-active", Some("inst-active"), false, cmd);

            assert!(verdict.is_blocked(), "Command {} should be blocked", cmd);
            assert_eq!(verdict.exit_code(), Some(CALLER_PROTECTION_EXIT_CODE));
        }
    }

    #[test]
    fn test_is_caller_target_normalization() {
        assert!(is_caller_target(
            "  inst-active  ",
            Some("inst-active\n")
        ));
        assert!(!is_caller_target("inst-active", Some("other-instance")));
        assert!(!is_caller_target("inst-active", None));
        assert!(!is_caller_target("inst-active", Some("")));
        assert!(!is_caller_target("inst-active", Some("   ")));
        assert!(!is_caller_target("", Some("inst-active")));
    }

    #[test]
    fn test_custom_policy_custom_exit_code_and_disallowed_force() {
        let strict_policy = CallerProtectionPolicy::new(99, false);

        // Even with is_forced = true, allow_force = false prevents bypass
        let verdict = strict_policy.evaluate(
            "inst-locked",
            Some("inst-locked"),
            true,
            TargetInstanceCommand::Kill,
        );

        assert!(verdict.is_blocked());
        assert_eq!(verdict.exit_code(), Some(99));
    }

    #[test]
    fn test_guidance_messages() {
        let blocked = evaluate_instance_action(
            "inst-1",
            Some("inst-1"),
            false,
            TargetInstanceCommand::Stop,
        );
        let msg = blocked.guidance_message().unwrap();
        assert!(msg.contains("exit code 2"));
        assert!(msg.contains("Pass --force to override"));

        let bypass =
            evaluate_instance_action("inst-1", Some("inst-1"), true, TargetInstanceCommand::Stop);
        let bypass_msg = bypass.guidance_message().unwrap();
        assert!(bypass_msg.contains("explicit --force flag"));

        let safe =
            evaluate_instance_action("inst-1", Some("inst-2"), false, TargetInstanceCommand::Stop);
        assert_eq!(safe.guidance_message(), None);
    }

    #[test]
    fn test_command_string_and_display() {
        assert_eq!(TargetInstanceCommand::Stop.as_str(), "stop");
        assert_eq!(TargetInstanceCommand::Remove.as_str(), "rm");
        assert_eq!(TargetInstanceCommand::Switch.as_str(), "switch");
        assert_eq!(TargetInstanceCommand::Restart.as_str(), "restart");
        assert_eq!(TargetInstanceCommand::Kill.as_str(), "kill");

        assert_eq!(TargetInstanceCommand::Stop.to_string(), "stop");
        assert_eq!(TargetInstanceCommand::Remove.to_string(), "rm");
    }

    #[test]
    fn test_serde_roundtrip() {
        let verdict = CallerProtectionVerdict::BlockedCallerProtectionFailFast {
            caller_id: "inst-primary".to_string(),
            command: TargetInstanceCommand::Switch,
            exit_code: 2,
        };

        let json = serde_json::to_string(&verdict).expect("serialize verdict");
        let deserialized: CallerProtectionVerdict =
            serde_json::from_str(&json).expect("deserialize verdict");

        assert_eq!(verdict, deserialized);

        let policy = CallerProtectionPolicy::default();
        let policy_json = serde_json::to_string(&policy).expect("serialize policy");
        let deserialized_policy: CallerProtectionPolicy =
            serde_json::from_str(&policy_json).expect("deserialize policy");

        assert_eq!(policy, deserialized_policy);
    }
}
