#[cfg(test)]
mod tests {
    use super::super::origin_tier_hierarchy_guard::{
        classify_origin_tier, format_hierarchy_tree_node, validate_tree_hierarchy, OriginTier,
        TreeNodeInput,
    };

    #[test]
    fn test_classify_origin_tier() {
        assert_eq!(classify_origin_tier("USER_INPUT", false), OriginTier::UserPrompt);
        assert_eq!(
            classify_origin_tier("USER_INPUT", true),
            OriginTier::SubagentInstruction
        );
        assert_eq!(
            classify_origin_tier("SUBAGENT_INSTRUCTION", false),
            OriginTier::SubagentInstruction
        );
        assert_eq!(
            classify_origin_tier("SYSTEM_MESSAGE", false),
            OriginTier::SystemMessage
        );
        assert_eq!(classify_origin_tier("TOOL_CALLS", false), OriginTier::ToolOutput);
    }

    #[test]
    fn test_origin_tier_metadata() {
        let user = OriginTier::UserPrompt;
        assert_eq!(user.as_str(), "USER_PROMPT");
        assert_eq!(user.badge_color_family(), "sky");
        assert_eq!(user.icon_name(), "user");

        let bot = OriginTier::SubagentInstruction;
        assert_eq!(bot.badge_color_family(), "purple");
        assert_eq!(bot.icon_name(), "bot");

        let sys = OriginTier::SystemMessage;
        assert_eq!(sys.badge_color_family(), "zinc");
        assert_eq!(sys.icon_name(), "terminal");

        let tool = OriginTier::ToolOutput;
        assert_eq!(tool.badge_color_family(), "amber");
        assert_eq!(tool.icon_name(), "wrench");
    }

    #[test]
    fn test_format_hierarchy_tree_node() {
        let root = format_hierarchy_tree_node(OriginTier::UserPrompt, 0, false, "Refactor proxy");
        assert_eq!(root, "[USER_PROMPT] Refactor proxy");

        let child1 = format_hierarchy_tree_node(
            OriginTier::SubagentInstruction,
            1,
            false,
            "Worker 1 researching",
        );
        assert_eq!(child1, "├── ↳ [SUBAGENT_INSTRUCTION] Worker 1 researching");

        let child2 = format_hierarchy_tree_node(
            OriginTier::SubagentInstruction,
            1,
            true,
            "Worker 2 assertions",
        );
        assert_eq!(child2, "└── ↳ [SUBAGENT_INSTRUCTION] Worker 2 assertions");
    }

    #[test]
    fn test_validate_tree_hierarchy_valid() {
        let nodes = vec![
            TreeNodeInput {
                id: "root-1".to_string(),
                parent_id: None,
                tier: OriginTier::UserPrompt,
                label: "Root prompt".to_string(),
            },
            TreeNodeInput {
                id: "sub-1".to_string(),
                parent_id: Some("root-1".to_string()),
                tier: OriginTier::SubagentInstruction,
                label: "Subagent worker".to_string(),
            },
        ];

        let report = validate_tree_hierarchy(&nodes);
        assert!(report.is_valid);
        assert_eq!(report.user_prompts_count, 1);
        assert_eq!(report.subagents_count, 1);
        assert!(report.unparented_subagents.is_empty());
    }

    #[test]
    fn test_validate_tree_hierarchy_unparented_subagent_rejected() {
        let nodes = vec![TreeNodeInput {
            id: "sub-orphan".to_string(),
            parent_id: None,
            tier: OriginTier::SubagentInstruction,
            label: "Orphan subagent".to_string(),
        }];

        let report = validate_tree_hierarchy(&nodes);
        assert!(!report.is_valid);
        assert_eq!(report.unparented_subagents, vec!["sub-orphan"]);
    }
}
