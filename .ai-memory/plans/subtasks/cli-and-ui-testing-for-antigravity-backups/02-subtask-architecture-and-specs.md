# Subtask 02: Architectural Specification & Subtask Roadmap

**Parent Plan Reference**: [02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md](../../../02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md)  
**Parent Task**: `cli-and-ui-testing-for-antigravity-backups`  
**Subtask ID**: `02-subtask-architecture-and-specs`  
**Target Area**: Specification Authoring, Data Contracts, Two-Tier JSON Envelopes & Subtask Roadmap  
**Status**: APPROVED SUBTASK PLAN  
**Attribution**: strictly `@aukgit`  

---

## 1. Scope & Objectives

1. Establish canonical architectural specifications (`02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md`) capturing verbatim requirements, system design, and problem classification.
2. Define the formal data contracts and resolution invariants for multi-instance prompt inspection, dual sequence numbering (`[AGM:P001 | GM:#1]`, `[AGM:C001 | GM:<cid>]`), configurable 100–200 word preview truncation, and multimodal image extraction.
3. Formulate the real-time 4-step progress logging protocol for running prompt backup and restoration workflows.
4. Specify the Two-Tier JSON Envelope v2.0 schema for structured machine ingestion via `agm running --json` and `gitmap agy running --json`.
5. Author actionable, modular subtask execution plans in `.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/` providing clear task separation and implementation contracts for parallel workers.

---

## 2. Invariants & Guardrails

- **Strict Relative Git Paths Invariant**: All file references in specifications and subtask plans MUST be strictly repository-relative (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Absolute paths with drive letters (`C:\...`, `D:\...`) are strictly banned.
- **Strict Lowercase Filename Invariant**: All authored file names, subtask plans, and directories must use lowercase hyphen-separated formatting.
- **Lossless Verbatim Ingestion**: Master specifications must contain the user's complete prompt text verbatim under `## User Request (Verbatim)`.
- **FIFO Restoration Invariant**: Data access designs across SQLite queues (`active_prompts`, `prompt_backups`) must mandate strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).
- **No Silent Errors Invariant**: All data contract fallbacks and error paths must route through `AppError` or record warnings via `crate::error::record_ignored` with documented justification.
- **Attribution Invariant**: Attribution in documentation and changelogs must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).

---

## 3. Detailed Implementation Steps

### Step 1: Formalize Multi-Instance Running Prompts Query Engine Contract
- In `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md`:
  - Detail instance discovery mechanism iterating `instance::load_registry()`.
  - Define active liveness detection utilizing `.antigravity_goal_prompt.log` timestamps and system process queries.
  - Specify status extraction from SQLite `active_prompts` and transcript reverse-scanning (`transcript_full.jsonl`).
  - Outline CLI routing fixes in `src-tauri/src/bin/agm/main.rs` routing `agm running` directly to `running_prompts_cmds::cmd_running_prompts`.

### Step 2: Define Dual Sequence Assignment Specification
- Formalize sequence hierarchy:
  - Instance Sequence: `#1` (default), `#2`, `#3`...
  - Project Sequence: `[AGM:P001 | GM:#1]` with zero-padded 3-digit AGM code and integer GitMap cluster sequence.
  - Conversation Sequence: `[AGM:C001 | GM:<cid>]` with zero-padded 3-digit AGM code and 8-character hex CID.
- Detail the bidirectional resolution function `resolve_agm_sequence_target` supporting tokens `P001`, `AGM:P001`, `GM:#1`, `C001`, `AGM:C001`, `GM:<cid>`, `1`, `#1`.

### Step 3: Formalize 100-200 Word Truncation Precedence & Data Model
- Define the configuration schema in `gui_config.json` under key `prompt_preview_max_words` / `promptPreviewWordLimit`.
- Establish resolution precedence:
  1. CLI Flag `--full` (no truncation).
  2. CLI Flag `--words <N>` / `--wc <N>` / `-w <N>`.
  3. User configuration in `gui_config.json` clamped to `[100, 200]`.
  4. Hard fallback: 100 words.
