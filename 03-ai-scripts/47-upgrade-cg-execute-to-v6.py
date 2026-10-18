#!/usr/bin/env python3
"""Upgrade all 01-prompts/15-cg-execute prompts and companion skills to V6 architecture.

Enforces:
- Header Parameters: N = 400, A = 3, H = 2, C = 30, Capacity = 6, Tree Hierarchy
- Extreme 'Understood' Turn-1 Showcase with 5 Mandatory Pillars & 4-part verification schema
- Step 0 Preflight & Data Schema Verification Gate (gitmap agent task init --budget 400, --json, --ddl)
- Phase 1 Step 3 Brain & Comprehensive Roadmap Creation (Mandatory Planning Gate)
- Section 3 GitMap High-Speed Command Primacy Table with Heatmap suite (log, agent heatmap, query, agent task)
- Section 7.2 Worker Brief with Rule 9 Mandatory Heatmap Telemetry, 5-part Heatmap suite, and 3-Tier DB table
- Section 7.4 Immediate Dynamic Respawn & Join Protocol (100% duty cycle, pinned at A = 3 active subagents)
- Section 12 Banned Operations Checklist with HEATMAP TELEMETRY & CONTENTION INSPECTION checkbox
- GitMap hyphen commit format without colons ('<module> - <summary>') and 'No Push = Not Done'
- Preservation of domain-specific rule instructions & Prompt 31 suffix precedence structure
- Mirroring to 01-prompts/v4/15-cg-execute/
- Synchronization to companion skills in .agents/skills/ and .cursor/skills/
"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

PROMPTS_DIR = Path("01-prompts/15-cg-execute")
V4_PROMPTS_DIR = Path("01-prompts/v4/15-cg-execute")
AGENTS_SKILLS_DIR = Path(".agents/skills")
CURSOR_SKILLS_DIR = Path(".cursor/skills")

V6_HEADER_PARAMETERS = """```text
N = 400 (Total self-loop steps budget — editable top-header parameter, default: 400)
A = 3   (MANDATORY number of spawned autonomous subagents running concurrently via invoke_subagent, default: 3)
H = 2   (Operational hands per agent: dual-task batch capacity & parallel tool dispatch, default: 2)
C = 30  (Tool calls per worker before it must report, default: 30)

System Concurrency Capacity = A × H = 3 agents × 2 hands = 6 concurrent subtask operations
Tree Hierarchy = Lead Orchestrator (Root Node) -> Branch Coordinators (Subtask Owners) -> Leaf Workers & Dynamic Assistants
PHASE_1_BUDGET = 100     (Steps 1 .. 100: Planning, Discovery Subagents, Up-Front Brain & Roadmap Creation, Lean Subtasks)
PHASE_2_BUDGET = 150     (Steps 101 .. 250: Intermediate Archival, Sanitization, Prerequisite Refactoring, Intermediate Checkpoints)
PHASE_3_BUDGET = 150     (Steps 251 .. 400: Tree Swarm Parallel Execution, Dynamic Join Capacity Saturation, Targeted Quality Linting, Atomic Push)
WAVES = ceil(subtasks / (A x H))
```"""

V6_META_TEMPLATE_TOP = """> [!IMPORTANT]
> Prompt Version: 6.0.0
> Runtime: Google Antigravity 2.0 (IDE and CLI)
> Invoke: /{invoke_name} <task>
>
> **Top-Instruction Priority Mandate (Above Precedence / Preamble Precedence):**
> Whatever directives, constraints, checklists, or user instructions are given ABOVE this prompt (including in the user preamble, header blocks, or incoming user request above) are HIGHEST PRIORITY and MUST BE FOLLOWED as strictly NON-NEGOTIABLE. They supersede and strictly override any conflicting general advice, default conventions, or lower-level guidelines below. The agent MUST inspect and follow the instructions above with absolute precedence."""

V6_META_TEMPLATE_BOTTOM = """> [!IMPORTANT]
> Prompt Version: 6.0.0
> Runtime: Google Antigravity 2.0 (IDE and CLI)
> Invoke: /cg-execute-in-below-steps <task>
>
> **Bottom-Instruction Priority Mandate (Below Precedence / Suffix Precedence):**
> Whatever directives, constraints, checklists, or user instructions are given BELOW this prompt (following the `--` divider border at the bottom) are HIGHEST PRIORITY and MUST BE FOLLOWED as strictly NON-NEGOTIABLE."""

MANDATORY_SPAWNING_GATE_V6 = """### 🚨 MANDATORY SUBAGENT SPAWNING GATE (A = 3, H = 2, CAPACITY = 6 — TREE SWARM & DYNAMIC JOIN)

- **ACTUAL TOOL CALL REQUIRED:** You must ACTUALLY CALL the `invoke_subagent` tool via your tool-calling API. Do NOT just print the text "Dispatched Worker..." and stop. If you only print text, the agents will not spawn and the task will fail! You must execute the `invoke_subagent` JSON tool payload.
- **TREE-STRUCTURED SWARM HIERARCHY:** Organize execution as an operational tree:
  1. **Root Orchestrator (Lead Agent):** Owns the unified Brain roadmap, audit ledger, and final GitMap commit. Never executes file edits solo.
  2. **Branch Coordinators (Subtask Primary Owners):** Own designated disjoint file boundaries (`Owned Files`) and execute modifications.
  3. **Leaf Workers & Dynamic Joining Assistants:** Assist primary owners with parallel reading, symbol caller discovery, memory generation, and companion lint verification.
- **3-STAGE MANDATORY DISPATCH:** You must invoke `A = 3` agents (`invoke_subagent`) at EVERY stage of the workflow:
  1. **Planning Step (A = 3 `research` subagents):** Spawn 3 read-only discovery subagents (`TypeName: "research"`) to research the codebase and return findings; lead writes the unified Brain execution plan and roadmap.
  2. **Spec Step (A = 3 `self` subagents or lead):** Spawn 3 authoring subagents (`TypeName: "self"`) to author modular, disjoint spec files and subtasks (never writing the same file).
  3. **Execution Step (A = 3 `self` worker subagents):** Spawn 3 worker subagents (`TypeName: "self"`) to execute code modifications in strictly disjoint file boxes.
- **DYNAMIC AGENT JOIN & IMMEDIATE RESPAWN (NO IDLE WORKERS):** Worker capacity `A` must remain pinned at 3 active agents (System Concurrency Capacity = 6) throughout execution. The lead orchestrator MUST NOT wait for an entire 3-agent wave to finish. As soon as ANY single worker completes, the lead immediately spawns a replacement worker for the next pending subtask, or commands the worker to dynamically join an active subtask in collaborative assistance mode (parallel reading, companion linters, memory logging) to keep concurrency at exactly 3 active subagents (100% duty cycle).
- **SOLO EXECUTION IS AN AUTO-REJECT FAILURE:** The lead orchestrator is **STRICTLY FORBIDDEN** from executing planning, spec writing, or code changes by itself without calling the `invoke_subagent` tool. Failing to call the actual tool is a critical protocol violation."""

EXTREME_UNDERSTOOD_TURN_1_SHOWCASE = """### Phase 1A: Verbatim Capture, Task Extraction & Chat Output Gate (Step 0)

Before executing any file searches, scans, spec writing, or code changes, you must execute Phase 1A:

1. **Top-Instruction Priority Verification:** Whatever directives, constraints, checklists, or instructions are given before this section or prompt (user preamble, header constraints, prior instructions) must be verified as highest priority and non-negotiable.
2. **Extreme "Understood" Turn-1 Showcase (Turn 1 Action):** In your VERY FIRST response turn upon receiving the prompt, you MUST output a comprehensive, in-depth understanding block directly in visible chat before any background tool execution. Any generic boilerplate or superficial 1-sentence summaries (e.g. `Understood: [YES] — I will do this task`) are protocol auto-reject failures. Turn 1 MUST display the top-level mandate containing all 5 Mandatory Pillars followed by the confirmed task breakdown using the 4-part verification schema:
   * **Pillar 1: Strategic Intent & Systems Reasoning:** Articulate overarching architectural goal, user intent, operational rationale, and systems integration mechanics. Explain *why* this change is necessary and how downstream components interact.
   * **Pillar 2: Explicit Scope Boundaries:** Strict two-column partition defining In-Scope vs. strictly Excluded or Deferred items.
   * **Pillar 3: Non-Negotiable Constraints & Negative Guardrails Ingestion:** Explicit recitation of active constraints: raw search ban (`rg`, `ripgrep`, `Select-String`, `git grep`), worker git ban (no git commands in subagents), zero builds/full test suites (CI verification only), strict relative Git paths, lowercase naming, zero secrets in repo (`repo-secrets` offloading), GitMap hyphen-separated commit format without colons.
   * **Pillar 4: Blast Radius & Dependency Topology:** Map impacted subsystems, files, downstream callers, and prompt/skill indexes with explicit risk containment strategies.
   * **Pillar 5: Multi-Agent Concurrency Topology:** Detail swarm structure (`A = 3`, `H = 2`, Capacity = 6), disjoint file assignments, and immediate dynamic respawn.
3. **Lossless Verbatim Capture:** Store incoming prompt losslessly under `## User Request (Verbatim)` in canonical spec and parent plan.
4. **Screenshots & Media:** Decode base64/screenshots immediately into `assets/screenshots/<slug>-<NN>.png`. Reference via relative markdown links (`![Screenshot](assets/screenshots/<slug>-<NN>.png)`).
5. **Discrete Deliverables Extraction:** Break down whatever user requirements were given into discrete, actionable items with ordered traceable IDs (`Task-01`, `Task-02`, ...).
6. **Mandatory Same-Turn Tool Chaining (TOTAL BAN ON TURNING OFF):** Emit the comprehension mandate and breakdown in chat with clean vertical formatting, and in the **EXACT SAME TURN**, invoke your first tool call (e.g. `write_to_file` to initialize ledger/spec, or run preflight). NEVER emit text alone (which ends the turn prematurely), and never ask "Should I proceed?".

```markdown
### 🧠 Deep Comprehension & Understood Mandate

#### Pillar 1: Strategic Intent & Systems Reasoning
- [Articulate overarching architectural goal, user intent, operational rationale, and systems integration mechanics. Explain why this change is necessary and how downstream components interact.]

#### Pillar 2: Explicit Scope Boundaries (In-Scope vs. Excluded / Deferred)
- **In-Scope:** [Strict enumeration of files, modules, and changes executed in this turn.]
- **Excluded / Deferred:** [Explicitly deferred items, preserved untouched areas, and follow-up backlog.]

#### Pillar 3: Non-Negotiable Constraints & Negative Guardrails Ingestion
- [Explicit recitation of active constraints: raw search ban (no rg/ripgrep/Select-String), worker git ban (no git commands in subagents), zero builds/full test suites (CI verification only), strict relative Git paths, lowercase naming, zero secrets in repo (repo-secrets offloading), GitMap hyphen-separated commit format without colons.]

#### Pillar 4: Blast Radius & Dependency Topology
- **Impacted Subsystems & Files:** [Files, downstream callers, and prompt/skill indexes affected.]
- **Containment & Risk Mitigation:** [File lock prevention via disjoint ownership, backward compatibility invariants.]

#### Pillar 5: Multi-Agent Concurrency Topology & Wave Orchestration
- **Swarm Structure:** Lead Orchestrator (Root Node) -> Branch Coordinators (Subtask Owners) -> Leaf Workers & Dynamic Joining Assistants
- **Capacity:** A = 3 agents × H = 2 hands = 6 concurrent operations, WAVES = ceil(subtasks / 6)
- **Disjoint Bounding Boxes:** Zero file ownership overlap; dynamic immediate respawn on worker completion (100% duty cycle pinned at 3 active subagents)

### 📋 Confirmed Task Breakdown & Requirement Ingestion

1. **Task-01: [Descriptive Task Title]**
   - **State:** `[IN PROGRESS — EXECUTING IMMEDIATELY]`
   - **Intent & Technical Deliverables:**
     - [Clear statement of technical goal and deliverable artifacts]
     - [Precise implementation scope: functions, types, schemas, or prompt sections being added/edited]
     - [Quantifiable success metrics: line count, field completeness, schema compliance]
   - **Negative Boundaries & Invariants:**
     - [Explicit list of things this task MUST NOT do or alter]
     - [Untouchable files, reserved packages, or excluded dependencies]
     - [Invariants that must remain preserved after modification]
   - **Disjoint File Boundaries:**
     - `Owned Files (Exclusive):` `[strictly/relative/path/to/file-01.md]`, `[strictly/relative/path/to/file-02.md]`
     - `Read-Only Context Files:` `[strictly/relative/path/to/reference.md]`
   - **Verification Evidence Contract:**
     - [Exact command or check to be executed to prove completion]
     - [Expected exit code: e.g. exit 0]
     - [Required diffstat evidence or structural regex check]

2. **Task-02: [Descriptive Task Title]**
   - **State:** `[QUEUED — EXECUTING NOW WITHOUT USER PROMPT]`
   - **Intent & Technical Deliverables:**
     - [Clear statement of technical goal and deliverable artifacts]
     - [Precise implementation scope: functions, types, schemas, or prompt sections being added/edited]
     - [Quantifiable success metrics: line count, field completeness, schema compliance]
   - **Negative Boundaries & Invariants:**
     - [Explicit list of things this task MUST NOT do or alter]
     - [Untouchable files, reserved packages, or excluded dependencies]
     - [Invariants that must remain preserved after modification]
   - **Disjoint File Boundaries:**
     - `Owned Files (Exclusive):` `[strictly/relative/path/to/file-03.md]`
     - `Read-Only Context Files:` `[strictly/relative/path/to/reference.md]`
   - **Verification Evidence Contract:**
     - [Exact command or check to be executed to prove completion]
     - [Expected exit code: e.g. exit 0]
     - [Required diffstat evidence or structural regex check]

Proceeding directly to Preflight & Phase 1B Spec Generation (Active Tool Call Running Below).
```"""

EXTREME_UNDERSTOOD_TURN_1_SHOWCASE_PROMPT_31 = """### Phase 1A: Verbatim Capture, Task Extraction & Chat Output Gate (Step 0)

