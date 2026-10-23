//! Unit Tests for Prompt Tree Branch Hierarchy Formatter (`Imp-257`).

use super::prompt_tree_branch_hierarchy_formatter::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_node_formatting() {
        let root = TreeNodeDescriptor::new(
            "node-1",
            None::<String>,
            PromptOriginTier::UserPrompt,
            0,
            false,
            "Implement Feature X",
        );

        let row = format_tree_node(&root);
        assert_eq!(row.id, "node-1");
        assert_eq!(row.glyph_prefix, "");
        assert_eq!(row.badge_label, "USER_PROMPT");
        assert_eq!(row.badge_color, "sky");
        assert_eq!(row.formatted_line, "[USER_PROMPT] Implement Feature X");
    }

    #[test]
    fn test_subagent_intermediate_child_glyph() {
        let child = TreeNodeDescriptor::new(
            "node-2",
            Some("node-1"),
            PromptOriginTier::SubagentInstruction,
            1,
            false,
            "Research existing codebase",
        );

        let row = format_tree_node(&child);
        assert_eq!(row.id, "node-2");
        assert_eq!(row.glyph_prefix, "↳ ");
        assert_eq!(row.badge_label, "SUBAGENT_INSTRUCTION");
        assert_eq!(row.badge_color, "purple");
        assert_eq!(
            row.formatted_line,
            "↳ [SUBAGENT_INSTRUCTION] Research existing codebase"
        );
    }

    #[test]
    fn test_subagent_terminal_child_glyph() {
        let terminal_child = TreeNodeDescriptor::new(
            "node-3",
            Some("node-1"),
            PromptOriginTier::SubagentInstruction,
            1,
            true,
            "Apply final patches",
        );

        let row = format_tree_node(&terminal_child);
        assert_eq!(row.id, "node-3");
        assert_eq!(row.glyph_prefix, "└── ");
        assert_eq!(row.badge_label, "SUBAGENT_INSTRUCTION");
        assert_eq!(row.badge_color, "purple");
        assert_eq!(
            row.formatted_line,
            "└── [SUBAGENT_INSTRUCTION] Apply final patches"
        );
    }

    #[test]
    fn test_all_four_origin_tiers_badge_props_and_icons() {
        let tiers = [
            (PromptOriginTier::UserPrompt, "USER_PROMPT", "sky", "User"),
            (
                PromptOriginTier::SubagentInstruction,
                "SUBAGENT_INSTRUCTION",
                "purple",
                "Bot",
            ),
            (
                PromptOriginTier::SystemMessage,
                "SYSTEM_MESSAGE",
                "zinc",
                "Terminal",
            ),
            (
                PromptOriginTier::ToolOutput,
                "TOOL_OUTPUT",
                "amber",
                "Wrench",
            ),
        ];

        for (tier, expected_label, expected_color, expected_icon) in tiers {
            let (label, color) = origin_badge_props(tier);
            assert_eq!(label, expected_label);
            assert_eq!(color, expected_color);
            assert_eq!(origin_icon_name(tier), expected_icon);
        }
    }

    #[test]
    fn test_deeply_nested_subagents_and_tool_outputs() {
        let depth_2_child = TreeNodeDescriptor::new(
            "tool-1",
            Some("sub-1"),
            PromptOriginTier::ToolOutput,
            2,
            false,
            "run_command: git status",
        );
        let row_d2 = format_tree_node(&depth_2_child);
        assert_eq!(row_d2.glyph_prefix, "  ↳ ");
        assert_eq!(row_d2.badge_label, "TOOL_OUTPUT");
        assert_eq!(row_d2.badge_color, "amber");
        assert_eq!(
            row_d2.formatted_line,
            "  ↳ [TOOL_OUTPUT] run_command: git status"
        );

        let depth_2_terminal = TreeNodeDescriptor::new(
            "tool-2",
            Some("sub-1"),
            PromptOriginTier::ToolOutput,
            2,
            true,
            "read_file: report.json",
        );
        let row_d2_term = format_tree_node(&depth_2_terminal);
        assert_eq!(row_d2_term.glyph_prefix, "  └── ");
        assert_eq!(
            row_d2_term.formatted_line,
            "  └── [TOOL_OUTPUT] read_file: report.json"
        );

        let depth_3_child = TreeNodeDescriptor::new(
            "sub-sub-1",
            Some("sub-1"),
            PromptOriginTier::SubagentInstruction,
            3,
            false,
            "Deep subagent recursion",
        );
        let row_d3 = format_tree_node(&depth_3_child);
        assert_eq!(row_d3.glyph_prefix, "    ↳ ");
        assert_eq!(
            row_d3.formatted_line,
            "    ↳ [SUBAGENT_INSTRUCTION] Deep subagent recursion"
        );
    }

    #[test]
    fn test_branch_glyph_helpers() {
        assert_eq!(BranchGlyph::Root.as_str(), "");
        assert_eq!(BranchGlyph::ChildBranch.as_str(), "↳ ");
        assert_eq!(BranchGlyph::TerminalBranch.as_str(), "└── ");

        assert_eq!(
            BranchGlyph::from_depth_and_position(0, false),
            BranchGlyph::Root
        );
        assert_eq!(
            BranchGlyph::from_depth_and_position(0, true),
            BranchGlyph::Root
        );
        assert_eq!(
            BranchGlyph::from_depth_and_position(1, false),
            BranchGlyph::ChildBranch
        );
        assert_eq!(
            BranchGlyph::from_depth_and_position(1, true),
            BranchGlyph::TerminalBranch
        );
    }

    #[test]
    fn test_format_prompt_tree_full_conversation_hierarchy() {
        let nodes = vec![
            TreeNodeDescriptor::new(
                "p-root",
                None::<String>,
                PromptOriginTier::UserPrompt,
                0,
                false,
                "Diagnose memory spike",
            ),
            TreeNodeDescriptor::new(
                "s-sys",
                Some("p-root"),
                PromptOriginTier::SystemMessage,
                1,
                false,
                "Context injected: 10 commits",
            ),
            TreeNodeDescriptor::new(
                "a-worker",
                Some("p-root"),
                PromptOriginTier::SubagentInstruction,
                1,
                false,
                "Launch heap analyzer subagent",
            ),
            TreeNodeDescriptor::new(
                "t-tool1",
                Some("a-worker"),
                PromptOriginTier::ToolOutput,
                2,
                false,
                "Heap snapshot captured: 120MB",
            ),
            TreeNodeDescriptor::new(
                "t-tool2",
                Some("a-worker"),
                PromptOriginTier::ToolOutput,
                2,
                true,
                "Dominator tree computed",
            ),
            TreeNodeDescriptor::new(
                "a-worker-done",
                Some("p-root"),
                PromptOriginTier::SubagentInstruction,
                1,
                true,
                "Synthesis complete",
            ),
        ];

        let rows = format_prompt_tree(&nodes);
        assert_eq!(rows.len(), 6);

        assert_eq!(
            rows[0].formatted_line,
            "[USER_PROMPT] Diagnose memory spike"
        );
        assert_eq!(
            rows[1].formatted_line,
            "↳ [SYSTEM_MESSAGE] Context injected: 10 commits"
        );
        assert_eq!(
            rows[2].formatted_line,
            "↳ [SUBAGENT_INSTRUCTION] Launch heap analyzer subagent"
        );
        assert_eq!(
            rows[3].formatted_line,
            "  ↳ [TOOL_OUTPUT] Heap snapshot captured: 120MB"
        );
        assert_eq!(
            rows[4].formatted_line,
            "  └── [TOOL_OUTPUT] Dominator tree computed"
        );
        assert_eq!(
            rows[5].formatted_line,
            "└── [SUBAGENT_INSTRUCTION] Synthesis complete"
        );
    }

    #[test]
    fn test_empty_prompt_tree_returns_empty_vec() {
        let rows = format_prompt_tree(&[]);
        assert!(rows.is_empty());
    }
}
