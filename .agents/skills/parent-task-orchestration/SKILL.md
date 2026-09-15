---
name: parent-task-orchestration
description: Autonomously orchestrate and execute complex multi-step parent tasks by decomposing into subtasks and running continuous self-loop execution until completion.
---

# Parent Task N-Step Continuous Loop & Multi-Agent Orchestration

## Core Protocol
1. **Planning Mode (Phase 1)**:
   - Deeply scan codebase for target changes and specifications.
   - Author master plan in `.lovable/plans/pending/xx-<slug>.md`.
   - Decompose into lean, atomic subtasks in `.lovable/plans/subtasks/xx-<slug>/01-*.md`.
   - Transition directly into execution without stopping.

2. **Execution Mode (Phase 2)**:
   - Execute subtasks with strict adherence to coding guidelines and error management.
   - Enforce positive boolean naming (`is*`, `has*` only).
   - Functions <= 8-15 lines, no nested `if`, universal `*AppError` wrapping.
   - Zero test running and zero build checking during routine turns.
   - Record modified files and run targeted file-level linters only.
