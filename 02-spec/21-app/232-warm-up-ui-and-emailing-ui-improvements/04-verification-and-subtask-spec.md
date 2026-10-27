# Verification & Subtask Execution Specification: Warm Up UI and Emailing UI Improvements

**Document Version:** 4.272.0  
**Specification Sequence:** 232  
**Task Slug:** `warm-up-ui-and-emailing-ui-improvements`  
**Parent Document:** `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
**Referenced Specifications:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/02-warmup-ui-improvements-spec.md` (Improvements 01–50)  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/03-emailing-ui-improvements-spec.md` (Improvements 51–100)  

---

## 1. Executive Summary & Verification Mandate

This document establishes the end-to-end verification framework, testability matrices, and modular subtask execution plans for the **100 concrete improvements** specified across the **Warm Up UI** and **Emailing UI** systems of Antigravity Manager (`agm-ui`) and CLI (`agm`).

All verification activities adhere strictly to the **8-Pillar Quality Gate** of `02-spec/02-coding-guidelines/`:
1. **Error Management & Leaf Error Sovereignty**: Pure direct error propagation without unhandled exits; structured `AppError` wrapping.
2. **Deterministic Exit Codes & Specialized Handlers**: Strongly-typed exit enums (`constants.ExitGeneralError`) with zero raw magic integers.
3. **Monadic Results & Null Safety**: Fallible operations bound to monadic results or typed error envelopes.
4. **Parameter Structs**: Any function exceeding 2–3 parameters consolidated into a dedicated `*Params` struct.
5. **String Constants & Enums**: Zero raw magic strings; PascalCase enums ending with `Type` suffix.
6. **Function & Conditional Bounds**: Strict <= 15 line maximum per function; zero nested `if` statements (guard clauses only).
7. **File Sizing Cap**: UI components <= 100 lines; backend modules <= 300 lines.
8. **Positive Booleans & Whitespace Discipline**: Affirmative `is*`/`has*` only; blank line before `if`, after `}`, before `return`.

---

## 2. 100 Improvements Traceability Matrix

The 100 improvements are partitioned across four modular execution subtasks:

| Cluster | Improvement Range | Domain Focus | Subtask Plan Mapping | Primary File Targets |
|---|---|---|---|---|
| **Cluster 1** | Improvements 01–10 | Warmup Dashboard & Telemetry Visualizers | `01-subtask-warmup-dashboard-and-visualizers.md` | `src/components/settings/SmartWarmup.tsx`, `src/components/warmup/` |
| **Cluster 2** | Improvements 11–20 | Quota Reset Countdown Timers & Waves | `01-subtask-warmup-dashboard-and-visualizers.md` | `src/hooks/useWarmupResetTimer.ts`, `src/components/warmup/` |
| **Cluster 3** | Improvements 21–30 | Batch Warmup Matrix & Dialog Restoration | `02-subtask-warmup-matrix-and-cli-ergonomics.md` | `src/pages/accounts/AccountsDialogs.tsx`, `src/pages/accounts/` |
| **Cluster 4** | Improvements 31–40 | Model Tuning, Sliders & Jitter Schedules | `02-subtask-warmup-matrix-and-cli-ergonomics.md` | `src-tauri/src/modules/scheduler.rs`, `src/components/settings/` |
| **Cluster 5** | Improvements 41–50 | Warmup CLI Ergonomics & GitMap Parity | `02-subtask-warmup-matrix-and-cli-ergonomics.md` | `src-tauri/src/bin/agm/warmup_cmds.rs`, `cli/` |
| **Cluster 6** | Improvements 51–60 | Unified Email Command Center & HUD | `03-subtask-email-command-center-and-diagnostics.md` | `src/components/settings/EmailNotificationSettings.tsx` |
| **Cluster 7** | Improvements 61–70 | Visual Template Designer & Dual Simulator | `04-subtask-email-template-builder-and-remote-playground.md` | `src/components/email/TemplateDesigner/` |
| **Cluster 8** | Improvements 71–80 | Inbound Command Playground & Security | `04-subtask-email-template-builder-and-remote-playground.md` | `src/components/email/CommandPlayground/`, `src-tauri/` |
| **Cluster 9** | Improvements 81–90 | 5-Stage Diagnostic Ladder & Self-Healing | `03-subtask-email-command-center-and-diagnostics.md` | `src/components/email/DiagnosticLadder/`, `src-tauri/` |
| **Cluster 10** | Improvements 91–100 | Email CLI Management & Headless Parity | `04-subtask-email-template-builder-and-remote-playground.md` | `src-tauri/src/bin/agm/email_cmds.rs`, `cli/` |

---

## 3. High-Priority Bug Restoration: `AccountsDialogs.tsx` Missing Modal

### 3.1 Defect Classification
- **Affected File**: `src/pages/accounts/AccountsDialogs.tsx`
- **Trigger**: Clicking the top action bar Warm Up button (`Sparkles` icon in `src/pages/Accounts.tsx:220-244`).
- **Observed Behavior**: The click sets `setIsWarmupConfirmOpen(true)`, but no dialog opens; the button appears non-responsive to the user.
- **Root Cause**: During modular component decomposition (commit `22a87aaec`), `isWarmupConfirmOpen` was defined in `useAccountsState.ts` and passed to `AccountsDialogsProps`, but the actual `ModalDialog` rendering block was omitted from the JSX tree in `AccountsDialogs.tsx`.

### 3.2 Verification & Restoration Contract
In `02-subtask-warmup-matrix-and-cli-ergonomics.md` (Improvement 29), the modal is restored:
```tsx
<ModalDialog
  isOpen={state.isWarmupConfirmOpen}
  onClose={() => state.setIsWarmupConfirmOpen(false)}
  title={
    state.selectedIds.size > 0
      ? `Warm Up ${state.selectedIds.size} Selected Accounts`
      : 'Warm Up All Accounts'
  }
  description="This will send minimal ping probes (1 token) to prime quota buckets for weekly reset."
  confirmLabel="Start Warm Up"
  confirmVariant="primary"
  isLoading={state.isWarmuping}
  onConfirm={async () => {
    state.setIsWarmupConfirmOpen(false);
    await handlers.handleWarmupAll();
  }}
