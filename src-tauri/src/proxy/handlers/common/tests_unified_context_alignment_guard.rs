//! Unit tests for Unified Context Alignment Guard (`Imp-261`).

#[cfg(test)]
mod tests {
    use super::unified_context_alignment_guard::*;

    #[test]
    fn test_clean_alternating_turns_unchanged() {
        let turns = vec![
            ConversationTurn::user("Hello, AI!"),
            ConversationTurn::model("Greetings, user!"),
            ConversationTurn::user("Can you summarize our project?"),
        ];

        let report = align_conversation_turns(&turns);

        assert!(report.is_strictly_alternating);
        assert_eq!(report.initial_turn_count, 3);
        assert_eq!(report.aligned_turn_count, 3);
        assert_eq!(report.aligned_turns.len(), 3);
        assert!(report.is_unchanged());
        assert!(!report.has_modifications());
        assert_eq!(report.actions, vec![AlignmentAction::Unchanged]);
    }

    #[test]
    fn test_adjacent_user_turns_merged() {
        let turns = vec![
            ConversationTurn::user("First part of prompt."),
            ConversationTurn::user("Second part of prompt."),
        ];

        let report = align_conversation_turns(&turns);

        assert!(report.is_strictly_alternating);
        assert_eq!(report.initial_turn_count, 2);
        assert_eq!(report.aligned_turn_count, 1);
        assert_eq!(
            report.aligned_turns[0].text,
            "First part of prompt.\n\nSecond part of prompt."
        );
        assert_eq!(report.aligned_turns[0].role, ConversationRole::User);
        assert!(report.has_modifications());
        assert!(!report.is_unchanged());
        assert_eq!(
            report.actions,
            vec![AlignmentAction::ContiguousRolesMerged {
                role: ConversationRole::User,
                merged_count: 2,
            }]
        );
    }

    #[test]
    fn test_adjacent_model_turns_merged() {
        let turns = vec![
            ConversationTurn::user("Explain recursion."),
            ConversationTurn::model("Recursion is defining a problem in terms of itself."),
            ConversationTurn::model("For example, computing factorial: n! = n * (n-1)!."),
            ConversationTurn::model("Base cases terminate the recursion."),
        ];

        let report = align_conversation_turns(&turns);

        assert!(report.is_strictly_alternating);
        assert_eq!(report.initial_turn_count, 4);
        assert_eq!(report.aligned_turn_count, 2);
        assert_eq!(report.aligned_turns[0].role, ConversationRole::User);
        assert_eq!(report.aligned_turns[1].role, ConversationRole::Model);
        let expected_model_text = "Recursion is defining a problem in terms of itself.\n\nFor example, computing factorial: n! = n * (n-1)!.\n\nBase cases terminate the recursion.";
        assert_eq!(report.aligned_turns[1].text, expected_model_text);
        assert_eq!(
            report.actions,
            vec![AlignmentAction::ContiguousRolesMerged {
                role: ConversationRole::Model,
                merged_count: 3,
            }]
        );
    }

    #[test]
    fn test_system_message_repositioning() {
        let turns = vec![
            ConversationTurn::system("You are a helpful coding assistant."),
            ConversationTurn::user("Refactor this method."),
            ConversationTurn::system("Remember to follow rustfmt standards."),
            ConversationTurn::model("Done!"),
        ];

        let report = align_conversation_turns(&turns);

        assert!(report.is_strictly_alternating);
        assert_eq!(report.initial_turn_count, 4);
        assert_eq!(report.aligned_turn_count, 2);
        assert_eq!(report.aligned_turns[0].role, ConversationRole::User);
        assert_eq!(report.aligned_turns[0].text, "Refactor this method.");
        assert_eq!(report.aligned_turns[1].role, ConversationRole::Model);
        assert_eq!(report.aligned_turns[1].text, "Done!");
        assert_eq!(
            report.actions,
            vec![
                AlignmentAction::SystemTurnMovedToPreamble,
                AlignmentAction::SystemTurnMovedToPreamble,
            ]
        );
    }

