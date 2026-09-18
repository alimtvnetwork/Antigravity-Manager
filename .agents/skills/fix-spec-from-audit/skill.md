---
name: fix-spec-from-audit
description: >-
  Autonomously ingest specification audit findings, decompose into a 1:1 remediation checklist, remediate specifications, and close audit gaps.
---

# Instruction (must follow): Specification Remediation from Audit Findings

/goal Autonomously ingest the latest specification audit file from `02-spec/25-app-spec-audit/`, decompose every finding into an exhaustive 1:1 remediation checklist, spawn parallel subagents to fix the specifications, verify 100% compliance, and remove the audit gap at the final stage.

```text
N = 200
PHASE_1_STEPS = N / 2   (Steps 1 .. N/2: Audit Ingestion, Finding Matrix & Subtask Decomposition)
PHASE_2_STEPS = N / 2   (Steps N/2+1 .. N: Parallel Remediation, CI Verification & Gap Removal)
```

## Shared Directory Contract (Non-Negotiable)

Both the auditor and fixer agents MUST operate against these exact paths:
- **Audit Reports Directory:** `02-spec/25-app-spec-audit/`
- **Default Target Spec Directory:** `02-spec/21-app/` (or user-specified subfolder)
- **Subtask Tracking:** `.ai-memory/plans/subtasks/xx-spec-fix/`
- **Completed Archive:** `.ai-memory/plans/completed/`
- **Agent State Directory:** `.ai-memory/temp-agents/`

## Master Pipeline (Atomic Numbered Steps)

1. [ ] **Phase 1: Audit Ingestion:** Scan `02-spec/25-app-spec-audit/` and select the audit file with the highest numerical sequence prefix (`NN-audit-*.md`).
2. [ ] **Phase 1: Finding Parsing:** Parse the Markdown Summary Table and individual findings without skipping a single issue.
3. [ ] **Phase 1: Remediation Ledger:** Create `.ai-memory/plans/pending/xx-spec-remediation.md` with explicit checkboxes for each finding.
4. [ ] **Phase 1: Subtask Decomposition:** Write lean subtasks to `.ai-memory/plans/subtasks/xx-spec-fix/01-<slug>.md`.
5. [ ] **Phase 1: Zero-Stop Transition:** Transition directly into Phase 2 without pausing.
6. [ ] **Phase 2: Parallel Remediation:** Fix each target spec file per the subtask specifications using dedicated temp-agent tracking.
7. [ ] **Phase 3: Validation Gate:** Verify relative links, markdown standards, and CI guards.
8. [ ] **Phase 4: Gap Removal & Archive:** Mark all items resolved, remove/archive the audit gap file, synchronize indexes, and commit.
