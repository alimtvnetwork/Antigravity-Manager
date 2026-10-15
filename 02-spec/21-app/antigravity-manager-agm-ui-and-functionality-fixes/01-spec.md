# Spec: Antigravity Manager (AGM) UI and Functionality Fixes

## slug
antigravity-manager-agm-ui-and-functionality-fixes

## Status
ACTIVE — plan enqueued at `.ai-memory/plans/antigravity-manager-agm-ui-and-functionality-fixes.md`

## 1. Objective

Fix the prompt-tree Send/Enqueue UX and functionality, add tree import/export,
polish UI professionalism, verify with E2E, and release with CI green.

## 2. Background — read before coding
- Commit `26e5a622` ("prompt-tree - decouple direct send and enqueue with
  rich copy and CLI parity") already did related work. **Do a gap analysis
  first**: diff each requirement below against the current implementation
  and build only what's missing. Do not rebuild what works.
- `.ai-memory/what-to-read.md` (canonical map; includes the E2E blind pack,
  prompt-tree send rules, and the FIFO queue invariant).
- `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/` and
  learned memory `27-e2e-instance-autoswitch-prompt-playbook.md`.
- AGENTS.md: FIFO invariant (`ORDER BY created_at ASC, id ASC` — LIFO
  forbidden), Headless & CLI parity, No Silent Errors, release workflow.

## 3. Requirements

### 3.1 Send/Enqueue UI (DetailHeader and prompt actions)
- The Send button must have a **dropdown** containing the Enqueue option.
  Send and Enqueue are two distinct actions with distinct buttons/entries.
- Professional UI polish of the prompt-tree action area (reviewer's
  discretion). **DO NOT touch the progress bar** (user hasn't reviewed it).

### 3.2 Send semantics
- Send = **immediately dispatch the prompt to the IDE on that conversation**.
- Send must work **regardless of whether the workspace/project is active**.
- Send must work for **all projects**, not just the active one.
- Currently Send secretly enqueues — this is the bug to fix.

### 3.3 Enqueue semantics
- Enqueue = append to the FIFO queue (`ORDER BY created_at ASC, id ASC`).
- The Enqueue button must actually work (currently broken).

### 3.4 New conversation
- Add a "new conversation" option in the prompt-tree action area
  (currently missing).

### 3.5 Copy with images
- Verify copy includes images in a compact form. At the end, **explain how
  the copy works** (format, image handling, compactness) in the final report.

### 3.6 Tree import/export
- In the tree view (right-hand side): copy-all-tree-prompts action.
- Export: single-file SQLite DB containing everything, AND JSON export.
- Import: from SQLite DB file, AND from JSON.
- One import/export section hosting all four operations.

### 3.7 E2E + release
- E2E-test the fixed flows where feasible (see `.ai-memory/e2e-testing/`).
- CI must be green at the end; minor-bump releases as needed to get there
  (explicitly authorized by the user).

## 4. Out of scope
- Progress bar changes (explicitly forbidden).
- `.github/workflows/` changes (standing rule).
- Deletions (standing rule).

## 5. Constraints (non-negotiable)
- GitMap-only search. Relative paths only. Lowercase filenames.
- Targeted `git commit <exact paths>` + `git push` by the lead only.
  NEVER `gitmap cpf`/`cpb`.
- No silent errors (AGENTS.md).
- Attribution strictly `(Thanks to @aukgit)`.
- `cargo fmt` clean; clippy clean; `npm run build` for frontend changes.
- FIFO invariant: never `ORDER BY updated_at DESC` in dispatch logic.
