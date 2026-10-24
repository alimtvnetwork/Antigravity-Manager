# Master Execution Plan: 224-repository-setup-and-installation-instructions

- **Specification Directory:** [02-spec/21-app/224-repository-setup-and-installation-instructions/](../../02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md)
- **Subtasks Directory:** [.ai-memory/plans/subtasks/224-repository-setup-and-installation-instructions/](../subtasks/224-repository-setup-and-installation-instructions/)
- **Task ID:** `224`
- **Slug:** `repository-setup-and-installation-instructions`
- **Status:** `COMPLETED`
- **Date:** 2026-10-24

---

## User Request (Verbatim)

```text
# Repository Setup and Installation Instructions: high priority instruction, non-negotiable task

Hey, make sure that in the root of the repository, you have the run.ps1 file and build.ps1 file, build and also e2e.ps1 file. These are specific files that will do specific tasks. For example, build will only do the build using Rust and other stuff. And if the installer is not there, then it would run the install.ps1 file if it is in the Windows. Same will go into the Ubuntu and other factors. It will install everything into the right order. Usually, user can prefer C drive or D drive or Windows, where they wanted to install the cargo Rust or things like that. If they don't, by default, it's going to be installed in C drive. The two tool chains should be installed, and then it would also mark with the SQLite database, so that anytime later it checks that it would say, "This is already installed." And also it can verify and install at the same time. Make sure that you update and improve these skills and also update the AI memory folder with what to read, what to understand, and also root README file improve with the latest versions. Make sure you do that and also compare that. The LLV is currently installed in D drive. It's wrong. It should be installed. If we follow the skips fixture, the skips fixture is default drive. In this machine, the default drive is C drive, I believe. Please correct all these factors.

## task: Repository Setup and Installation Instructions
## slug: repository-setup-and-installation-instructions

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/XX-repository-setup-and-installation-instructions/ and enqueue plan task in .ai-memory/plans/pending/XX-repository-setup-and-installation-instructions.md (subtasks in .ai-memory/plans/subtasks/XX-repository-setup-and-installation-instructions/) first, (xx should be figured out by the folder structure and running AI tools, xx is sequence 01, 02)
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Ensure run.ps1, build.ps1, and e2e.ps1 files are present in the root of the repository.
6. Verify that build.ps1 performs the build using Rust and other necessary components.
7. Ensure install.ps1 is executed if the installer is not present, applicable for both Windows and Ubuntu environments.
8. Confirm installation order is correct and defaults to C drive if no preference is specified.
9. Install and verify two tool chains, marking them with SQLite database for future checks.
10. Update AI memory folder with relevant reading and understanding materials.
11. Improve the root README file with the latest versions and compare with existing documentation.
12. Correct LLV installation path to default C drive as per skips fixture.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing work for improved work or less friction, faster execution for AI and coding without errors and following Coding Guidelines.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Track in-flight file touches and prevent write collisions using GitMap RC Heatmap (`gitmap rc heatmap status` / `gitmap rc heatmap tree` / `gitmap rc heatmap live` / `gitmap rc heatmap sync`).
- Only add relative paths, never add absolute paths during work; this must be respected on release page and in release notes as well.
```

---

## 1. Specification & Subtask Index

- **Master Audit Ledger:** [02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md](../../02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md)
- **Architecture Spec:** [02-spec/21-app/224-repository-setup-and-installation-instructions/01-architecture-spec.md](../../02-spec/21-app/224-repository-setup-and-installation-instructions/01-architecture-spec.md)
- **Component Spec:** [02-spec/21-app/224-repository-setup-and-installation-instructions/02-component-spec.md](../../02-spec/21-app/224-repository-setup-and-installation-instructions/02-component-spec.md)
- **Subtask Directory:** [.ai-memory/plans/subtasks/224-repository-setup-and-installation-instructions/](../subtasks/224-repository-setup-and-installation-instructions/)
  1. `01-root-runners-architecture.md` (build.ps1, e2e.ps1, build.sh, e2e.sh, run.ps1) - COMPLETED
  2. `02-toolchain-drive-and-sqlite-tracking.md` (LLVM C: drive default, MSVC/Rust, toolchain.db) - COMPLETED
  3. `03-installer-parity-and-e2e-harness.md` (install.ps1/sh fallback, cross-platform parity) - COMPLETED
  4. `04-ai-memory-and-readme-sync.md` (README.md, README_EN.md, what-to-read.md) - COMPLETED
  5. `05-verification-and-atomic-commit.md` (Syntax validation, test checks, atomic commit) - COMPLETED

---

## 2. Multi-Agent Delegation Plan (A = 3, H = 2)

```text
Tree Hierarchy:
Root Orchestrator (Lead Agent)
├── Worker 01 (Toolchain & SQLite Specialist)
│   ├── Fix LLVM unpack target (C:\Program Files\LLVM / C:\LLVM default)
│   └── Implement toolchain.db SQLite tracking helper
├── Worker 02 (Runners & Build Specialist)
│   ├── Implement build.ps1, e2e.ps1, build.sh, e2e.sh
│   └── Align run.ps1 prerequisites self-healing
└── Worker 03 (Documentation & AI Memory Specialist)
    ├── Update README.md and README_EN.md
    └── Update .ai-memory/what-to-read.md & plans registry
```

---

## 3. Execution Phases & Progress Tracking

- [x] **Phase 1: Spec Authoring & Discovery (A = 3 discovery/authoring subagents)**
  - [x] Initialized Master Audit Ledger `00-master-audit-ledger.md`
  - [x] Enqueued Master Plan `224-repository-setup-and-installation-instructions.md`
  - [x] Author `01-architecture-spec.md` (Worker 02)
  - [x] Author `02-component-spec.md` (Worker 01)
  - [x] Author `03-e2e-and-installer-spec.md` (Worker 02)
  - [x] Author `04-memory-and-readme-spec.md` (Worker 03)
  - [x] Author Subtasks 01–05
- [x] **Phase 2: Code Implementation & Script Generation**
  - [x] Remediate `scripts/install-rust-toolchain.ps1` & `install.ps1` (C: drive default for LLVM, user drive preference parameter `-Drive`)
  - [x] Implement SQLite `toolchain.db` tracker (`scripts/toolchain-db.ps1` and `scripts/toolchain_db.py`)
  - [x] Implement root `build.ps1` and `build.sh`
  - [x] Implement root `e2e.ps1` and `e2e.sh`
  - [x] Update root `run.ps1` and `run.sh`
- [x] **Phase 3: Documentation, Memory & Quality Gates**
  - [x] Synchronize `README.md` and `README_EN.md`
  - [x] Update `.ai-memory/what-to-read.md`
  - [x] Verify script syntax (`pwsh -NoProfile -Command "Get-Command ..."` and shell syntax)
  - [x] Stage atomic GitMap commit: `gitmap cpf "scripts - add build.ps1 e2e.ps1 and enforce c-drive toolchain sqlite tracking"`
