# Subtask 02: Prompt Tree 4-Tier Origin Hierarchy Formatter (`Imp-696`)
Sequence: 214
Step: 282
Status: IN_PROGRESS
Module: `src-tauri/src/modules/instance/prompt_tree_origin_hierarchy_formatter.rs`
Test Suite: `src-tauri/src/modules/instance/tests_prompt_tree_origin_hierarchy_formatter.rs`

## Description
Enforces AGENTS.md rule: "Prompt tree views and inspectors must discriminate among 4 distinct origin tiers: USER_PROMPT (Sky badge, User icon), SUBAGENT_INSTRUCTION (Purple badge, Bot icon), SYSTEM_MESSAGE (Zinc badge, Terminal icon), and TOOL_OUTPUT (Amber badge, Wrench icon). Subagents must nest hierarchically beneath their parent user prompt with branch glyphs (↳, └──)."
