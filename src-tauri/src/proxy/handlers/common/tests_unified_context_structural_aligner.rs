//! Unit Tests for Unified Context Structural Aligner (`Imp-626`)

#[cfg(test)]
mod tests {
    use super::super::unified_context_structural_aligner::*;
    use serde_json::json;

    #[test]
    fn test_merge_adjacent_duplicate_user_turns() {
        let input = vec![
            RoleTurn::text("user", "Hello"),
            RoleTurn::text("user", "World"),
        ];

        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 1);
        assert_eq!(aligned[0].role, "user");
        assert_eq!(aligned[0].parts.len(), 2);
        assert_eq!(aligned[0].parts[0], json!({ "text": "Hello" }));
        assert_eq!(aligned[0].parts[1], json!({ "text": "World" }));

        assert!(!report.is_aligned);
        assert_eq!(report.turns_merged_count, 1);
        assert_eq!(report.empty_parts_pruned_count, 0);
        assert_eq!(report.final_turn_count, 1);
        assert!(matches!(
            &report.violations_detected[0],
            AlignmentViolation::ConsecutiveDuplicateRole { role, index: 1 } if role == "user"
        ));
    }

    #[test]
    fn test_merge_adjacent_duplicate_model_turns() {
        let input = vec![
            RoleTurn::text("user", "What is 2+2?"),
            RoleTurn::text("model", "It is 4."),
            RoleTurn::text("model", "Can I help with anything else?"),
        ];

        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].role, "user");
        assert_eq!(aligned[1].role, "model");
        assert_eq!(aligned[1].parts.len(), 2);
        assert_eq!(aligned[1].parts[0], json!({ "text": "It is 4." }));
        assert_eq!(
            aligned[1].parts[1],
            json!({ "text": "Can I help with anything else?" })
        );

        assert!(!report.is_aligned);
        assert_eq!(report.turns_merged_count, 1);
        assert_eq!(report.final_turn_count, 2);
        assert!(matches!(
            &report.violations_detected[0],
            AlignmentViolation::ConsecutiveDuplicateRole { role, index: 2 } if role == "model"
        ));
    }

    #[test]
    fn test_prune_empty_text_parts() {
        let input = vec![
            RoleTurn::new(
                "user",
                vec![
                    json!({ "text": "" }),
                    json!({ "text": "   " }),
                    json!({ "text": "Valid message" }),
                ],
            ),
            RoleTurn::text("model", "Reply"),
        ];

        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].parts.len(), 1);
        assert_eq!(aligned[0].parts[0], json!({ "text": "Valid message" }));

        assert!(!report.is_aligned);
        assert_eq!(report.empty_parts_pruned_count, 2);
        assert_eq!(report.turns_merged_count, 0);
        assert!(report.violations_detected.is_empty());
    }

    #[test]
    fn test_empty_parts_in_turn_backfilled_with_default() {
        let input = vec![
            RoleTurn::new("user", vec![json!({ "text": "   " })]),
            RoleTurn::text("model", "Response"),
        ];

        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].parts.len(), 1);
        assert_eq!(
            aligned[0].parts[0],
            json!({ "text": DEFAULT_FALLBACK_TEXT })
        );

        assert!(!report.is_aligned);
        assert_eq!(report.empty_parts_pruned_count, 1);
        assert!(matches!(
            &report.violations_detected[0],
            AlignmentViolation::EmptyPartsInTurn { index: 0 }
        ));
    }

    #[test]
    fn test_already_aligned_turns_pass_unmodified() {
        let input = vec![
            RoleTurn::text("user", "Hello"),
            RoleTurn::text("model", "Hi there"),
            RoleTurn::text("user", "How are you?"),
            RoleTurn::text("model", "I am doing well!"),
        ];

        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned, input);
        assert!(report.is_aligned);
        assert_eq!(report.turns_merged_count, 0);
        assert_eq!(report.empty_parts_pruned_count, 0);
        assert!(report.violations_detected.is_empty());
        assert_eq!(report.final_turn_count, 4);
    }

    #[test]
    fn test_edge_case_empty_turns_list() {
        let input: Vec<RoleTurn> = Vec::new();
        let (aligned, report) = align_context_turns(&input);

        assert!(aligned.is_empty());
        assert!(report.is_aligned);
        assert_eq!(report.final_turn_count, 0);
        assert_eq!(report.turns_merged_count, 0);
        assert_eq!(report.empty_parts_pruned_count, 0);
    }

    #[test]
    fn test_edge_case_single_user_turn() {
        let input = vec![RoleTurn::text("user", "Single prompt")];
        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 1);
        assert_eq!(aligned[0].role, "user");
        assert!(report.is_aligned);
        assert_eq!(report.final_turn_count, 1);
    }

    #[test]
    fn test_edge_case_initial_model_turn_prepends_user() {
        let input = vec![RoleTurn::text("model", "Unsolicited greeting")];
        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].role, "user");
        assert_eq!(
            aligned[0].parts[0],
            json!({ "text": DEFAULT_FALLBACK_TEXT })
        );
        assert_eq!(aligned[1].role, "model");
        assert_eq!(
            aligned[1].parts[0],
            json!({ "text": "Unsolicited greeting" })
        );

        assert!(!report.is_aligned);
        assert_eq!(report.final_turn_count, 2);
        assert!(matches!(
            &report.violations_detected[0],
            AlignmentViolation::InvalidRoleSequence { expected, found, index: 0 }
            if expected == "user" && found == "model"
        ));
    }

    #[test]
    fn test_non_canonical_assistant_role_mapped_to_model() {
        let input = vec![
            RoleTurn::text("user", "Hi"),
            RoleTurn::text("assistant", "Greetings"),
        ];

        let (aligned, report) = align_context_turns(&input);

        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].role, "user");
        assert_eq!(aligned[1].role, "model");

        assert!(!report.is_aligned);
        assert!(matches!(
            &report.violations_detected[0],
            AlignmentViolation::InvalidRoleSequence { expected, found, index: 1 }
            if expected == "model" && found == "assistant"
        ));
    }

    #[test]
    fn test_serde_roundtrip_all_alignment_structures() {
        let turn = RoleTurn::text("user", "Test message");
        let turn_json = serde_json::to_string(&turn).expect("serialize turn");
        let deserialized_turn: RoleTurn =
            serde_json::from_str(&turn_json).expect("deserialize turn");
        assert_eq!(turn, deserialized_turn);

        let violation = AlignmentViolation::ConsecutiveDuplicateRole {
            role: "user".to_string(),
            index: 2,
        };
        let violation_json = serde_json::to_string(&violation).expect("serialize violation");
        let deserialized_violation: AlignmentViolation =
            serde_json::from_str(&violation_json).expect("deserialize violation");
        assert_eq!(violation, deserialized_violation);

        let report = ContextAlignmentReport::new(1, 2, vec![violation], 3);
        let report_json = serde_json::to_string(&report).expect("serialize report");
        let deserialized_report: ContextAlignmentReport =
            serde_json::from_str(&report_json).expect("deserialize report");
        assert_eq!(report, deserialized_report);
    }
}