1. **Bottom-Instruction Priority Verification:** Verify whatever directives, constraints, or task instructions are given BELOW this prompt (following the `--` divider border at the bottom) as highest priority and non-negotiable.
2. **Extreme "Understood" Turn-1 Showcase (Turn 1 Action):** In your VERY FIRST response turn upon receiving the prompt, you MUST output a comprehensive, in-depth understanding block directly in visible chat before any background tool execution. Any generic boilerplate or superficial 1-sentence summaries are protocol auto-reject failures. Turn 1 MUST display the top-level mandate containing all 5 Mandatory Pillars followed by the confirmed task breakdown using the 4-part verification schema:
   * **Pillar 1: Strategic Intent & Systems Reasoning:** Articulate overarching architectural goal, user intent, operational rationale, and systems integration mechanics.
   * **Pillar 2: Explicit Scope Boundaries:** Strict two-column partition defining In-Scope vs. strictly Excluded or Deferred items.
   * **Pillar 3: Non-Negotiable Constraints & Negative Guardrails Ingestion:** Explicit recitation of active constraints: raw search ban (`rg`, `ripgrep`, `Select-String`, `git grep`), worker git ban (no git commands in subagents), zero builds/full test suites, strict relative Git paths, lowercase naming, zero secrets in repo (`repo-secrets` offloading), GitMap hyphen-separated commit format without colons.
   * **Pillar 4: Blast Radius & Dependency Topology:** Map impacted subsystems, files, downstream callers, and prompt/skill indexes with explicit risk containment strategies.
   * **Pillar 5: Multi-Agent Concurrency Topology:** Detail swarm structure (`A = 3`, `H = 2`, Capacity = 6), disjoint file assignments, and immediate dynamic respawn.
3. **Lossless Verbatim Capture:** Store incoming prompt losslessly under `## User Request (Verbatim)` in canonical spec and parent plan.
4. **Mandatory Same-Turn Tool Chaining (TOTAL BAN ON TURNING OFF):** Emit the comprehension mandate and breakdown in chat with clean vertical formatting, and in the **EXACT SAME TURN**, invoke your first tool call. Never pause or ask "Should I proceed?".

```markdown
### 🧠 Deep Comprehension & Understood Mandate

#### Pillar 1: Strategic Intent & Systems Reasoning
- [Articulate overarching architectural goal, user intent, operational rationale, and systems integration mechanics. Explain why this change is necessary.]

#### Pillar 2: Explicit Scope Boundaries (In-Scope vs. Excluded / Deferred)
- **In-Scope:** [Strict enumeration of files, modules, and changes executed in this turn.]
- **Excluded / Deferred:** [Explicitly deferred items, preserved untouched areas, and follow-up backlog.]

#### Pillar 3: Non-Negotiable Constraints & Negative Guardrails Ingestion
- [Explicit recitation of active constraints: raw search ban (no rg/ripgrep/Select-String), worker git ban (no git commands in subagents), zero builds/full test suites (CI verification only), strict relative Git paths, lowercase naming, zero secrets in repo (repo-secrets offloading), GitMap hyphen-separated commit format without colons.]

#### Pillar 4: Blast Radius & Dependency Topology
- **Impacted Subsystems & Files:** [Files, downstream callers, and prompt/skill indexes affected.]
- **Containment & Risk Mitigation:** [File lock prevention via disjoint ownership, backward compatibility invariants.]

#### Pillar 5: Multi-Agent Concurrency Topology & Wave Orchestration
- **Swarm Structure:** Lead Orchestrator (Root Node) -> Branch Coordinators (Subtask Owners) -> Leaf Workers & Dynamic Joining Assistants
- **Capacity:** A = 3 agents × H = 2 hands = 6 concurrent operations, WAVES = ceil(subtasks / 6)
- **Disjoint Bounding Boxes:** Zero file ownership overlap; dynamic immediate respawn on worker completion (100% duty cycle pinned at 3 active subagents)

### 📋 Confirmed Task Breakdown & Requirement Ingestion

1. **Task-01: [Descriptive Task Title]**
   - **State:** `[IN PROGRESS — EXECUTING IMMEDIATELY]`
   - **Intent & Technical Deliverables:**
     - [Clear statement of technical goal and deliverable artifacts]
     - [Precise implementation scope: functions, types, schemas, or prompt sections being added/edited]
     - [Quantifiable success metrics: line count, field completeness, schema compliance]
   - **Negative Boundaries & Invariants:**
     - [Explicit list of things this task MUST NOT do or alter]
     - [Untouchable files, reserved packages, or excluded dependencies]
   - **Disjoint File Boundaries:**
     - `Owned Files (Exclusive):` `[strictly/relative/path/to/file-01.md]`
     - `Read-Only Context Files:` `[strictly/relative/path/to/reference.md]`
   - **Verification Evidence Contract:**
     - [Exact command or check to be executed to prove completion: e.g. exit 0]

Proceeding directly to Phase 1B: Spec & Subtask Generation (Active Tool Call Running Below).
```"""

GITMAP_COMMAND_PRIMACY_TABLE_V6 = """## 3. GitMap High-Speed Command Primacy (Run Everything Faster)

GitMap is your **PRIMARY** acceleration engine. NEVER use `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`. Execute all searches and operations through GitMap:

| Operation | Primary GitMap Command | High-Speed Alias | Purpose & Advantage |
| :--- | :--- | :--- | :--- |
| **Live Streaming Search** | `gitmap aum search "<pattern>" [dir] [-e <.ext>] [-r] [-i]` | `gitmap aum grep "<pat>"` | Multi-core streaming live file search (replaces `Select-String`, `git grep`) |
| **Indexed Symbol Search** | `gitmap search "<query>" [--limit <n>]` | `gitmap search` | Instant SQLite cached keyword/symbol search across indexed repos |
| **Wildcard File Search** | `gitmap find "<pattern>" [-ext <ext>]` | `gitmap f "<pat>"` | Index-accelerated multi-core glob filename finder |
| **Directory Inventory** | `gitmap list-files [pattern] [-ext <ext>]` | `gitmap lf [pat]` | Instant indexed repository inventory |
| **Substring File Search** | `gitmap find-files-any "<substring>"` | `gitmap ffa "<str>"` | High-speed partial filename matcher |
| **Stream File** | `gitmap cat <filepath>` | `gitmap cat` | Zero-disk memory streaming to stdout |
| **PowerShell Runner** | `gitmap pwsh "<command>"` | `gitmap ps "<cmd>"` | High-speed PowerShell execution with `-NoProfile` |
| **Bash Runner** | `gitmap bash "<command>"` | `gitmap sh "<cmd>"` | Standard cross-platform Bash command execution |
| **Offload Secrets** | `gitmap rs file <filepath>` / `folder` / `text` | `gitmap rs` | Auto-commits into `repo-secrets` in work directory |
| **Offload Scripts** | `gitmap rc file <file.ps1>` / `text` | `gitmap rc` | Auto-commits reusable scripts into `repo-cache` |
| **Atomic Commits** | `gitmap cpf "<module> - <msg>"` (Feature) / `cpb` (Bug) | `gitmap cpf` | Stages, formats with prefix, and pushes atomically. Mention as a hyphen `-` (no need to provide a colon in GitMap `cpf`/`cpb`/CVF commit arguments because the colon is already automatically provided by GitMap in `Feature: ` or `Bug: `). |
| **Pipeline Waiting** | `gitmap pipeline-ai status --json` | `gitmap pl-ai` | Non-polling dynamic ETA CI/CD monitor |
| **Agent Task Engine** | `gitmap agent task <init|status|ls|pending>` | `gitmap agent task` | Sub-millisecond compiled Go SQLite task manager for multi-agent workflows |
| **Heatmap Action Log** | `gitmap heatmap log -t <slug> -a "READ"|"EDIT" -f <path> -r <reason>` | `gitmap heatmap log` | Concurrency-safe cognitive action & reasoning telemetry into `.ai-memory/heatmap.db` |
| **Contention & Leases** | `gitmap agent heatmap [--json]` / `gitmap heatmap status` | `gitmap agent heatmap` | Pre-edit file lease conflict & contention hotspot analysis across active agents |
| **Heatmap Diagnostics** | `gitmap heatmap query|tables|view` / `gitmap rc heatmap ...` | `gitmap heatmap` | Non-blocking read-only SQL queries, tree view, and live activity stream |
| **Fleet Synchronization** | `gitmap sync [--workers 8] [--projects <json>]` | `gitmap sync` | Parallel Go sync engine across 43 repositories in <5s with 6-stage ceremony |

### 🔍 Code & Symbol Search Protocol (TOTAL BAN ON `rg`, `ripgrep`, `Select-String` & `git grep`)
- **Live Disk Search (Default for discovery, symbol tracking & blast radius):**
  - Search string/symbol: `gitmap aum search "<symbol>" [dir] [-e <.ext>]` (e.g. `gitmap aum search "RunFleetPASCommand" cli -e .go`)
  - Search regex: `gitmap aum search -r "<regex>" [dir] [-e <.ext>]` (e.g. `gitmap aum search -r "(\"pas\"|\"pa\")" cli/cmd -e .go`)
  - Case-insensitive: `gitmap aum search -i "<query>" [dir]`
