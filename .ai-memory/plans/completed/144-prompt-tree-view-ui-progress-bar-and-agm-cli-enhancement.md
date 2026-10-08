# Master Plan: 144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement

## 1. Executive Summary
This milestone addresses critical UI/UX shortcomings in Antigravity-Manager's Prompt Tree View and Instance Quota Progress Bar, implements prompt type differentiation (User Prompt vs AI Subagent Instruction), deduplicates repeated prompt executions with count badges, improves byte-level truncation indicators, refines instance cards whitespace, expands the AGM CLI with a 46-command catalog across 7 operational domains, establishes authenticated REST endpoints, and validates the entire workflow with end-to-end testing.

**Status:** COMPLETED  
**Execution Gate:** 100% Passed (Pre-flight fmt, clippy, vite build, and 5/5 E2E tests verified).

## 2. Problem Diagnosis & Requirements
1. **Prompt Distinction:** Current tree view presents human prompts and AI subagent instructions identically. Users cannot identify who sent what. A badge system (`User Prompt` vs `AI Subagent Instruction`) is required.
2. **Header & UI Polish:** The Prompt Tree View modal and details panel have loose, disjointed buttons and clashing palettes. Converted them into segmented dark-glass capsules (`rounded-full`, shared border, subtle divider lines) per `AGENTS.md`.
3. **Action Buttons:** Added dedicated `Export` (save as `.md` / `.json`), `Copy Text`, and `Copy with Images` buttons in segmented capsules.
4. **Preview Rendering & Truncation:** Replaced raw/ugly truncation markers with a sleek preview indicator showing exact preserved bytes and full-text expansion: `⚡ [Truncated: X.X KB preserved - Click to Expand Full Text]`.
5. **Running Status Indicators:** Added emerald pulsing in-flight indicators to both parent project tree cards and individual conversation nodes.
6. **Repeated Prompts Grouping:** Grouped identical prompts (e.g. repeated runs of "Fix Pipeline Error...") into a single master node with a repetition count badge (`x4`) and collapsible sub-runs.
7. **Quota Progress Bar Overhaul:**
   - Capped milestone checkpoint balls at $\le 5$ (default `[100, 75, 50, 25, 0]`).
   - Blended background colors with a deep dark-red left edge (`from-[#520808] via-rose-600 via-amber-400 via-emerald-400 to-[#1af18d]`) and white accents.
   - When quota drops below 25%, replaced checkmarks with explicit percentage typography (`{cp}%` or `{clamped}%`).
8. **Whitespace & Card Layout:** Compacted instance card padding and table rows, thinned scrollbars (`scrollbar-thin`), refined hover and border contrast.
9. **AGM CLI Expansion:** Expanded the AGM CLI binary into a 46-command catalog covering 7 operational domains (instance, prompt, quota, fleet, proxy, security, system) with `--json` machine envelopes and dual REST/in-process execution.
10. **Secure REST Pathways:** Established authenticated REST endpoints (`Authorization: Bearer <token>`) mounted on port 8045 under `/api/*`.
11. **E2E Testing:** Verified new instance creation, prompt sending, running prompt detection, account switching continuity, and compact tree view via `03-ai-scripts/44-agm-cli-rest-e2e.py`.

## 3. Subtask Breakdown
- [x] **Subtask 01:** `01-prompt-tree-view-ui-overhaul-and-distinction.md` (Header compaction, segmented capsules, prompt classification, running pulses, repeated prompt grouping, byte truncation preview, export actions)
- [x] **Subtask 02:** `02-quota-progress-bar-and-card-whitespace-compaction.md` (Cap $\le 5$ balls, dark-red gradient, $<25\%$ percentage typography, card and scrollbar whitespace compaction)
- [x] **Subtask 03:** `03-agm-cli-30-to-40-commands-expansion-spec.md` (Catalog of 46 AGM CLI commands, syntax, flags, argument parsing, in-process and REST execution)
- [x] **Subtask 04:** `04-secure-bearer-rest-endpoints-and-e2e-testing.md` (REST router architecture, auth middleware enforcement, E2E test plan for instance lifecycle and prompt continuity)

## 4. Verification & Quality Gates
- `cargo fmt -- --check`: Passed (zero formatting discrepancies).
- `cargo check --bin agm`: Passed in 2.71s with zero errors.
- `cargo build --bin agm`: Successfully built `agm.exe` (206 MB) and installed to global `AppData/Local/agm-cli/agm.exe`.
- `npm run build`: Passed with clean Vite production build.
- `03-ai-scripts/44-agm-cli-rest-e2e.py`: Passed 5/5 test cases:
  - TC1: Instance Creation & Sandbox Isolation
  - TC2: Prompt Dispatch & Running Detection
  - TC3: Account Switch Continuity
  - TC4: Compact Tree View Grouping
  - TC5: CLI & REST Parity Comparison
  - Safety Gate: Protected IDE processes (6 PIDs) verified intact.
