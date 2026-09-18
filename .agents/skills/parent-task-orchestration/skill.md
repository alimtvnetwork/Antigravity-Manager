---
name: parent-task-orchestration
description: Autonomously orchestrate and execute complex multi-step parent tasks by decomposing into subtasks and running continuous self-loop execution until completion.
---

# [V2] Parent Task N-Step Continuous Loop & Multi-Agent Orchestration — Workflow

> **Prompt Version:** 2.2.0
> **Synchronization:** Main Meta-Repo & Connected Workspaces

## Core Protocol

### Phase 1: Planning Mode & Subtask Generation FIRST (Steps 1 .. N/2)
1. **Scan & Discover:** Spawn planning subagents to scan codebase for target changes.
2. **Master Spec Generation:** Save master architectural plan into `.ai-memory/plans/pending/xx-<slug>.md` with 3-5 custom rules/constraints.
3. **Lean Subtask Decomposition:** Break down master plan into granular subtasks in `.ai-memory/plans/subtasks/xx-<slug>/01-<subtask>.md`.
4. **Mandatory Auto-Loop:** Transition directly into execution mode without stopping.

### Phase 2: Execution Mode & Parallel Refactoring (Steps N/2+1 .. N)
1. **Parallel Dispatch:** Spawn execution subagents assigned to disjoint subtasks.
2. **Coding Guidelines:** Strict adherence to function size (<= 8-15 lines), positive boolean prefixes (`is*`, `has*` only), Unix LF, UTF-8 without BOM.
3. **Banned Operations:** Zero routine test running, zero build checking, zero per-file commits.
4. **Targeted Quality Linting:** Run fast file-level linters on modified files only.

### Phase 3: Task Consolidation & File Reduction (End of Loop)
1. Consolidate completed subtasks into `.ai-memory/plans/completed/xx-<slug>.md`.
2. Delete subtask folder and pending plan.
3. Update `.ai-memory/plans/01-index.md`.
4. Mandatory final commit and push to git in a single atomic commit.