- **TOTAL BAN:** NEVER run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`. Running raw grep, ripgrep, or slow shell search pipelines wastes execution steps, causes Windows process hangs, and violates GitMap primacy. Search exclusively through `gitmap aum search` or `gitmap find`."""

STEP_0_PREFLIGHT_V6 = """## 5. Step 0: Preflight, Platform Handshake & SQLite Task DB Initialization (Phase 1 Budget)

1. **Platform Handshake:** Confirm tools (`invoke_subagent`, `send_message`, `manage_subagents`, `ask_question`, `write_to_file`, `replace_file_content`, `run_command`). If `task_boundary` exists: set `PLANNING` (Phase 1), `EXECUTION` (Phase 2), `VERIFICATION` (Phase 3).
2. **Commands & Directory:** Confirm `gitmap --version` and `python --version` exit 0. Verify GitMap with harmless call (`gitmap lf readme.md`), not `--help`. `run_command` uses `Cwd` in workspace root, paths relative. Never cd to other drives or tool folders.
3. **Working Tree Cleanliness:** Run `git status --porcelain`. Record modified files in ledger; never touch them. Confirm root `readme.md` is lowercase. Read `.ai-memory/what-to-read.md`, `strictly-avoid.md`, `coding-guidelines.md`.
4. **SQLite Task DB & Deterministic Slug Initialization (Check Before Creating):**
   - Initialize or inspect task state via GitMap SQLite task manager (compiled Go, sub-millisecond):
     `gitmap agent task init --name "<task name>" --budget 400`
   - Inspect Task DB Schema & Data Integrity Rules (Mandatory Preflight):
     `gitmap agent task status`
     Verify tables (`ParentTask`, `Subtask`, `AgentActionLog`), positive boolean flags (`IsActive`, `HasCompleted`, `IsBlocked`), and payload constraints before populating subtasks. (DDL: `gitmap agent task status --ddl`; JSON: `gitmap agent task status --json`).
   - Mandatory Heatmap & Companion RC Preflight:
     `gitmap rc heatmap tables`
     `gitmap heatmap status`
     `gitmap agent heatmap`
   - TOTAL BAN ON RAW PYTHON SQLITE: NEVER execute `python -c "import sqlite3..."` or ad-hoc Python DB scripts. Inspect status via `gitmap agent task status` or list subtasks via `gitmap agent subtask ls`.
   - Output Analysis & Crash Forensics:
     - If `action: "RESUME_FOUND"`: A matching or similar slug exists in `.ai-memory/temp-agents/`! If `diagnostics.hasCrashesDetected: true`, inspect the forensic report (`diagnostics.crashedAgents`) to identify which agent crashed, what file it was touching, and the last logged action. Resume execution from the uncompleted subtask.
     - If `action: "INITIALIZED"`: Created dedicated run directory `.ai-memory/temp-agents/<nn>-<slug>/` and SQLite database `agent-task.db` with WAL mode (`PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;`).
5. **Ledger Creation:** Also create `.ai-memory/temp-agents/<nn>-<slug>/ledger.md` mirroring the DB initialization for human readability:

```markdown
# Ledger: nn-<slug>
Request slug: <slug>
Request first line: <verbatim first line>
Status: ACTIVE
Phase: 1    Wave: 0 / WAVES    Step: 1 / N
Last completed action: Phase 1A Capture & Task Breakdown
Next action: Phase 1B Spec & Plan
Workers in flight: none
Commits: none    Pushed: no
Branch: <branch> | Tree at start: clean (or dirty with <paths>)
Tools: invoke_subagent=yes send_message=yes ask_question=yes gitmap=yes sqlite_db=<databasePath>
| Task-ID | Subtask | Owner | Owned files | Status | Evidence |
|---|---|---|---|---|---|
| Task-01 | 01-<name> | Worker 01 | <paths> | PENDING | - |
Assumptions: <list or none>
Conflicts: <list or none>
Stage list: <every path this run creates or modifies>
```"""

PHASE_1_PLANNING_SPEC_V6 = """## 6. Phase 1: Planning Step & Spec Step (Steps 1 .. PHASE_1_BUDGET)

You must use `invoke_subagent` to delegate both planning discovery and spec authoring.

> [!CAUTION]
> **SUBAGENT CRASH PREVENTION (MANDATORY ARCHITECTURAL RULES):**
> 1. **Tool Capabilities (`research` vs `self`):** Subagents with `TypeName: "research"` have READ-ONLY tools (`view_file`, `run_command`, web search). They DO NOT have `write_to_file` or `replace_file_content`. Commanding a `research` subagent to write files will cause an immediate tool failure and crash. Subagents that modify or author files MUST use `TypeName: "self"`.
> 2. **File Write Collisions & Windows Locks:** Two subagents MUST NEVER be assigned to create or write the same file path simultaneously in `Workspace: "inherit"`. Windows file locks and race conditions will crash both agents. In Phase 1 Planning, discovery subagents return findings via structured messages to the lead, and the lead alone writes the unified plan. In Phase 1 Spec, subagents author strictly disjoint modular files.
> 3. **Valid TypeName Strings:** `TypeName` MUST strictly be `"research"` (read-only discovery) or `"self"` (read-write execution). Passing custom strings (e.g. `"TypeName": "Research 01"` or `"TypeName": "Worker 01"`) is an invalid schema and immediately crashes `invoke_subagent`. Put descriptive labels in `Role` only.
> 4. **Worker Git Ban (Index Lock Prevention):** Subagents NEVER run git commands (`git add`, `git commit`, `git push`, `git status`, `git diff`, `git checkout`). In shared workspaces, worker git calls create `.git/index.lock` collisions that immediately crash parallel agents with exit code 128.
> 5. **Clean Turn-Yielding:** When calling `invoke_subagent`, make it the final tool action of the turn. Emit the status line and IMMEDIATELY STOP CALLING TOOLS to allow platform reactive wakeup. Chaining additional tools or tight polling crashes the message queue.

1. **Planning Step (A = 3 `research` Discovery Subagents):** Lead agent calls `invoke_subagent` to spawn 3 read-only discovery subagents (`TypeName: "research"`, `Role: "Research 01: Architecture & Blast Radius"`, `Role: "Research 02: Specs & Dependency Mapping"`, `Role: "Research 03: Repository Dependencies & Fleet Sync Targets"`). Their prompt MUST instruct them to research the codebase using GitMap high-speed search (`gitmap aum search "<symbol>" [dir] [-e <.ext>]`, `gitmap find`, `gitmap lf`, `gitmap cat`) — NEVER `rg`, `ripgrep`, PowerShell `Select-String`, or `git grep` — define symbol boundaries and caller dependencies, and return their structured findings to the lead in their final message.
   - *Master Plan Generation (Lead Agent):* The lead orchestrator receives all three discovery reports, synthesizes findings, and writes the unified Execution Plan (`.ai-memory/plans/pending/nn-<slug>.md`) and the Root Task JSON Manifest.
   - *Tool Call:* Lead must execute the `invoke_subagent` tool as the final action in the turn, print `Dispatched Planning Agents`, and then STOP CALLING TOOLS to wait for `<SYSTEM_MESSAGE>` reactive wakeup.
2. **Spec Step (A = 3 `self` Authoring Subagents):** Once planning is synthesized, the lead agent calls `invoke_subagent` to spawn 3 authoring subagents (`TypeName: "self"`). Their prompt MUST instruct them to author modular, strictly disjoint spec files and subtask plans:
   - Subagent 1 writes `02-spec/21-app/nn-<slug>/01-architecture-spec.md` and subtasks `.ai-memory/plans/subtasks/nn-<slug>/01-<name>.md`.
   - Subagent 2 writes `02-spec/21-app/nn-<slug>/02-component-spec.md` and subtasks `.ai-memory/plans/subtasks/nn-<slug>/02-<name>.md`.
   - Subagent 3 writes `02-spec/21-app/nn-<slug>/03-migration-and-verification-spec.md` and subtasks `.ai-memory/plans/subtasks/nn-<slug>/03-<name>.md`.
   - NEVER have subagents write to the same file path!
   - *Tool Call:* Lead must execute the `invoke_subagent` tool as the final action in the turn, print `Dispatched Spec Agents`, and then STOP CALLING TOOLS to wait for `<SYSTEM_MESSAGE>` reactive wakeup.
3. **Brain & Comprehensive Roadmap Creation (Mandatory Planning Gate):** Before entering Phase 2 execution, the lead orchestrator synthesizes discovery findings into a unified, concrete "Brain" roadmap authored in the specification (`02-spec/21-app/`) and execution plan (`.ai-memory/plans/`):
   - **Target Files Inventory:** Exhaustive list of every file to read, create, or modify.
   - **Modification Sites & Functions:** Exact function names, structs, interfaces, and logic blocks targeted for changes.
   - **Data Contracts & Error Wrapping:** Concrete type definitions and structured errors (`*appfault.AppError`).
   - **Caller Blast Radius:** Callers and dependent packages mapped via GitMap search.
   - **Disjoint Subtask Allocation:** Clear file-ownership boundaries assigned to subtask codes (`Task-01`, `Task-02`) so parallel workers never collide.
4. **Populate Subtasks in SQLite Task DB:** Once subtasks are decomposed, inspect schema constraints (`gitmap agent task status`) and populate them into the SQLite database for atomic worker claiming:
   `gitmap agent subtask add --json '[{"code": "Task-01", "title": "<title>", "owned_files": ["<paths>"], "agent_role": "Worker 01"}]'`
5. **Readiness Gate:** Complete Phase 1 planning, Brain roadmap generation, and spec authoring within `PHASE_1_BUDGET` steps, then proceed **UNCONDITIONALLY** into Phase 2. ZERO intermediate git commits during Phase 1!"""

PHASE_2_EXECUTION_V6 = """## 7. Phase 2: Execution Step (Worker Waves) (Steps (PHASE_1_BUDGET + 1) .. N)

> [!CRITICAL]
> **MANDATORY `invoke_subagent` DISPATCH (ZERO SOLO EXECUTION):**
> You MUST ACTUALLY CALL the `invoke_subagent` tool to spawn A workers (`A = 3`, `TypeName: "self"`, up to H subtasks per worker) in parallel. Executing all subtasks solo in main agent without the tool call is an immediate auto-reject failure.

### 7.1 Dispatch Payload Examples (`invoke_subagent`)

#### Phase 1 Planning Payload (`TypeName: "research"` — Read-Only Discovery):
```json
{
  "Subagents": [
    {
      "TypeName": "research",
      "Role": "Research 01: Architecture & Blast Radius Discovery",
      "Model": "inherit",
      "Workspace": "inherit",
      "Prompt": "You are Research 01 for task <nn>-<slug>. You have no prior chat context; this brief is your complete specification.\\n\\n### Core Objective:\\nExplore the codebase, map symbol callers, trace dependencies, and discover relevant source files for: <task title and objectives>.\\n\\n### Tool Capabilities & Strict Read-Only Boundary:\\n- You are a READ-ONLY subagent (`TypeName: 'research'`).\\n- You have read tools: `run_command`, `view_file`, `search_web`, `read_url_content`.\\n- You DO NOT have `write_to_file` or `replace_file_content`. NEVER attempt to create or edit files.\\n\\n### Mandatory Search Primacy & Total Ban on Raw Grep:\\n- Execute ALL searches and symbol discoveries exclusively via GitMap:\\n  * Live symbol search: `gitmap aum search \\"<symbol>\\" [dir] [-e <.ext>]`\\n  * Path-scoped search: `gitmap aum search \\"<symbol>\\" --path <relative-dir>`\\n  * Indexed keyword search: `gitmap search \\"<query>\\"`\\n  * File discovery: `gitmap find \\"<pattern>\\"`\\n  * Directory inventory: `gitmap lf [dir]`\\n  * View files: `gitmap cat <path>` or native `view_file`\\n- TOTAL BAN (AUTO-REJECT FAILURE): NEVER run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`.\\n- TOTAL BAN ON GIT COMMANDS: NEVER run `git` commands (`git add`, `git commit`, `git status`, `git diff`, etc.).\\n\\n### Required Findings Report (Send via send_message to Parent):\\nWhen your research is complete, send a message to the caller containing:\\n1. Key symbol definitions, structures, and entry points.\\n2. Call sites and blast radius (files that will be affected by modifications).\\n3. Recommended modular file ownership boundaries for worker subtasks.\\n4. Verification commands to validate changes."
    },
    {
      "TypeName": "research",
      "Role": "Research 02: Specs & Dependency Mapping",
      "Model": "inherit",
      "Workspace": "inherit",
      "Prompt": "You are Research 02 for task <nn>-<slug>. You have no prior chat context; this brief is your complete specification.\\n\\n### Core Objective:\\nInspect existing specifications under `02-spec/`, coding guidelines under `02-spec/02-coding-guidelines/`, and prior plan tasks for: <task title and objectives>.\\n\\n### Tool Capabilities & Strict Read-Only Boundary:\\n- You are a READ-ONLY subagent (`TypeName: 'research'`).\\n- You DO NOT have file authoring tools. NEVER attempt to create or edit files.\\n\\n### Mandatory Search Primacy & Total Ban on Raw Grep:\\n- Execute ALL searches exclusively via GitMap:\\n  * Live search in specs: `gitmap aum search \\"<term>\\" 02-spec`\\n  * Indexed search: `gitmap search \\"<term>\\"`\\n  * File discovery: `gitmap find \\"*.md\\"`\\n  * View files: `gitmap cat <file>` or native `view_file`\\n- TOTAL BAN: NEVER run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem`, or `findstr`.\\n- TOTAL BAN ON GIT COMMANDS: NEVER run `git` commands.\\n\\n### Required Findings Report (Send via send_message to Parent):\\nSend a message to the caller detailing existing specs, architectural invariants, acceptance criteria, and positive boolean rules."
    },
    {
      "TypeName": "research",
      "Role": "Research 03: Repository Dependencies & Fleet Sync Targets",
      "Model": "inherit",
      "Workspace": "inherit",
      "Prompt": "You are Research 03 for task <nn>-<slug>. You have no prior chat context; this brief is your complete specification.\\n\\n### Core Objective:\\nExplore external dependencies, script pipelines (`03-ai-scripts/`), skills (`.agents/skills/` and `.cursor/skills/`), and fleet synchronization manifests for: <task title and objectives>.\\n\\n### Tool Capabilities & Strict Read-Only Boundary:\\n- You are a READ-ONLY subagent (`TypeName: 'research'`).\\n- You DO NOT have file authoring tools. NEVER attempt to create or edit files.\\n\\n### Mandatory Search Primacy & Total Ban on Raw Grep:\\n- Execute ALL searches exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap lf`, `gitmap cat`).\\n- TOTAL BAN: NEVER run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem`, or `findstr`.\\n- TOTAL BAN ON GIT COMMANDS: NEVER run `git` commands.\\n\\n### Required Findings Report (Send via send_message to Parent):\\nSend a message to the caller detailing script dependencies, skill catalogs, and cross-repo synchronization targets."
    }
  ]
}
```

#### Phase 2 Execution Payload (`TypeName: "self"` — Read/Write Workers):
```json
{
  "Subagents": [
    {
      "TypeName": "self",
      "Role": "Worker 01: [Assigned Subtask]",
      "Model": "inherit",
      "Workspace": "inherit",
      "Prompt": "<Worker Brief Below>"
    },
    {
      "TypeName": "self",
      "Role": "Worker 02: [Assigned Subtask]",
      "Model": "inherit",
      "Workspace": "inherit",
      "Prompt": "<Worker Brief Below>"
    },
    {
      "TypeName": "self",
      "Role": "Worker 03: [Assigned Subtask]",
      "Model": "inherit",
      "Workspace": "inherit",
      "Prompt": "<Worker Brief Below>"
    }
  ]
}
```

### 7.1.B Self-Contained Research Discovery Brief (TypeName: 'research')

Read-only discovery subagents spawn with zero prior chat context. The prompt envelope MUST inject complete instructions, strictly enforce GitMap search primacy, and impose an absolute ban on raw search tools:

```text
You are Research <NN> for task nn-<slug>. You have no prior chat context; this brief is your complete specification.

