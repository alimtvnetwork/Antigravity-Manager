//! Unit tests for Caller IDE Exit Code 2 Enforcer (`Imp-693`, Step 279).

#[cfg(test)]
mod tests {
    use crate::modules::instance::caller_ide_exit_code_enforcer::{
        evaluate_instance_action, evaluate_instance_action_with_policy, evaluate_instance_command,
        is_caller_target, CallerInstanceContext, CallerProtectionPolicy, CallerProtectionVerdict,
        CommandInterceptionResult, InstanceCommandType, TargetInstanceCommand,
        CALLER_PROTECTION_EXIT_CODE,
    };

    #[test]
    fn test_exit_code_constant_is_two() {
        assert_eq!(CALLER_PROTECTION_EXIT_CODE, 2);
    }

    #[test]
    fn test_destructive_commands_flagged_correctly() {
        assert!(InstanceCommandType::Stop.is_destructive());
        assert!(InstanceCommandType::Remove.is_destructive());
        assert!(InstanceCommandType::Switch.is_destructive());
        assert!(InstanceCommandType::Restart.is_destructive());

        assert!(!InstanceCommandType::NonDestructive("status".to_string()).is_destructive());
        assert!(!InstanceCommandType::NonDestructive("logs".to_string()).is_destructive());
        assert!(!InstanceCommandType::NonDestructive("inspect".to_string()).is_destructive());
    }

    #[test]
    fn test_command_string_representation() {
        assert_eq!(InstanceCommandType::Stop.as_str(), "stop");
        assert_eq!(InstanceCommandType::Remove.as_str(), "rm");
        assert_eq!(InstanceCommandType::Switch.as_str(), "switch");
        assert_eq!(InstanceCommandType::Restart.as_str(), "restart");
        assert_eq!(
            InstanceCommandType::NonDestructive("status".to_string()).as_str(),
            "status"
        );
        assert_eq!(InstanceCommandType::Stop.to_string(), "stop");
    }

    #[test]
    fn test_different_target_returns_exit_code_0_not_intercepted() {
        let ctx = CallerInstanceContext::new(
            Some("inst-developer-main".to_string()),
            Some(4200),
            Some("Antigravity-IDE".to_string()),
        );

        let res =
            evaluate_instance_command("inst-worker-01", InstanceCommandType::Stop, false, &ctx);

        assert!(!res.is_intercepted);
        assert_eq!(res.exit_code, 0);
        assert!(!res.is_caller_match);
        assert!(!res.was_force_provided);
        assert_eq!(res.target_instance_id, "inst-worker-01");
        assert!(res.reason.contains("safe"));
    }

    #[test]
    fn test_same_target_destructive_without_force_fails_fast_with_exit_code_2() {
        let ctx = CallerInstanceContext::new(
            Some("inst-developer-main".to_string()),
            Some(4200),
            Some("Antigravity-IDE".to_string()),
        );

        let destructive_commands = vec![
            InstanceCommandType::Stop,
            InstanceCommandType::Remove,
            InstanceCommandType::Switch,
            InstanceCommandType::Restart,
        ];

        for cmd in destructive_commands {
            let res = evaluate_instance_command("inst-developer-main", cmd.clone(), false, &ctx);

            assert!(
                res.is_intercepted,
                "Command {:?} targeting caller must be intercepted",
                cmd
            );
            assert_eq!(
                res.exit_code, 2,
                "Exit code must be 2 per AGENTS.md for {:?}",
                cmd
            );
            assert!(res.is_caller_match);
            assert!(!res.was_force_provided);
            assert_eq!(res.target_instance_id, "inst-developer-main");
            assert!(res.reason.contains("4200"));
            assert!(res.reason.contains("Antigravity-IDE"));
            assert!(res.remediation_hint.contains("--force"));
            assert!(res.remediation_hint.contains(cmd.as_str()));
        }
    }

    #[test]
    fn test_same_target_destructive_with_force_is_permitted() {
        let ctx = CallerInstanceContext::from_caller_id("inst-developer-main");

        let res = evaluate_instance_command(
            "inst-developer-main",
            InstanceCommandType::Remove,
            true,
            &ctx,
        );

        assert!(!res.is_intercepted);
        assert_eq!(res.exit_code, 0);
        assert!(res.is_caller_match);
        assert!(res.was_force_provided);
        assert!(res.reason.contains("Bypassing caller self-preservation"));
        assert!(res.remediation_hint.is_empty());
    }

    #[test]
    fn test_nondestructive_command_on_caller_is_permitted_without_force() {
        let ctx = CallerInstanceContext::new(
            Some("inst-developer-main".to_string()),
            Some(1234),
            Some("VSCode".to_string()),
        );

        let res = evaluate_instance_command(
            "inst-developer-main",
            InstanceCommandType::NonDestructive("status".to_string()),
            false,
            &ctx,
        );

        assert!(!res.is_intercepted);
        assert_eq!(res.exit_code, 0);
        assert!(res.is_caller_match);
        assert!(!res.was_force_provided);
        assert!(res.reason.contains("non-destructive"));
    }

    #[test]
    fn test_empty_or_whitespace_handling() {
        let ctx = CallerInstanceContext::from_caller_id("  inst-developer-main  ");

        // Target with leading/trailing spaces should match trimmed caller
        let res = evaluate_instance_command(
            "  inst-developer-main  ",
            InstanceCommandType::Stop,
            false,
            &ctx,
        );
        assert!(res.is_intercepted);
        assert_eq!(res.exit_code, 2);

        // Empty target should not intercept
        let res_empty = evaluate_instance_command("", InstanceCommandType::Stop, false, &ctx);
        assert!(!res_empty.is_intercepted);
        assert_eq!(res_empty.exit_code, 0);
    }

    #[test]
    fn test_terminal_error_formatter() {
        let ctx = CallerInstanceContext::new(
            Some("inst-active".to_string()),
            Some(9999),
            Some("Antigravity".to_string()),
        );

        let blocked =
            evaluate_instance_command("inst-active", InstanceCommandType::Stop, false, &ctx);
        let error_msg = blocked.format_terminal_error();
        assert!(error_msg.contains("Error [Exit Code 2]"));
        assert!(error_msg.contains("Remediation:"));
        assert!(error_msg.contains("--force"));

        let display_str = blocked.to_string();
        assert!(display_str.contains("[BLOCKED - exit code 2]"));

        let permitted =
            evaluate_instance_command("inst-active", InstanceCommandType::Stop, true, &ctx);
        assert!(!permitted.format_terminal_error().contains("Error"));
        assert!(permitted.to_string().contains("[PERMITTED]"));
    }

    #[test]
    fn test_legacy_evaluate_instance_action_compatibility() {
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

        let verdict_blocked = evaluate_instance_action(
            "inst-developer-main",
            Some("inst-developer-main"),
            false,
            TargetInstanceCommand::Stop,
        );
        assert!(verdict_blocked.is_blocked());
        assert_eq!(verdict_blocked.exit_code(), Some(2));

        let verdict_forced = evaluate_instance_action(
            "inst-developer-main",
            Some("inst-developer-main"),
            true,
            TargetInstanceCommand::Remove,
        );
        assert!(verdict_forced.is_allowed());
    }

    #[test]
    fn test_legacy_policy_custom_exit_code() {
        let custom_policy = CallerProtectionPolicy::new(42, true);
        let verdict = custom_policy.evaluate(
            "inst-main",
            Some("inst-main"),
            false,
            TargetInstanceCommand::Stop,
        );
        assert_eq!(verdict.exit_code(), Some(42));
    }
}