    #[test]
    fn test_empty_conversation_handling() {
        let turns: Vec<ConversationTurn> = Vec::new();
        let report = align_conversation_turns(&turns);

        assert!(report.is_strictly_alternating);
        assert_eq!(report.initial_turn_count, 0);
        assert_eq!(report.aligned_turn_count, 0);
        assert!(report.aligned_turns.is_empty());
        assert_eq!(report.actions, vec![AlignmentAction::Unchanged]);
        assert!(report.is_unchanged());
    }

    #[test]
    fn test_initial_turn_not_user_backfills_synthetic_user() {
        let turns = vec![ConversationTurn::model(
            "Spontaneous greeting from assistant.",
        )];

        let report = align_conversation_turns(&turns);

        assert!(report.is_strictly_alternating);
        assert_eq!(report.initial_turn_count, 1);
        assert_eq!(report.aligned_turn_count, 2);
        assert_eq!(report.aligned_turns[0].role, ConversationRole::User);
        assert_eq!(report.aligned_turns[0].text, DEFAULT_SYNTHETIC_USER_PROMPT);
        assert_eq!(report.aligned_turns[1].role, ConversationRole::Model);
        assert_eq!(
            report.aligned_turns[1].text,
            "Spontaneous greeting from assistant."
        );
    }

    #[test]
    fn test_backfill_model_turn_action() {
        let mut report = ContextAlignmentReport::new(
            1,
            vec![ConversationTurn::user("User query without response")],
            vec![AlignmentAction::Unchanged],
        );

        report.backfill_model_turn(1, "Synthetic model acknowledgement.");

        assert_eq!(report.aligned_turn_count, 2);
        assert_eq!(report.aligned_turns[1].role, ConversationRole::Model);
        assert_eq!(
            report.aligned_turns[1].text,
            "Synthetic model acknowledgement."
        );
        assert!(report.is_strictly_alternating);
        assert_eq!(
            report.actions.last(),
            Some(&AlignmentAction::ModelTurnBackfilled { at_index: 1 })
        );
    }

    #[test]
    fn test_role_and_action_display_and_string_conversions() {
        assert_eq!(role_as_str(ConversationRole::User), "user");
        assert_eq!(role_as_str(ConversationRole::Model), "model");
        assert_eq!(role_as_str(ConversationRole::System), "system");

        assert_eq!(ConversationRole::User.to_string(), "user");
        assert_eq!(ConversationRole::Model.to_string(), "model");
        assert_eq!(ConversationRole::System.to_string(), "system");

        assert!(ConversationRole::User.is_user());
        assert!(!ConversationRole::User.is_model());
        assert!(!ConversationRole::User.is_system());

        assert!(ConversationRole::Model.is_model());
        assert!(!ConversationRole::Model.is_user());

        assert!(ConversationRole::System.is_system());
        assert!(!ConversationRole::System.is_user());

        let unch = AlignmentAction::Unchanged;
        assert!(unch.is_unchanged());
        assert_eq!(unch.to_string(), "Unchanged");

        let merged = AlignmentAction::ContiguousRolesMerged {
            role: ConversationRole::User,
            merged_count: 3,
        };
        assert!(!merged.is_unchanged());
        assert_eq!(
            merged.to_string(),
            "Contiguous user turns merged (count: 3)"
        );

        let backfill = AlignmentAction::ModelTurnBackfilled { at_index: 2 };
        assert_eq!(backfill.to_string(), "Model turn backfilled at index 2");

        let sys = AlignmentAction::SystemTurnMovedToPreamble;
        assert_eq!(sys.to_string(), "System turn moved to preamble");
    }

    #[test]
    fn test_turn_constructors_and_predicates() {
        let u = ConversationTurn::user("hello");
        assert!(u.is_user());
        assert!(!u.is_model());
        assert!(!u.is_system());
        assert_eq!(u.text, "hello");

        let m = ConversationTurn::model("response");
        assert!(m.is_model());
        assert!(!m.is_user());

        let s = ConversationTurn::system("instruction");
        assert!(s.is_system());
        assert!(!s.is_user());
    }
}