### Boundaries & Crash Prevention:
- Read-Only Mode: You possess read-only tools (`view_file`, `run_command`, web search). You DO NOT have `write_to_file` or `replace_file_content`. NEVER attempt to create or modify files. Commanding a research agent to write files crashes the subagent.
- TOTAL BAN ON RAW SEARCH TOOLS: NEVER run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`. Spawning external search binaries hangs execution, causes OS lock collisions, and wastes steps.
- GitMap Search Primacy: Execute all code, symbol, and pattern searches exclusively via GitMap commands:
  - Live streaming code/regex search: `gitmap aum search "<pattern>" [dir] [-e <.ext>] [-r] [-i]`
  - Fast file finder: `gitmap find "<pattern>" [-ext <ext>]`
  - High-speed directory inventory: `gitmap lf [pat]`
  - Stream file content: `gitmap cat <filepath>`
- TOTAL BAN ON GIT COMMANDS: NEVER run `git add`, `git commit`, `git push`, `git status`, `git diff`, `git checkout`.
- Strict Relative Git Paths: All paths cited in your report must be relative to the repository root; only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well (TOTAL BAN on absolute filesystem paths and `file:///` URIs).

### Assigned Research Scope:
- Research Target: <symbol / architectural area / module>
- Primary Objective: Map callers, define type contracts, identify blast radius, and discover existing specifications.

### Output Contract:
Return your structured findings via send_message to the Lead Orchestrator with this JSON envelope, then stop:
{
  "agent": "Research <NN>",
  "target": "<symbol or module>",
  "filesExamined": ["<path1>", "<path2>"],
  "callers": ["<caller1>", "<caller2>"],
  "architecturalBoundaries": "<findings>",
  "existingSpecs": ["<path>"],
  "risksAndBlastRadius": "<risks>"
}
```

### 7.2 Self-Contained Worker Brief (Eliminate Context Blindness)

Subagents spawn with clean context. The prompt envelope MUST inject complete instructions:

```text
You are Worker <NN> for task nn-<slug>. You have no prior chat context; this brief is your complete specification.

### Boundaries & Crash Prevention:
- Read any file in the workspace; edit ONLY your Owned Files: <relative paths>.
- TOTAL BAN ON GIT COMMANDS (LOCK COLLISION PREVENTION): NEVER run ANY git commands (`git add`, `git commit`, `git push`, `git status`, `git diff`, `git checkout`). In shared workspaces, worker git calls create `.git/index.lock` collisions that immediately crash parallel agents. Only the lead orchestrator runs git commands after workers complete.
- TOTAL BAN ON COMMITS: Workers NEVER commit, stage, or push. Committing is exclusively reserved for the Lead Agent at Phase 3 via GitMap (`gitmap cpf "<module> - <summary>"` using hyphen `-`; no colon needed in GitMap cpf as colon is already provided).
- Code & Symbol Search: Use GitMap exclusively: `gitmap aum search "<pattern>" [dir] [-e <.ext>] [-r] [-i]` or `gitmap find`. TOTAL BAN on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`.
- After C tool calls, stop and report what you have.
- A tool failing twice: reply "STATUS: BLOCKED" with exact error and stop. Never guess paths and never troubleshoot machine.
- Workers that find a secret stop and report "BLOCKED: secret at <file>:<line>". They do not handle it themselves.
- Adhere to R1, R2, and R11 by ID.

### Assigned Subtasks (up to H subtasks):
- Subtask 1: .ai-memory/plans/subtasks/nn-<slug>/01-<name>.md
- Subtask 2: .ai-memory/plans/subtasks/nn-<slug>/02-<name>.md (if assigned)

### 100% Non-Negotiable Coding Guidelines (AUTO-REJECT ON VIOLATION):
1. Positive booleans ONLY: use `is` and `has` prefixes exclusively. NEVER evaluate explicit `== true`. NEVER combine positive and negative checks in the same condition (`if isA && !isB` is BANNED).
2. Go Structured Errors: return `*appfault.AppError`, never bare `error`.
3. Function Sizing: <= 8 lines preferred, hard cap 15 lines. Extract domain structs and raw generics to `types.go`.
4. Strict Relative Git Paths & Lowercase: zero absolute filesystem paths and zero `file:///` URIs. Only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well. All new files strictly lowercase.
5. Repo Secrets: if any credentials or private tokens are needed, store them in the `repo-secrets` folder in the default work directory (via `gitmap rs`). Never commit secrets.
6. Zero Builds or Tests: NEVER run `go build`, `npm run build`, `go test`, or `pytest`.
7. Targeted Verification: Run only fast file-scoped linters (e.g. `python 03-ai-scripts/05-guideline-autofixer.py <folder> --check-only`). A check scanning 0 files is a FAIL.
8. GitMap Search Primacy (TOTAL BAN on rg / ripgrep / Select-String / git grep): NEVER execute `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, or `findstr`. Always use `gitmap aum search "<pattern>" [dir] [-e <.ext>] [-r]` for live symbol/regex discovery.
9. Mandatory Heatmap Telemetry: In multi-agent autonomous sessions, agents MUST record all file read/edit decisions with reasoning via gitmap heatmap log or gitmap agent log, and inspect active file leases and contention hotspots via gitmap agent heatmap (or gitmap heatmap status) before editing shared files.

### Concurrency-Safe SQLite Action Logging & Heatmap Telemetry (CRASH FORENSICS MANDATE):
- Worker subtasks are tracked in the run database: `<databasePath>`.
- Inspect Database Schema & Constraints:
  `gitmap agent task status`
  Inspect tables, positive boolean fields, and JSON payload specifications before claiming or logging actions.
- Claim assigned subtask atomically:
  `gitmap agent subtask claim --agent "Worker <NN>"`

#### 1. Heatmap File Contention & Pre-Edit Blast Radius Check:
- BEFORE modifying any shared files, you MUST inspect active file leases and collision contention hotspots:
  `gitmap agent heatmap [--json]` (or `gitmap heatmap status`)
  If high write contention is reported on a file, coordinate disjoint boundaries or sequence edits to prevent file lock crashes.

#### 2. In-Flight Cognitive Decision Logging (Heatmap & Action Log):
- BEFORE touching, inspecting, or modifying any file, you MUST log your in-flight action and reasoning for GitMap heat map tracking and crash forensics:
  * Heatmap File Touch & Exploration (reasoning):
    `gitmap heatmap log -t <task-slug> -a "READ" -f "<path>" -r "Reasoning: <why this file is being inspected and what information is sought>" -s "EXPLORATION"`
    (or via agent engine: `gitmap agent log --subtask <id> --agent "Worker <NN>" --action "read_file" --file "<path>" --details "Reasoning: <why this file is inspected>"`)
  * Heatmap File Modification (editing decision & rationale):
    `gitmap heatmap log -t <task-slug> -a "EDIT" -f "<path>" -r "Decision: <why this file is being modified and planned refactor/fix>" -s "EXECUTION"`
    (or via agent engine: `gitmap agent log --subtask <id> --agent "Worker <NN>" --action "write_to_file" --file "<path>" --details "Decision: <why this file is being modified>"`)
  *(Note: This guarantees that GitMap indexes full cognitive heat map telemetry into `.ai-memory/heatmap.db` and agent task DB, and if a tool execution crashes or the session restarts, the database permanently records the exact file and reasoning trace).*

#### 3. Real-Time Telemetry & Heatmap Stream:
- Inspect live heat map, tree, and action status:
  * Hierarchical tree view: `gitmap heatmap tree [-t <task-slug>]`
  * Action summary and metrics: `gitmap heatmap status [-t <task-slug>]`
  * Real-time continuous stream: `gitmap heatmap live`
  * Action search: `gitmap heatmap search [query] [-f <file>] [-t <task>]`
  * 48-hour active repository activity: `gitmap full-summary` (alias `gitmap fs`)

#### 4. Multi-AI Companion & repo-cache Synchronization:
- Lists all supported AI namespaces: `gitmap heatmap rc list-ai`
- Logs cognitive actions into isolated split-databases: `gitmap heatmap rc log [flags]`
- Checkpoint commit in `repo-cache`: `gitmap heatmap rc commit [-m "<msg>"]`
- Bidirectional reconciliation: `gitmap heatmap rc sync`

#### 5. Non-Blocking Read-Only Heatmap Queries & Diagnostics:
- Execute read-only SQL queries with automatic `mode=ro` protection:
  `gitmap heatmap query "<sql>" [--format json|yaml|table]`
  `gitmap heatmap rc query "<sql>" [--ai <name>] [--repo <slug>] [--tier <repo|root>]`
- Inspect SQLite master catalog, tables, and row counts:
  `gitmap heatmap tables [--db <root|repo|local>]` / `gitmap heatmap rc tables`
- Query records from specific heatmap table:
  `gitmap heatmap view <table> [--limit <n>]` / `gitmap heatmap rc view <table>`
*(Note: Supports tri-alias routing parity: `gitmap heatmap rc ...`, `gitmap rc heatmap ...`, `gitmap heatmap rc-heatmap ...`).*

- When your subtask passes targeted checks, mark completion in the database:
  `gitmap agent subtask complete <id> --agent "Worker <NN>" --evidence "PASS exit 0, <files>"`
- If blocked or failing, record the failure:
  `gitmap agent subtask fail <id> --agent "Worker <NN>" --reason "<reason>"`
*(Note: Native compiled Go engine `gitmap agent` executes in <1ms without Python VM overhead).*

### 3-Tier Database Location Architecture & Verification Queries:

| Database Tier | Physical File Location | Access Mode & Concurrency Protocol | Data Retained & Ownership |
| :--- | :--- | :--- | :--- |
| **Tier 1: Host Root DB** | `~/.gitmap/agent-root.db` | WAL Mode + Cross-Repo Catalog | Machine sessions, active leases, workspace catalog |
| **Tier 2: Repo-Cache Telemetry** | `repo-cache/.heatmap/<ai-slug>/<repo-slug>/heatmap.db` | Isolated Split-DB with periodic sync (`gitmap heatmap rc sync`) | Cross-repository multi-agent actions, tool usage traces, commit checkpoints |
| **Tier 3: Local Task & Heatmap DB** | `.ai-memory/temp-agents/<slug>/agent-task.db`<br/>`.ai-memory/heatmap.db` | WAL Mode (`PRAGMA journal_mode=WAL; busy_timeout=5000`) | Task status, worker claims, subtasks, in-flight action logs, local contention |

#### Executable Commands Block:
```bash
# Initialize and verify agent task database
gitmap agent task init --name "<task name>" --budget 400
gitmap agent task status --json

# Pre-edit contention and lease check
gitmap agent heatmap --json

# Cognitive file exploration telemetry
gitmap heatmap log -t <task-slug> -a "READ" -f "<path>" -r "Reasoning: <explanation>" -s "EXPLORATION"
gitmap heatmap log -t <task-slug> -a "EDIT" -f "<path>" -r "Decision: <refactor-rationale>" -s "EXECUTION"

# Subtask lifecycle
gitmap agent subtask claim --agent "Worker 01"
gitmap agent subtask complete <id> --agent "Worker 01" --evidence "PASS exit 0"

