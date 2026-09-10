# Workflow: Conversation Log & Context Wrapper

**Date:** 2026-09-09
**Status:** Active Workflow
**Scope:** `conversation/`, `prompts/`, `.lovable/memory/`

---

## 1. Trigger Conditions & Workflow Purpose

- **When Triggered:** Whenever the user pastes the conversation log workflow prompt before a complex multi-task execution, or requests context persistence and prompt rewriting.
- **Goal:** Safely snapshot the ongoing chat history, verify memory and guidelines, stage the rewritten prompt for execution, and require human-in-the-loop review before taking actions on codebase files.

---

## 2. Phase Breakdown

### Phase 1: Conversation History Persistence
1. Verify root directory `conversation/` exists.
2. Determine next zero-padded 3-digit sequence prefix (`001`, `002`, ...).
3. Chunk multi-topic conversations into logical units or write a unified session log (`conversation/NNN-<topic-slug>.md`).
4. Preserve all user directives verbatim within blockquotes.
5. Provide concise factual bullets for assistant responses and operations.

### Phase 2: Memory & Guidelines Audit
1. Read `.lovable/memory/01-index.md` and check for missing standard pointers (`.lovable/coding-guidelines.md`, `.lovable/plan.md`, `mem://workflow/conversation-log`).
2. Identify proposed memory writes without modifying memory silently.

### Phase 3: Instruction Rewriting & Staging
1. Rewrite the follow-up prompt into a production-grade specification in `prompts/NNN-<slug>.md`.
2. Register the staged prompt in `01-prompts/01-prompt-library-setup/01-prompt-library-setup.md` under `## Instructions Index`.
3. Verify all relative links.

### Phase 4: Local CI/CD Quality Verification
1. Run `python 03-ai-scripts/06-cicd-local-runner.py`.
2. Ensure all 36 quality gates pass with zero failures.

### Phase 5: Reporting & Staging Hold
1. Begin response with `"Understood - staging only, not executing."`.
2. Emit the 5 required sections (files written, memory check, rewritten prompt summary, ambiguities/questions, next steps).
3. Halt execution. Await user's explicit `"go"` or `"execute"` command.
