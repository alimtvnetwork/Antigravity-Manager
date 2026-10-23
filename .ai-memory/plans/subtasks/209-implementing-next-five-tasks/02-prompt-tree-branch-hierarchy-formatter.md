# Subtask 02: Prompt Tree Branch Hierarchy Formatter (`Imp-671`)
Status: COMPLETED
Parent Plan: 209-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md 4-Tier Prompt Origin Classification & Hierarchy:
"Prompt tree views and inspectors must discriminate among 4 distinct origin tiers: `USER_PROMPT` (Sky badge, User icon), `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon), `SYSTEM_MESSAGE` (Zinc badge, Terminal icon), and `TOOL_OUTPUT` (Amber badge, Wrench icon). Subagents must nest hierarchically beneath their parent user prompt with branch glyphs (`↳`, `└──`)."

Implement `src-tauri/src/modules/instance/prompt_tree_branch_hierarchy_formatter.rs` and unit tests in `src-tauri/src/modules/instance/tests_prompt_tree_branch_hierarchy_formatter.rs`.

## Requirements
- Define 4 origin tiers: UserPrompt, SubagentInstruction, SystemMessage, ToolOutput.
- Assign branch glyphs: root nodes have no glyph, intermediate child nodes use `↳`, terminal child nodes use `└──`.
- Calculate nesting depth and format hierarchical tree display string with badges and glyphs.
- Provide structured report tracking origin tiers, depth hierarchy, and formatting results.