# Forensic autopsy and status inspection
gitmap agent task status
gitmap agent diagnose
```

### Output Contract:
Write your subtask output to .ai-memory/plans/subtasks/nn-<slug>/01-<name>.json and reply with this JSON block, once per subtask, then stop:
{
  "task": "Task-01",
  "status": "DONE",
  "filesChanged": ["<path1>", "<path2>"],
  "checks": "<command> -> exit <code>, <files scanned>",
  "acceptance": { "ac1": "PASS <evidence>" },
  "assumptions": [],
  "blockers": []
}
```

### 7.3 Turn-Yielding, Verification & Crash Forensics Protocol

1. **Invoke & Yield:** You must ACTUALLY CALL the `invoke_subagent` tool as the final action in your turn. Print the progress line (`Dispatched Worker 01 .. Worker <A> (wave k / WAVES); waiting for their results.`) and **STOP CALLING TOOLS** to end your turn.
2. **Automated Crash Forensics & Status Inspection:**
   - Lead inspects overall completion status at any time:
     `gitmap agent task status` (or list subtasks via `gitmap agent subtask ls`)
   - TOTAL BAN ON RAW PYTHON SQLITE: NEVER execute `python -c "import sqlite3..."` or ad-hoc DB scripts. Inspect status via `gitmap agent task status` or list subtasks via `gitmap agent subtask ls`.
   - If any worker fails to report, crashes, or times out, inspect status or diagnose:
     `gitmap agent task status` or `gitmap agent diagnose`
     This pinpoints the autopsy: which agent crashed, on which subtask, targeting which file, and the exact action that was executing when it failed.
3. **Verify Worker Reports Independently:** Confirm `git diff --stat -- <owned files>` matches `filesChanged`, no files outside owned files modified, re-run targeted checks for `exit 0` on non-zero files.
4. **Reject Violations:** Send failures via `send_message`. On `BLOCKED`, lead does work and logs `LEAD_FALLBACK: <reason>`. After two failed rounds, mark `FAILED`, write RCA, continue (R13).
5. **Update Ledger:** Record status, evidence, changed paths in `ledger.md` via `replace_file_content`.
6. **Loop:** Dispatch subsequent waves via `invoke_subagent` until all subtasks are `DONE` or `FAILED`.

### 7.4 Immediate Dynamic Respawn & Join Protocol (100% Duty Cycle Concurrency)

To maximize execution velocity and prevent idle worker slots during Phase 2 and Phase 3:
1. **Zero-Idle Immediate Respawn Mandate:** The lead orchestrator MUST NOT wait for an entire 3-agent wave to finish. As soon as ANY single worker completes (`send_message` or `gitmap agent subtask complete`), the lead immediately checks the queue for the next pending subtask and spawns a replacement worker (`invoke_subagent`), keeping concurrency pinned at exactly `A = 3` active subagents (System Concurrency Capacity = 6).
2. **Dynamic Collaborative Assistant Mode:** If all pending subtasks are already claimed or in-flight, any newly freed agent MUST NOT terminate, halt, or idle. The lead immediately commands or spawns the worker to dynamically join an active subtask in collaborative assistance mode:
   - **Mode A: Parallel Reading & Blast Radius Discovery:** The joining agent uses GitMap (`gitmap aum search`, `gitmap cat`, `gitmap find`) to read caller files, inspect imports, and verify type signatures, feeding findings to the primary worker via structured message.
   - **Mode B: Cognitive Memory & Spec Documentation:** The joining agent records architectural discoveries, updates memory registers, and generates companion audit specs.
   - **Mode C: Companion Lint & Relative Path Validation:** The joining agent executes targeted linters (`check-relative-paths.py`, `check-forbidden-strings.py`, guideline autofixer check) across related workspaces while the primary worker completes editing.
   - **Mode D: Disjoint File Partitioning:** If the ongoing subtask touches multiple files, the subtask is subdivided dynamically so the joining agent takes exclusive ownership of File 2 while the primary worker focuses on File 1.
3. **Disjoint Ownership Invariant (Rule R6):** Dynamic assistants NEVER write to the primary worker's owned files. File ownership remains strictly disjoint to eliminate file write collisions.
4. **GitMap Telemetry Tracking:**
   - Log join actions: `gitmap agent subtask join --subtask <id> --agent "Worker <NN>" --role "Assistant: <focus>"`
   - Inspect tree hierarchy: `gitmap agent task tree` (renders live Root -> Branch -> Leaf/Assistant hierarchy)."""

PHASE_3_CONSOLIDATION_V6 = """## 8. Phase 3: Consolidation, Evidence Verification & Atomic GitMap Push

1. **Consolidate Subtasks:** Merge completed subtasks into `.ai-memory/plans/completed/nn-<slug>.md`, logging real steps from ledger; link to canonical spec. Delete `.ai-memory/plans/subtasks/nn-<slug>/` and pending plan. Canonical spec in `02-spec/21-app/` stays permanently.
2. **Update Registers:** Update `.ai-memory/plans/readme.md` and `02-spec/21-app/readme.md`. Update `01-prompts/readme.md` and `.ai-memory/prompts.md` only when adding/modifying prompts. Register recent completed tasks before push gate.
3. **Push Gate Verification:** Before GitMap call, verify: (a) targeted checks exit 0 (>0 files), (b) secrets gate clean, (c) `.gitignore` covers caches, build outputs, logs, reports, `.env*` (untrack any tracked ignored files via `git rm --cached <file>` or `git rm -r --cached <dir>`). On failure: abort GitMap call; mark task `FAILED` with RCA.
4. **Atomic Commit & Push:** Call `gitmap cpf "<module> - <feature summary>"` (features, e.g. `gitmap cpf "CBF - implement user profile dashboard"`) or `gitmap cpb "<module> - <fix summary>"` (fixes, e.g. `gitmap cpb "AUM - validate regex without nil fallback"`). Mention as a hyphen `-` because the colon is already going to be provided by GitMap (which automatically prepends `Feature: ` or `Bug: `). So, there is no need to provide the colon in GitMap `cpf`/`cpb`/CVF message arguments (TOTAL BAN on colons `:` inside the message argument); always use a hyphen `-` to separate module from summary (e.g. `<module> - <summary>`). NEVER include redundant conventional prefixes (`feat(...)`, `fix(...)`, `docs(...)`) in the commit string. Push rejected: `git pull --rebase` and re-run. Miss after push: allow one follow-up `gitmap cpb "<module> - <fix summary>"`, logged as `FOLLOW_UP_PUSH: <sha>`. Never amend pushed commits."""

BANNED_OPERATIONS_CHECKLIST_V6 = """## 12. Banned Operations Checklist (TOTAL BAN — Auto-Reject on Violation)

