# Subtask 01: Agent Prompt Sanitization Regex Engine (`Imp-660`)
Status: COMPLETED
Parent Plan: 207-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Code Quality & Backend Fix Strategy:
"For agent-client prompt sanitization, prioritize regex-based pattern matching over static keyword replacement, ensuring full coverage without stripping pipeline system prompts or user queries."

Implement `src-tauri/src/proxy/handlers/common/agent_prompt_sanitization_regex_engine.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_agent_prompt_sanitization_regex_engine.rs`.

## Requirements
- Define structured sanitization rules with compiled regexes.
- Mask dangerous prompt injection payloads and unescaped instruction overrides.
- Preserve pipeline system instructions and legitimate multi-turn user conversation queries.
- Reject static string keyword replacements in favor of regex pattern matching.
- Provide structured `PromptSanitizationReport` tracking replacements count, matched rule names, and modified content.
