# Subtask 02: Warmup Matrix, Dialog Fix & CLI Ergonomics

**Subtask ID:** `232-02`  
**Parent Task:** `232-warm-up-ui-and-emailing-ui-improvements`  
**Status:** `IN_PROGRESS` (Cluster 3: Improvements 21–30 IMPLEMENTED; Cluster 4: Improvements 31–40 IMPLEMENTED; Cluster 5: Improvements 41–50 PENDING)  
**Improvements Covered:** Improvements 21–50 (Clusters 3, 4, 5)  
**Target Specs:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/02-warmup-ui-improvements-spec.md`  

---

## Technical Scope & Deliverables
1. **Interactive Account × Model Matrix Grid (Improvement 21)**: Per-account and per-model warmup enablement matrix (`AccountModelWarmupMatrix.tsx`).
2. **Tier-Based Profiles (Improvement 22)**: Preset rules for Ultra/Pro/Eco tiers (`WarmupTierPresets.tsx`).
3. **Per-Account Warmup Opt-Out Toggle (Improvement 23)**: Account exemption switch and badge (`AccountWarmupOptOut.tsx`).
4. **Dynamic Priority Model Cascade (Improvement 24)**: Automatic fallback ladder on rate limits/failures (`warmupCascade.ts`).
5. **Custom Probe Prompt Editor (Improvement 25)**: Configurable probe strings and token ceiling indicator (`ProbePromptEditor.tsx`).
6. **Bulk Batch Model Assignment Popover (Improvement 26)**: Multi-account model selector popover (`BatchWarmupModelPopover.tsx`).
7. **Dry-Run Simulation Mode (Improvement 27)**: Simulation modal and report generator (`WarmupDryRunModal.tsx`).
8. **Quota Lockout Override & Safe Forcing (Improvement 28)**: 6-day cooldown force confirmation dialog (`ForceWarmupDialog.tsx`).
9. **Surgical Fix for Missing Confirmation Modal (Improvement 29)**: Connected `isWarmupConfirmOpen` in `src/pages/accounts/AccountsDialogs.tsx` wired to `WarmupConfirmModal.tsx` and account store.
10. **Multi-Instance Fleet Sync (Improvement 30)**: Distributed warmup lease locking helper (`warmupFleetSync.ts`).
11. **Gaussian Schedule Jitter & Throttling (Improvements 31, 32)**: Configurable jitter window (±0–180s) and batch sizing (1–10 accounts).
12. **Caller IDE Active Preservation (Improvement 35)**: Pause automated warmup runs while active developer IDE session is streaming.
13. **Quota Reserve Guard Slider (Improvement 40)**: Minimum quota threshold slider (default 5%) preventing warmup on low quota.
14. **Warmup CLI Command Tree (Improvements 41–50)**:
   - `agm warmup status` (Headless status HUD)
   - `agm warmup list` (High-speed `termtable` account matrix)
   - `agm warmup run` (Targeted account/all dispatcher)
   - `agm warmup test` (Probe latency tester)
   - `agm warmup config` (Headless key-value manager)
   - `agm warmup cooldown` (Pruning tool)
   - `agm warmup history` (SQLite audit query)
   - Standardized `--json` envelope parity on all subcommands.

---

## Acceptance Criteria
- [x] Top action bar Warm Up button in `Accounts.tsx` successfully opens the confirmation modal (`WarmupConfirmModal.tsx`).
- [x] Account matrix grid permits granular per-account model toggling (`AccountModelWarmupMatrix.tsx`).
- [x] Model fallback cascade automatically tries secondary flash model upon 503 error (`warmupCascade.ts`).
- [x] Multi-instance fleet synchronization acquires leases to eliminate duplicate probes (`warmupFleetSync.ts`).
- [x] Gaussian schedule jitter slider and batch delay controls (Cluster 4: `ScheduleJitterSlider.tsx`, `StaggerBatchControls.tsx`, `ScanIntervalSlider.tsx`, `QuotaReserveSlider.tsx`).
- [x] Resilient backoff, quiet hours, connectivity gates, and 3-strike circuit breaker (Cluster 4: `QuietHoursScheduler.tsx`, `NetworkConnectivityGate.tsx`, `warmupBackoff.ts`, `callerIdePreservation.ts`, `warmupCircuitBreaker.ts`, `CircuitBreakerBanner.tsx`).
- [ ] Headless CLI warmup command tree and `--json` envelope (Cluster 5).
