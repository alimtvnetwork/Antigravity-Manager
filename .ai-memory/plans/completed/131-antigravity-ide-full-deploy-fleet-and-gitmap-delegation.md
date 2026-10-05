# Master Execution Plan: 131-antigravity-ide-full-deploy-fleet-and-gitmap-delegation

## User Request (Verbatim)

```text
Look, so far the projects are there in the Antigravity. I appreciate that, but there is a great issue. The issue is when I go to the Antigravity, I do see you do not apply the same thing. Not this instance, but original, the default instances theme. And also the preset mode is not copied from the default instance. And plugins are not applied, skills are not added. So, so many things are not correct. So write the script, PowerShell additional script in the repo secrets folder properly, additional, and then after you do that, make modifies to the `gitmap` so that we can delegate in the future to `gitmap`, fully deploy the Antigravity IDE to another machine, and it will do it automatically. Do you understand? Is it clear?

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Write the PowerShell script in the repo secrets folder properly.
4. Modify `gitmap` to enable future delegation for fully deploying the Antigravity IDE to another machine automatically.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]
```

---

## 1. Actionable Deliverables & Responsibilities

| Subtask ID | Title | Owner | Target Artifacts | Status |
|---|---|---|---|---|
| `Task-01` | Architecture Spec, Component Spec, RCA & Master Plan | Lead | `02-spec/21-app/131-.../`, `.ai-memory/plans/131-...md` | `DONE` |
| `Task-02` | Default Instance Asset Ingestion & Rust Parity Engine | Worker 01 | `src-tauri/src/modules/instance.rs`, `instances/default/home/.gemini/` | `QUEUED` |
| `Task-03` | Standalone Fleet Deployment Script in repo-secrets | Worker 01 | `d:/work/repo-secrets/02-antigravity-manager/scripts/deploy-antigravity-ide-fleet.ps1`, `scripts/deploy-antigravity-ide-fleet.ps1` | `QUEUED` |
| `Task-04` | GitMap Remote Delegation Integration | Worker 02 | `scripts/gitmap-delegate-deploy.ps1`, `D:/work/gitmap/cli/cmdagy/` | `QUEUED` |
| `Task-05` | 7-Gate Verification Scorecard & Deployment Simulation | Worker 02 | `scratch/verify_ide_deployment.py` | `QUEUED` |
| `Task-06` | Plan Consolidation & Single Atomic GitMap Push | Lead | `.ai-memory/plans/readme.md`, `.ai-memory/plans/completed/131-...md` | `QUEUED` |

---

## 2. Multi-Agent Execution Waves

### Wave 1: Research, Spec & Plan Authoring (Lead + Research Subagents)
- Completed verbatim capture and task breakdown in chat.
- Authored Architecture Spec (`01-architecture-spec.md`), Component Spec (`02-component-spec.md`), Root Cause Analysis (`03-root-cause-analysis.md`), and Master Plan (`131-antigravity-ide-full-deploy-fleet-and-gitmap-delegation.md`).
- Authored granular subtasks under `.ai-memory/plans/subtasks/131-antigravity-ide-full-deploy-fleet-and-gitmap-delegation/`.

### Wave 2: Implementation & Parity Enforcement (Workers 01 & 02)
- **Worker 01**:
  - Ingest host `.gemini/config/config.json`, plugins, and builtin skills into `instances/default/home/.gemini/`.
  - Update `src-tauri/src/modules/instance.rs` with `sync_instance_ide_parity` ensuring theme, presets, plugins, and skills propagate unconditionally.
  - Finalize `deploy-antigravity-ide-fleet.ps1` in `d:\work\repo-secrets\02-antigravity-manager\scripts\` and `scripts/`.
- **Worker 02**:
  - Update `scripts/gitmap-delegate-deploy.ps1` and verify GitMap CLI delegation.
  - Author and run `scratch/verify_ide_deployment.py` to evaluate the 7-Gate Scorecard (`VG-01` to `VG-07`) with 100% pass outcome.

### Wave 3: Consolidation & Release (Lead)
- Verify preflight checks.
- Update plans index in `.ai-memory/plans/readme.md`.
- Copy master plan to `.ai-memory/plans/completed/131-antigravity-ide-full-deploy-fleet-and-gitmap-delegation.md`.
- Commit and push single atomic commit via `gitmap cpf`.
