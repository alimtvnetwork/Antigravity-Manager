# Subtask 02: Warmup Matrix, Dialog Fix & CLI Ergonomics

**Subtask ID:** `232-02`  
**Parent Task:** `232-warm-up-ui-and-emailing-ui-improvements`  
**Status:** `PENDING`  
**Improvements Covered:** Improvements 21–50 (Clusters 3, 4, 5)  
**Target Specs:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/02-warmup-ui-improvements-spec.md`  

---

## Technical Scope & Deliverables
1. **Interactive Account × Model Matrix Grid (Improvement 21)**: Per-account and per-model warmup enablement matrix.
2. **Tier-Based Profiles & Opt-Out Switches (Improvements 22, 23)**: Preset rules for Ultra/Pro/Free tiers and individual account opt-outs.
3. **Dynamic Priority Model Cascade (Improvement 24)**: Automatic fallback to secondary model on upstream rate limits.
4. **Surgical Fix for Missing Confirmation Modal (Improvement 29)**: Re-insert `isWarmupConfirmOpen` `ModalDialog` in `src/pages/accounts/AccountsDialogs.tsx` wired to `handlers.handleWarmupAll`.
5. **Gaussian Schedule Jitter & Throttling (Improvements 31, 32)**: Configurable jitter window (±0–180s) and batch sizing (1–10 accounts).
6. **Caller IDE Active Preservation (Improvement 35)**: Pause automated warmup runs while active developer IDE session is streaming.
7. **Quota Reserve Guard Slider (Improvement 40)**: Minimum quota threshold slider (default 5%) preventing warmup on low quota.
8. **Warmup CLI Command Tree (Improvements 41–50)**:
   - `agm warmup status` (Headless status HUD)
   - `agm warmup list` (High-speed `termtable` account matrix)
   - `agm warmup run` (Targeted account/all dispatcher)
   - `agm warmup test` (Probe latency tester)
   - `agm warmup config` (Headless key-value manager)
   - `agm warmup cooldown` (Pruning tool)
   - `agm warmup history` (SQLite audit query)
   - Standardized `--json` envelope parity on all subcommands.

---

## File Boundaries
- `Exclusive Files`:
  - `src/pages/accounts/AccountsDialogs.tsx`
  - `src/components/warmup/WarmupAccountMatrix.tsx`
  - `src-tauri/src/modules/scheduler.rs`
  - `src-tauri/src/bin/agm/warmup_cmds.rs`
- `Read-Only Context Files`:
  - `src/pages/Accounts.tsx`
  - `src/pages/accounts/useAccountsHandlers.ts`
  - `src-tauri/src/proxy/handlers/warmup.rs`

---

## Acceptance Criteria
- [ ] Top action bar Warm Up button in `Accounts.tsx` successfully opens the confirmation modal.
- [ ] Account matrix grid permits granular per-account model toggling.
- [ ] `agm warmup status`, `agm warmup list`, and `agm warmup run` execute with exit 0 and valid `--json` outputs.
