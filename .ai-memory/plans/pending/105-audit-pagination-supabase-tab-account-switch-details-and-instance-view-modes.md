# Plan 105: Audit Pagination, Supabase Top-Level Tab, Account Switch Notification & Instance View Modes

## Metadata
- **Task ID**: `105-audit-pagination-supabase-tab-account-switch-details-and-instance-view-modes`
- **Spec Reference**: `02-spec/21-app/105-audit-pagination-supabase-tab-account-switch-details-and-instance-view-modes.md`
- **Total Steps**: 300 (Continuous N-Step Self-Loop)
- **Subagent Concurrency**: A = 2, H = 2

---

## Task Decomposition & Waves

### Wave 0: Ingestion & Discovery (Current Step)
- [x] Ingest Supabase credentials from `scripts/supabase-endpoints.json` into `%APPDATA%\antigravity-manager\supabase_config.json` and `~/.antigravity_tools/supabase_config.json`.
- [x] Copy user uploaded media to `assets/screenshots/105-*.png`.
- [ ] Spawn 2 Research subagents for Architecture & Spec discovery.

### Wave 1: Frontend Navigation, Supabase Tab & Audit Table Overhaul
- [ ] Subtask 105-01: Add `Supabase` to top-level navigation (`Sidebar.tsx`, `MiniView.tsx`, routes).
- [ ] Subtask 105-02: Audit view pagination (100/200 limit), caching, column reordering (`From -> To`, `Action`, `Status`, `Time`), time format `DD-MMM-YYYY HH:MM:SS`, double-click row detail modal, and email click-to-unmask.
- [ ] Subtask 105-03: Accounts view toolbar buttons (`Focus`, `+`, `Refresh`, `Warm`, `Show All Quotas`), email hover/double-click temporary unmasking with mouseleave re-masking, and glowing colorful gradient progress bar.

### Wave 2: Instances Page View Modes, Toolbar Polish & Prompt Tree View
- [ ] Subtask 105-04: Instances page Card View vs. List View toggle, search bar (filter by instance name, email, status, PID), and toolbar padding overhaul.
- [ ] Subtask 105-05: Duplicate/Clone Profile modal input text box padding fix (`px-4 py-3`).
- [ ] Subtask 105-06: Instance prompt/conversation tree view dialog (Left: projects & conversations; Right: queued & running prompt items with double-click modal for images/files).

### Wave 3: Backend Lifecycle, Restoration Stabilization & Settings
- [ ] Subtask 105-07: Account switch enriched audit trail logging (`[from_email, to_email]`, instance ID, IDE path, IDC alias, switch reason) and 7-second stabilization delay before prompt re-injection.
- [ ] Subtask 105-08: Settings proxy configuration: default path rewrite toggle, allow all URLs by default, comma-separated disallowed URLs.
- [ ] Subtask 105-09: Full system backup & restore with optional audit inclusion.

### Wave 4: CLI Parity, Local E2E Verification & Release
- [ ] Subtask 105-10: CLI global `--json` flag, `ls` and `--help` options verification.
- [ ] Subtask 105-11: Local-only E2E test script (`scripts/e2e-instance-commands-test.ps1`) executing all CLI commands.
- [ ] Subtask 105-12: Minor release bump, changelog update attributing `@aukgit`, push, and generate AI verification prompt.