- [ ] TOP-INSTRUCTION PRIORITY MANDATE: Whatever directives, constraints, checklists, or instructions are given before this section or prompt (user preamble, header constraints, prior instructions) are verified as highest priority and non-negotiable, overriding all lower-level guidelines below.
- [ ] MANDATORY SUBAGENT SPAWNING GATE (ZERO SOLO EXECUTION): Verified that `invoke_subagent` (`A = 3, H = 2, Capacity = 6`) was explicitly called in Phase 1 (parallel discovery/spec modules) and Phase 2 (`TypeName: "self"` parallel subtask execution). Executing all reads or code modifications solo without calling `invoke_subagent` is an immediate auto-reject failure.
- [ ] TREE SWARM & DYNAMIC JOIN MANDATE (ZERO IDLE CAPACITY): Verified that execution followed the Tree Swarm Hierarchy (Root Orchestrator -> Branch Coordinators -> Leaf Workers & Dynamic Assistants), that Phase 1 authored the comprehensive Brain roadmap up front before worker dispatch, and that newly idle workers immediately respawned for pending tasks or dynamically joined in-flight tasks for parallel reading, caller analysis, or companion verification to keep concurrency capacity pinned at 3 active subagents (Capacity = 6) until completion.
- [ ] NO SUBAGENT FILE WRITE COLLISIONS (TOTAL BAN): Never assign two subagents to create or modify the same file. Shared files belong exclusively to the lead orchestrator. Subagents must have strictly disjoint, non-overlapping file boundaries to prevent file lock crashes.
- [ ] NO READ-ONLY SUBAGENT FILE WRITES (TOTAL BAN): Subagents of `TypeName: "research"` have read-only tools and cannot write files. Never command a research subagent to write files; research subagents discover and report back, and the lead writes the unified plan.
- [ ] NO INVALID SUBAGENT TYPENAMES (TOTAL BAN): Never pass custom strings like "Research 01" or "Worker 01" in the `TypeName` field of `invoke_subagent`. `TypeName` MUST be strictly `"research"` (read-only discovery) or `"self"` (read/write execution). Put descriptive names in `Role` only.
- [ ] NO SUBAGENT GIT EXECUTION (TOTAL BAN): Subagents must NEVER run any git commands (`git add`, `git commit`, `git push`, `git status`, `git diff`, `git checkout`) in shared workspaces. Worker git execution creates `.git/index.lock` collisions that immediately crash parallel workers with exit code 128.
- [ ] ISSUE & RCA DESTINATION ROUTING: Whenever resolving an issue or performing a fix with RCA, verified that CI/CD failures are documented in `.ai-memory/cicd-issues/nn-<slug>.md` (indexed in `.ai-memory/cicd-index.md`), while non-CI/CD issues (application bugs, logic/runtime defects) are documented in `02-spec/22-app-issues/nn-<slug>.md` (indexed in `02-spec/22-app-issues/readme.md`).
- [ ] NO TEST RUNNING (TOTAL BAN): Never run any tests using Python scripts (`06-cicd-local-runner.py`, `pytest`, runner scripts), Go (`go test ./...`), or any test runner during routine execution turns. Testing is strictly checked later on in CI/CD.
- [ ] NO BUILD CHECKING (TOTAL BAN): Never run build commands (`go build`, `npm run build`, compiler checks) to verify compilation. Build verification is checked later on in CI/CD.
- [ ] NO RUNNER SCRIPTS (TOTAL BAN): Never launch background test runners, worker pools, or test inventory loops during routine execution.
- [ ] NO AUTOMATIC RELEASES (TOTAL BAN): Never bump versions, update changelogs, or trigger releases unless explicitly commanded by the user.
- [ ] NO RAW GIT COMMITS OR CONVENTIONAL COMMIT PREFIXES (TOTAL BAN): Never execute `git commit`, `git commit -m "..."`, `git add -A`, `git add .`, or raw `git push`. Never use conventional commit prefixes (`docs(...):`, `feat(...):`, `fix(...):`, `chore(...):`) in raw git commands. All staging, committing, and pushing MUST be executed exclusively by GitMap: `gitmap cpf "<module> - <summary>"` (features) or `gitmap cpb "<module> - <summary>"` (fixes). GitMap automatically formats, stages, commits, and pushes atomically. Always mention and format as a hyphen (`-`); no need to provide a colon in GitMap commit commands (`gitmap cpf` / `cpb` / CVF) because the colon is already automatically provided by GitMap (`Feature: ` or `Bug: `).
- [ ] NO INTERMEDIATE COMMITS (TOTAL BAN): Never commit after Phase 1 (e.g. committing plans or specs) or mid-Phase 2 (committing individual files or tests). Committing early pollutes git history, creates race conditions, and breaks atomicity. All changes (specs, plans, code modifications, index updates) MUST be committed together in ONE single atomic GitMap commit at the final step of Phase 3.
- [ ] NO PER-FILE COMMITTING (TOTAL BAN): Never commit each file individually as you work (e.g. running `git commit` or `gitmap cpf "<module> - <summary>"` after editing File 1, then another commit after File 2). Committing file-by-file pollutes git log history, creates subagent lock collisions, and breaks atomic changes. All modified files across the turn must be accumulated and committed together in a single atomic commit at the final step.
- [ ] NO UNCOMMITTED WORK OR UNPUSHED CODE (TOTAL BAN — NO PUSH = NOT DONE): Never conclude a turn, claim success, or mark any task as DONE while leaving changes uncommitted or unpushed. If the code is not committed to Git and pushed upstream to GitHub (main/master/tracking branch), the task is strictly considered INCOMPLETE and NOT DONE. Leaving uncommitted dirty changes, untracked files, or unpushed local commits means the execution is unfinished and has failed. Concluding without a confirmed successful GitMap push to remote is an immediate auto-reject failure.
- [ ] NO RAPID CI/CD POLLING (TOTAL BAN): Never query or loop rapidly (`gh run view` in tight loops) when inspecting remote CI/CD pipelines. Agents must query pipeline state using GitMap Pipeline-AI (`gitmap pipeline-ai status --json` or `gitmap pl-ai status -t <sec>`) and strictly wait or sleep based on `etaSeconds` to eliminate credit waste.
- [ ] NO PREMATURE TURN CLOSING BEFORE EXECUTION (TOTAL BAN): Never halt execution, conclude the turn, or ask the user for permission after generating specs or subtasks. Planning constitutes only 50% of the task budget; you must proceed unconditionally to Phase 2 code execution. (Note: When dispatching asynchronous background subagents via `invoke_subagent`, yielding control to allow platform reactive wakeup is mandatory and is exempt from this ban).
- [ ] NO HORIZONTAL TASK CONCATENATION (TOTAL BAN): Never concatenate tasks horizontally in the Task Completion Summary (e.g. NEVER `✅ #1... ✅ #2...` run-on). Every completed task MUST be rendered on its OWN SEPARATE LINE starting with an individual markdown list bullet (`- ✅`).
- [ ] INDEPENDENT AI VERIFICATION PROMPT MANDATE: Emitted the self-contained independent AI verification and audit prompt linking to the canonical spec, consolidated plan, and modified files with verbatim score audit criteria.
- [ ] GITMAP HEAVY USAGE & ROUTINE PULL BAN: Heavily leveraged GitMap commands (`cpf`, `cpb`, `cpr`, `search`, `find`, `pwsh`) for discovery, execution, and commits. Never ran `pull-all` (`gitmap pa` or `gitmap pae`) unconditionally during routine turns; only ran `gitmap pae --json` when explicitly commanded by the user.
- [ ] NO RG, RIPGREP, OR RAW SHELL SEARCHES (TOTAL BAN): Never run `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `Get-ChildItem -Recurse`, `findstr`, or slow shell search pipelines to search code. All code searching, symbol discovery, and regex scans MUST use GitMap high-speed search tools: `gitmap aum search "<query>" [dir] [-e <.ext>] [-r]` (streaming multi-core text/regex search) or `gitmap search "<query>"` (indexed symbol search). Running `rg`, `ripgrep`, `Select-String`, or `git grep` is an immediate auto-reject failure.
- [ ] HEATMAP TELEMETRY & CONTENTION INSPECTION: Verified that multi-agent contention was inspected via gitmap agent heatmap (or gitmap heatmap status) before editing shared files, and that all file exploration and modifications were logged with reasoning via gitmap heatmap log (or gitmap agent log) into .ai-memory/heatmap.db for crash forensics and repo-cache synchronization."""

SECTION_14_ANTI_HALLUCINATION_V6 = """## 14. Anti-Hallucination & Blast Radius Checklist

- [ ] Echo Back the Spec: Verified Acceptance Criteria from the Spec file verbatim.
- [ ] Pre-Commit Diff Proof: Verified `git status` shows actual modified files before committing.
- [ ] No Placeholder Search: Confirmed zero `TODO` or `\\[.*\\]` placeholders remain in modified files.
- [ ] Index Sync Deadman Switch: Every new file is explicitly linked in `readme.md` and enqueued in `.ai-memory/what-to-read.md`.
- [ ] Blast Radius Acknowledgment: Global search across codebase performed via `gitmap aum search "<symbol>" [dir]` to update all callers of modified symbols (never `Select-String` or `git grep`).
- [ ] Continuous Loop Maintained: Continuous self-loop executed until 100% complete without running banned test/build commands.
- [ ] Final Step Commit & Push Verified: Staged and committed all changes atomically via GitMap semantic commit commands using `<module> - <summary>` format: `gitmap cpf "<module> - <summary>"` (features, e.g. `gitmap cpf "CBF - implement user profile dashboard"`) or `gitmap cpb "<module> - <summary>"` (bug fixes, e.g. `gitmap cpb "AUM - validate regex without nil fallback"`). Mentioned and formatted with a hyphen `-` to separate module from summary; no need to provide a colon in GitMap `cpf`/`cpb`/CVF commit arguments because the colon is already automatically provided by GitMap (`Bug: ` or `Feature: `). TOTAL BAN on colons `:` inside the GitMap argument and conventional prefixes (`fix(...)`, `feat(...)`, `docs(...)`). TOTAL BAN on raw `git add -A` and `git commit`. Pushed to remote via GitMap in a single final command.
- [ ] Strict Completion Invariant ('No Push = Not Done'): Confirmed that all modifications are staged, committed atomically via GitMap, and pushed upstream to GitHub. If the code is not committed to Git and pushed upstream to GitHub (main/master/tracking branch), the task is strictly considered INCOMPLETE and NOT DONE. Verified that `git status --porcelain` is completely clean (zero uncommitted or untracked files) and that the remote tracking branch reflects the pushed commit before declaring task completion."""

SECTION_15_FINAL_STEP_MANDATE_V6 = """## 15. Final Step Git Commit & Push Mandate (Strict Checklist)

- [ ] MANDATORY FINAL COMMIT & PUSH VIA GITMAP (ANYHOW): At the final step of the turn, after all targeted files have been refactored, verified with targeted linters, and plans/subtasks consolidated, use GitMap semantic commit commands exclusively: `gitmap cpf "<module> - <summary>"` (features, e.g. `gitmap cpf "CBF - implement user profile dashboard"`) or `gitmap cpb "<module> - <summary>"` (bug fixes, e.g. `gitmap cpb "AUM - validate regex without nil fallback"`), which automatically stage, format commit messages, and push directly to remote. Mention and format as a hyphen `-` to separate module and summary; no need to provide a colon `:` in the GitMap `cpf`/`cpb`/CVF message argument as the colon is already provided automatically by GitMap (`Feature: ` or `Bug: `). TOTAL BAN on raw `git commit`, `git commit -m`, `git add -A`, colons `:` in the message argument, or conventional prefixes inside GitMap arguments (`docs(...)`, `feat(...)`, `fix(...)`). ZERO intermediate commits during Phase 1 or Phase 2; all files across the run are committed together at the final step. Leaving uncommitted changes or unpushed commits on the active branch at the end of a turn is an immediate failure.
- [ ] TOTAL BAN ON PER-FILE COMMITS (DO NOT COMMIT EACH FILE INDIVIDUALLY): You must not create separate git commits for each individual file as you edit them (e.g. running `git commit` or `gitmap cpf` after editing File 1, then committing again after File 2 is strictly forbidden). Committing file-by-file pollutes git log history, creates subagent lock collisions, and breaks atomic rollback. All modified files, test change caches, and plan records across the turn must be accumulated in the working tree and committed together in a single grouped atomic commit at the final step before pushing.
- [ ] MANDATORY COMPLETION INVARIANT (NO PUSH = NOT DONE): If the code is not committed to Git and pushed upstream to GitHub (main/master/tracking branch), the task is strictly considered INCOMPLETE and NOT DONE. Leaving uncommitted dirty changes or unpushed commits means the execution is unfinished and has failed. A task cannot be marked completed or successful until all changes are committed and confirmed pushed to GitHub. The orchestrator must verify that `git status --porcelain` returns completely empty and that the remote tracking branch is up to date before rendering the final completion summary."""

MUST_FOLLOW_FOOTER = """## MUST FOLLOW NON-NEGOTIABLE

Listen, past runs of these turns have been sloppy and stupid as fuck: wrong step counts, partial task lists dumped into chat instead of files, plans and session summaries half-filled with "[N]" placeholders, folders skimmed, open ambiguities ignored, CI/CD issues and `plans/subtasks/` forgotten, user commands dropped, coding guidelines bypassed, detailed specs chopped and summarized into useless junk, uppercase readme.md files left uncorrected, `.ai-memory/memory/` created by accident, `strictly-avoid.md` overwritten, and explicit user instructions softened after being told not to. WTF. How on earth are you reverting to this carelessness, are you stupid?? Stop doing that, you stupid fuck. Read the whole codebase, read every folder in `02-spec/` and `.ai-memory/`, confirm root `readme.md` is strictly lowercase, find the root cause in one sentence, capture commands, issues, and pending tasks without omitting a single item, write the spec files and memory files in the right paths, update every index in the same turn, sync `readme.md` with `what-to-read.md`, preserve detailed specs verbatim with zero truncation, group commits with clear messages, and push everything to git before ending. Going deep IS the job. If you are not going deep, you are not doing the job. Violating this is auto-reject on the same tier as RULE 0. Avoid stupidity and being careless, you stupid fuck. Where is your attention, are you stupid? Tell me. Your stupidity is going on top of my head. Where did you learn this stupidity? If I could find you, I could slap you."""


def extract_prompt_slug(file_path: Path) -> str:
    """Extract prompt slug identifier from filename."""
    stem = file_path.stem
    if "-" in stem:
        return stem.split("-", 1)[1]
    return stem


def upgrade_prompt_31(content: str) -> str:
    """Specialized upgrade for 31-cg-execute-in-below-steps.md preserving bottom precedence."""
    slug = "cg-execute-in-below-steps"

    # Replace parameter block
    content = re.sub(
        r"```text\s*\nN = \d+[\s\S]*?```",
        V6_HEADER_PARAMETERS,
        content,
        count=1,
    )

    # Ensure Bottom metadata block
    if "Bottom-Instruction Priority Mandate" not in content:
        content = re.sub(
            r"(```text[\s\S]*?```\s*\n)",
            r"\1" + V6_META_TEMPLATE_BOTTOM + "\n\n",
            content,
            count=1,
        )

    # Normalize slash commands
    content = content.replace("(slashCommand:goal)", "(slashCommand;goal)")
    content = content.replace("(slashCommand:learn)", "(slashCommand;learn)")
    content = content.replace("(slashCommand:plan)", "(slashCommand;plan)")
    content = re.sub(
        r"A = 2,\s*H = 2",
        "A = 3, H = 2, Capacity = 6",
        content,
    )

    # Upgrade Mandatory Spawning Gate
    gate_pattern = re.compile(
        r"### 🚨 MANDATORY SUBAGENT SPAWNING GATE[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    if gate_pattern.search(content):
        content = gate_pattern.sub(MANDATORY_SPAWNING_GATE_V6 + "\n\n", content, count=1)

    # Upgrade Phase 1A with Understood Turn-1 showcase (prompt 31 variation)
    phase1a_pattern = re.compile(
        r"## Phase 1A: Verbatim Capture, Task Extraction & Chat Output Gate[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    if phase1a_pattern.search(content):
        content = phase1a_pattern.sub(
            "## " + EXTREME_UNDERSTOOD_TURN_1_SHOWCASE_PROMPT_31 + "\n\n",
            content,
            count=1,
        )

    # Upgrade Phase 1B with High-Speed GitMap discovery & Step 0 Task DB
    phase1b_pattern = re.compile(
        r"## Phase 1B: Planning Mode, Detailed Spec Generation & Lean Subtasks[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    phase1b_replacement = """## Phase 1B: Planning Mode, Detailed Spec Generation & Lean Subtasks (Steps 1 .. 100)

1. **High-Speed GitMap Discovery (PRIMARY):**
   - `gitmap find "<wildcard*>" [-ext <ext>]` (`gitmap f`), `gitmap find-files <name>` (`gitmap ff`), `gitmap find-files-any <str>` (`gitmap ffa`), `gitmap find-files-startswith <prefix>` (`gitmap ffs`), `gitmap find-files-endswith <suffix>` (`gitmap ffe`)
   - `gitmap aum search "<query>" [dir] [-e <.ext>] [-r] [-i]` (alias `gitmap aum grep`), `gitmap list-files [pattern] [-ext <ext>]` (`gitmap lf`), `gitmap cat <filepath>`, `gitmap search "<term>"`. TOTAL BAN on rg, ripgrep, grep, git grep, PowerShell `Select-String`, `Get-ChildItem -Recurse`, or `findstr`.
2. **Step 0 Preflight & SQLite Task DB Data Schema Verification:**
   - Initialize task state: `gitmap agent task init --name "cg-execute-in-below-steps" --budget 400`
   - Inspect Task DB Schema & Data Integrity Rules: `gitmap agent task status`
   - DDL: `gitmap agent task status --ddl`; JSON: `gitmap agent task status --json`. Verify singular PascalCase tables, positive boolean fields (`IsActive`, `HasCompleted`, `IsBlocked`), foreign keys, and payload constraints before populating subtasks. TOTAL BAN on raw Python SQLite queries.
3. **Brain & Comprehensive Roadmap Creation (Mandatory Planning Gate):**
   - Exhaustive target files inventory, modification sites, concrete type contracts (`types.go`), structured errors (`*appfault.AppError`), and caller blast radius.
   - Author parent plan `.ai-memory/plans/pending/xx-<slug>.md` and lean, disjoint subtasks in `.ai-memory/plans/subtasks/xx-<slug>/01-<subtask>.md`. Populate into SQLite DB: `gitmap agent subtask add --json '[...]'`.
   - Complete all spec and subtask writing within Phase 1 budget (`PHASE_1_BUDGET = 100`) and unconditionally transition to Phase 2 without stopping."""
    if phase1b_pattern.search(content):
        content = phase1b_pattern.sub(phase1b_replacement + "\n\n", content, count=1)

    # Upgrade Phase 2 with Worker Brief, Heatmap Telemetry, 3-Tier DB and Dynamic Respawn
    phase2_pattern = re.compile(
        r"## Phase 2: Execution Mode & Parallel Refactoring[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    phase2_replacement = """## Phase 2: Execution Mode & Parallel Refactoring (Steps 101 .. 250)

1. **Parallel Subagent Dispatch (`TypeName: "self"`, A = 3, H = 2, Capacity = 6):**
   - Dispatch subagents with self-contained Prompt Envelopes to refactor disjoint file batches (5–8 files per batch).
   - Yield turn after `invoke_subagent` to await reactive wakeup (`<SYSTEM_MESSAGE>`).
   - Section 7.4 Immediate Dynamic Respawn & Join Protocol (100% duty cycle, pinned at A = 3 active subagents).
2. **Rule 9 Mandatory Heatmap Telemetry & Pre-Edit Contention:**
   - Inspect active leases and file contention before editing shared files: `gitmap agent heatmap [--json]` (or `gitmap heatmap status [--json]`).
   - Log all file touches and editing decisions with reasoning into `.ai-memory/heatmap.db`:
     `gitmap heatmap log -t cg-execute-in-below-steps -a "READ" -f "<path>" -r "Reasoning: <explanation>" -s "EXPLORATION"`
     `gitmap heatmap log -t cg-execute-in-below-steps -a "EDIT" -f "<path>" -r "Decision: <rationale>" -s "EXECUTION"`
3. **3-Tier Database Location Transparency:**
   - Tier 1 Host Root: `~/.gitmap/agent-root.db`
   - Tier 2 Repo-Cache: `repo-cache/.heatmap/<ai-slug>/<repo-slug>/heatmap.db`
   - Tier 3 Local: `.ai-memory/temp-agents/<slug>/agent-task.db` and `.ai-memory/heatmap.db`
   - Verification commands: `gitmap agent task status`, `gitmap heatmap tree`, `gitmap heatmap rc query "<sql>"`.
4. **Coding Guidelines Enforced:**
   - Positive booleans only (`is`/`has`), no `== true`.
   - Structured Go errors (`*appfault.AppError`), concrete types in `types.go`, functions <= 8–15 lines, vertical blank lines, strict relative paths, lowercase filenames.
5. **Total Ban on Build & Test Commands:**
   - NEVER run `go build`, `npm run build`, `go test`, `pytest`, or `06-cicd-local-runner.py` during routine execution turns. Run only targeted file-level linters (`05-guideline-autofixer.py`)."""
    if phase2_pattern.search(content):
        content = phase2_pattern.sub(phase2_replacement + "\n\n", content, count=1)

    # Upgrade Phase 3 with atomic commit format and No Push = Not Done
    phase3_pattern = re.compile(
        r"## Phase 3: Task Consolidation & Atomic GitMap Push[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    phase3_replacement = """## Phase 3: Task Consolidation & Atomic GitMap Push (End of Loop)

1. Consolidate completed subtasks from `.ai-memory/plans/subtasks/xx-<slug>/*.md` into `.ai-memory/plans/completed/xx-<slug>.md`, delete granular subtasks and pending plan, and update `.ai-memory/plans/readme.md`.
2. Commit and push all modified files in a single grouped atomic commit via GitMap using hyphen format:
   - `gitmap cpf "<module> - <summary>"` (features) or `gitmap cpb "<module> - <summary>"` (fixes).
   - TOTAL BAN on colons inside the commit message argument.
   - Strict Completion Invariant ('No Push = Not Done'): If the code is not committed to Git and pushed upstream to GitHub, the task is strictly considered INCOMPLETE and NOT DONE."""
    if phase3_pattern.search(content):
        content = phase3_pattern.sub(phase3_replacement + "\n\n", content, count=1)

    # Upgrade Section 12 Banned Operations Checklist
    sec12_pattern = re.compile(
        r"## 12\. Banned Operations Checklist[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    if sec12_pattern.search(content):
        content = sec12_pattern.sub(BANNED_OPERATIONS_CHECKLIST_V6 + "\n\n", content, count=1)

    # Upgrade Section 14
    sec14_pattern = re.compile(
        r"## 14\. Anti-Hallucination & Blast Radius Checklist[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    if sec14_pattern.search(content):
        content = sec14_pattern.sub(SECTION_14_ANTI_HALLUCINATION_V6 + "\n\n", content, count=1)

    # Upgrade Section 15
    sec15_pattern = re.compile(
        r"## 15\. Final Step Git Commit & Push Mandate[\s\S]*?(?=## MUST FOLLOW NON-NEGOTIABLE)",
        re.MULTILINE,
    )
    if sec15_pattern.search(content):
        content = sec15_pattern.sub(SECTION_15_FINAL_STEP_MANDATE_V6 + "\n\n", content, count=1)

    # Ensure trailing divider and High Priority Instructions Below boundary is preserved
    divider_anchor = "\n--\n\n## 🚨 High Priority Instructions Below\n"
    if "--\n\n## 🚨 High Priority Instructions Below" not in content:
        content = content.rstrip() + "\n" + divider_anchor

    return content


def upgrade_standard_prompt(content: str, filename: str) -> str:
    """Deterministic upgrade for standard prompts (01-30, 32-37) to V6 architecture."""
    stem = Path(filename).stem
    slug = extract_prompt_slug(Path(filename))

    # 1. Replace Header Parameter block
    content = re.sub(
        r"```text\s*\nN = \d+[\s\S]*?```",
        V6_HEADER_PARAMETERS,
        content,
        count=1,
    )

    # 2. Metadata Block
    if "Top-Instruction Priority Mandate" not in content:
        v6_meta = V6_META_TEMPLATE_TOP.format(invoke_name=slug)
        content = re.sub(
            r"(```text[\s\S]*?```\s*\n)",
            r"\1" + v6_meta + "\n\n",
            content,
            count=1,
        )
    else:
        # Ensure Prompt Version: 6.0.0 and correct invoke slug
        v6_meta = V6_META_TEMPLATE_TOP.format(invoke_name=slug)
        content = re.sub(
            r"> \[!IMPORTANT\][\s\S]*?\*\*Top-Instruction Priority Mandate[\s\S]*?precedence\.",
            v6_meta,
            content,
            count=1,
        )

    # 3. Normalize slash command semicolons
    content = content.replace("(slashCommand:goal)", "(slashCommand;goal)")
    content = content.replace("(slashCommand:learn)", "(slashCommand;learn)")
    content = content.replace("(slashCommand:plan)", "(slashCommand;plan)")

    # 4. Update [/goal] concurrency to A = 3, H = 2, Capacity = 6
    content = re.sub(
        r"A = 2,\s*H = 2",
        "A = 3, H = 2, Capacity = 6",
        content,
    )

    # 5. Replace Mandatory Subagent Spawning Gate
    gate_pattern = re.compile(
        r"### 🚨 MANDATORY SUBAGENT SPAWNING GATE[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    if gate_pattern.search(content):
        content = gate_pattern.sub(MANDATORY_SPAWNING_GATE_V6 + "\n\n", content, count=1)

    # 6. Replace Phase 1A with Extreme Understood Turn-1 Showcase
    phase1a_pattern = re.compile(
        r"### Phase 1A: Verbatim Capture, Task Extraction & Chat Output Gate[\s\S]*?(?=---)",
        re.MULTILINE,
    )
    if phase1a_pattern.search(content):
        content = phase1a_pattern.sub(EXTREME_UNDERSTOOD_TURN_1_SHOWCASE + "\n\n", content, count=1)

    # 7. Upgrade Section 2 Rules
    content = re.sub(
        r"- \*\*R5 Mandatory Subagents[\s\S]*?(?=- \*\*R6)",
        "- **R5 Mandatory Subagents (`invoke_subagent`), Concurrency Scaling & Immediate Dynamic Respawn.** Spawning subagents via `invoke_subagent` (`A = 3`, `H = 2`, System Concurrency Capacity = 6 concurrent operations) is an **ABSOLUTE MUST** (`research` for discovery in Phase 1, `self` for spec authoring and code edits in Phase 2/3). The lead agent is STRICTLY FORBIDDEN from executing all reads or edits solo. Organize execution as a Tree Swarm Hierarchy (Root Orchestrator -> Branch Coordinators -> Leaf Workers & Dynamic Assistants). Under the Immediate Dynamic Respawn & Join Protocol, the lead orchestrator MUST NOT wait for an entire 3-agent wave to finish. As soon as ANY single worker completes, the lead immediately spawns a replacement worker for the next pending subtask, or commands the worker to dynamically join an active subtask in collaborative assistance mode (parallel reading, companion linters, memory logging) to keep concurrency at exactly 3 active subagents (100% duty cycle) without file write collisions (respecting R6). Solo execution without calling `invoke_subagent` is an auto-reject failure on the same tier as Rule 0.\n",
        content,
        count=1,
    )

    content = re.sub(
        r"- \*\*R7 Git Safety & Isolation[\s\S]*?(?=- \*\*R8)",
        "- **R7 Git Safety & Isolation (Worker Git Ban).** Subagents NEVER run git commands (`git add`, `git commit`, `git push`, `git status`, `git diff`, `git checkout`). In shared workspaces (`Workspace: \"inherit\"`), worker git calls create `.git/index.lock` collisions that immediately crash parallel agents. Nobody runs `git reset --hard`, `git checkout --`, `git clean`, `git stash`, or force pushes.\n",
        content,
        count=1,
    )

    content = re.sub(
        r"- \*\*R8/R9 Atomic Commit & Push via GitMap[\s\S]*?(?=- \*\*R10)",
        "- **R8/R9 Atomic Commit & Push via GitMap (TOTAL BAN ON RAW GIT COMMITS).** The run ends with ONE GitMap call:\n  - For features: `gitmap cpf \"<module> - <feature summary>\"` (using hyphen `-`; no colon needed in GitMap cpf as colon is already provided).\n  - For bug fixes: `gitmap cpb \"<module> - <fix summary>\"` (using hyphen `-`; no colon needed in GitMap cpb as colon is already provided).\n  GitMap stages, formats, commits, and pushes atomically. TOTAL BAN on raw git commits (`git commit`, `git add -A`, raw `git push`), colons inside the GitMap message argument, and conventional prefixes (`docs(...)`, `feat(...)`, `fix(...)`). ZERO intermediate commits during Phase 1 or Phase 2; all files across the turn MUST be committed together at the final step of Phase 3. Before GitMap, all push gates must pass (targeted checks, secrets gate, and `.gitignore` hygiene). Mandatory Completion Invariant ('No Push = Not Done'): If the code is not committed to Git and pushed upstream to GitHub, the task is strictly considered INCOMPLETE and NOT DONE. Workers never run git commands or GitMap commit tools; only lead does.\n",
        content,
        count=1,
    )

    content = re.sub(
        r"- \*\*R11 Strict Relative Git Paths[\s\S]*?(?=- \*\*R12)",
        "- **R11 Strict Relative Git Paths & Lowercase Hygiene.** Strict ban on absolute paths (`C:\\...`, `/home/...`) and `file:///` URIs. Only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well. Paths relative from git root. All filenames, documentation, and specs strictly lowercase (e.g. `readme.md`, `agents.md`, `skill.md`).\n",
        content,
        count=1,
    )

    # 8. Upgrade Section 3 (GitMap Table & Search Protocol)
    sec3_pattern = re.compile(
        r"## 3\. GitMap High-Speed Command Primacy[\s\S]*?(?=---\s*\n\s*##\s*[457]\.)",
        re.MULTILINE,
    )
    if sec3_pattern.search(content):
        content = sec3_pattern.sub(GITMAP_COMMAND_PRIMACY_TABLE_V6 + "\n\n", content, count=1)

    # 9. Upgrade Step 0 (Preflight & Task DB Initialization)
    # Check if prompt has domain sections (like 37 which numbers Step 0 as 7)
    step0_pattern = re.compile(
        r"## (\d+)\. Step 0: Preflight, Platform Handshake & Ledger Creation[\s\S]*?(?=---\s*\n\s*##\s*\d+\.\s*Phase 1)",
        re.MULTILINE,
    )
    match_step0 = step0_pattern.search(content)
    if match_step0:
        sec_num = match_step0.group(1)
        step0_replacement = STEP_0_PREFLIGHT_V6.replace("## 5. Step 0:", f"## {sec_num}. Step 0:")
        content = step0_pattern.sub(step0_replacement + "\n\n", content, count=1)

    # 10. Upgrade Phase 1 (Planning, Spec & Brain Roadmap)
    phase1_pattern = re.compile(
        r"## (\d+)\. Phase 1: Planning Step & Spec Step[\s\S]*?(?=---\s*\n\s*##\s*\d+\.\s*Phase 2)",
        re.MULTILINE,
    )
    match_phase1 = phase1_pattern.search(content)
    if match_phase1:
        sec_num = match_phase1.group(1)
        phase1_replacement = PHASE_1_PLANNING_SPEC_V6.replace("## 6. Phase 1:", f"## {sec_num}. Phase 1:")
        content = phase1_pattern.sub(phase1_replacement + "\n\n", content, count=1)

    # 11. Upgrade Phase 2 (Execution, Worker Brief, Heatmap Telemetry, Dynamic Respawn)
    phase2_pattern = re.compile(
        r"## (\d+)\. Phase 2: Execution Step \(Worker Waves\)[\s\S]*?(?=---\s*\n\s*##\s*\d+\.\s*Phase 3)",
        re.MULTILINE,
    )
    match_phase2 = phase2_pattern.search(content)
    if match_phase2:
        sec_num = match_phase2.group(1)
        phase2_replacement = PHASE_2_EXECUTION_V6.replace("## 7. Phase 2:", f"## {sec_num}. Phase 2:")
        content = phase2_pattern.sub(phase2_replacement + "\n\n", content, count=1)

    # 12. Upgrade Phase 3 (Consolidation & Push)
    phase3_pattern = re.compile(
        r"## (\d+)\. Phase 3: Consolidation, Evidence Verification & Atomic GitMap Push[\s\S]*?(?=---\s*\n\s*##\s*\d+\.\s*Final Report)",
        re.MULTILINE,
    )
    match_phase3 = phase3_pattern.search(content)
    if match_phase3:
        sec_num = match_phase3.group(1)
        phase3_replacement = PHASE_3_CONSOLIDATION_V6.replace("## 8. Phase 3:", f"## {sec_num}. Phase 3:")
        content = phase3_pattern.sub(phase3_replacement + "\n\n", content, count=1)

    # 13. Upgrade Section 12 (or 14 in 37) Banned Operations Checklist
    banned_pattern = re.compile(
        r"## (\d+)\. Banned Operations Checklist \(TOTAL BAN — Auto-Reject on Violation\)[\s\S]*?(?=---\s*\n\s*##\s*\d+\.\s*Non-Negotiable)",
        re.MULTILINE,
    )
    match_banned = banned_pattern.search(content)
    if match_banned:
        sec_num = match_banned.group(1)
        banned_replacement = BANNED_OPERATIONS_CHECKLIST_V6.replace("## 12. Banned", f"## {sec_num}. Banned")
        content = banned_pattern.sub(banned_replacement + "\n\n", content, count=1)

    # 14. Upgrade Section 14 (or 16 in 37) Anti-Hallucination & Blast Radius Checklist
    ah_pattern = re.compile(
        r"## (\d+)\. Anti-Hallucination & Blast Radius Checklist[\s\S]*?(?=---\s*\n\s*##\s*\d+\.\s*Final Step)",
        re.MULTILINE,
    )
    match_ah = ah_pattern.search(content)
    if match_ah:
        sec_num = match_ah.group(1)
        ah_replacement = SECTION_14_ANTI_HALLUCINATION_V6.replace("## 14. Anti-Hallucination", f"## {sec_num}. Anti-Hallucination")
        content = ah_pattern.sub(ah_replacement + "\n\n", content, count=1)

    # 15. Upgrade Section 15 (or 17 in 37) Final Step Git Commit & Push Mandate
    fs_pattern = re.compile(
        r"## (\d+)\. Final Step Git Commit & Push Mandate \(Strict Checklist\)[\s\S]*?(?=## MUST FOLLOW NON-NEGOTIABLE)",
        re.MULTILINE,
    )
    match_fs = fs_pattern.search(content)
    if match_fs:
        sec_num = match_fs.group(1)
        fs_replacement = SECTION_15_FINAL_STEP_MANDATE_V6.replace("## 15. Final Step", f"## {sec_num}. Final Step")
        content = fs_pattern.sub(fs_replacement + "\n\n", content, count=1)

    return content


def upgrade_prompt_file(file_path: Path) -> bool:
    """Read, upgrade, and save prompt file in 01-prompts/15-cg-execute/."""
    content = file_path.read_text(encoding="utf-8")
    original = content
    filename = file_path.name

    if filename == "31-cg-execute-in-below-steps.md":
        content = upgrade_prompt_31(content)
    else:
        content = upgrade_standard_prompt(content, filename)

    if content != original:
        file_path.write_text(content, encoding="utf-8", newline="\n")
        return True

    return False


def mirror_to_v4(src_file: Path) -> bool:
    """Mirror upgraded prompt file to 01-prompts/v4/15-cg-execute/ with exact parity."""
    if not src_file.exists():
        return False

    v4_target = V4_PROMPTS_DIR / src_file.name
    content = src_file.read_text(encoding="utf-8")

    if not v4_target.exists() or v4_target.read_text(encoding="utf-8") != content:
        v4_target.parent.mkdir(parents=True, exist_ok=True)
        v4_target.write_text(content, encoding="utf-8", newline="\n")
        return True

    return False


def sync_skills() -> int:
    """Synchronize companion skills across .agents/skills/ and .cursor/skills/."""
    synced = 0
    prompts = sorted([p for p in PROMPTS_DIR.glob("*.md") if p.name != "readme.md"])

    for p in prompts:
        slug = extract_prompt_slug(p)

        # Candidate skill names based on naming conventions
        skill_candidates = [
            slug,
            f"cg-{slug}",
        ]
        if slug.startswith("cg-"):
            skill_candidates.append(slug[3:])

        # Known manual mapping exceptions from specification
        manual_map = {
            "execute-coding-guideline-fix": ["execute-coding-guideline-fix", "cg-execute", "audit-execute-coding-guideline-fix"],
            "error-management": ["cg-error-management"],
            "booleans-and-complex-conditions": ["boolean-improvements-v2", "cg-boolean-and-naming"],
            "naming-conventions-and-boolean-prefixes": ["cg-boolean-and-naming"],
            "constants-and-enums": ["cg-enum-standards"],
            "data-and-schema": ["cg-database-conventions"],
            "extract-generic-types-to-types-go": ["cg-extract-types"],
            "clean-work-artifacts-and-os-caches": ["clean-artifacts-and-git-history"],
            "map-dispatch-and-handler-lookup": ["cg-map-dispatch-and-handler-lookup"],
        }
        if slug in manual_map:
            skill_candidates.extend(manual_map[slug])

        for sname in sorted(set(skill_candidates)):
            agent_skill_file = AGENTS_SKILLS_DIR / sname / "skill.md"
            cursor_skill_file = CURSOR_SKILLS_DIR / sname / "skill.md"

            if agent_skill_file.exists():
                text = agent_skill_file.read_text(encoding="utf-8")
                # Ensure cursor has exact copy
                cursor_skill_file.parent.mkdir(parents=True, exist_ok=True)
                if not cursor_skill_file.exists() or cursor_skill_file.read_text(encoding="utf-8") != text:
                    cursor_skill_file.write_text(text, encoding="utf-8", newline="\n")
                    synced += 1

    return synced


def verify_upgraded_prompt(file_path: Path) -> list[str]:
    """Verify that a prompt satisfies all 10 V6 specifications."""
    errors: list[str] = []
    text = file_path.read_text(encoding="utf-8")

    # 1. Header Parameters
    if "N = 400" not in text or "A = 3" not in text or "H = 2" not in text or "Capacity = 6" not in text:
        errors.append("Missing V6 header parameter block (N=400, A=3, H=2, Capacity=6)")

    # 2. Extreme Understood Turn-1 Showcase
    if "Pillar 1: Strategic Intent & Systems Reasoning" not in text:
        errors.append("Missing Extreme Understood Turn-1 Showcase (Pillar 1)")
    if "Pillar 5: Multi-Agent Concurrency Topology" not in text:
        errors.append("Missing Extreme Understood Turn-1 Showcase (Pillar 5)")

    # 3. Step 0 Data Schema Verification Gate
    if "gitmap agent task init --name" not in text and file_path.name != "readme.md":
        errors.append("Missing Step 0 gitmap agent task init")
    if "gitmap agent task status --json" not in text and file_path.name != "readme.md":
        errors.append("Missing Step 0 gitmap agent task status --json")
    if "gitmap agent task status --ddl" not in text and file_path.name != "readme.md":
        errors.append("Missing Step 0 gitmap agent task status --ddl")

    # 4. Step 3 Brain & Comprehensive Roadmap Creation
    if "Brain & Comprehensive Roadmap Creation" not in text and file_path.name != "readme.md":
        errors.append("Missing Phase 1 Step 3 Brain & Comprehensive Roadmap Creation")

    # 5. Section 3 GitMap Table with Heatmap suite
    if "gitmap heatmap log" not in text and file_path.name != "readme.md":
        errors.append("Missing gitmap heatmap log in GitMap command table")
    if "gitmap agent heatmap" not in text and file_path.name != "readme.md":
        errors.append("Missing gitmap agent heatmap in GitMap command table")

    # 6. Section 7.2 Worker Brief & Rule 9 Heatmap Telemetry
    if "Mandatory Heatmap Telemetry" not in text and file_path.name != "readme.md":
        errors.append("Missing Rule 9 Mandatory Heatmap Telemetry in Worker Brief")
    if (
        "agent-task.db" not in text
        and "heatmap.db" not in text
        and "agent-root.db" not in text
        and file_path.name != "readme.md"
    ):
        errors.append("Missing 3-Tier Database Location architecture in Worker Brief")

    # 7. Section 7.4 Immediate Dynamic Respawn & Join Protocol
    if "Immediate Dynamic Respawn & Join Protocol" not in text and file_path.name != "readme.md":
        errors.append("Missing Section 7.4 Immediate Dynamic Respawn & Join Protocol")

    # 8. Section 12 Banned Operations Checklist
    if "HEATMAP TELEMETRY & CONTENTION INSPECTION" not in text and file_path.name != "readme.md":
        errors.append("Missing HEATMAP TELEMETRY & CONTENTION INSPECTION checkbox in Section 12")

    # 9. Hyphen commit format
    if 'gitmap cpf "<module> - <summary>"' not in text and 'gitmap cpf "<module> - <feature summary>"' not in text and file_path.name != "readme.md":
        errors.append("Missing hyphen format in GitMap commit command")

    # 10. Prompt 31 suffix precedence
    if file_path.name == "31-cg-execute-in-below-steps.md":
        if "## 🚨 High Priority Instructions Below" not in text:
            errors.append("Missing trailing High Priority Instructions Below boundary in prompt 31")
        if "Bottom-Instruction Priority Mandate" not in text:
            errors.append("Missing Bottom-Instruction Priority Mandate in prompt 31")

    return errors


def main() -> None:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    parser = argparse.ArgumentParser(description="Upgrade 15-cg-execute prompts and skills to V6 parity.")
    parser.add_argument("--file", "-f", type=Path, help="Upgrade a single prompt file")
    parser.add_argument("--dry-run", "-n", action="store_true", help="Preview changes without writing")
    parser.add_argument("--verify", "-v", action="store_true", help="Verify all prompts against V6 specifications")
    args = parser.parse_args()

    prompts = [args.file] if args.file else sorted([p for p in PROMPTS_DIR.glob("*.md") if p.name != "readme.md"])

    if args.verify:
        print(f"Verifying {len(prompts)} prompt specifications against V6 invariants...")
        has_errors = False
        for p in prompts:
            errs = verify_upgraded_prompt(p)
            if errs:
                has_errors = True
                print(f"[FAIL] {p.name}: {len(errs)} verification issues")
                for e in errs:
                    print(f"   - {e}")
            else:
                print(f"[PASS] {p.name}: V6 Parity Verified")
        if has_errors:
            sys.exit(1)
        print("All prompt specifications verified with 100% V6 parity.")
        sys.exit(0)

    print(f"Processing {len(prompts)} prompts...")
    upgraded_count = 0
    mirrored_count = 0

    for p in prompts:
        if args.dry_run:
            print(f"[DRY-RUN] Would upgrade: {p.name}")
            continue

        if upgrade_prompt_file(p):
            upgraded_count += 1
            print(f"Upgraded prompt: {p.name}")
        else:
            print(f"Already upgraded: {p.name}")

        if mirror_to_v4(p):
            mirrored_count += 1
            print(f"Mirrored to v4: {p.name}")

    if not args.dry_run:
        synced_skills = sync_skills()
        print(f"\nExecution Summary:")
        print(f"- Upgraded prompts: {upgraded_count}")
        print(f"- Mirrored v4 prompts: {mirrored_count}")
        print(f"- Synchronized companion skills: {synced_skills}")


if __name__ == "__main__":
    main()
