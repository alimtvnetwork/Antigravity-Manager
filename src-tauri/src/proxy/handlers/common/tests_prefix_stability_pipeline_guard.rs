//! Unit Tests for Prefix Stability Pipeline Guard (`Imp-200`)

#[cfg(test)]
mod tests {
    use super::super::prefix_stability_pipeline_guard::*;

    #[test]
    fn test_clean_appended_new_turn_preserves_prefix() {
        let baseline = build_dialog_turns([
            ("system", "You are an AI coding assistant."),
            ("user", "Write a hello world program in Rust."),
            ("assistant", "fn main() { println!(\"Hello, world!\"); }"),
        ]);
        let new_history = build_dialog_turns([
            ("system", "You are an AI coding assistant."),
            ("user", "Write a hello world program in Rust."),
            ("assistant", "fn main() { println!(\"Hello, world!\"); }"),
            ("user", "Now do it in Go."),
        ]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(report.is_stable());
        assert!(!report.has_violations());
        assert_eq!(report.prefix_turns_preserved, 3);
        assert_eq!(report.total_new_turns, 4);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_mutating_system_prompt_on_subsequent_turn() {
        let baseline = build_dialog_turns([
            ("system", "Original system prompt."),
            ("user", "Hello there."),
        ]);
        let new_history = build_dialog_turns([
            ("system", "Mutated system prompt."),
            ("user", "Hello there."),
            ("assistant", "General Kenobi!"),
        ]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(!report.is_stable());
        assert!(report.has_violations());
        assert_eq!(report.prefix_turns_preserved, 0);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            PrefixStabilityViolation::SystemPromptMutated {
                initial_hash,
                mutated_hash,
            } => {
                assert_eq!(initial_hash, &baseline[0].content_hash);
                assert_eq!(mutated_hash, &new_history[0].content_hash);
            }
            other => panic!("Expected SystemPromptMutated, got: {:?}", other),
        }
    }

    #[test]
    fn test_mutating_previous_user_message() {
        let baseline = build_dialog_turns([
            ("system", "System instructions."),
            ("user", "Original user question."),
            ("assistant", "First assistant answer."),
        ]);
        let new_history = build_dialog_turns([
            ("system", "System instructions."),
            ("user", "Retroactively modified user question."),
            ("assistant", "First assistant answer."),
        ]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(!report.is_stable());
        assert_eq!(report.prefix_turns_preserved, 1);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            PrefixStabilityViolation::EarlierTurnContentMutated {
                turn_index,
                snippet,
            } => {
                assert_eq!(*turn_index, 1);
                assert_eq!(snippet, "Retroactively modified user question.");
            }
            other => panic!("Expected EarlierTurnContentMutated, got: {:?}", other),
        }
    }

    #[test]
    fn test_mutating_previous_assistant_message() {
        let baseline = build_dialog_turns([
            ("system", "System prompt."),
            ("user", "Question 1."),
            ("assistant", "Original answer 1."),
        ]);
        let new_history = build_dialog_turns([
            ("system", "System prompt."),
            ("user", "Question 1."),
            ("assistant", "Altered answer 1."),
            ("user", "Question 2."),
        ]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(!report.is_stable());
        assert_eq!(report.prefix_turns_preserved, 2);
        assert_eq!(report.violations.len(), 1);

        match &report.violations[0] {
            PrefixStabilityViolation::EarlierTurnContentMutated {
                turn_index,
                snippet,
            } => {
                assert_eq!(*turn_index, 2);
                assert_eq!(snippet, "Altered answer 1.");
            }
            other => panic!("Expected EarlierTurnContentMutated, got: {:?}", other),
        }
    }

    #[test]
    fn test_swapping_order_of_earlier_turns() {
        let baseline =
            build_dialog_turns([("user", "Turn 0 User"), ("assistant", "Turn 1 Assistant")]);
        let new_history =
            build_dialog_turns([("assistant", "Turn 1 Assistant"), ("user", "Turn 0 User")]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(!report.is_stable());
        assert_eq!(report.prefix_turns_preserved, 0);

        let has_role_swap = report.violations.iter().any(|v| match v {
            PrefixStabilityViolation::TurnOrderAltered {
                turn_index,
                expected_role,
                actual_role,
            } => *turn_index == 0 && expected_role == "user" && actual_role == "assistant",
            _ => false,
        });
        assert!(
            has_role_swap,
            "Expected TurnOrderAltered violation for turn 0"
        );
    }

    #[test]
    fn test_dropping_earlier_turns_truncation() {
        let baseline = build_dialog_turns([
            ("system", "System prompt."),
            ("user", "Question 1."),
            ("assistant", "Answer 1."),
            ("user", "Question 2."),
        ]);
        let new_history =
            build_dialog_turns([("system", "System prompt."), ("user", "Question 1.")]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(!report.is_stable());
        assert_eq!(report.prefix_turns_preserved, 2);
        assert_eq!(report.total_new_turns, 2);

        let truncation_violation = report
            .violations
            .iter()
            .find(|v| matches!(v, PrefixStabilityViolation::TurnHistoryTruncated { .. }));

        match truncation_violation {
            Some(PrefixStabilityViolation::TurnHistoryTruncated {
                baseline_turns,
                new_turns,
            }) => {
                assert_eq!(*baseline_turns, 4);
                assert_eq!(*new_turns, 2);
            }
            _ => panic!("Expected TurnHistoryTruncated violation"),
        }
    }

    #[test]
    fn test_violation_display_formatting_and_error_trait() {
        let v1 = PrefixStabilityViolation::SystemPromptMutated {
            initial_hash: "hash_init".to_string(),
            mutated_hash: "hash_mut".to_string(),
        };
        let v2 = PrefixStabilityViolation::TurnOrderAltered {
            turn_index: 2,
            expected_role: "user".to_string(),
            actual_role: "model".to_string(),
        };
        let v3 = PrefixStabilityViolation::EarlierTurnContentMutated {
            turn_index: 1,
            snippet: "Mutated content preview".to_string(),
        };
        let v4 = PrefixStabilityViolation::TurnHistoryTruncated {
            baseline_turns: 5,
            new_turns: 3,
        };

        assert!(v1.to_string().contains("System prompt mutated"));
        assert!(v2.to_string().contains("Turn order altered at turn 2"));
        assert!(v3
            .to_string()
            .contains("Earlier turn content mutated at turn 1"));
        assert!(v4
            .to_string()
            .contains("Turn history truncated: baseline has 5"));

        // Verify std::error::Error trait bound
        let error_ref: &dyn std::error::Error = &v1;
        assert_eq!(error_ref.to_string(), v1.to_string());

        let report = PrefixStabilityReport {
            is_stable: false,
            prefix_turns_preserved: 1,
            total_new_turns: 3,
            violations: vec![v1, v2],
        };
        let summary = report.violation_summary();
        assert!(summary.contains("System prompt mutated"));
        assert!(summary.contains("Turn order altered"));
    }

    #[test]
    fn test_empty_baseline_initial_turns() {
        let baseline = Vec::<DialogTurn>::new();
        let new_history =
            build_dialog_turns([("system", "System prompt."), ("user", "Hello world.")]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(report.is_stable());
        assert_eq!(report.prefix_turns_preserved, 0);
        assert_eq!(report.total_new_turns, 2);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_both_histories_empty() {
        let baseline = Vec::<DialogTurn>::new();
        let new_history = Vec::<DialogTurn>::new();

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(report.is_stable());
        assert_eq!(report.prefix_turns_preserved, 0);
        assert_eq!(report.total_new_turns, 0);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_snippet_preview_truncation_and_utf8() {
        let long_ascii = "A".repeat(120);
        let turn_ascii = create_dialog_turn(0, "user", &long_ascii);
        assert_eq!(turn_ascii.content_snippet.chars().count(), 60);

        let utf8_multibyte = "✨ 统一上下文结构对齐与前缀稳定性检测保护 🚀 确保模型缓存效率提升并防止 prompt 漂移与状态不一致！继续输入更多内容以验证截断处理。";
        let turn_utf8 = create_dialog_turn(1, "system", utf8_multibyte);
        assert!(turn_utf8.content_snippet.chars().count() <= 60);
        assert!(turn_utf8
            .content_snippet
            .starts_with("✨ 统一上下文结构对齐"));
    }

    #[test]
    fn test_compute_turn_hash_properties() {
        let hash_a = compute_turn_hash("user", "identical content");
        let hash_b = compute_turn_hash("user", "identical content");
        let hash_diff_role = compute_turn_hash("assistant", "identical content");
        let hash_diff_content = compute_turn_hash("user", "different content");

        assert_eq!(hash_a, hash_b, "Hash should be deterministic");
        assert_ne!(hash_a, hash_diff_role, "Hash should discriminate roles");
        assert_ne!(
            hash_a, hash_diff_content,
            "Hash should discriminate content"
        );
        assert_eq!(
            hash_a.len(),
            64,
            "SHA-256 hex digest should be 64 characters"
        );
    }

    #[test]
    fn test_multiple_simultaneous_violations() {
        let baseline = build_dialog_turns([
            ("system", "System baseline."),
            ("user", "User prompt."),
            ("assistant", "Model reply."),
        ]);
        let new_history =
            build_dialog_turns([("system", "System altered."), ("assistant", "Model reply.")]);

        let report = verify_prefix_stability(&baseline, &new_history);
        assert!(!report.is_stable());
        assert_eq!(report.total_new_turns, 2);
        // TurnHistoryTruncated + SystemPromptMutated + TurnOrderAltered at turn 1 + EarlierTurnContentMutated at turn 1
        assert!(report.violations.len() >= 3);
        assert!(report.has_violations());
    }

    #[test]
    fn test_report_serde_roundtrip() {
        let original_report = PrefixStabilityReport {
            is_stable: false,
            prefix_turns_preserved: 2,
            total_new_turns: 4,
            violations: vec![PrefixStabilityViolation::EarlierTurnContentMutated {
                turn_index: 2,
                snippet: "Sample mutated snippet".to_string(),
            }],
        };

        let json = serde_json::to_string(&original_report).expect("Serialization failed");
        let deserialized: PrefixStabilityReport =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(original_report, deserialized);
    }

    #[test]
    fn test_dialog_turn_constructors() {
        let turn_manual = DialogTurn::new(0, "user", "abc123hash", "hello snippet");
        assert_eq!(turn_manual.turn_index, 0);
        assert_eq!(turn_manual.role, "user");
        assert_eq!(turn_manual.content_hash, "abc123hash");
        assert_eq!(turn_manual.content_snippet, "hello snippet");

        let turn_from_content = DialogTurn::from_role_and_content(1, "system", "sys prompt");
        assert_eq!(turn_from_content.turn_index, 1);
        assert_eq!(turn_from_content.role, "system");
        assert_eq!(turn_from_content.content_snippet, "sys prompt");
        assert!(!turn_from_content.content_hash.is_empty());
    }
}