>
  <WarmupBatchSummary
    selectedCount={state.selectedIds.size}
    totalCount={accounts.length}
    monitoredModels={config.scheduled_warmup?.monitored_models}
  />
</ModalDialog>
```
- **Acceptance Criterion**: Clicking the Warm Up button in `Accounts.tsx` opens the modal with account count, model summary, and active execution spinner; confirming dispatches `handleWarmupAll()`.

---

## 4. End-to-End Test Plan & Verification Matrix

### 4.1 Automated UI & Component Test Contracts
1. **Countdown Ticker Precision**: Verify `useWarmupResetTimer` updates timestamps once per second without causing unmounting or rerendering of parent account tables.
2. **Zero-Clipping Responsive Test**: Verify at container widths `280px`, `360px`, `480px`, and `1200px` that no button in segmented capsules is clipped or wrapped awkwardly.
3. **Transparent Dropdown Style Invariant**: Verify inner `<select>` elements within segmented pill capsules render with `bg-transparent border-0 focus:ring-0` without nested white borders.
4. **Dark-Glass Truncation Banners**: Verify CLI outputs exceeding 50 lines render interactive dark-glass callouts (`TruncatedContextCallout`) displaying byte and line counts.

### 4.2 Backend Rust & IPC Test Contracts
1. **Warmup Dispatch Latency & Token Guard**: Verify loopback calls to `POST /internal/warmup` consume <= 2 output tokens and record latency to SQLite `warmup_audit_logs`.
2. **5-Stage Diagnostic Ladder Isolation**: Verify `test_email_connection_ladder` reports isolated stage results (DNS -> TCP -> TLS -> SASL Auth -> IMAP Ping) with granular error codes.
3. **Caller Instance Self-Preservation**: Verify remote commands (`* | agm instances stop`) targeting the host developer instance fail-fast with exit code 2 unless `--force` is provided.

### 4.3 CLI Parity & Envelope Test Contracts
1. **`agm warmup status --json`**: Verify valid JSON output matching `CliEnvelope<WarmupStatusPayload>`.
2. **`agm email doctor --json`**: Verify diagnostic ladder produces structured stage results matching UI output.
3. **`agm warmup list`**: Verify `termtable` output aligns columns with ANSI coloring and honors `NO_COLOR`.

---

## 5. Non-Negotiable Rules & Protocol Invariants

1. **Strictly Relative Git Paths**: Zero absolute paths (`C:\...`, `/home/...`) or `file:///` URIs.
2. **Zero Builds / Zero Tests During Planning**: Zero compilation or test execution during Phase 1.
3. **Worker Git Ban**: Subagents never run git commands; lead handles atomic commit and push.
4. **GitMap Search Primacy**: All code searches via `gitmap aum search`, `gitmap find`, `gitmap lf`.
