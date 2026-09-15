# Conversation Log & Context Wrapper Protocol

**Date:** 2026-09-09
**Status:** Learned & Consolidated
**Scope:** `conversation/`, `prompts/`, `01-prompts/`, `.lovable/memory/`

---

## 1. Purpose & Motivation

Agentic context windows are vulnerable to truncation and catastrophic memory loss across extended multi-turn sessions.

The **Conversation Log & Context Wrapper** workflow establishes a strict pre-execution protocol:
1. Persist the entire preceding chat history into structured, numbered markdown logs under `conversation/NNN-<topic-slug>.md`.
2. Rewrite and stage follow-up user instructions into unambiguous specifications under `prompts/NNN-<slug>.md`.
3. Verify coding guidelines and memory references without executing unapproved actions.
4. Require explicit user confirmation before executing staged work.

---

## 2. Hard Rules & Invariants

1. **Non-Destructive Append-Only:**
   - Existing conversation files are immutable history. New sessions append the next zero-padded 3-digit prefix (`001`, `002`, ...).
2. **Verbatim User Blockquotes:**
   - User prompts and directives must be preserved verbatim inside markdown blockquotes (`> ...`).
   - Summarizing, paraphrasing, or softening user language is strictly forbidden.
3. **Factual Assistant Logs:**
   - Assistant actions are recorded as concise, factual bullet points detailing files touched, commands executed, and concrete outcomes.
   - Internal model scratchpad, thinking tokens, and speculative commentary must not be persisted into conversation logs.
4. **Zero-Execution Staging Boundary:**
   - The agent MUST NOT execute or implement any part of the staged follow-up prompt during the staging turn.
   - The agent MUST begin its response with the mandatory acknowledgment: `"Understood - staging only, not executing."`
   - Execution begins ONLY when the user explicitly provides approval (e.g. `"go"`, `"execute"`).
5. **Strict Relative Git Paths:**
   - Conversation and prompt files must never contain absolute paths or `file:///` URIs.
   - All references must use repository root-relative paths.

---

## 3. Library Integration & Cross-Referencing

- Staged prompts in `prompts/NNN-<slug>.md` must be registered in `01-prompts/01-prompt-library-setup/01-prompt-library-setup.md` under the `## Instructions Index` table.
- All 36 CI/CD local runner gates (`python 03-ai-scripts/06-cicd-local-runner.py`) must pass cleanly with 0 broken links or sequence integrity errors.