- Detail the Rust implementation contract in `AppConfig` (`src-tauri/src/models/config.rs`) and TypeScript contract in `src/types/config.ts`.
- Specify bounded truncation algorithm in `src-tauri/src/modules/repo_db/text_utils.rs`.

### Step 4: Specify Multimodal Image Extraction & Tracing Engine
- Formulate the multimodal image taxonomy:
  - Inline Base64 data URIs (`data:image/...;base64,...`).
  - Local filesystem disk paths (`.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.svg`, `.bmp`).
  - Multimodal sub-images in nested tool calls and payloads.
  - Screenshot tokens (`SS_ID_<digits>`).
- Establish counting contract:
  $$\text{image\_count} = \text{data\_uris.len}() + \text{found\_paths.len}()$$
- Define reporting attributes: `image_count`, `image_uris_count`, `image_paths_count`, and `image_paths: Vec<String>`.
- Specify deep multimodal tracing logs for CLI and UI feedback.

### Step 5: Document Step-by-Step Progress Logging Protocol & JSON Envelopes
- Document the 4-step real-time progress logging ladders for:
  - `agm backup running [instance]`:
    - Step 1/4: Discover running prompts across instances.
    - Step 2/4: Trace and extract multimodal image payloads.
    - Step 3/4: Persist snapshot to `backup_prompts.db` & state cache.
    - Step 4/4: Verify backup record integrity.
  - `agm restore running [instance]`:
    - Step 1/4: Locate latest backup snapshot in `backup_prompts.db`.
    - Step 2/4: Reconstitute multimodal image assets and payload caches.
    - Step 3/4: Insert prompts into `active_prompts` FIFO queue.
    - Step 4/4: Seed workspace resumption file & refresh Prompt Tree View.
- Define Two-Tier JSON Envelope v2.0 schema matching `attributes`, `variables`, and `data` blocks.

### Step 6: Author Subtask Roadmap & Cross-Worker Coordination
- Structure discrete subtask plans to guide parallel workers:
  - `01-subtask-git-sync-and-preflight.md`: Preflight discovery and baseline synchronization.
  - `02-subtask-architecture-and-specs.md`: Architecture spec, data models, and subtask roadmap.
  - Companion subtasks in `.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/` covering CLI implementation, GitMap delegation, settings UI, help documentation, and E2E testing.
- Update plan cross-references in `.ai-memory/plans/cli-and-ui-testing-for-antigravity-backups.md`.

---

## 4. Verification & Quality Gates

| Verification Item | Command / Check | Expected Result | Pass/Fail Criteria |
|:---|:---|:---|:---|
| **Spec Existence** | Check `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md` | Contains all required sections | File present & >200 lines |
| **Verbatim Request** | Inspect `## User Request (Verbatim)` | Matches original prompt text word-for-word | Exact verbatim match |
| **Path Hygiene** | Search for `C:\` or `D:\` in spec files | No absolute filesystem paths found | Zero absolute paths |
| **Dual Sequence Model** | Review sequence section | Defines `[AGM:P001 | GM:#1]` and resolution | Clear formal spec |
| **Word Truncation Model** | Review truncation section | Defines 100–200 clamping and precedence | Complete model |
| **Subtask Plan Links** | Cross-link validation | Relative links resolve to existing files | Valid links |

---

## 5. Traceability & Cross-References

- Canonical Spec: [02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md](../../../02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md)
- Preceding Subtask: [.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/01-subtask-git-sync-and-preflight.md](./01-subtask-git-sync-and-preflight.md)
- Companion Specs:
  - `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-backup-restore-and-gitmap-spec.md`
  - `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/03-verification-and-help-spec.md`
- Master Plan: `.ai-memory/plans/cli-and-ui-testing-for-antigravity-backups.md`
