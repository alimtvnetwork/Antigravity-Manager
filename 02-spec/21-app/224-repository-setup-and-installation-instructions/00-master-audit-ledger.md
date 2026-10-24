# Master Audit Ledger: 224-repository-setup-and-installation-instructions

- **Parent Spec:** [01-architecture-spec.md](01-architecture-spec.md)
- **Component Spec:** [02-component-spec.md](02-component-spec.md)
- **Task ID:** `224`
- **Date:** 2026-10-24
- **Status:** `COMPLETED`

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

## 1. Specification & Subtask Matrix

| Step ID | Specification File | Subtask Plan | Focus Area | Status |
| :---: | :--- | :--- | :--- | :---: |
| **01** | `01-architecture-spec.md` | `subtasks/224-repository-setup-and-installation-instructions/01-root-runners-architecture.md` | Runner suite architecture (`run.ps1`, `build.ps1`, `e2e.ps1`, `build.sh`, `e2e.sh`). | `COMPLETED` |
| **02** | `02-component-spec.md` | `subtasks/224-repository-setup-and-installation-instructions/02-toolchain-drive-and-sqlite-tracking.md` | LLVM C: drive default, MSVC/Rust toolchain verification, SQLite `toolchain.db` tracking. | `COMPLETED` |
| **03** | `03-e2e-and-installer-spec.md` | `subtasks/224-repository-setup-and-installation-instructions/03-installer-parity-and-e2e-harness.md` | Fallback to `install.ps1` / `install.sh`, cross-platform parity, E2E test runner. | `COMPLETED` |
| **04** | `04-memory-and-readme-spec.md` | `subtasks/224-repository-setup-and-installation-instructions/04-ai-memory-and-readme-sync.md` | `README.md`, `README_EN.md`, `.ai-memory/what-to-read.md` synchronization. | `COMPLETED` |
| **05** | `05-verification-and-release.md` | `subtasks/224-repository-setup-and-installation-instructions/05-verification-and-atomic-commit.md` | Verification gate and atomic GitMap commit. | `COMPLETED` |

---

## 2. Multi-Agent Tree Swarm Topology (A = 3, H = 2)

- **Root Orchestrator (Lead):** Master ledger, plan enqueuing, wave dispatching, and final atomic GitMap commit.
- **Worker 01 (Toolchain & SQLite Specialist):**
  - Fixes LLVM unpack logic in `scripts/install-rust-toolchain.ps1` and `install.ps1`.
  - Implements SQLite `toolchain.db` tracking and fast verification.
- **Worker 02 (Runners & Build Specialist):**
  - Implements `build.ps1`, `e2e.ps1`, `build.sh`, `e2e.sh`.
  - Aligns `run.ps1` with prerequisites self-healing.
- **Worker 03 (Documentation & AI Memory Specialist):**
  - Synchronized `README.md` and `README_EN.md` with Root Runner Suite section and `v4.257.2` version tags.
  - Codified Task 224 changelog and toolchain database invariants in `.ai-memory/what-to-read.md`.
  - Authored Subtasks 04 and 05.

---

## 3. Execution Ledger & Completion Tracking

- [x] **Phase 1: Specifications & Subtasks Authored**
  - [x] Step 01: `01-architecture-spec.md` & `01-root-runners-architecture.md`
  - [x] Step 02: `02-component-spec.md` & `02-toolchain-drive-and-sqlite-tracking.md`
  - [x] Step 03: `03-e2e-and-installer-spec.md` & `03-installer-parity-and-e2e-harness.md`
  - [x] Step 04: `04-memory-and-readme-spec.md` & `04-ai-memory-and-readme-sync.md`
  - [x] Step 05: `05-verification-and-atomic-commit.md`
- [x] **Phase 2: Documentation & Memory Synchronization (Worker 03)**
  - [x] `README.md`: Added `## 🚀 Root Runner Suite & Developer Automation`, updated Option A one-liner to `v4.257.2`, prepended latest tags to version pinning table.
  - [x] `README_EN.md`: Updated subtitle to `(v4.257.2)`, version badge to `Version-4.257.2-blue`, added matching English runner suite section, updated installation commands to `v4.257.2`.
  - [x] `.ai-memory/what-to-read.md`: Updated timestamp to `2026-10-24T17:00:00Z`, prepended Task 224 changelog entry, added `00-master-audit-ledger.md` and `.ai-memory/toolchain.db` to "Before any task (always)".
- [ ] **Phase 3: Final Quality Gate & Atomic Commit (Root Orchestrator)**
  - [ ] Static syntax verification across runner scripts.
  - [ ] Relative Git paths invariant verification.
  - [ ] Atomic GitMap commit ceremony: `gitmap cpf "scripts - add build.ps1 e2e.ps1 and enforce c-drive toolchain sqlite tracking"`.
