# Plan 68: Instance Duplicate / Clone, Default Selector, Fast-Forward Shortcut, and Security Fixes

> Subtask of parent task: Comprehensive Instances UX & Diagnostics Remediation
> Context: Specifications `02-spec/21-app/24-instance-duplicate-default-selector-and-security-ip-fixes.md` and `02-spec/22-app-issues/03-ip-security-null-and-ipc-query-fix.md`
> Status: Completed

## Tasks

- [x] Task 1: Fix IP Security backend null error in `src-tauri/src/modules/security_db.rs` (COALESCE aggregate queries and Option<u64> safe fallback)
- [x] Task 2: Fix IP Access Logs argument mismatch in `src-tauri/src/commands/security.rs` and `src/components/security/IpAccessLogs.tsx` (support both query object and flat fallback args)
- [x] Task 3: Implement Backend `set_default_instance` IPC command in `instance.rs` and register in `lib.rs` with default instance deletion protection
- [x] Task 4: Add `fast_forward_shortcut` to `AutoProfileSwitcherConfig` in backend and frontend models
- [x] Task 5: Enhance `InstanceSelector.tsx` (remove outside play button, keep Fast-Forward only with shortcut tooltip, add duplicate to header and items, enlarge sequence numbers `#1`, `#2`, `#3`...)
- [x] Task 6: Enhance `Instances.tsx` (prominent sequence numbers `#1`, `#2`..., Default badge, Set as Default button, and Duplicate action)
- [x] Task 7: Add shortcut configuration to `AutoSwitcherSettings.tsx` and register global shortcut listener in frontend (`src/hooks/useFastForwardShortcut.ts` & `src/components/layout/Layout.tsx`)
- [x] Task 8: Run focused pre-flight checks (`npx tsc --noEmit`, `cargo fmt --check`, `cargo clippy`)
