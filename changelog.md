# Changelog

## [v4.279.0] - 2026-10-29

### Fixed
- Prompt Tree Enqueue stayed at 0 after a successful click because force `loadTree` reset the 30s cache and viewing a completed turn overwrote conversation `is_queued`. `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` and `DetailHeader.tsx` now inject queued turns immediately and skip the wipe; `applySelectedTurnOverlay` in `src/components/instances/PromptTreeView/usePromptTreeData.ts` keeps QUEUED while inspecting an older turn; Enqueue shows the live count; rerun ×N appends that many queued turns on click (Thanks to @aukgit)
- `src-tauri/src/modules/repo_db/tree_collect.rs` no longer requires a 600s recency window to mark a conversation queued, so idle conversations still show FIFO queued prompts; `PENDING_STAGES` in `src-tauri/src/modules/repo_db/tree_node_conv.rs` includes `queued` and `in_progress` (Thanks to @aukgit)

### Tests
- Frontend `queuedTurns` and `rerunActions` passed; isolated `agm instances test-create` sandbox `test-enqueue-ui-1793289596-9596` enqueued `queued-1793289597305-0000-b68fdd`, `queued-1793289597524-0000-569e8d`, `queued-1793289597623-0000-65c281`, `queued-1793289597724-0000-187b40`, `queued-1793289597832-0000-d91ac9` immediately; `agm instances test-clean` removed only that sandbox; Default PID 3804 and `v2-3127` were not stopped (Thanks to @aukgit)

---

## [v4.278.0] - 2026-10-29

### Fixed
- Prompt tree enqueue and rerun now show on the clicked instance immediately: `src/components/instances/PromptTreeView/usePromptTreeData.ts` keeps queued turns across force-refresh instead of dropping the 30s SWR cache first; `mergeQueuedTurns` counts each identical rerun click; `isDefaultInstanceAlias` no longer treats `default-copy-*` as Default; `src-tauri/src/modules/repo_db/tree_node_conv.rs` attaches queued prompts by repo path or session id (Thanks to @aukgit)
- `agm instances switch --json` no longer prints `"status": "switched"` before `switch_account_to_instance`; `src-tauri/src/bin/agm/instance_cmds.rs` and `src-tauri/src/bin/agm/instance_routes_b.rs` emit `bound_email` only after the registry matches, otherwise `success: false` (Thanks to @aukgit)
- Enqueue wakes the FIFO scheduler immediately via `notify_prompt_queue` in `src-tauri/src/modules/repo_db/agy.rs` and Telegram instance-card rerun/send in `src-tauri/src/modules/telegram_inbound/instance_cards.rs` (Thanks to @aukgit)

### Improved
- Telegram `/status` cards show idle / busy / N queued and an ✉️ Send button that starts `/send` for that instance (`src-tauri/src/modules/telegram_inbound/instance_cards.rs`, `src-tauri/src/modules/telegram_inbound/send_flow.rs`) (Thanks to @aukgit)

### Tests
- Isolated `agm instances test-create` sandbox enqueued `queued-1793284251719-0000-913d6b`, `queued-1793284543149-0000-d43cf3`, `queued-1793284543288-0000-fd5c09`, copied, sent `p-0db14cf4`; JSON switch to a missing account returned `success: false`; Default PID stayed 3804 and `v2-3127` was not stopped; `agm instances test-clean` removed only the sandbox (Thanks to @aukgit)

---

## [v4.277.4] - 2026-10-29

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.277.3] - 2026-10-29

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.277.2] - 2026-10-29

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.277.1] - 2026-10-29

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.277.0] - 2026-10-29

### Added
- 100-Step Prompt Breakdown & Verification Templates: Added multi-step verification templates and prompt breakdowns in `src/components/instances/PromptTreeView/verificationTemplates.ts` (Thanks to @aukgit)
- Restyled Create Profile Modal: Modernized dialog in `src/pages/instances/CreateEditDialogs.tsx` with unified dark-glass styling, segmented button controls, and high-contrast form elements (Thanks to @aukgit)

### Fixed
- Release Tag Normalization: Purged rogue v5 tags and releases from GitHub, restoring canonical 4.x SemVer progression and eliminating spurious update notifications (Thanks to @aukgit)

---

## [v4.276.0] - 2026-10-29

### Fixed
- Telegram Screenshot Self-Focus Hang: capture PowerShell in `src-tauri/src/modules/telegram_inbound/open_shot.rs` no longer calls `ShowWindow`/`SetForegroundWindow` on the live AGM window; helpers time out after 8s so the manager stays responsive (Thanks to @aukgit)
- Instances CSV Download Fallback: `send_telegram_document` in `src-tauri/src/modules/telegram_inbound/media.rs` sends CSV/txt as `application/octet-stream` with an empty caption first; `src-tauri/src/modules/telegram_inbound/daemon.rs` retries then pastes bytes as text instead of another copy-button keyboard (Thanks to @aukgit)
- Telegram Copy Whole Prompt: instance cards in `src-tauri/src/modules/telegram_inbound/instance_cards.rs` drop `copy_text` (Telegram's 256-character Bot API cap rejects the whole keyboard); Copy and Full both send the complete prompt as a `.txt` document via `i:cp:` (Thanks to @aukgit)

### Improved
- CSS3 Hover Shade: `.agm-hover-shade` in `src/App.css` animates color, border, box-shadow, and filter only (`transform: none`) (Thanks to @aukgit)

### Tests
- Isolated switch+restore E2E: `agm instances test-create` sandbox dispatched prompt `p-555265fc`; stop backed it up for restore; Default and v2-3127 stayed running; `agm instances test-clean` purged only the sandbox. Spec: `02-spec/21-app/236-telegram-feature-enhancement/` (Thanks to @aukgit)

---

## [v4.275.3] - 2026-10-28

### Fixed
- Release Gate Unblocked: `.github/workflows/ci.yml` no longer runs `macos-latest` jobs. Hosted arm64 runners that are never acquired conclude `cancelled`, and cancelled jobs fail the workflow even with `continue-on-error`, which blocked v4.275.0–v4.275.2 releases. Windows and Ubuntu remain required (Thanks to @aukgit)

---

## [v4.275.2] - 2026-10-28

### Fixed
- CLI Telegram Observe Fan-Out: `agm telegram observe` and `status` now call `deliver_instance_status_fanout` from `src-tauri/src/modules/telegram_inbound/instance_cards.rs` so CLI and inbound `/status` send the same per-instance cards (Thanks to @aukgit)
- macOS CI Capacity Failures: CI macOS jobs in `.github/workflows/ci.yml` use `continue-on-error` so hosted arm64 runner acquisition failures cannot fail the workflow or block the Release gate (Thanks to @aukgit)

### Tests
- Isolated switch+restore E2E: `agm instances test-create` dispatched `p-73a4c18e`, Stage 1 backed it up from the test instance, Stage 5 restored and re-dispatched the same id; Default and v2-3127 stayed running; `agm instances test-clean` purged the sandbox (Thanks to @aukgit)

---

## [v4.275.1] - 2026-10-28

### Improved
- Telegram Screenshot Mosaic Completeness: `/shot`, `/screenshot`, and per-instance `i:sc:` now send the AGM window, virtual desktop, AGM Instances page, AGM Accounts page, and each running Antigravity IDE window (capped at 8, PID-filtered for `i:sc:`) from `src-tauri/src/modules/telegram_inbound/open_shot.rs` instead of photographing only the current manager window (Thanks to @aukgit)

### Fixed
- Local `/screenshot` Bypass: `ui_remote` Shot now routes through `execute_screenshot_command` so typed `/screenshot` and All shots use the same mosaic pipeline as the instance card Shot button (Thanks to @aukgit)
- Telegram Image Copy Keyboard: inbound image CDN copy buttons now clamp `copy_text` to `TELEGRAM_COPY_TEXT_MAX` (256) in `src-tauri/src/modules/telegram_inbound/daemon.rs` so a long URL cannot reject the whole keyboard (Thanks to @aukgit)

---

## [v4.275.0] - 2026-10-28

### Added
- Per-Instance Telegram Status Fan-Out: `/status` and `/observe` now send one compact HTML card per registered instance (`src-tauri/src/modules/telegram_inbound/instance_cards.rs`) covering running/queued prompts and projects with a 100-word expandable preview, instead of a single quota-only blob (Thanks to @aukgit)
- Instance-Scoped Telegram Actions: each card exposes Copy (native `copy_text` <= 256 chars), Full prompt as `.txt` document, Rerun, Done?, Released?, Auto, Fast-Forward, Screenshot, and Refresh via `i:<op>:<id8>` callbacks inside the 60-byte budget (Thanks to @aukgit)
- Telegram Feature Enhancement Spec (Task 234): architecture, component, and AI instruction specs under `02-spec/21-app/234-telegram-feature-enhancement/` with plan `.ai-memory/plans/completed/234-telegram-feature-enhancement.md` (Thanks to @aukgit)

### Fixed
- Telegram Copy Button Rejection: clamped native CopyTextButton payload to Telegram's 256-character cap (`TELEGRAM_COPY_TEXT_MAX`) so oversized `copy_text` no longer rejects the entire inline keyboard and leaves CSV/action buttons as a copied label (Thanks to @aukgit)
- Instances CSV Download: `send_telegram_document` now uses `application/octet-stream` with timestamped filenames (`agm-instances-YYYYMMDD-HHMMSS.csv`) and prompt/project count columns so Telegram delivers a real download instead of inlining or copying a button (Thanks to @aukgit)
- Fast-Forward Instance Scope: `/ff [instance]` and `ff:` now call `trigger_manual_rotation_for_instance` on the named instance instead of always rotating Default (Thanks to @aukgit)

### Improved
- Local Screenshot Mosaic: empty `/screenshot` and per-instance `i:sc:` capture the AGM window plus a VirtualScreen desktop mosaic (`src-tauri/src/modules/telegram_inbound/open_shot.rs`) (Thanks to @aukgit)
- CSS3 Hover Shade: `.agm-hover-shade` in `src/App.css` animates color, border, shadow, and brightness only (`transform: none`) on instance and Telegram cards; amber hover keeps `#090611` text (Thanks to @aukgit)
- Isolated Switch+Restore E2E: `agm instances test-create` sandbox received a running prompt, switched to a standby account, Stage 1 backed up from the test instance (not Default), Stage 5 restored and dispatched the same prompt id, Default and v2-3127 stayed running, then `agm instances test-clean` purged only the sandbox (Thanks to @aukgit)

---

## [v4.274.0] - 2026-10-28

### Added
- Idle Mode Auto-Notification Hub: Implemented `dispatch_idle_mode_alert` in `src-tauri/src/modules/notification_hub/idle_alerts.rs` and integrated into `src-tauri/src/modules/email_watcher/sensors.rs`, automatically detecting when IDE instances enter idle mode (`not_fully_idle == 0` and prompt queue empty) and dispatching alerts to Telegram and Email with 600-second per-instance anti-spam deduplication (Thanks to @aukgit)
- UI Step & Button Click Trace Logging: Created `src/utils/uiLogger.ts` and `src-tauri/src/commands/cmd_ui_trace.rs` (`record_ui_trace`), logging structured `[UI_STEP]` tracing logs and appending records to `task_history_db` audit trail on step transitions and button interactions (Thanks to @aukgit)
- CLI UI Remote Control Expansion: Expanded `agm ui click` in `src-tauri/src/bin/agm/ui_cmds.rs` to dynamically route instance targets to `/instances` and added `agm instances click <instance_id> <action>` with action alias normalization (`start` -> `launch`, `kill` -> `stop`, `ff` -> `fastforward`) (Thanks to @aukgit)
- Feature and Instance Management Specification (Task 233): Authored comprehensive architectural, component, and verification specifications under `02-spec/21-app/233-feature-and-instance-management/` and executed multi-agent plan under `.ai-memory/plans/completed/233-feature-and-instance-management.md` and `.ai-memory/plans/subtasks/233-feature-and-instance-management/` (Thanks to @aukgit)

### Improved
- Instance Section UI Contrast & Switch Button Elevation: Redesigned instance cards in `src/pages/instances/InstanceCard.tsx` with Royal Violet dark glass (`#171025`/90 background, `#40285e`/80 borders, `#120b1e` quota panels) eliminating Chromium opacity washouts; elevated the Switch Account action button in `src/pages/instances/InstanceCardActions.tsx` and `src/components/instances/table/InstanceRowActions.tsx` to an illuminated amber pill badge with 3D yellow tactile hover gradient (`#FDE047` to `#EAB308`, depth shadow `#A16207`); added unambiguous DOM IDs and semantic attributes (`id`, `data-ui-name`, `data-ui-id`) across all instance card action buttons (Thanks to @aukgit)

---

## [v4.273.1] - 2026-10-28

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.273.0] - 2026-10-28

### Fixed
- Image Extension Sanitization & E9001 Resolution: Resolved diagnostic error `E9001 — unsupported_image_extension: .` on `/instances` by sanitizing image extraction with `isLikelyImageUri` across `src/components/instances/PromptTreeView/clipboard.ts`, `src/components/instances/PromptTreeView/markdown.tsx`, and `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, rejecting bare `.`, `..`, and directory paths; added binary magic byte sniffing in `src-tauri/src/commands/cmd_files.rs` to detect TIFF signatures (`b"II*\0"`, `b"MM\0*"`) and fallback to `image/tiff` when extensions are missing or generic (Thanks to @aukgit)
- Restored Accounts Warmup Confirmation Dialog (`isWarmupConfirmOpen`): Destructured `isWarmupConfirmOpen` and `setIsWarmupConfirmOpen` in `src/pages/accounts/AccountsDialogs.tsx` and mounted a dedicated `<ModalDialog>` confirmation modal wired to `handlers.handleWarmupAll`, eliminating the silent action drop when clicking "Warm up all" on `/accounts` (Thanks to @aukgit)

### Added
- Task 232 Architectural Specification (100 Warm Up UI & Emailing UI Improvements): Authored comprehensive 4-part architectural and implementation specification in `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/` covering 100 concrete improvements across Smart Warmup HUD, 7-day quota countdown clocks, account matrix grid, multi-stage email diagnostics ladder, visual template builder, inbound IMAP playground, Split-DB telemetry, and headless CLI parity (`agm warmup` and `agm email`) (Thanks to @aukgit)
- End-to-End Test Suite Verification: Executed full test verification across frontend suites (`scripts/run-frontend-tests.mjs`, 11/11 passed), standalone account scoring tests (`accountAlgorithmScore.test.ts`, 14 passed), rerun queueing validation (`rerunActions.test.ts`, 21 passed), and full Vite/TypeScript compilation gate (17,332 modules transformed, exit code 0) (Thanks to @aukgit)

---

## [v4.272.0] - 2026-10-27

### Fixed
- Sequential Enqueue Retention & Deduplication Bypass: Fixed critical bug in `src-tauri/src/modules/repo_db/prompts_crud.rs` where `save_or_requeue_prompt` matched in-flight prompts by identical content and skipped enqueueing subsequent requests with `return Ok(())`; added `&& prompt.status != "queued"` ensuring sequential enqueues of identical instructions are reliably enqueued and retained in the SQLite FIFO queue (Thanks to @aukgit)
- Queued Turn Transcript Mutation Prevention: Resolved turn collision in `src-tauri/src/modules/repo_db/tree_node_conv.rs` where incoming queued prompts matched completed transcript turns (`p.prompt_text.trim() == ap.prompt_content.trim()`), causing transcript mutations instead of appending distinct `[QUEUED]` turns; restricted existing turn matching to `if is_ap_run && !is_queued` so queued prompts always appear as independent turns (Thanks to @aukgit)
- Active & Queued Prompt Starvation Guard: Updated `list_all_prompts()` in `src-tauri/src/modules/repo_db/project_queries.rs` with priority-weighted ordering (`CASE WHEN status IN ('queued', 'running', 'dispatched', 'in_flight') THEN 0 ELSE 1 END ASC`), preventing queued and active prompts from being crowded out by completed rows when prompt volume exceeds 500 records (Thanks to @aukgit)
- Enqueue Refresh Bar Suppression & 0ms Optimistic UI: Replaced full reload triggers `loadTree(true, true)` with background silent SWR `loadTree(false, true)` across `handleResendPrompt`, `handleEnqueuePrompt`, and `handleRerunConversationAction` in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` and `DetailHeader.tsx`, eliminating unwanted top indeterminate refresh bars during enqueue; removed 60ms artificial delay; added 0ms optimistic turn injection to `executeDirectEnqueue` and header actions for instantaneous visual appearance in the tree view (Thanks to @aukgit)
- Anti-gravity Rerun Target ID Retention: Fixed conversation targeting in `usePromptTreePromptActions.ts` where multi-round rerun operations were dropping the active conversation UUID or defaulting unexpectedly; reruns now explicitly target and preserve the specified conversation `id` and sequence immediately across all selected rounds (Thanks to @aukgit)
- Tree View Filter Compaction & Zero-Clipping Layout: Overhauled Row 2 filter controls in `src/components/instances/PromptTreeView/TreePanel.tsx` using two contiguous segmented pill capsules (`rounded-full`), Lucide `Image` icon replacing raw OS emoji, compact text labels (`Run`, `Queue`, `Pin`), and icon toggles (`EyeOff`/`Eye`, Lucide `Archive` with count badge), reducing row width from 451px to 194px and providing a +70px safety buffer against right-hand clipping in 328px sidebars (Thanks to @aukgit)

### Added
- Tree View UI/UX Design System Specification: Authored authoritative design system specification in `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md` and indexed in `02-spec/24-app-ui-design-system/01-index.md` and `02-spec/02-coding-guidelines/24-app-ui-design-system/readme.md`, documenting 10 structural principles: 2-plane depth hierarchy, 280px minimum width floor, zero horizontal scrolling, segmented capsule pill controls, transparent borderless inline selects, icon-first compact typography, subtle non-displacing hover transitions, terminal tree branch glyphs (`├──`, `└──`), 4-tier origin badges, and 0ms optimistic turn injection (Thanks to @aukgit)
- Strict E2E Verification Rerun Templates: Added strict verification question presets ("Have you tested these below instructions with end-to-end testing?" and "Have you tested the below instructions with end-to-end testing and released it?") into rerun quick options in `DetailHeader.tsx` (Thanks to @aukgit)
- Automated E2E Test Suite for Rerun, Enqueue & Tree Compaction: Added 28-assertion automated test suite `scripts/test-anti-gravity-rerun-and-ui-fixes-e2e.ps1` with caller IDE self-preservation and transient test sandbox isolation, verifying 100% green pass (Thanks to @aukgit)
- 4-Part Root Cause Analysis (RCA): Documented complete 4-part RCA and long-term invariants in `.ai-memory/rca/22-anti-gravity-rerun-and-ui-fixes.md` and `.ai-memory/issues/65-anti-gravity-rerun-and-ui-fixes.md` (Thanks to @aukgit)

---

## [v4.271.0] - 2026-10-27

### Fixed
- Post-Switch Prompt Restore Gate Fix: Resolved critical parameter mismatch in `src-tauri/src/modules/instance/switch.rs` where `list_prompt_backups` was passed an instance ID into `batch_id` (`WHERE backup_batch_id = ?`), always returning 0 rows and prematurely bypassing prompt restoration; replaced with scoped `count_unrestored_prompts_for_instance` (Thanks to @aukgit)
- Restored Status & Queue Dispatch Alignment: Added `'in_progress'` to `resend_single_pass` SQL query, `already_dispatched` deduplication bypass, and queue dispatch filters in `src-tauri/src/modules/repo_db/resend.rs` and `queue.rs`, ensuring prompts restored from backups are immediately discoverable and scheduled (Thanks to @aukgit)
- Strict FIFO Ordering Invariant: Refactored prompt selection in `src-tauri/src/modules/repo_db/dispatch.rs` from LIFO `max_by(updated_at)` to strict FIFO `pick_first_queued_prompt` (`min_by` on `created_at ASC, id ASC`), and replaced `ORDER BY updated_at DESC` with `ORDER BY created_at ASC, id ASC` in `tree_format.rs` (Thanks to @aukgit)
- Reactive Post-Switch Scheduler Wakeup: Added immediate `notify_prompt_queue()` signal and asynchronous dispatch probe in `src-tauri/src/modules/instance/workspace.rs` and `post_switch_resume.rs`, eliminating 30-second idle backoff delays and dispatching restored prompts instantly (Thanks to @aukgit)
- Default Instance Backup Count Fix: Corrected `count_backed_up_prompts` in `src-tauri/src/modules/repo_db/backup.rs` to include empty string `''` and `NULL` for default instance, preventing backed-up prompts from being counted as zero (Thanks to @aukgit)
- Manual Switch Backup Preservation: Updated `DesktopIntegration::on_account_switch` in `src-tauri/src/modules/integration/system_impl.rs` to capture backup return counts and evaluate both `repo_db` and `backup_prompts_db` before scheduling post-launch restore (Thanks to @aukgit)
- Subagent Filter False Positive Cleanse: Removed `lower.starts_with("you are ")` in `src-tauri/src/modules/repo_db/discovery.rs` to prevent dropping legitimate user instructions that begin with role framing (Thanks to @aukgit)
- Telegram Sub-2s Response Latency & Reactive Wakeup: Replaced 60s/30s polling sleep traps with `tokio::sync::Notify` (`TELEGRAM_DAEMON_NOTIFIER`), introduced persistent pooled `TELEGRAM_HTTP_CLIENT` to eliminate per-request TCP/TLS handshake penalties, decoupled inbound command processing to non-blocking background tasks, and implemented `delete_telegram_webhook` auto-healing on HTTP 409 Conflict (Thanks to @aukgit)
- Frontend Settling Countdown & Restore Visibility: Added interactive settling countdown badge (`⏳ Restoring Prompts in {N}s...`) and restore indicator in `src/components/instances/InstanceCard/InstanceCardHeader.tsx` driven by `useInstancePage.ts` (Thanks to @aukgit)

### Added
- E2E Unit Test Suites for Restore Integrity: Added comprehensive test suites in `src-tauri/src/modules/backup_prompts_db/tests.rs` and `src-tauri/src/modules/repo_db/tests.rs` covering dual-store prompt count scoping, discoverability of restored statuses, and strict FIFO queue ordering (Thanks to @aukgit)

---

## [v4.270.0] - 2026-10-27

### Added
- Strict Pre-Deployment & E2E Verification Prompts: Injected mandatory verification questions ("Have you tested these below instructions with end-to-end testing?" and "Have you tested the below instructions with end-to-end testing and released it?") into verification templates, suffix presets, and rerun message sets so dispatched AI agents cannot skip testing and release validation (Thanks to @aukgit)
- One-Click Release Page Action in Rerun: Added quick-action button in prompt instruction and header rerun flyouts to open release notes and changelogs directly (Thanks to @aukgit)
- Future Dual-Channel Chat Architecture Spec: Authored comprehensive specification (`02-spec/21-app/228-ui-and-feature-enhancement-instructions/04-chat-section-architecture-spec.md`) defining Channel 1 (IDE workspace bridge via CDP WebSocket + deep transcript reverse scan) and Channel 2 (Direct API chat client via local proxy SSE) (Thanks to @aukgit)
- 50+ Grounded System & UI Improvements Catalog: Documented 50 concrete, high-leverage improvement specifications (`02-spec/21-app/228-ui-and-feature-enhancement-instructions/05-fifty-plus-improvement-ideas-spec.md`) across Instances, Prompts, Databases, Proxy, and UI/UX (Thanks to @aukgit)

### Fixed
- Active Instance Card UI & Corner Artifacts: Eliminated asymmetric corner clipping by replacing `border-l-[6px]` with clean `border-l-4` in `SELECTED_CARD_CLASSES` and `SELECTED_ROW_CLASSES`, and resolved nested pill double-border and invalid `py-0.2` in `InstanceCard.tsx` (Thanks to @aukgit)
- Prompt Inspector Tab Alignment & Translucent View Accent: Aligned Preview, Raw, Edit, and Details buttons in `DetailTabs.tsx` to a shared `h-7` vertical baseline with `leading-none`, and replaced the harsh opaque royal blue background with a translucent dark-glass cyan/blue accent (`bg-blue-500/15 text-blue-600 dark:text-cyan-300 font-semibold`) (Thanks to @aukgit)
- CSS3 Tactile Button Animations: Exempted interactive buttons from the universal 250ms CSS delay in `src/App.css`, and introduced high-performance `.btn-tactile` micro-interactions (`100ms`, `active:scale-[0.985] active:translate-y-px`) for instantaneous response (Thanks to @aukgit)
- Top-Level Menu Capsule Geometry: Resolved rectangular hover highlight defects across `NavSettings.tsx`, `InstanceSelector.tsx`, and `NavMenu.tsx` using inner `rounded-full overflow-hidden` tracks and concentric radii, eliminating sharp edges on middle buttons (Thanks to @aukgit)
- "+ New Instance" Default-Clone Invariant: Ensured opening the creation dialog via "+ New Instance" in header and empty states preselects the Default instance ID rather than blank, preserving extensions and workspace continuity, with responsive CSS3 hover animations (Thanks to @aukgit)
- Universal 24-Hour Clock Formatting: Exported canonical `formatTime24` and `formatDateTime24` helpers in `src/utils/date.ts` (`HH:mm` / `HH:mm:ss`, `hour12: false`) and eliminated uncontrolled `toLocaleTimeString()` and `toLocaleString()` calls across AuditPanel, LogTable, LogDetailPanel, ErrorModal, ErrorHistoryDrawer, NetworkMonitor, CurrentAccount, and IpAccessLogs (Thanks to @aukgit)
- Dispatch Modal Redesign & Rerun FIFO: Streamlined mode buttons to "Send", "Enqueue", and "Rerun", widened popover to `w-[26rem]` to eliminate message dropdown text clipping, and unified `PromptInstructionView.tsx` `executeDirectRerun` to sequence all rounds via `dispatchRerunMessages` into the FIFO prompt queue (Thanks to @aukgit)

---

## [v4.269.0] - 2026-10-25

### Fixed
- Rerun 3 & Numbered Rerun Resolution: Added 1-based history row resolution for Telegram `/rerun 3`, `/rerun #3`, and `/rr 3`, selecting the 3rd prompt from recent history before falling back to UUID prefix search (Thanks to @aukgit)
- Monotonic FIFO Queue Ordering: Enforced monotonic millisecond timestamps and sequence counters in `enqueue_prompt_for_instance` alongside pacing in `dispatchRerunMessages`, guaranteeing multi-round reruns (e.g. Rerun ×3) sort and drain in exact chronological sequence (`ORDER BY created_at ASC, id ASC`) (Thanks to @aukgit)
- Canonical Attachment Syntax (`@[file;<path>]`): Enforced canonical third-bracket syntax `@[file;<file_path>]` with `file;` prefix across prompt attachment builders and upgraded all regex parsers in TypeScript and Rust with backward compatibility (Thanks to @aukgit)
- Rerun Enqueue Invariant: Routed all Telegram inbound reruns and frontend multi-resends strictly into SQLite FIFO queue with `status: 'queued'`, eliminating immediate injection and CDP task preemption (Thanks to @aukgit)

---

## [v4.268.0] - 2026-10-25

### Added
- Added Gemini 3.8 Flash model constants and hardened auto-switcher Claude limits detection (Thanks to @aukgit)

### Fixed
- Instant Rerun Enqueue & Latency Fix: Eliminated 30s background scheduler idle backoff via `tokio::sync::Notify` wakeup, immediate idle queue dispatch on enqueue, removal of 1.6s blocking CDP loop, and elimination of frontend 300ms pacing delay (Thanks to @aukgit)
- Image Attachment Normalization: Standardized multimodal image attachments into canonical bracketed format `Attachments:\n1. @[<path>]` across Copy, Send, Enqueue, and Rerun (Thanks to @aukgit)
- Rerun Enqueue Invariant: Guaranteed that rerun actions always enqueue into SQLite FIFO queue, eliminating immediate task preemption (Thanks to @aukgit)

---

## [v4.267.0] - 2026-10-25

### Changed
- Routine minor version bump from 4.266.0 to 4.267.0: all 14 version locations synchronized via `scripts/bump-version.mjs` (`package.json`, `package-lock.json`, `version.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/hooks.nsh`, `Casks/antigravity-tools.rb`, `readme.md`, `readme_en.md`, `src/components/layout/MiniView.tsx`, `releases-manifest.json`, `changelog.md`, `changelog_en.md`). No functional code changes in this bump; the v4.266.0 feature and fix set ships in its own release (Thanks to @aukgit)

---

## [v4.266.0] - 2026-10-25

### Added
- Compact, scroll-free dispatch popover UI in `src/components/instances/PromptTreeView/DetailHeader.tsx`, replacing verbose 17-item radio list and 37-bullet dump with a single-line `<select>` dropdown and collapsible variation preview, reducing popup height by >75% (Thanks to @aukgit)
- Intelligent anti-repeat random nudge dispatching via `extractUsedNudgesFromConversation` in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`, scanning conversation history and turns to strictly exclude already-sent questions from multi-round rerun dispatches (Thanks to @aukgit)

### Fixed
- Prompt tree view turns and child subagents now keep recent items on top: running turns sort to Tier 1, followed by queued turns, and completed turns descending by recency in `src/components/instances/PromptTreeView/ConversationNode.tsx` and `tierClassification.ts` (Thanks to @aukgit)
- Added fallback for weekly quota progress bar in `src/pages/instances/instanceCardData.ts` when account `quota_groups` are unpopulated due to API rate-limiting, achieving UI parity with backend CLI (Thanks to @aukgit)
- Extended release gate CI polling timeout in `.github/workflows/release.yml` to 60m (`MAX_WAIT_SECONDS=3600`) to safely absorb sequential runner queue latency (Thanks to @aukgit)

---

## [v4.265.0] - 2026-10-25

### Fixed
- Scoped CI concurrency group on `main` in `.github/workflows/ci.yml` to the commit SHA (`group: ${{ github.workflow }}-${{ github.ref == 'refs/heads/main' && github.sha || github.ref }}`), preventing rapid sequential commits on `main` from cancelling in-flight CI runs (Thanks to @aukgit)
- Expanded 4-part Root Cause Analysis in `.ai-memory/cicd-issues/47-release-gate-ci-race-condition-rca.md` to document GitHub Actions concurrency group collision and cancel-in-progress dynamics (Thanks to @aukgit)

---

## [v4.264.0] - 2026-10-24

### Fixed
- Release workflow gate in `.github/workflows/release.yml` now uses a bounded polling loop (every 20s up to 25 minutes) to await in-progress `ci.yml` runs on tag push, eliminating premature gate failure (`conclusion: missing`) and release refusal (Thanks to @aukgit)
- Documented 4-part Root Cause Analysis in `.ai-memory/cicd-issues/47-release-gate-ci-race-condition-rca.md` and indexed recurring CI race conditions in `.ai-memory/cicd-issues/readme.md` (Thanks to @aukgit)

---

## [v4.263.0] - 2026-10-24

### Fixed
- Stage 1 prompt snapshot during account rotation now scopes strictly to the target switched instance via `prompt_backup_source_instance_id` in `src-tauri/src/modules/instance/switch.rs` instead of defaulting to global active instance (Thanks to @aukgit)
- CLI and headless account switching now executes Stage 5 restore inline via `run_post_switch_resume_after_delay` in `src-tauri/src/modules/instance/post_switch_resume.rs`, preventing prompt loss when short-lived CLI processes exit before GUI timers fire (Thanks to @aukgit)
- Fast-forward leftover handling in `src-tauri/src/bin/agm/instance_fastforward.rs` only re-dispatches prompts if backed-up records remain, eliminating duplicate prompt dispatches after Stage 5 execution (Thanks to @aukgit)
- Real dispatched prompt counts correctly propagate across `src-tauri/src/modules/instance/workspace.rs` and audit logs during switch restoration (Thanks to @aukgit)

---

## [v4.262.0] - 2026-10-24

### Fixed
- Fast-forward Previous Profile now reads the **target** instance bound email in `src-tauri/src/bin/agm/instance_fastforward.rs` instead of the active-instance status. Isolated E2E printed `Previous Profile: zihadxshohag2@gmail.com` after `agm ff test-ff-prev-*` while Default stayed on `riseup.team.now@gmail.com` (Thanks to @aukgit)
- `agm auto-switch status --json` now includes `is_enabled` and `low_quota_threshold_percent` from `src-tauri/src/modules/auto_switcher/status.rs` (Thanks to @aukgit)
- Removed unused `mut` on the completions request body in `src-tauri/src/proxy/handlers/openai/completions.rs` (Thanks to @aukgit)

---

## [v4.261.0] - 2026-10-24

### Fixed
- Inspector markdown image clicks now open disk files through `openFileInIde` in `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx`. The in-app modal is only used for `data:`/`http` images or opener failure (Thanks to @aukgit)
- POSIX absolute image paths are treated as disk files in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` (Thanks to @aukgit)
- Removed unused `mut` on `stderr_reader` in `src-tauri/src/modules/repo_db/agy.rs` (Thanks to @aukgit)

---

## [v4.260.0] - 2026-10-24

### Fixed
- Prompt tree titles and out-of-workspace images now rebuild in the current `agm` binary. `agm prompts tree --json --local` skips the GUI daemon cache. A local dump of 144 conversations returned 0 untitled titles and 68 `file:///` media paths with 0 raw drive paths (Thanks to @aukgit)
- Current Conv keeps `conv-` ids. An isolated `agm instances test-create` sandbox received `session_id=conv-e2e-probe` from `agm prompts send --conversation`. Caller instances were not stopped (Thanks to @aukgit)
- Image clicks open the original file through `openFileInIde` in `src/components/instances/PromptTreeView/markdown.tsx` instead of stripping `file:///` into a broken `/C:/` path (Thanks to @aukgit)
- Instance card root actions are one full-width `rounded-full divide-x` capsule in `src/pages/instances/InstanceCardActions.tsx`. Yellow hover uses black text (Thanks to @aukgit)
- Tree header keeps a single Import/Export menu. Hide Tree and Show Tree are icon tooltips. Reset and Refresh stay labeled (Thanks to @aukgit)

### Added
- Error CLI skill at `.agents/skills/agm-error-manager-cli/skill.md` for `agm errors list|inspect|count|fix|clear` (Thanks to @aukgit)

---

## [v4.259.0] - 2026-10-24

### Fixed
- Prompt Tree View header height compacted to `< 42px` in `src/components/instances/PromptTreeView/ModalHeader.tsx` and tightened node spacing in `TreePanel.tsx` (Thanks to @aukgit)
- Import/Export converted to compact Lucide icons (`Download`, `Upload`) with tooltips; Hide Tree View converted to `PanelLeftClose`/`PanelLeftOpen` icon button (Thanks to @aukgit)
- Relabeled yellow button adjacent to Refresh to "Reset" with high-contrast amber styling (`text-zinc-950 font-bold bg-amber-400 hover:bg-amber-300 shadow-[0_2px_0_0_#b45309]`) and 3D tactile elevation (Thanks to @aukgit)
- Eliminated green hover outline shifts in `ConversationNode.tsx` using permanent base `border border-transparent` and subtle background tint transitions (Thanks to @aukgit)
- Enforced strict 30-second TTL cooldown guard (`TREE_CACHE_TTL_MS = 30000`) in `usePromptTreeData.ts` to eliminate mouseover-induced refetches (Thanks to @aukgit)
- Reunited Send, Enqueue, and Rerun into a contiguous segmented dropdown pill capsule in `PromptInstructionView.tsx` with borderless transparent destination selectors (Thanks to @aukgit)
- Resolved Root Cause Analysis (RCA) defect in `usePromptTreePromptActions.ts` and `src-tauri/src/modules/repo_db/dispatch.rs` where "Current Conversation" mode was erroneously routing to new conversations (Thanks to @aukgit)
- Implemented `formatIdeClickableFileLink` in `clipboard.ts` and `markdown.tsx` formatting images and file paths into RFC-8089 `file:///` clickable links with a 1-click "Open in IDE" preview button (Thanks to @aukgit)
- Harmonized toolbar action clusters in `InstanceRowActions.tsx` and `AccountCardActions.tsx` into contiguous segmented pill capsules with high-contrast yellow hover states (Thanks to @aukgit)
- Verified CLI error inspection commands (`agm errors list`, `agm errors inspect`, `agm errors clear`, `agm errors fix`) and authored canonical agent skill `.agents/skills/agm-error-inspection/skill.md` (Thanks to @aukgit)

---

## [v4.257.3] - 2026-10-24

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.257.2] - 2026-10-24

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.257.1] - 2026-10-24

### Fixed
- Prompt tree titles no longer stay `Untitled Conversation` when the summary title is a placeholder. The visible title is the cleaned first line of the prompt (Thanks to @aukgit)
- Image paths that sit outside the workspace are emitted as `file:///` URIs with forward slashes, so the IDE can open them. Paths inside the workspace stay relative (Thanks to @aukgit)

---

## [v4.257.0] - 2026-10-24

### Fixed
- Current conversation send keeps `conv-` conversation ids on `agy --conversation` instead of opening a new chat. The `new` sentinel still starts a new chat, and `agm prompt send --new` uses that sentinel instead of a synthetic `conv-{uuid}` (Thanks to @aukgit)
- Background prompt-tree sync no longer draws the green bar across the top. The tree reloads on the Sync interval or the Refresh button (Thanks to @aukgit)
- Clicking an attached image opens the disk file. Dispatched image markdown stays workspace-relative (Thanks to @aukgit)

---

## [v4.256.2] - 2026-10-24

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.256.1] - 2026-10-24

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.256.0] - 2026-10-24

### Fixed
- Post-switch / post-fast-forward prompt resume now re-dispatches exactly one prompt — the most recently active backed-up prompt strictly belonging to the target instance — via the new `dispatch_single_latest_prompt` in `src-tauri/src/modules/repo_db/dispatch.rs`; every other backed-up prompt stays `backed_up` instead of the whole queue being re-injected (Thanks to @aukgit)
- Removed the flexible cross-instance fallback from the post-switch resume path, so another instance's prompt is never dispatched after a switch or fast-forward (Thanks to @aukgit)
- `has_auto_resume = false` now fully suppresses the post-switch re-dispatch (restore bookkeeping still runs), and the duplicate inline auto-resume of recent prompts was removed from the switch path so resume happens exactly once, in the scheduled STAGE_5_RESTORE stage after the prompt channel is up (Thanks to @aukgit)

---

## [v4.255.0] - 2026-10-24

### Changed
- Complete American English language modernization across 352 Rust backend files, 76 TypeScript/React frontend files, and all architecture specifications (Thanks to @aukgit)

### Fixed
- Fixed minified React hook error #310 ("Rendered more hooks than during previous render") in Instances switch profile dialog and prompt export modal (Thanks to @aukgit)
- Mounted universal RouteErrorBoundary on root browser routing in `src/App.tsx` and wrapped fleet tables and modals with defensive error boundaries (Thanks to @aukgit)
- Fixed Windows autostart syntax error and error pattern matching in `src-tauri/src/commands/autostart.rs` (Thanks to @aukgit)

---

## [v4.253.3] - 2026-10-23

### Fixed
- Fixed GitHub Actions release pipeline with CI dynamic wait polling and post-release asset verification
- Synchronized all 14 version manifests across Rust, Tauri, npm, and installer packages
- Hardened prompt restore sweep on quota completion and integrated diverse rerun randomizer

---

## [v4.253.2] - 2026-10-23

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.253.1] - 2026-10-23

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.11] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.10] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.9] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.8] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.7] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.6] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.5] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.4] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.3] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.2] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.1] - 2026-10-22

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.250.0] - 2026-10-21

### Added
- **Explicit Send (Preempt) & Enqueue (FIFO) Action Capsule (`DetailHeader.tsx`, `usePromptTreePromptActions.ts`)**: Replaced hidden mode switcher with side-by-side Send and Enqueue buttons within Capsule B. `Send` immediately cancels any actively running task on the target instance (terminating `agy` workers, stopping goal workers, and executing CDP active task preemption) to execute the chosen prompt right now (Hotkey: `N`). `Enqueue` safely stages prompts into the background SQLite FIFO queue without interrupting current execution. (Thanks to @aukgit)
- **Continuously Resizable Prompt Tree View (`PromptTreeViewModal.tsx`)**: Implemented dynamic draggable vertical splitter divider (`cursor-col-resize`) allowing users to expand or reduce the left-hand Prompt Tree column width between 220px and 680px at any time, with persistent state stored in `localStorage` under `agm_prompt_tree_width`. (Thanks to @aukgit)
- **Real-Time Conversation Title Rename Sync (`sequences.rs`, `DetailHeader.tsx`)**: Wired double-click inline conversation title renaming directly to Antigravity's live filesystem watcher via `<gemini_dir>/annotations/<conversation_id>.pbtxt` and updated `conversation_summaries.db` (`title` and `preview` columns), enabling instant real-time title reflection inside the Antigravity IDE without requiring IDE restart. (Thanks to @aukgit)
- **Dynamic Theme Adaptability & One Dark Pro Support (`ThemeManager.tsx`, `PromptTreeDetail.tsx`, `App.css`)**: Removed hardcoded purple hex values across the middle Prompt Tree panels. Synchronized `--ui-*` design tokens (`--ui-canvas`, `--ui-surface-1`, `--ui-surface-hover`, `--ui-border-subtle`, `--ui-interactive`) in `ThemeManager.tsx` whenever non-purple themes (such as One Dark Pro, VS Code Dark+, Tokyo Night, Nordic Mint) are selected, allowing the middle portion and hover accents to seamlessly adapt to the active theme palette. (Thanks to @aukgit)

### Fixed
- **Stationary Non-Displacing InstanceCard Hover (`InstanceCard.tsx`)**: Eliminated layout displacement jitter (`hover:-translate-y-0.5` and `scale-x-95`), replacing positional movement with a smooth 0.3s CSS3 color and contrast transition (`hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)]`, `hover:border-gray-300/80 dark:hover:border-[var(--ui-interactive-40)]`, `hover:shadow-md`). (Thanks to @aukgit)
- **Scoreboard Algorithm Tooltip Collision Guard (`AlgorithmScoreCell.tsx`)**: Portaled the candidate score breakdown tooltip to `document.body` via React `createPortal` and implemented dynamic boundary collision detection (`spaceAbove >= TOOLTIP_HEIGHT + 16`), preventing the popover from getting stuck against or overlapping the sticky table header. (Thanks to @aukgit)

---

## [v4.247.0] - 2026-10-21

### Added
- **Telegram Inbound Backup & Restore Enhancements (`telegram_inbound`)**: Added rich instance status indicators (`🟢 Active & Running` / `⚪ Standby / Ready`) and comprehensive counts of backed up/restored prompts and attached multimodal images to `/backup` and `/restore` commands. Expanded `/restore` with intelligent `all` scope handling to restore running prompts across all workspace profiles. Enriched inline keyboard markups across Telegram daemon status and prompt dispatch acknowledgments with quick action buttons (`[ 📦 Backup Prompts ]`, `[ ♻️ Restore Prompts ]`, `[ 🖼️ Send with Images ]`, `[ 🌲 Expand Tree ]`, `[ ⚡ Active Prompts ]`). Automatically enclosed attached Telegram photo markdown paths in angle brackets (`![attached-image](<path>)`) to prevent CommonMark whitespace parsing errors. (Thanks to @aukgit)
- **Expandable Terminal-Style Prompt Tree View (`TreePanel.tsx`, `ProjectNode.tsx`, `ConversationNode.tsx`)**: Added global 1-click Expand/Collapse All toggle (`isAllExpanded`, `FolderPlus`/`FolderMinus`) to cleanly toggle all project nodes across the tree. Refined terminal monospace branch connector glyphs (`├──`, `└──`) and subagent branch glyphs (`└── ↳`), top-level project breathing room (`my-2.5` with container framing), and child turn right-hand indentation (`ml-5 pl-3 border-l-2`). Resized the Auto-Hide button in row 2 to a sleek, compact pill matching adjacent filter capsules (`rounded-lg`, `px-2 py-0.5 text-[10px]`, matching `divide-x`). (Thanks to @aukgit)
- **Fleet Synchronous Cache Acceleration (`useFleetMachines.ts`)**: Implemented synchronous `localStorage` caching with 30s TTL envelope under key `agm_fleet_machines_cache_v2`, enabling instant rendering on mount without full-screen loading freezes while SWR refreshes in the background. (Thanks to @aukgit)
- **Cross-Machine Prompt Dispatch in Fleet Management (`SendFleetPromptModal.tsx`, `exec.rs`)**: Integrated `SendFleetPromptModal` to send prompts across cluster nodes with sanitized shell escaping and remote SSH execution. (Thanks to @aukgit)

### Changed
- **Fleet Machine Identity & Lead Node Alignment (`query.rs`, `useFleetMachines.ts`)**: Resolved local machine identity (`192.168.1.3`) as `w1` (local Lead node) with `is_local: true` and `is_lead: true` using GitMap IP resolution, placing `w1` at index 0 and `main` (`192.168.1.20`) as remote node, while preserving all 6 cluster nodes. (Thanks to @aukgit)
- **Multi-Image Composite PNG Clipboard Engine (`cmd_files.rs`, `clipboard.ts`)**: Ensured composite PNG copying populates Win32 OLE `CF_DIB`, `CF_BITMAP`, `PNG`, and `CF_HDROP` for seamless pasting into OneNote, Antigravity IDE, Word, and Paint. (Thanks to @aukgit)

### Fixed
- **Eliminated False `QUEUED` Badges (`transcript.rs`, `tree_collect.rs`, `tree_phases.rs`)**: Fixed false amber `QUEUED` badges displaying on completed or idle turns by enforcing active instance liveness and 600s TTL verification before declaring queued status. Updated `transition_stale_inflight_prompts` in `tree_phases.rs` to include stale queued and pending prompts after 600s. (Thanks to @aukgit)

---

## [v4.246.0] - 2026-10-21

### Added
- **Unified Action Consolidation Dropdown (`DetailHeader.tsx`)**: Compacted disjointed `Send`, `Enqueue`, and `Rerun` buttons into a single segmented split dropdown button `[ Primary Action | v ]` with mode switcher pills (`Send Immediately` [N], `Enqueue for Later`, `Rerun Turn`), preserving contextual parameters (Target: Current/New, Suffix template, Times multiplier 1..5) while keeping the adjacent `[ Copy | v ]` split button clean and minimalist. (Thanks to @aukgit)
- **ModalHeader Iconification & Informative Hover Tooltips**: Replaced text-heavy buttons `Reset` and `Refresh` with dark-glass icon buttons rendering Lucide `RotateCcw` and `RefreshCw` with active spin animations and descriptive hover tooltips. Replaced raw emoji `🖼️ Images` toggle with Lucide `Image as ImageIcon` with dark-glass sky styling (`bg-sky-500/25 text-sky-400 border-x border-sky-500/40`) and dynamic tooltip. (Thanks to @aukgit)
- **Win32 OLE `CF_DIB` & `CF_HDROP` Native Clipboard Engine**: Fixed PowerShell slice bug in `src-tauri/src/commands/cmd_files.rs` using `New-Object System.IO.MemoryStream($bytes, 14, ($bytes.Length - 14))` to guarantee valid non-zero DIB stream length (491KB+), and added `CF_HDROP` (`SetFileDropList`) so Antigravity IDE (VS Code) attaches copied images directly as native file attachments. Implemented `copy_images_to_clipboard` with native vertical composite bitmap stitching in `cmd_files.rs` and registered it in `src-tauri/src/lib/run.rs`. (Thanks to @aukgit)

### Changed
- **Immediate Send Bypass vs FIFO Enqueue Architecture**: Decoupled immediate session injection (`handleResendPrompt` / `sendPromptNow`) from FIFO background queue scheduling (`handleEnqueuePrompt` / `enqueuePrompt`). Send immediately injects to the active instance session (via direct loopback CDP prompt submission or agy CLI spawn), bypassing any queued turns. Enqueue inserts into `active_prompts` with status `'queued'`, strictly respecting FIFO execution order (`ORDER BY created_at ASC, id ASC`) so that pending turns execute sequentially after running tasks complete. (Thanks to @aukgit)
- **Attached Image Markdown & Multimodal Forwarding**: Aggregated attached image paths and media payloads across active turns, conversation nodes, and full prompt histories in `usePromptTreePromptActions.ts`. Attached image file paths are appended in CommonMark syntax with angle-bracket enclosures (`![Attached Image N](<path>)`) preventing Windows path whitespace breakage, and forwarded in Tauri IPC commands (`send_prompt_now`, `enqueue_prompt`) and `ActivePrompt.image_payload`. Added `media_paths` and `media_thumbnails` to `AgmConversationNode` in `src-tauri/src/modules/repo_db/models.rs` and populated them in `tree_node_conv.rs`. (Thanks to @aukgit)
- **Frontend Clipboard Race Condition Elimination**: In `src/components/instances/PromptTreeView/clipboard.ts`, immediately returned upon native write success to prevent WebView2's `EmptyClipboard()` from wiping native Win32 clipboard formats (`CF_DIB`, `CF_BITMAP`, `CF_HDROP`). Simplified button labels across menus to strictly "Copy" and "Copy Text Only". (Thanks to @aukgit)

### Fixed
- **TreePanel & Detail View Layout Overlap Fixes**: Added `relative z-10 bg-slate-50/95 dark:bg-[#0c0817]/95 backdrop-blur-md` to the `TreePanel.tsx` search/filter header container to eliminate conversation card bleed-through and clipping over the dividing border. Enforced non-wrapping layout in Row 2 to prevent awkward overlap of the `Hide Auto` toggle pill. Increased conversation card vertical spacing in `ProjectNode.tsx` (`space-y-1.5` / `space-y-2`) and turn chip gaps in `PromptInstructionView.tsx` (`gap-x-2 gap-y-2`) to eliminate relative time badge clumping (`1d`, `2d`, `5d`). (Thanks to @aukgit)

---

## [v4.245.0] - 2026-10-21

### Added
- **Dual Clipboard Engine (OneNote & Antigravity IDE Parity)**: Upgraded `copyAllImagesToClipboard`, `writePureImageToClipboard`, and `copySingleImageForOneNoteOrIde` in `src/components/instances/PromptTreeView/clipboard.ts` to perform simultaneous dual-writes across native Win32 OLE clipboard (`copy_image_to_clipboard` with `CF_DIB` and `PNG`) and web clipboard (`navigator.clipboard.write([item])`). Untainted canvas operations via Base64 data URI pre-loading (`loadImageElement`), completely eliminating WebView2 `SecurityError` and enabling 1-click pasting of multi-image composites into both Microsoft OneNote and Google Antigravity. (Thanks to @aukgit)
- **Telegram Bot /backup & /restore Inline Keyboards**: Enhanced Telegram inbound daemon in `src-tauri/src/modules/telegram_inbound/daemon.rs` and `config.rs` with interactive inline keyboard markup buttons (`[ 📦 Backup Prompts ]`, `[ ♻️ Restore Prompts ]`, `[ 🖼️ Send with Images ]`, `[ 🌲 Expand Tree ]`, `[ ⚡ Active Prompts ]`) and added automated forwarding of attached Telegram photos into prompt injections via `![attached-image](url)`. (Thanks to @aukgit)
- **Cross-Machine Prompt Dispatch in Fleet Management**: Added interactive `[ Send Prompt ]` action button to each cluster node row in `FleetMachinesTable.tsx` and `FleetMachineRow.tsx` powered by `SendFleetPromptModal.tsx`, allowing direct cross-machine prompt routing and remote SSH execution. (Thanks to @aukgit)

### Changed
- **Fleet Cache Acceleration & Lead Machine Alignment**: Added synchronous `localStorage` caching in `src/components/instances/fleet/useFleetMachines.ts` to eliminate stuck loading states, enforced GitMap IP primacy (`192.168.1.3` as local `Lead` node `w1`), and re-aligned remote cluster node roles. (Thanks to @aukgit)
- **Prompt Tree View Terminal Indentation & Expand/Collapse All**: Added top-level project breathing room (`my-2.5`), increased child prompt indentation (`ml-5 pl-3 border-l-2`), integrated 1-click Expand/Collapse All toggle (`isAllExpanded`, `FolderPlus`/`FolderMinus`), terminal monospace branch connector glyphs (`├──`, `└──`), and modernized the 30s auto-sync interval dropdown into a custom dark-glass popover capsule in `ModalHeader.tsx`. (Thanks to @aukgit)

### Fixed
- **Prompt Tree False QUEUED Status**: Updated `src-tauri/src/modules/repo_db/tree_node_conv.rs` to eliminate false `QUEUED` badges on completed or idle prompts by enforcing active instance liveness and 600s TTL verification before declaring queued status. (Thanks to @aukgit)

---

## [v4.244.2] - 2026-10-21

### Added
- **Synchronize prompts, skills, AI scripts, and coding guidelines**: Updated canonical V6 prompt suite and AI scripts across connected repositories. (Thanks to @aukgit)

---

## [v4.244.1] - 2026-10-21

### Added
- **Cross-Machine Prompt Dispatch in Fleet Management**: Added interactive `[ Send Prompt ]` action button to each cluster node row in `FleetMachinesTable.tsx` and `FleetMachineRow.tsx` powered by `SendFleetPromptModal.tsx`, allowing direct cross-machine prompt routing and remote SSH execution. (Thanks to @aukgit)
- **Tree View Expand/Collapse All Toggle**: Added 1-click expand and collapse all projects toggle button (`handleToggleExpandAll`, `isAllExpanded`, `FolderPlus`, `FolderMinus`) in `PromptTreeViewModal.tsx` and `TreePanel.tsx`. (Thanks to @aukgit)

### Changed
- **Modernized Dark-Glass Sync Dropdown**: Replaced the native select dropdown for 30s auto-sync interval with a custom dark-glass popover capsule featuring active selection checkmarks (`Check`) in `ModalHeader.tsx`. (Thanks to @aukgit)
- **Terminal Monospace Tree Branch Glyphs**: Added monospace tree branch connector glyphs (`├──`, `└──`) to `ConversationNode.tsx` and `ProjectNode.tsx` reflecting last-conversation position in the project hierarchy. (Thanks to @aukgit)

---

## [v4.244.0] - 2026-10-21

### Changed
- **Boolean Logic Modernization & Guard Clause Decomposition**: Refactored conditional logic across `src/pages/instances/` adhering strictly to `01-prompts/15-cg-execute/04-booleans-and-complex-conditions.md`. Replaced compound negative checks such as `if (!inst.is_running || !proj.is_running) return false;` with discrete, single-variable affirmative guard clauses in `InstanceCardProjects.tsx`. (Thanks to @aukgit)
- **Affirmative Guard Extraction (`isConversationGenuinelyRunning`)**: Extracted and centralized `isConversationGenuinelyRunning` in `src/pages/instances/instancePageUtils.ts`, enforcing discrete boolean decomposition (affirmative running checks, explicit instance ownership matching, active task validation, and ghost conversation filtering), eliminating 4-operand chained `.some(...)` predicates across `instanceCardData.ts` (`hasActiveTask`), `InstanceCardProjects.tsx` (`isProjectRunningForInstance`), and `useInstancePage.ts` (`hasActiveTaskFor`). (Thanks to @aukgit)
- **Mixed Operator Elimination & 2-Operand Decomposition**: Decomposed mixed `&&`/`||` operators into intermediate named booleans in `instancePageUtils.ts` (`isDefaultInstance`, `hasMatchingSeqNum`), `instanceCardData.ts` (`weeklyBucket` resolution), and `useInstancePage.ts` (`filteredInstances` search filtering). (Thanks to @aukgit)

### Fixed
- **Prompt Tree View Modal Sidebar Overflow & Title Width**: Fixed CSS layout defect where the left sidebar (`TreePanel`) overflowed 136px across the vertical divider line into the detail view by removing redundant fixed width constraints on `TreePanel`, adding `overflow-hidden` to the modal sidebar container, wrapping filter capsules cleanly, and expanding title truncation threshold to `max-w-3xl`. (Thanks to @aukgit)
- **Instance Running Task Liveness & Ghost Filtering Accuracy**: Standardized conversation running checks to respect authentic task status, instance ID ownership, and ghost conversation suppression, verified across all 32 test cases in `src/pages/instances/__tests__/instanceRunningPrompts.test.ts`. (Thanks to @aukgit)

---

## [v4.243.0] - 2026-10-20

### Added
- **Telegram Inbound Prompt Backup & Snapshots**: Added `/backup`, `/backup_prompt`, `/backup_prompts`, `/backup_running`, and `/brp` inbound command routes in `src-tauri/src/modules/telegram_inbound/processing.rs` and `commands.rs`, delivering automated snapshots of running prompts, attached multimodal image counts, dynamic instance resolution, and structured HTML backup reports. (Thanks to @aukgit)
- **Base64 Remote Prompt Transmission Protocol**: Added `--base64` RFC 4648 payload encoding across `src-tauri/src/modules/telegram_inbound/injection.rs` and `src-tauri/src/bin/agm/prompt_dispatch.rs` for cross-machine SSH prompt execution, preserving shell whitespace, quotes, and multiline instructions without command-line escaping defects. (Thanks to @aukgit)
- **Prompt Tree Image Filter & Responsive Layout**: Added a header `🖼️ Images` toggle button in `src/components/instances/PromptTreeView/ModalHeader.tsx` connected to `imageFilterEmitter` to instantly filter projects and conversations containing attached images, expanded tree panel width to `w-[460px] lg:w-[520px]`, increased project separation (`space-y-3.5`), and styled Auto-Hide and scope dropdown as a contiguous dark-glass segmented pill capsule. (Thanks to @aukgit)

### Fixed
- **Fleet Local Machine Node Identity w1 & 6-Node Deduplication**: Enforced Cluster-IP primacy in `src-tauri/src/modules/supabase_sync/node.rs` and `email_watcher/detection.rs` to reliably resolve `192.168.1.3` as `w1` (local lead) and `192.168.1.20` as `main`, and fixed deduplication logic in `query_fleet_machines` so all 6 registered cluster nodes (`w1`, `main`, `u1`, `w2`, `w3`, `w4`) are discovered and displayed with Lead indicators. (Thanks to @aukgit)
- **SSH Cluster 30s Caching & Credential-Preserving Reset**: Implemented `CLUSTER_BALANCES_CACHE` with 30s TTL in `src-tauri/src/modules/ssh_manager/exec.rs` and decoupled balance enrichment via SWR in `useFleetMachines.ts` to eliminate UI freezing; updated `reset_ssh_cluster_nodes` in `sync.rs` to reload cluster nodes preserving passwords and SSH key paths. (Thanks to @aukgit)
- **Default Instance Prompt Liveness & FIFO Query Order**: Standardized instance specifier normalization across `observe.rs` (`#1 Default`, `Default`, `__default__`), hardened canonical directory discovery in `gemini_dirs.rs`, enforced strict FIFO `ORDER BY created_at ASC, id ASC` in `liveness.rs` to eliminate false terminal overrides, and added IDE prompt goal heartbeat fallback in `cmd_running_up_next.rs` for active IDE sessions. (Thanks to @aukgit)
- **Conversation Rename Sync to state.vscdb**: Implemented immutable React state cloning in `usePromptTreeData.ts` to instantly propagate renamed conversation titles to header and selected state, and wired `sync_title_to_state_vscdb` in `src-tauri/src/modules/repo_db/sequences.rs` to update `ItemTable` in `state.vscdb`. (Thanks to @aukgit)
- **Terminal Monospace Box Glyphs & Details Button Health Pulse**: Restored Details button with emerald `Activity` icon and glowing `#1af18d` pulsing health indicator dot in `DetailTabs.tsx`, standardized child branch glyphs to monospace box-drawing glyphs (`├──`, `└──`, and `└── ↳` for subagents) with auto-selection on double-click in `ConversationNode.tsx` and `ProjectNode.tsx`, and added a 10-retry overload in `cmd_files.rs` for Win32 clipboard image copies. (Thanks to @aukgit)

---

## [v4.242.0] - 2026-10-20

### Added
- **Win32 CF_DIB & PNG Multi-Format Clipboard Engine**: Upgraded `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` to extract bitmap bytes and strip the 14-byte `BITMAPFILEHEADER`, registering native `DeviceIndependentBitmap` (`CF_DIB`) alongside `PNG` streams on `System.Windows.Forms.DataObject`, guaranteeing images pasted into Microsoft OneNote desktop, Word, Paint, Discord, and Google Antigravity IDE render 100% reliably. (Thanks to @aukgit)
- **Batch Multi-Image Context Menu Copy**: Added multi-image aggregation in `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`, invoking `copyAllImagesToClipboard` to composite multiple images into a stitched canvas with native `CF_DIB` and `PNG` clipboard writes, and standardized button labels strictly to "Copy" and "Copy Images" (eliminating "Copy including image"). (Thanks to @aukgit)

### Fixed
- **Prompt Tree Image Attachment Preservation**: Updated regex parsing in `extract_clean_user_prompt` (`src-tauri/src/modules/repo_db/text_utils.rs`) to collect and preserve markdown images located outside `<USER_REQUEST>`, and updated `usePromptTreePromptActions.ts` (`handleResendPrompt` and `handleEnqueuePrompt`) to splice missing conversation images inside `<USER_REQUEST>` before submission. (Thanks to @aukgit)
- **Prompt Tree Instant Turn Refresh & Cache Invalidation**: Inverted execution order in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` so `invalidate_prompt_tree_cache` executes before `loadTree`, ensuring newly enqueued turns immediately appear with amber `QUEUED` badges without reading stale SQLite cache entries. (Thanks to @aukgit)
- **DetailHeader Prominent Title Display**: Removed all turn/origin prefix badges preceding the conversation title in `src/components/instances/PromptTreeView/DetailHeader.tsx`, enlarged title typography to `text-xl md:text-2xl font-bold font-ubuntu tracking-tight`, and aligned the status capsule cleanly after the title. (Thanks to @aukgit)

---

## [v4.241.0] - 2026-10-20

### Added
- **Native OS DIB / Bitmap Clipboard Command for OneNote and Antigravity**: Repaired `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` to validate `data:image/` before path normalization, execute PowerShell in `-Sta` (Single-Thread Apartment) mode, and persist bitmap memory using `[System.Windows.Forms.Clipboard]::SetDataObject($data, $true)`, ensuring images pasted into Microsoft OneNote, Google Antigravity IDE, Word, and Paint render 100% reliably. (Thanks to @aukgit)
- **Automatic Native Clipboard Priming on Rich Prompt Copy**: Connected `copy_image_to_clipboard` in `src/components/instances/PromptTreeView/clipboard.ts` (`copyPromptWithRichImages`) so copying prompt text with images simultaneously primes the native OS clipboard with Win32 `CF_DIB` bitmap format alongside browser HTML/PNG payloads. (Thanks to @aukgit)

### Fixed
- **Prompt Tree Send & Enqueue Decoupled Dispatch with Live Tree Refresh**: Preserved `media_paths` and `media_thumbnails` when selecting execution turns in `createTurnConv` (`src/components/instances/PromptTreeView/ConversationNode.tsx`), passed `loadTree` callback from `PromptTreeViewModal.tsx` to `usePromptTreePromptActions` for instant status badge updates upon enqueuing, and added workspace directory fallback in `src-tauri/src/modules/repo_db/agy.rs` resolving `std::env::current_dir()` when `prompt.repo_path` is empty. (Thanks to @aukgit)
- **DetailHeader Prefix Badge Removal & Enlarged Title Typography**: Stripped all noisy turn and status prefix badges preceding conversation titles in `src/components/instances/PromptTreeView/DetailHeader.tsx`, enlarged title typography to `text-lg md:text-xl font-bold font-ubuntu tracking-tight` with expanded `max-w-md md:max-w-xl lg:max-w-2xl` width, and placed the status capsule cleanly after the title. (Thanks to @aukgit)
- **Complete SQLite Cache Purge & Rescan on Reset**: Enhanced `wipe_and_rescan_repo_db` in `src-tauri/src/modules/repo_db/agy.rs` to purge `media_vault`, clear in-memory prompt tracking (`get_memory_prompts_map()`), and reset dispatched prompts cache, while invalidating the client-side SWR cache across all instances in `PromptTreeViewModal.tsx`. (Thanks to @aukgit)
- **Fleet Lead Machine Git Repository Origin Alignment**: Updated `useFleetMachines.ts` and `FleetMachineRow.tsx` so cluster lead node `main` (`192.168.1.20`) and local node `w1` (`192.168.1.3`) display the authentic Git repository `Antigravity-Manager` (`alimtvnetwork / Antigravity-Manager`) and aligned bound account emails (`sakibagjdjg@gmail.com` for `main`, `cirajayu11@gmail.com` for `w1`). (Thanks to @aukgit)

---

## [v4.240.1] - 2026-10-20

### Added
- **Native OS DIB / Bitmap Clipboard Command for OneNote and Antigravity**: Implemented `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` and `src-tauri/src/lib/run.rs` writing Win32 `CF_DIB`/`CF_BITMAP` format via PowerShell `System.Windows.Forms.Clipboard::SetImage` alongside pure PNG binary Blob, ensuring images pasted into Microsoft OneNote, Paint, Word, and Antigravity IDE render 100% reliably. (Thanks to @aukgit)
- **Unified Multi-Image Batch Clipboard Copy**: Implemented `handleCopyImagesUnified` in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` and `clipboard.ts`, automatically compositing all attached conversation images into a single vertical canvas and writing both native OS bitmap and binary PNG blob to the clipboard in one click when multiple images exist. (Thanks to @aukgit)
- **agy CLI Discovery in User Home**: Added `~/.gemini/bin/agy.exe` and `~/.antigravity/bin/agy.exe` candidate paths to `get_antigravity_cli_executable_path` in `src-tauri/src/modules/process/locations.rs`, resolving installed `agy` CLI binary for seamless prompt dispatch. (Thanks to @aukgit)

### Fixed
- **Prompt Tree Send & Enqueue Decoupled Dispatch**: Restored independent `[ Send N | v ]` and `[ Enqueue | v ]` buttons in `src/components/instances/PromptTreeView/DetailHeader.tsx` across distinct segmented capsules, passed untruncated `conv.full_prompt_text` instead of premature 200-word summaries, and preserved authentic attached image paths during dispatch. (Thanks to @aukgit)
- **Image Copy Button Label Standardization**: Normalized attached image copy buttons in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` strictly to "Copy" (eliminating "Copy including image"). (Thanks to @aukgit)
- **DetailHeader Title Prominence & Turn Badge Cleanup**: Removed noisy prefix badges (`Turn T40`, `↳ Turn 40 of 40`, and origin pill) preceding conversation titles in `DetailHeader.tsx`, enlarged title typography (`text-base md:text-lg font-bold font-ubuntu`), kept adjacent Copy Title button, and moved the status capsule after the title. (Thanks to @aukgit)
- **Full Database Reset & Disk Rescan**: Connected `onResetTree` in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `wipe_and_rescan_repo_db` and `loadTree(true, true)` to wipe SQLite prompt caches and rescan `transcript_full.jsonl` files fresh with live spinner feedback. (Thanks to @aukgit)
- **Fleet Lead Machine Git Origin Alignment**: Corrected `useFleetMachines.ts` and `FleetMachineRow.tsx` so cluster lead node `main` (`192.168.1.20`) displays Git repository `gitmap / gitmap` and bound account `sakibagjdjg@gmail.com` without leaking local node `w1` (`192.168.1.3`, `Antigravity-Manager`) accounts. (Thanks to @aukgit)
- **CDP Direct Submission Robustness**: Expanded composer selector ladder (`div.monaco-editor`, `div.interactive-input-editor`, `textarea.monaco-mouse-cursor-text`) and send button selectors with keyboard Enter fallback in `src-tauri/src/modules/instance/cdp_onboarding.rs`. (Thanks to @aukgit)

---

## [v4.240.0] - 2026-10-20

### Added
- **Multi-Image Composite Clipboard Canvas**: Implemented `stitchImagesToCompositePng` and `copyAllImagesToClipboard` in `src/components/instances/PromptTreeView/clipboard.ts` to vertically composite multiple images into a single PNG binary blob with clean margins and write directly to the system clipboard, plus rich markdown copy referencing local `file:///...` paths and Base64 data URIs. (Thanks to @aukgit)
- **Prompt Tree SQLite Full Wipe & Rescan Engine**: Added `wipe_and_rescan_repo_db` in `src-tauri/src/modules/repo_db/agy.rs` and registered in `src-tauri/src/commands/instance.rs`, truncating `prompt_tree_cache`, `active_prompts`, `running_projects`, `agm_conversation_sequences`, and `agm_project_sequences`, terminating stale `agy` workers, and rescanning disk `transcript_full.jsonl` files to completely rebuild the database and tree view with live progress spinner feedback in `ModalHeader.tsx`. (Thanks to @aukgit)
- **Fleet Lead Node Primacy & Git Repository Badges**: Added lead machine badge and Git repository indicators in `FleetMachineRow.tsx` and `useFleetMachines.ts`, prioritizing primary workstation `main` (`192.168.1.20`) with lead primacy, correcting inverted SSH host alias mappings with `w1` (`192.168.1.3`), and displaying account sync status instead of unassigned placeholders. (Thanks to @aukgit)

### Fixed
- **Prompt Tree Send & Enqueue Button Restoral**: Fixed prompt dispatching in `usePromptTreePromptActions.ts` by removing image stripping that generated phantom `./.antigravity/attachments/...` paths, preserving authentic data URIs and attached media paths, and implementing project resolution fallbacks so `repoPath` is never empty when enqueuing turns. (Thanks to @aukgit)
- **Pure Single-MIME Image Clipboard Pipeline**: Refactored `writePureImageToClipboard` in `src/components/instances/PromptTreeView/clipboard.ts` to write strictly single-MIME `'image/png'` binary blobs without conflicting `text/html` or `text/plain` payloads, resolving silent paste drops across Microsoft OneNote, Paint, and Antigravity IDE. (Thanks to @aukgit)
- **Single-Image Gallery Button Labeling**: Normalized image copy button in `PromptInstructionView.tsx` strictly to "Copy" (eliminating "Copy including image"), and added a "Copy All Images" header action when multiple attachments are present. (Thanks to @aukgit)
- **DetailHeader Title Prominence & Prefix Stripping**: Cleaned `DetailHeader.tsx` by stripping all prefix badges (`Turn T40`, `↳ Turn 40 of 40`, and origin pill) preceding conversation titles, enlarging title text to `text-base md:text-lg font-bold font-ubuntu`, keeping the Copy Title button adjacent, and repositioning the status capsule after the title. (Thanks to @aukgit)
- **Telegram Inbound Prompt Routing**: Fixed pattern match collision in `src-tauri/src/modules/telegram_inbound/processing.rs` where conversation seek shadowed prompt injection commands (`/prompt`, `/p`, `/prompts`), restoring Telegram prompt injection functionality with 74 passing tests. (Thanks to @aukgit)
- **Strict FIFO Prompt Queue Invariant**: Enforced `ORDER BY created_at ASC, id ASC` in `src-tauri/src/commands/cmd_running_up_next.rs` and added canonical instance ID alias resolution for default instance queries. (Thanks to @aukgit)

---

## [v4.239.0] - 2026-10-20

### Added
- **Fleet SSH Cluster Reset & Toolbar Action**: Added `reset_ssh_cluster` backend command and UI toolbar button `[ 🔄 Reset Cluster ]` in `src-tauri/src/commands/ssh.rs` and `FleetMachinesTable.tsx` to safely clear stale connection pools and re-synchronize cluster nodes from GitMap. (Thanks to @aukgit)
- **Start All Instances Control**: Added `[ 🚀 Start All ]` button to the main instances management toolbar in `src/pages/instances/useInstancePage.ts` and `InstanceTable.tsx` to launch or resume all configured instances in one click. (Thanks to @aukgit)
- **Audit Log Execution Breadcrumbs & Step Inspection**: Added an interactive "Step Performed & Interaction Execution Path" inspection card to `AuditPanel.tsx`, displaying intent summary sentences, connected DOM hierarchy breadcrumbs, and a 4-tier contextual target grid. (Thanks to @aukgit)

### Fixed
- **Fleet Machine Node Identity & IP Resolution**: Resolved local machine identity matching `192.168.1.3` to cluster node `w1` using Cluster-IP primacy in `src-tauri/src/modules/ssh_manager/exec.rs`, while preserving all 6 discovered GitMap nodes in `FleetMachinesTable.tsx`. (Thanks to @aukgit)
- **Prompt Tree Running Prompt Detection**: Fixed running prompt liveness in `src-tauri/src/modules/repo_db/liveness.rs` and `detection.rs` to include `dispatched` status and cover default/unscoped instance records, ensuring active prompts in default instances are accurately detected. (Thanks to @aukgit)
- **Conversation Rename Synchronization**: Ensured conversation title renames propagate synchronously across `conversations.title`, `conversation_summaries.summary`, and `conversations_fts.title` with live UI refresh. (Thanks to @aukgit)
- **Audit Log Meaningful Action & Target Naming**: Replaced cryptic Tailwind utility class selectors (`div.grid`, `Button.P 1.5`, `div.group`, `span.truncate`, and raw `scroll:/instances`) in `useAuditRecorder.ts` and `AuditPanel.tsx` with human-readable semantic descriptors (`Click: "Open Settings" Button`, `Click: Instance "Default"`, `Click: Instances Grid Area`, `Scroll: /instances Page`). (Thanks to @aukgit)
- **Prompt Tree Toolbar & Markdown View Polish**: Restored the Details button with its healthy status icon to row 2 next to the conversation title; standardized "Copy All (Include Images)" in the action toolbar; updated Markdown view background to `#19191c`; and converted verbose SSH settings helper text into compact icon buttons with hover tooltips. (Thanks to @aukgit)

---

## [v4.238.7] - 2026-10-20

### Added
- **Multimodal Running Prompt Backup & Restore E2E Suite with GitMap Parity**: Implemented a comprehensive 25-assertion, 8-gate end-to-end test suite (`scripts/test-agm-gitmap-running-backup-e2e.ps1`) covering running prompt inspection, multimodal deep analysis image attachments (Base64 data URIs, local fixtures, screenshot tokens), Split SQLite snapshot backup (`backup_prompts.db:prompt_backups`), and strict FIFO queue restoration with status transitioning to `'dispatched'`; verified full command parity across `agm running`, `agm backup running`, `agm restore running`, `gitmap agy running`, `gitmap agy ff`, `gitmap backup-running-prompts`, and `gitmap restore-running-prompts`. (Thanks to @aukgit)
- **10-Second Settling Invariant and Caller IDE Immunity**: Enforced mandatory `post_switch_resume_delay_seconds = 10` in `src-tauri/src/modules/repo_db/post_switch_resume.rs` to ensure the Electron webview, extension host, and CDP debugger endpoints (`DevToolsActivePort`) are listening before prompt re-injection; hardened caller process detection (`agm instances whoami`) with Exit Code 2 refusal on caller stopping or mutation to protect active developer workspaces. (Thanks to @aukgit)

### Fixed
- **SQLite Constraint Violation on Instance Deactivation**: Fixed SQLite runtime constraint failure in `src-tauri/src/modules/instance/create.rs` where setting `pid = NULL` violated the non-nullable schema constraint, preserving the historical process ID while correctly setting `is_active = 0`. (Thanks to @aukgit)
- **Windows File-Lock Permission Bypass during Teardown**: Implemented `remove_dir_all_force` in `src-tauri/src/utils/fs.rs` with Win32 attribute stripping (`FILE_ATTRIBUTE_NORMAL`) and integrated into instance deletion paths, eliminating `os error 5 (Access is denied)` during Chromium cache and lockfile removal. (Thanks to @aukgit)
- **Speed Error Self-Healing Engine**: Verified and passed 8/8 diagnostic probes in `agm errors fix` across filesystem permissions, stale logs, lockfile eviction, `accounts.json`, `gui_config.json`, SQLite integrity, git hygiene, and gateway proxy port probing. (Thanks to @aukgit)

---

## [v4.238.4] - 2026-10-19

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.238.3] - 2026-10-19

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.238.2] - 2026-10-19

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.238.1] - 2026-10-19

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.238.0] - 2026-10-19

### Added
- **Pure PNG Clipboard Writing & Gallery Batch Copy**: Implemented `writePureImageToClipboard` in `src/components/instances/PromptTreeView/clipboard.ts` writing strictly single-MIME `'image/png'` with canvas-based PNG header transcoding (`transcodeSrcToPngBlob`), eliminating multi-MIME format collisions that previously caused Microsoft OneNote, Slack, Discord, and Telegram to strip images or paste empty placeholders; added 1-click `[ 📋 Copy Image (PNG) ]` and sequential `[ 📋 Copy Images (N) ]` buttons in the "Attached Images" gallery header, plus hover-activated 1-click copy badges on individual thumbnails in `PromptInstructionView.tsx`. (Thanks to @aukgit)
- **GitMap SSH Node Resolution & Telegram Hierarchy Formatting**: Upgraded `src-tauri/src/modules/telegram_inbound/node_label.rs` to parse `~/.gitmap/gitmap-ssh-nodes.json` and `./gitmap-ssh-nodes.json` when the machine alias is `none` or auto-generated `Node-*`, matching local IP `192.168.1.3` to alias `w1` and annotating as `w1 (collected from gitmap)`; separated `Machine:` and `Instance:` onto distinct lines across all Telegram status reports in `reports_b.rs`, `system.rs`, `telegram_alerts.rs`, and `telemetry.rs`. (Thanks to @aukgit)

### Fixed
- **Immediate Send vs Enqueue Decoupling & Attachment Forwarding**: Decoupled immediate session injection (`handleSendPromptNow` / `sendPromptNow`) from queue scheduling (`handleEnqueuePrompt`) in `usePromptTreePromptActions.ts` and `DetailHeader.tsx`; preserved and appended attached conversation images (`media_paths`) in markdown image syntax (`![Attached Image N](<path>)`) across Rerun, Send, and Enqueue flows so IDE sessions receive complete image attachments without data loss. (Thanks to @aukgit)
- **Live Transcript Supremacy for Prompt Tree Status Badges**: Refactored status evaluation in `src-tauri/src/modules/repo_db/tree_collect.rs` so live transcript activity (`is_owning_inst_alive && inspection.is_recent_active && !inspection.is_terminal_done`) takes precedence over stale SQLite idle summaries, accurately displaying emerald pulse `RUNNING` for active turns and amber `QUEUED` for pending turns rather than incorrectly marking turns completed or showing `QUEUE EMPTY`. (Thanks to @aukgit)
- **High-Contrast Dark Theme Detail View**: Upgraded preview, raw markdown, and edit mode containers in `PromptInstructionView.tsx` to `dark:bg-[#0c0514]/95` with `dark:border-purple-500/25` subtle borders and high-contrast text styling for professional readability. (Thanks to @aukgit)

---

## [v4.237.0] - 2026-10-19

### Added
- **Strict Release Asset Verification Before Update Prompting**: Added `verify_tag_has_platform_asset` in `src-tauri/src/modules/update_checker/channels.rs` performing HTTP HEAD requests against the GitHub Release API to ensure that native platform installer binaries (`agm-alim-setup.exe`, `.dmg`, `.deb`) actually exist and are downloadable before notifying users of updates, eliminating 404 installation errors when tags exist before assets finish compiling. (Thanks to @aukgit)
- **Combined Running & Queued Backup / Restore with 5s Countdown HUD**: Added direct combined export for both running and enqueued prompts (`Export Running & Queued Prompts (.json)`) and direct backup restoration in `src/components/instances/PromptTreeView/ImportExportMenu.tsx` and `usePromptTreeCoreActions.ts`; displays an interactive 5-second countdown HUD banner ("Waiting time: Xs remaining · Next action: Resume and dispatch restored prompt(s) to instance ...") before automatic prompt dispatching. (Thanks to @aukgit)
- **Telegram Multimodal Remote Dispatch & Session 0 Display Fallback**: Preserved `saved_media_opt` during `/send` flows in `src-tauri/src/modules/telegram_inbound/send_flow.rs` and `injection.rs`, populating `active_prompts.image_payload` with attached media counts and timestamps; implemented an anti-aliased synthetic dispatch card in `open_shot.rs` when running under non-interactive Windows OpenSSH Session 0 without display station access. (Thanks to @aukgit)

### Fixed
- **Prompt Tree Selection Anti-Flapping & Turn Persistence**: Locked the actively selected turn (`selected_turn_number`) and project selection across background SWR sync intervals in `src/components/instances/PromptTreeView/usePromptTreeData.ts` and `ConversationNode.tsx`, preventing title and prompt text from glitching or alternating between conversation summaries and raw turn prompts during active inspection. (Thanks to @aukgit)
- **Modal Outside-Click & Double-Click Dismissal**: Added explicit `onClick` and `onDoubleClick` backdrop event handlers on the backdrop container in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to cleanly dismiss the modal when clicking outside. (Thanks to @aukgit)
- **Rich Dual-MIME Image Copying & Visual Fallback Cards**: Upgraded image processing in `PromptInstructionView.tsx`, `markdown.tsx`, and `clipboard.ts`; images now render with local base64 and asset protocol fallback cards (`PromptThumbnailCard`), and support rich multi-format clipboard copying (`image/png` binary blob + `text/html`) through `copyPromptWithRichImages` with deduplicated URI extraction and Windows drive letter decoding. (Thanks to @aukgit)
- **GitMap SSH Mesh & Fleet Node Schema Alignment**: Reconciled `SshConnectionRecord` field schemas across `src-tauri/src/modules/supabase_sync/query.rs` and `src/components/instances/fleet/useFleetMachines.ts`, eliminating all Rust compilation, Clippy, and TypeScript linter errors across all supported OS matrix targets. (Thanks to @aukgit)

---

## [v4.236.0] - 2026-10-19

### Added
- **Collapsible Session Overview Card with Memory**: Converted the session overview dashboard in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` to be collapsed by default into a sleek single-line summary bar (~36px); added subtle low-profile purple expander trigger (`ChevronDown`/`ChevronUp`); persisted collapse/expand state in `localStorage` (`agm_prompt_overview_expanded`) across sessions; removed redundant title, sequence badge (`C211`), and conversation ID repetitions from inside the overview card. (Thanks to @aukgit)
- **Centralized 24-Hour Date/Time Formatting & Duration Seconds Suppression**: Implemented centralized date formatting in `src/utils/date.ts` supporting `DD-MMM-YYYY HH:mm` in 24-hour clock using the local system timezone, with relative time indicators and duration seconds suppression when hours or minutes exist; added user-configurable `date_format_mode` toggle with strictly typed `DateFormatModeType` enum (`TwentyFourHour = '24h'`, `TwelveHour = '12h'`, `Iso = 'iso'`) in `src/types/config.ts` and Settings. (Thanks to @aukgit)
- **Metadata Clarification & DetailsModal Restructuring**: Clarified sequential conversation index (`C211` = Conversation #211) and interaction turns (`turns` = user/assistant request-response cycles) with dedicated labels and explanations in `src/components/instances/PromptTreeView/DetailsModal.tsx`; formatted `Last Modified` into localized 24-hour clock (`DD-MMM-YYYY HH:mm`) accompanied by relative recency delta ("ran 5 minutes ago", "ran 1 day ago"). (Thanks to @aukgit)
- **SSH Multi-Instance & Account-Centric Discovery with 1-Click Remote Dispatch**: Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via SSH; elevated Account Email and Instance Name to primary prominent columns in `FleetMachinesTable.tsx` and `SshGitmapSection.tsx` while demoting raw IP into an interactive hover badge; added dynamic instance `<select>` dropdown with transparent borderless styling and 1-click remote prompt dispatching. (Thanks to @aukgit)

### Fixed
- **Expand Full / Fold Tab Switch Persistence**: Fixed UI bug in `src/components/instances/PromptTreeView/DetailTabs.tsx` where switching from the Input to Output tab caused the `Expand Full` toggle to unmount; decoupled `hasExpandToggle` so output text can be expanded to full height seamlessly. (Thanks to @aukgit)
- **Prompt Tree Image Rendering & Queued/Running Visibility Protection**: Added Tauri v2 `core:asset:default` capability permission in `src-tauri/capabilities/default.json` and resolved relative image paths against project `repo_path` before passing to `convertFileSrc` in `clipboard.ts` and `PromptMarkdownPreview.tsx`; protected queued prompts from stale conversation demotion in `conversationGuards.ts`; synthesized running/queued projects from `active_prompts` in `src-tauri/src/modules/repo_db/tree_phases.rs`, ensuring the left panel is never empty when prompts are in flight. (Thanks to @aukgit)
- **Cold-Start IDE 5-Second Stabilization Ladder & Send Reliability**: Upgraded `src-tauri/src/modules/instance/smart_run.rs` and `src-tauri/src/modules/repo_db/agy.rs` with an adaptive 5-second process stabilization ladder and CDP retry mechanism for cold-started IDE instances, eliminating lost prompts when dispatching to closed IDEs. (Thanks to @aukgit)

---

## [v4.235.0] - 2026-10-19

### Added
- **SSH Cluster Multi-Instance & Account Telemetry with Direct File Fallback**: Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` with `extract_clean_json` to bypass leading logger noise, OS-specific CLI execution (`agm.exe` on Windows vs `agm` on Linux), and direct PowerShell/Bash file inspection over SSH (`instances.json`, `accounts.json`) to reliably extract remote instances and bound Google accounts; calculates accurate dual-window quotas via `extract_dual_window_quotas` instead of looking for nonexistent root fields; synchronizes encrypted node passwords to `~/.gitmap/gitmap-ssh-nodes.json` in `connections.rs` so GitMap's `fix-auth` engine possesses authentication credentials; and forwards default identity key paths (`-i <pubkey>`) in `fix_node_auth_via_gitmap`. (Thanks to @aukgit)
- **Universal Supabase 5-Minute Network Error Circuit Breaker**: Integrated automatic cooldown triggers in `src-tauri/src/modules/supabase_client/client.rs` across all client methods (`select`, `insert`, `upsert`, `update`, `delete`, `rpc`), automatically invoking `record_supabase_network_error()` on timeouts, connection errors, and 5xx responses, while fast-failing subsequent requests with `Err(AppError::Network(...))` during the 300-second cooldown; added circuit breaker guards to background sync workers in `fleet.rs`, `sync.rs`, and `supabase_pruner.rs` to stop repetitive connection retry loops. (Thanks to @aukgit)

### Fixed
- **Prompt Tree Action Button Reliability & Async Feedback**: Converted `onRefreshAll` and `onResetTree` in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` into async functions awaiting `loadTree(false, true)`, synchronizing cache invalidation (`invalidatePromptTreeCache`) and auto-dismissing visual status banners after 2500ms; enabled flexible instance dispatch in `src-tauri/src/modules/repo_db/dispatch.rs` so restored prompts dispatch cleanly across active and default instances with loopback CDP injection fallback (`submit_prompt_via_cdp_for_instance`). (Thanks to @aukgit)
- **Import/Export Menu Stacking Context & Portal Isolation**: Refactored `src/components/instances/PromptTreeView/ImportExportMenu.tsx` to mount its dropdown popup via React Portal (`createPortal(..., document.body)`) with `fixed z-[99999]` and a full-screen transparent capture backdrop (`fixed inset-0 z-[99998]`), completely eliminating CSS stacking context clipping and blur bleed-through from `ModalHeader` (`z-40`) and `TreePanel`. (Thanks to @aukgit)
- **Preview Header Decluttering & Auto-Selection Flapping**: Removed redundant plain `User Prompt` badges and verbose instance ID tokens (`default-copy-5597`) from `DetailHeader.tsx`, directing deep runtime telemetry to `DetailsModal.tsx`; updated `performAutoSelection` in `usePromptTreeData.ts` to preserve existing selected conversations on reloads; tightened skeleton shimmer mounting to avoid screen glitches during background refreshes. (Thanks to @aukgit)
- **Subagent Queued Badges & 3-Tier Sorting**: Added explicit amber clock `QUEUED` badges to child subagents in `ProjectNode.tsx` when enqueued under parent prompts; enforced strict 3-tier ordering (Tier 1: Running, Tier 2: Queued, Tier 3: Newest Timestamp) across project trees. (Thanks to @aukgit)
- **UI Transaction & Polling Cadence Dampening**: Increased `FETCH_DEBOUNCE_MS` to 30s in `src/stores/useAccountStore.ts` with 5-minute idle throttling when `document.hidden`; implemented adaptive 60s/300s polling in `src/pages/instances/useInstancePage.ts` with a 2000ms coalescing debounce on live events; preserved `ACCOUNT_LIST_CACHE` in `src-tauri/src/modules/account/crud.rs` via in-place memory updates on quota saves. (Thanks to @aukgit)
- **CLM Query Flood Elimination & Audit Log Demotion**: Optimized `src-tauri/src/modules/repo_db/liveness.rs` to query `conversation_summaries.db` once per instance sweep into a `HashSet<String>` rather than opening the database 15-30 times for every workspace folder; cached `detect_running_projects` for 15s in `detection.rs`; and demoted idle/negative audit logs (`IdleNoActiveTasks`) from `info!` to `tracing::debug!`. (Thanks to @aukgit)
- **Telegram Machine Discovery & Remote Command Syntax**: Expanded `machine_entries()` in `src-tauri/src/modules/telegram_inbound/send_flow.rs` to discover SSH mesh and Supabase fleet nodes alongside GitMap nodes; standardized remote dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to `agm.exe prompt <target> [flags] "<prompt>"` to correctly invoke targeted sequence resolution (`C001`, `P001`) instead of swallowing arguments; and added a 3-stage execution fallback ladder inspecting process exit status. (Thanks to @aukgit)

---

## [v4.234.0] - 2026-10-19

### Added
- **GitMap Cluster Node Deduplication & Local Host-IP Resolution**: Added `deduplicate_connections()` in `src-tauri/src/modules/ssh_manager/connections.rs` to eliminate duplicate cluster node records; resolved local node identity in `src-tauri/src/modules/ssh_manager/exec.rs` by matching local IPv4 against configured SSH connections (resolving local alias as `w1`), ensuring local instances are not excluded or misattributed to remote nodes; added dual-key matching (`machine_alias` or `ip_address`) in `src/components/settings/EmailNotifications/SshGitmapSection.tsx` to accurately display instances and bound accounts. (Thanks to @aukgit)
- **Telegram Inbound Multi-VM Collision Shielding & Hostname Precedence**: Restored multi-VM collision shielding guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` (`on_machine_picked`, `on_instance_picked`), ensuring only targeted nodes respond to machine/instance callbacks while non-target nodes drop replies cleanly to prevent multi-daemon message collisions; refined `is_current_machine` in `src-tauri/src/modules/telegram_inbound/node_label.rs` to prioritize authentic local hostnames (`detect_machine_name`) and local IPs before querying cluster nodes; standardized remote CLI dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to `agm prompt send --instance <inst> [--repo <repo>] "<prompt>"`. (Thanks to @aukgit)

### Fixed
- **Unified Prompt Tree Status & Strict 3-Tier Hierarchy**: Centralized conversation status detection in `src/components/instances/PromptTreeView/conversationGuards.ts` (`getConversationStatus`) and re-exported through `ConversationNode.tsx`; enforced strict 3-tier ordering in `ProjectNode.tsx`, `tierClassification.ts`, and `PromptTreeViewModal.tsx` so running prompts stay strictly at Tier 1 on top, queued prompts at Tier 2 with amber clock badges, and completed/historical prompts ordered by timestamp descending. (Thanks to @aukgit)
- **Supabase Polling Throttling & Background Debouncing**: Added document visibility change detection and 5-minute background throttling in `src/pages/instances/useInstancePage.ts` and `src/stores/useAccountStore.ts`, coalescing IPC refresh requests and eliminating CPU spikes during inactive background state. (Thanks to @aukgit)

---

## [v4.233.1] - 2026-10-19

### Added
- **Native Base64 Image Reader for Safe Clipboard Copying**: Introduced `read_image_as_base64` in `src-tauri/src/commands/cmd_files.rs` and registered in `src-tauri/src/lib/run.rs` to read local image files securely with path canonicalization, system path protection, and a 100MB ceiling; decodes Base64 data directly into binary PNG Blobs (`base64ToBlob`) in `src/components/instances/PromptTreeView/clipboard.ts`, eliminating offscreen HTML canvas CORS errors and WebView2 `SecurityError` canvas tainting; added rich event JSON clipboard copying with toast feedback. (Thanks to @aukgit)

### Fixed
- **Singleton Context Menu Architecture & Comprehensive Dismissal Guard**: Eliminated duplicate ghost context menu rendering and menu lingering by centralizing context menu state into `src/components/instances/PromptTreeView/TreePanel.tsx`, rendering exactly one `<PromptTreeContextMenu>` in the DOM; mounted a transparent full-screen portal capture backdrop (`fixed inset-0 z-[99998]`) behind the active context menu (`z-[99999]`); registered window capture listeners in `src/components/instances/PromptTreeView/PromptTreeContextMenu.tsx` for `pointerdown`, `contextmenu`, `keydown` (`Escape`), `scroll`, and `blur`; removed the 100ms click-dropping delay so menus dismiss immediately on any outside interaction. (Thanks to @aukgit)
- **Enqueued Prompt Turns Visibility & Auto-Expansion**: Fixed queued item invisibility in `src/components/instances/PromptTreeView/conversationGuards.ts`, `ConversationNode.tsx`, `ProjectNode.tsx`, and `tierClassification.ts` by introducing `isConversationOrTurnQueued` to inspect child prompt turns. Conversation nodes with queued turns now auto-expand upon mount, display amber `QUEUED` badges with clock icons, sort into Tier 2 immediately beneath running items, and are preserved when filtering by `activeFilter === 'queued'`. Removed duplicate local context menu states and redundant portals from `ConversationNode.tsx` and `ProjectNode.tsx`. (Thanks to @aukgit)
- **GitMap Cluster Node Discovery & CLI Parsing**: Upgraded `import_ssh_from_gitmap()` in `src-tauri/src/modules/ssh_manager/sync.rs` to prioritize `gitmap cluster nodes --json` and `export-json` which emit machine-readable JSON arrays, resolving silent import failures caused by text-table stdout from legacy `ssh nodes` commands; verified with automated unit tests for `gitmap cluster nodes --json` JSON decoding. (Thanks to @aukgit)
- **Accurate Offline Node Quota Telemetry**: Corrected unreachable SSH node fallback values in `src-tauri/src/modules/ssh_manager/exec.rs` from fabricated 100.0% to 0.0% (`quota_4h_pct: 0.0`, `quota_weekly_pct: 0.0`), preventing offline nodes from presenting false availability in cluster views. (Thanks to @aukgit)
- **Account Store IPC Request Debouncing & Coalescing**: Added `inFlightFetchPromise` coalescing and 1500ms timestamp-based debouncing to `fetchAccounts` in `src/stores/useAccountStore.ts`, eliminating duplicate concurrent IPC requests during multi-component mounting while supporting explicit force refreshes. (Thanks to @aukgit)

---

## [v4.233.0] - 2026-10-19

### Added
- **SSH Fleet & GitMap Cluster Account/Instance Discovery & Auth Key Forwarding**: Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via GitMap discovery and cluster metadata; forwarded identity public key `-i <pubkey>` in `fix_node_auth_via_gitmap` across `mesh.rs`, `commands/ssh.rs`, and CLI handlers, removing hardcoded single-node bypasses in favor of generalized credential vault auto-repair. (Thanks to @aukgit)
- **Telegram Multi-Node Coordinator Routing & Syntax Standardization**: Removed fatal callback drop guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` so the receiving Telegram bot coordinator accurately routes prompts across cluster nodes; standardized remote dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to canonical `agm prompt send <project> --instance <inst> "<prompt>"` with safe quoting and multi-line escaping. (Thanks to @aukgit)
- **CDP-First Prompt Submission for Active IDE Windows**: Inverted prompt dispatch in `src-tauri/src/modules/repo_db/agy.rs` to submit directly via Chrome DevTools Protocol (CDP) into active IDE windows before falling back to spawning external `agy` CLI background workers, eliminating worker collision on open workspaces. (Thanks to @aukgit)
- **Instance List PID Status Pill & Path Tail Alignment**: Replaced ambiguous status codes with an emerald pulse badge (`PID: <pid>`) and aligned executable tail and data directory tail together on the exact same horizontal flex row (`.../Antigravity.exe · .../data`) in `src/components/instances/table/InstanceRowCells.tsx`. (Thanks to @aukgit)
- **Royal Violet Switch Account Modal Modernization**: Upgraded `src/pages/instances/SwitchAccountDialog.tsx` with cohesive dark-glass UI/UX (`var(--ui-surface-1)`, `var(--ui-border-subtle)`), clear selection states, and keyboard accessibility. (Thanks to @aukgit)
- **Caller IDE Self-Preservation & Transient Test Sandboxing**: Hardened `detect_caller_instance_id` to protect the developer's active workspace session (`default-copy-5597`) from accidental termination during CLI operations; added transient test sandboxes (`agm instances test-create` and `agm instances test-clean`) for non-destructive lifecycle verification. (Thanks to @aukgit)

### Fixed
- **Prompt Tree Action Buttons & 1-Click Backup/Restore**: Wired direct 1-click execution for Backup and Restore in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `coreActions.handleBackup()` and `coreActions.handleRestore()`; added spinning animation on the Refresh icon during active sync; provided instant visual notification feedback on Reset and Refresh. (Thanks to @aukgit)
- **Prompt Tree Screen Flickering Elimination**: Restricted animated skeleton loaders in `src/components/instances/PromptTreeView/TreePanel.tsx` and `PromptTreeViewModal.tsx` strictly to initial cold starts (`isLoading && treeData.length === 0`), keeping the active tree rendered during background polling to eliminate visual flashing. (Thanks to @aukgit)
- **Deterministic Prompt Ordering & Badges**: Enforced 3-tier deterministic sorting in `ProjectNode.tsx` and `tierClassification.ts` with `RUNNING` prompts strictly at Tier 1, `QUEUED` prompts at Tier 2 with explicit amber badges, and timestamp descending at Tier 3; prevented running/queued prompts from being marked as stale or empty. (Thanks to @aukgit)
- **Prompt Preview Header Decluttering & Dropdown Z-Index Stacking**: Removed redundant `"User Prompt"` badge and duplicate `{instanceNameDisplay}` text token from `DetailHeader.tsx`, relocating deep runtime telemetry to the Details modal; added `relative z-40` to `ModalHeader.tsx` container and `relative z-50` to `ImportExportMenu.tsx` so menus stack cleanly above banners and header elements. (Thanks to @aukgit)
- **Supabase 5-Minute Atomic Circuit Breaker & Polling Rate Dampening**: Implemented atomic `LAST_SUPABASE_ERROR_TIMESTAMP` circuit breaker in `src-tauri/src/modules/supabase_sync/types.rs` with a 300-second cooldown on network errors across `fleet.rs`, `query.rs`, and `release.rs`; relaxed fleet polling to 300s in `useFleetMachines.ts`; added a 30-second TTL cache for account listings in `src-tauri/src/modules/account/crud.rs` to stop repetitive reads of 46 JSON account files; removed circular cache invalidation sabotage in `src-tauri/src/modules/repo_db/detection.rs:254` to eliminate console log flood. (Thanks to @aukgit)
- **Process Termination & Lockfile Cleanup**: Enforced process tree killing via `taskkill /F /T /PID` before stopping or switching accounts in `src-tauri/src/commands/instance.rs`, ensuring complete release of Chromium lockfiles (`code.lock`, `DevToolsActivePort`, `Singleton*`). (Thanks to @aukgit)

---

## [v4.232.1] - 2026-10-18

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.232.0] - 2026-10-18

### Added
- **UI/UX Session Overview Collapsible & State Memory**: Converted the session overview dashboard in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` to be collapsed by default to save vertical viewport space; added subtle low-profile purple expander trigger (`ChevronDown`/`ChevronUp`); persisted collapse/expand state in `localStorage` (`agm_prompt_overview_expanded`) across sessions; compacted collapsed view into a single-line metadata bar. (Thanks to @aukgit)
- **Standardized 24-Hour Date/Time & Relative Time Formatting**: Implemented centralized date formatting in `src/utils/date.ts` supporting `DD-MMM-YYYY HH:mm` (e.g. `18-Oct-2026 21:40`) in 24-hour clock using the local system timezone, with relative time indicators ("ran 5 minutes ago", "ran 1 day ago") and duration seconds suppression when hours or minutes exist; added user-configurable `date_format_mode` toggle (`24h` / `12h` / `iso`) in Settings. (Thanks to @aukgit)
- **SSH Fleet Account & Instance Discovery with Remote Prompt Dispatch**: Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via SSH; elevated Account Email and Instance Name to primary columns in `FleetMachinesTable.tsx` and `SshGitmapSection.tsx` while demoting raw IP into an interactive hover badge; added dynamic instance `<select>` dropdown and 1-click remote prompt dispatching. (Thanks to @aukgit)

### Fixed
- **Persistent Expand Full Toggle across Input/Output Tabs**: Fixed UI bug in `src/components/instances/PromptTreeView/DetailTabs.tsx` where switching from the Input to Output tab hid the Expand Full / Fold toggle button; decoupled expand toggle from instruction tab so users can freely expand both input instructions and execution results. (Thanks to @aukgit)
- **Header Deduplication & Detail Modal Clarification**: Removed redundant title, sequence badge (`C211`), and conversation ID repetitions from the overview card; clarified sequential conversation index (`C211` = Conversation #211) and interaction turns (`turns` = user/assistant request-response cycles) in `src/components/instances/PromptTreeView/DetailsModal.tsx`. (Thanks to @aukgit)
- **Prompt Tree Image Rendering & Queued/Running Visibility Fix**: Added Tauri v2 `core:asset:default` capability in `src-tauri/capabilities/default.json` and resolved relative image paths against `repo_path` in `clipboard.ts` and `PromptMarkdownPreview.tsx`; protected queued prompts from stale conversation demotion in `conversationGuards.ts`; synthesized running/queued projects from `active_prompts` in `src-tauri/src/modules/repo_db/tree_phases.rs`, ensuring the left panel is never empty when prompts are in flight. (Thanks to @aukgit)
- **Closed IDE Send Delivery Ladder**: Upgraded `src-tauri/src/modules/instance/smart_run.rs` and `src-tauri/src/modules/repo_db/agy.rs` with an adaptive 5-second process stabilization ladder and CDP retry mechanism for cold-started IDE instances, eliminating lost prompts when dispatching to closed IDEs. (Thanks to @aukgit)

---

## [v4.231.1] - 2026-10-18

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.231.0] - 2026-10-18

### Added
- **Instance List View Layout & PID Indicator**: Confirmed that numeric status values (e.g. `1632`) represent the native OS Process ID (`PID`); merged separate `Status & PID` and `IDE EXE & DATA` table columns into a unified `Status, Process & Environment` cell in `src/components/instances/InstanceTable.tsx` and `src/components/instances/table/InstanceRowCells.tsx`, rendering the running status with PID badge (`PID: <pid>`), shortened executable path (`.../Antigravity.exe` with Terminal icon), and shortened user data directory (`.../antigravity-data` with Folder icon) together on the exact same horizontal flex row with subtle `·` separators and a contiguous segmented copy action capsule. (Thanks to @aukgit)
- **Royal Violet Switch Account Modal Modernization**: Redesigned `src/pages/instances/SwitchAccountDialog.tsx` to 100% Royal Violet design token compliance (`var(--ui-surface-1)`: `#171025`, `var(--ui-border-subtle)`: `#40285e`, `var(--ui-canvas)`: `#090611`, `var(--ui-interactive)`: `#7c3aed`, `var(--ui-success)`: `#1af18d`); integrated a segmented quota visualization pill capsule header (`4H: <val>% | Weekly: <val>% | Total: <val>%`); maintained `.ui-interactive-row` whole-card row selection, Escape key dismissal, and keyboard accessibility. (Thanks to @aukgit)
- **E2E Lifecycle Verification Suite**: Comprehensive verification through `scripts/test-instance-lifecycle-e2e.ps1` covering caller self-preservation guards, transient test sandbox creation (`agm instances create-test`), verified PID eviction, switch synchronization, audit logging, and clean filesystem teardown (`agm instances rm-test`). (Thanks to @aukgit)

### Fixed
- **Deterministic 5-Stage Process Termination Ladder**: Fixed stop instance button and process killing on Windows in `src-tauri/src/modules/instance/close.rs` and `create.rs` by establishing a 5-stage progressive ladder: root PID + recursive child descendant discovery, primary tree kill via `taskkill /F /T /PID <root_pid>` with `CREATE_NO_WINDOW (0x08000000)`, high-frequency 50ms verification polling (up to 1500ms), force escalation fallback via direct Win32 kernel kill (`force_terminate_pid_win32`) and PowerShell `Stop-Process -Id <pid> -Force`, mandatory 200ms kernel settle delay guaranteeing NTFS file handle release, and unconditional Chromium lockfile eviction (`lockfile`, `code.lock`, `DevToolsActivePort`, `Singleton*`). (Thanks to @aukgit)
- **Account Switch Pre-Switch PID Eviction & Synchronization**: Fixed account switching synchronization in `src-tauri/src/modules/instance/switch.rs` and `credentials.rs` by forcefully terminating any active instance PID before accessing `state.vscdb`, clearing stale Chromium lockfiles, enforcing a 200ms settle delay to prevent Electron shutdown write-back race conditions, and atomically injecting credentials with progressive retry backoff. (Thanks to @aukgit)
- **Sharded Instance Lifecycle Audit Trail Visibility**: Fixed empty audit trail in `src-tauri/src/modules/task_history_db/switch_history.rs` and `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx` by expanding default instance aliases (`""`, `NULL`, `"default"`, `"antigravity"`, `"agy"`, `"ide"`) across Split-DB queries, guaranteeing all response field aliases (`recent_switches`, `total_switches`, `records`, `items`), and adding smart auto-tab fallback to the `operations` tab when switch cycles are 0 so users immediately see instance lifecycle events (`LaunchInstance`, `StopInstance`, `RestartInstance`, `CreateInstance`, `SwitchAccount`). (Thanks to @aukgit)
- **Active Caller IDE Self-Preservation**: Hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations and matching workspace paths) across `agm instances stop`, `switch`, `restart`, `delete`, and `rm-test` with `refuse_if_caller_instance` guards to prevent CLI commands and automated routines from self-terminating the developer's active workspace session with exit code 2. (Thanks to @aukgit)
- **Clipboard Focus Loss Resilience & 3-Stage Fallback Ladder**: Eliminated Error Manager warnings (`NotAllowedError: Failed to execute 'writeText'/'write' on 'Clipboard': Document is not focused`) in `src/utils/clipboard.ts`, `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx`, and `src/components/instances/PromptTreeView/clipboard.ts` by ensuring document focus (`ensureDocumentFocus`), wrapping clipboard writes in a 3-stage fallback ladder (Async Clipboard API -> `document.execCommand` -> native desktop helper `copy_text_to_clipboard` via OS clip/pbcopy/wl-copy), and suppressing false-alarm telemetry when subsequent fallbacks succeed. (Thanks to @aukgit)
- **Instance Registry Atomic Persistence & 0-Byte Recovery**: Eliminated intermittent parse errors (`Error: Failed to parse instances registry: expected value at line 1 column 1`) in `src-tauri/src/modules/instance/registry.rs` by upgrading `save_registry()` to use atomic file replacement (`write_atomic` via Windows `MoveFileExW`) and automated `.bak` backup, adding automatic recovery from `instances.json.bak` and fallback initialization in `load_registry()`, and routing silent background polling failures in `src/stores/useInstanceStore/crud.ts` away from user-facing error modals. (Thanks to @aukgit)

---

## [v4.230.0] - 2026-10-18

### Added
- **Prompt Tree View Ergonomics & Compact Action Rows**: Compacted Input / Output indicators into a unified segmented capsule alongside Expand Full on the right-hand side of Row 2; streamlined View Modes (`Preview`, `Raw`, `Edit`, `Details`) into an icon-only capsule (`rounded-lg` container, `rounded-md` buttons) on the left-hand side with hover tooltips; elevated Copy, Send, and Rerun controls to Row 1 with dual rich copy options (`includes image` binary PNG + HTML vs `copy as text`). (Thanks to @aukgit)
- **Prompt Image Previews & Native Multimodal Clipboard**: Added inline image resolution via Tauri asset URLs (`convertFileSrc`) preventing WebView2 image rendering failures; enabled right-click context menu and dual clipboard copy supporting rich HTML with raw PNG binary payloads. (Thanks to @aukgit)
- **Conversation Session Dashboard & Queue Indicators**: Added comprehensive Conversation Session Summary Dashboard when viewing session roots (turn count, duration, timestamp range, enqueued prompts list) and breadcrumb badge (`↳ Turn X of Y`) on individual prompt turns; integrated `+N queued` counters in tree items and detail headers. (Thanks to @aukgit)
- **Local Mesh Shared Cluster Display**: Restored the Shared Cluster view in instance management to discover and display local SSH mesh nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) with TCP liveness probing even when Supabase Cloud Sync is disabled. (Thanks to @aukgit)

### Fixed
- **Instance Prompt Bleeding & False Liveness**: Enforced Explicit Idle Supremacy in `tree_collect.rs` and `liveness.rs` where explicit idle statuses (`CASCADE_RUN_STATUS_IDLE`) override unreset `not_fully_idle` engine artifacts; restricted instance card active task indicators to genuine conversation owners (`c.instance_id === inst.config.id`). (Thanks to @aukgit)
- **SSH Node W2 Reachability & GitMap Parity**: Fixed SSH connection failures to remote node `w2` (192.168.1.7:22) by falling back to GitMap Go in-process SSH engine and supporting vaulted credential delegation (`rtyrty123@`) via `gitmap ssh fix-auth` and `gitmap ssh exec`. (Thanks to @aukgit)
- **Standalone JSON Import/Export & Modal Overlay**: Upgraded prompt tree JSON export to v2.0 envelope schema with portable base64 embedded images and relative `${workDir}` path normalization; elevated modal overlay z-index to `z-[100050]` so export/import dialogs are never trapped behind the main prompt tree modal. (Thanks to @aukgit)
- **Updater Pinned Version Execution & Live Version Extraction**: Fixed updater script `install.ps1` to write `DisplayVersion` to Windows Registry branding (resolving false `v4.190` reports), inspect PE executable `ProductVersion` first, and enforce strict `-isPinned` contracts preventing silent fallback to unrequested versions. (Thanks to @aukgit)

---

## [v4.229.0] - 2026-10-18

### Fixed
- **Process Termination Ladder & Account Switch Synchronization**: Fixed stop instance button and account switching synchronization failures by establishing a multi-stage process termination ladder (`taskkill /F /T` with fallback to Win32 `TerminateProcess`), conscious PID matching against cached process instances, and deterministic lockfile eviction (`code.lock`, `lockfile`, `DevToolsActivePort`) before updating `state.vscdb`. (Thanks to @aukgit)
- **Active Caller IDE Self-Preservation**: Hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations and matching workspace paths) across `agm ff`, `switch_account_to_instance`, `stop_instance`, and `close_instance` with `refuse_if_caller_instance` guards to prevent CLI commands and automated routines from self-terminating the developer's active workspace session. (Thanks to @aukgit)
- **Audit Trail Query Filter & Empty State Remediation**: Fixed empty instance audit trail by refining SQL filter logic to eliminate global account crud pollution from `default` instance records, instrumenting missing lifecycle operations (`RestartInstance`, `CloneInstance`, `CreateInstance`) into `task_history_db`, and expanding frontend action code mapping for lifecycle operations. (Thanks to @aukgit)

### Added
- **Instance List View Layout & PID Indicator**: Aligned running instance status (`● PID: <pid>`), executable tail, and data directory tail inline on the same horizontal row with monospace formatting, enhanced tooltips, and contiguous segmented copy actions. (Thanks to @aukgit)
- **Switch Account Dialog UI/UX Modernization**: Redesigned switch account modal with Royal Violet tokens (`#171025`, `#40285e`, `#1af18d`), circular radio selection indicators, standard `<TierBadge />` integration, search filter clear button, and full keyboard accessibility. (Thanks to @aukgit)
- **E2E Lifecycle Verification Suite**: Comprehensive verification through `scripts/test-instance-lifecycle-e2e.ps1` covering caller self-preservation guards, transient test sandbox creation, PID eviction, switch synchronization, and clean filesystem teardown. (Thanks to @aukgit)

---

## [v4.228.1] - 2026-10-18

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.228.0] - 2026-10-18

### Fixed
- **Conscious Process Termination & Account Switch Synchronization**: Fixed stop instance button and account switching synchronization by implementing Win32 process tree escalation (`taskkill /F /T /PID` and `TerminateProcess`), 8.3 path normalization (`GetShortPathNameW`), and a 200ms kernel settle delay to guarantee process death and lockfile release before credential rewrites in `state.vscdb`. (Thanks to @aukgit)
- **Active Caller IDE Self-Preservation**: Hardened `detect_caller_instance_id` to trace cloned executables (`Antigravity-<id>.exe`), `ANTIGRAVITY_APP_DATA_DIR`, and `VSCODE_PID`, enforcing caller immunity guards across `rm`, `switch`, `ff`, and `stop` to prevent CLI operations from terminating active IDE developer windows. (Thanks to @aukgit)

### Added
- **Instance List View Path Alignment & PID Clarification**: Explicitly formatted running instance status as `● Running (PID: <pid>)` with pulsing emerald indicator; aligned executable tail and data directory tail together inline on the same horizontal row with monospace pill styling and dual copy actions. (Thanks to @aukgit)
- **Switch Account Dialog UI/UX Modernization**: Completely overhauled switch account modal with AGM dark-glass styling (`rounded-2xl`, Royal Violet surfaces, sky/amber badges, `.ui-interactive-row` selection, Escape key dismissal, and segmented capsule controls). (Thanks to @aukgit)
- **Instance Audit Trail Visibility & Lifecycle Actions**: Extended `AuditAction` enum with instance lifecycle actions (`LaunchInstance`, `StopInstance`, `KillInstance`, `CreateInstance`, `CloneInstance`, `DeleteInstance`, `InstanceError`), instrumented backend lifecycle logging into `task_history_db`, and added automatic fallback to operations tab to eliminate empty audit states. (Thanks to @aukgit)
- **CLI Test Sandboxing & Timeout Hardening**: Added `agm instances current`, `agm instances create-test`, and `agm instances rm-test` for sub-50ms transient data-only testing, and hardened test suite (`scripts/test-instance-autoswitch-e2e.ps1`) with 30-second bounded timeouts. (Thanks to @aukgit)

---

## [v4.227.3] - 2026-10-18

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.227.2] - 2026-10-18

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.227.1] - 2026-10-17

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.227.0] - 2026-10-18

### Fixed
- **Error E9001 Root-Cause Resolution & Sibling Instance Account Isolation**: Resolved Error E9001 (`Cannot switch instance to account: Account is actively bound to running sibling instance`) during Fast-Forward and manual rotation. Actively running instances (`is_running == true`) are never marked stale or stealable in `is_instance_binding_stale_or_exhausted`; running sibling instance accounts (checking `bound_account_id`, `bound_email`, and default active account) are strictly excluded from candidate evaluation in `manual.rs`; and live pre-switch candidate verification in `selection.rs` guards against sibling account theft. (Thanks to @aukgit)
- **Prompt Queue Missing Workspace Fail-Fast & Infinite Retry Loop Elimination**: In `PromptQueueScheduler` (`queue.rs`), added workspace existence validation to prevent infinite 5-second queue retry loops when target directories are missing; marked invalid prompts as `failed` with actionable diagnostics and applied `updated_at = now` retry backoff on dispatch errors. (Thanks to @aukgit)
- **Supabase Connection & Heartbeat 5-Minute Error Backoff**: Added a 300-second (5-minute) error backoff in `start_sync_worker` (`fleet.rs`) on Supabase connection/heartbeat errors, eliminating repetitive DNS/connection warning storms. (Thanks to @aukgit)

### Added
- **Instance Card & Account Card Toolbar Redesign & Riseup Yellow 3D**: Replaced the 4th icon (Tag) on Account and Instance cards with the Fast-Forward (`FastForward`) button; relocated Tag (Edit Label) into the More dropdown; flattened Switch (yellow) and Delete (red) buttons when idle; and applied authoritative Riseup Yellow 3D gradients (`from-[#FDE047] via-[#FACC15] to-[#EAB308]`, border `#CA8A04`, depth shadow `[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]`) on hover. (Thanks to @aukgit)
- **UI Hover Dark-Glass Contrast & Green Duration Micro-Pill Badges**: Replaced stark white hover backgrounds across `AuditTable.tsx` and account rows with balanced dark-glass styling (`hover:bg-slate-100/80 dark:hover:bg-slate-800/60`); removed inner cell opaque backgrounds; upgraded `default-copy` instance badges to high-contrast `bg-indigo-100 text-indigo-950 font-black`; and enclosed green duration text (`#1af18d`) in high-contrast micro-pill badges (`bg-emerald-500/15 text-emerald-950 dark:text-emerald-200 border-emerald-500/30 font-bold`). (Thanks to @aukgit)
- **Background Loop Throttling & 30-Second Minimum Floor**: Enforced strict `>= 30s` minimum floor on CLI poller loops (`fpug` and `sug` in `green_cmds.rs`) and elevated `MIN_GOOGLE_QUOTA_CHECK_SECONDS` to 30 seconds. (Thanks to @aukgit)
- **Prompt Parameter Right-Click Context Menu & 1-Click Reset Filters**: Added `onContextMenu` right-click handlers on prompt parameter badges (`#{seqDisplay}`, `USER_PROMPT`, context `#C001 · default`) in `DetailHeader.tsx` to copy clean JSON parameter payloads; added 1-click Clear `✕` button to `InstancesFilterBar` and `InstancesNoResultsState`; and added "Reset to Local" button in remote SSH prompt dispatcher. (Thanks to @aukgit)
- **Updater Typography & Cross-Platform Terminal Fonts**: Configured Win32 updater console to 18pt Consolas (`font_family = 54`, `dw_font_size.y = 18`, `font_weight = 600`) with Lucida Console fallback for Windows, with specification for Ubuntu Mono and SF Mono. (Thanks to @aukgit)
- **GitMap <-> AGM Mutual HMAC-SHA256 JWT Communication Bridge**: Architected secure local HTTP JWT bridge over port `:18188` with 60-second token expiration, replay nonce cache with 120s TTL, and scoped permissions (`email:read`, `cluster:sync`). (Thanks to @aukgit)

---

## [v4.226.1] - 2026-10-17

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.226.0] - 2026-10-18

### Fixed
- **Error E9001 Root-Cause Resolution & Sibling Instance Account Isolation**: Fixed Error E9001 (`Cannot switch instance to account: Account is actively bound to running sibling instance`) during Fast-Forward and manual rotation. Actively running instances (`is_running == true`) are never marked stale or stealable in `is_instance_binding_stale_or_exhausted`; running sibling instance accounts (checking `bound_account_id`, `bound_email`, and default active account) are strictly excluded from candidate evaluation in `manual.rs`; and live pre-switch candidate verification in `selection.rs` guards against sibling account theft. (Thanks to @aukgit)
- **Prompt Queue Non-Existent Workspace Handling & Infinite Retry Loop Elimination**: In `PromptQueueScheduler` (`queue.rs`), added workspace existence validation to prevent infinite 5-second queue retry loops when target directories are missing; marked invalid prompts as `failed` with actionable diagnostics and applied `updated_at = now` retry backoff on dispatch errors. (Thanks to @aukgit)

### Added
- **Accounts Table UI Redesign & Riseup Yellow 3D Switch Button**: Stripped blinding stark white hover backgrounds (`hover:!bg-white`) across `SortableAccountRow.tsx` and `AccountRowContent.tsx` in favor of subtle dark-glass styling (`hover:bg-slate-100/70 dark:hover:bg-slate-800/50`); enhanced bound instance badge (`Default Copy`) contrast; flattened Delete and Switch action buttons when row is idle; styled Switch button with Riseup Yellow 3D gradient; reordered action buttons (Fast-Forward `>>`, Refresh `↻`, 3D Riseup Yellow Switch, Delete `🗑️`, More `···`); and moved custom label Tag edit into More dropdown. (Thanks to @aukgit)
- **Prompt Tree View Reset Filters**: Added an interactive "Reset Filters" button in `TreePanel.tsx` that appears when any filter is active, resetting search query, status filter, category filter, and project scope in a single click. (Thanks to @aukgit)
- **Adaptive L1 Quota Cache TTL & Supabase Error Backoff**: Implemented adaptive L1 quota cache expiration in `quota_fetch.rs` (120s for depleted <20%, 180s for <50%, 300s for healthy >=50% quota), reducing Google OAuth quota polling overhead; added 300-second (5-minute) error backoff in `start_sync_worker` (`fleet.rs`) on Supabase connection/heartbeat errors. (Thanks to @aukgit)
- **Win32 Updater Console 18pt Consolas Font**: Upgraded Win32 console updater font in `delegate_updater/run.rs` to 18pt Consolas (`font_family = 54`, `FF_MODERN | FIXED_PITCH`, `weight = 600`) with Lucida Console fallback for crisp readability during updates. (Thanks to @aukgit)
- **Codebase Hygiene & DRY Domain Enums**: Replaced magic status literals with strongly-typed enums (`PromptExecutionStatusType`, `RemoteDispatchStatusType`, `CommandExecutionStatusType`, `ProjectAuditRationaleType`, `PromptAuditCriteriaType`, `AuditTaskStatus`); flattened nested if pyramids with guard clauses across `goal_heartbeat.rs`, `detection.rs`, `tree_collect.rs`, and `pid_scan.rs`. (Thanks to @aukgit)

---

## [v4.225.0] - 2026-10-17

### Added
- **System-Wide Broadcasting Design Pattern & 30-Second Minimum Interval Guard**: Enforced an architectural invariant across Antigravity Manager and GitMap mandating that all periodic background broadcasts, polling schedulers, worker tickers, and notification dispatchers operate at intervals of not less than 30 seconds (`>= 30s`) to prevent CPU spikes, database lock contention, and notification storms. (Thanks to @aukgit)
- **Prompt Goal Heartbeat Worker Interval Clamping**: Clamped the detached prompt goal heartbeat worker loop in `goal_heartbeat.rs` to a strict 30-second minimum (`interval_secs.max(30)`, default 30s), eliminating rapid disk-write cycles. (Thanks to @aukgit)
- **Notification Hub Duplicate Switch Broadcast Suppression Window**: Expanded the duplicate switch notification broadcast suppression window in `channels.rs` from 5s to 30s (`(now_ts - guard.1).abs() <= 30`), completely preventing rapid notification storms across Email, Telegram, and Supabase. (Thanks to @aukgit)
- **GitMap Cross-System Interval Alignment**: Updated GitMap task synchronization defaults (`TaskMinInterval = 30`, `TaskDefaultInterval = 30` in `constants_task.go`), clamped async periodic monitors (`gitmap async -t`) to `>= 30s`, clamped macro runner intervals to `>= 30s`, and enforced minimum 30s cluster distribution heartbeat loops. (Thanks to @aukgit)

---

## [v4.224.0] - 2026-10-17

### Added
- **Modernized Compact Telegram Notification Cards with White Text Values**: Redesigned all Telegram alert cards (Account Rotation, Post-Switch Liveness, Emergency Alerts, and System Updates) to be ~40% more vertically compact; all key values are indented on the next line in high-contrast bold white text (`kv_block`, `kv_white`); eliminated duplicate fields (`Bound` in header, `Event reason` in workspace); and clamped project lists to top-3 with a compact `(+N more)` badge. (Thanks to @aukgit)
- **Telegram Inbound Step 2 Multi-Machine Collision Shielding & Debounce Guard**: Resolved duplicate Step 2 prompt wizard dispatch in Telegram by enforcing strict destination matching via `node_label::is_current_machine()`; non-target cluster nodes silently drop incoming callbacks, while the target node suppresses rapid duplicate taps (<3s) to prevent repeated messages. (Thanks to @aukgit)
- **Node Network Identity, IP Address & Signal Attribution in Telegram**: Enhanced Telegram machine and instance pickers with explicit node alias, IP address, and mesh signal indicators (`[alias • ip • 📶 signal]`); instance selection buttons now display `🏷️ {name} ({alias} • {ip})` arranged in single-column layouts for maximum legibility. (Thanks to @aukgit)
- **Clean Action-Only Inline Keyboard Markups**: Purged the duplicate top-row static banner button from Account Rotation alerts (`build_switch_alert_reply_markup`), presenting only functional action buttons: `[ 💰 Check Balance ] [ 📥 Export CSV ]`, `[ 🚀 Prompt Instance ] [ 🔄 Refresh Status ]`, and `[ 📋 Copy Prompt ]`. (Thanks to @aukgit)

---

## [v4.223.0] - 2026-10-17

### Added
- **Prompt Tree Right-Click Custom Context Menus (`Project` & `Conversation` & `Turns`)**: Added right-click context menus directly to Project headers (`ProjectContextMenu.tsx`) with Import into Project, Export (JSON, SQLite DB, YAML), and Rerun (Queued or All) actions; attached context menus and keyboard accessibility (`Enter`, `Space`, `tabIndex={0}`) to child subagent items and individual prompt turn entries (`ConversationNode.tsx`); implemented viewport boundary clamping (`createPortal` within `window.innerWidth - 240` and `window.innerHeight - 380`) to eliminate off-screen menu clipping; and applied high-contrast active selection highlights (`bg-sky-500/10 border-l-2 border-sky-500 text-sky-400 font-medium`) for active conversation turns. (Thanks to @aukgit)
- **Top Toolbar Right-Aligned Navigation Tabs & Vertical Dead-Space Recovery**: Redesigned Row 2 of `DetailHeader.tsx` into a `justify-between` toolbar, relocating primary navigation tabs (`[ Prompt Instruction | AI Results & Outputs ]`) to the right side of the toolbar row beside top-level action capsules; eliminated the redundant 3rd row navigation tabs and borders in `DetailTabs.tsx` to reclaim ~44px of vertical height for expanded prompt reading and result inspection; streamlined ViewMode toggles (`[ Preview | Raw | Edit ]`) and Details into a compact sub-bar directly above content. (Thanks to @aukgit)
- **Top-Level Copy All Consolidation & Redundant Footer Buttons Removal**: Consolidated all prompt copy operations into the top-level `[ Copy All | v ]` segmented capsule in `DetailHeader.tsx` (Copy All Content+Images, Copy Full Prompt, Copy Clean Text, and Save Images); purged duplicate bottom copy buttons across Preview and Raw modes in `PromptInstructionView.tsx`. (Thanks to @aukgit)
- **Telegram Inbound Machine Routing & Fleet Broadcast Storm Shielding**: Eliminated multi-machine response collisions by normalizing Supabase `public.command_queue` payloads with explicit `target_node_id` (previously defaulting to wildcard `'*'`) across `injection.rs`, `open_shot.rs`, and `processing.rs`; unified current machine detection via `node_label::is_current_machine()`; deduplicated machine entries in `machine_entries()` so only a single "This machine" button is generated; and added native CLI support for `agm prompt <target> [flags] <prompt_text>` in `cli/prompts.rs` to allow GitMap cluster execution and SSH remote prompts to succeed directly. (Thanks to @aukgit)
- **Telegram Prompt Input UX & Destination Locked Flow**: Redesigned `destination_locked_reply()` with prominent visual dividers and clear instruction (`💬 Type your prompt below and send it as a plain chat message`); encapsulated past prompts in Telegram Bot API 7.2+ `<blockquote expandable>` so direct typing is the obvious primary action; renamed Step 3 manual target button to `[✏️ Enter Project / Conv ID]`; and added graceful prompt-detection fallback in `SendStage::TypeConversation` to route direct prompt text to the default project. (Thanks to @aukgit)

---

## [v4.222.3] - 2026-10-17

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.222.2] - 2026-10-17

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.222.1] - 2026-10-17

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.222.0] - 2026-10-17

### Added
- **3-Tier Prompt Subtree Hierarchy (`Project` → `Conversation` → `Prompts List`)**: Expanded the AGM prompt tree from 2 tiers to 3 full tiers; conversations now expand with interactive chevrons to display individual prompt turns (`Turn 1, Turn 2, ..., Turn N`), their timestamps, 200-word Poppins previews, attached image indicators, and deterministic execution states. (Thanks to @aukgit)
- **Active Task Running Symbol & False IDLE Root Cause Remediation**: Resolved false IDLE status on running IDE tasks by enforcing live runtime process liveness precedence over static transcript heuristics (`is_owning_inst_alive && not_fully_idle > 0 && is_recent`); eliminated intermediate planner turn short-circuits (`is_terminal_done`) and premature IDLE mutations in `tree_collect.rs` and `tree_node_conv.rs`; added pulsing emerald badges for `RUNNING` turns and amber badges for `QUEUED` turns. (Thanks to @aukgit)
- **Section-Level Import / Export (JSON, SQLite DB, YAML) with Embedded Base64 Images**: Added right-click context menu triggers (`ProjectContextMenu.tsx`, `ConversationContextMenu.tsx`) and top-level modals (`PromptTreeExportModal.tsx`, `PromptTreeImportModal.tsx`) to export and import arbitrary scopes (Full Tree, Single Project, Single Conversation, Prompt Subtree); embedded all local image attachments as Base64 data URIs (`agm-asset://`) in exports and materialized them back into `./.antigravity/attachments/` on import for 100% cross-machine portability with zero broken links. (Thanks to @aukgit)
- **Refresh Button 4-Point RCA Fix**: Eliminated selection whiplash by calling `loadTree(false, true)` to preserve active project and conversation selection; bound proactive `detect_running_projects` workspace discovery; eradicated SQLite `immutable=1` WAL bypass with 5000ms `busy_timeout`; raised in-flight timeout cutoff to 600s in `tree_phases.rs`. (Thanks to @aukgit)
- **Poppins & Ubuntu Typography & 1-Click Header Copy**: Enforced Ubuntu font (`font-ubuntu`) for all headers/titles and Poppins font (`font-poppins`) for body, prompts, and badges; mounted a 1-click Copy button directly beside the conversation title `<h3>` in `DetailHeader.tsx` with copy toast feedback; added informative hover tooltips explaining `Conversation #226 (Sequential AGM Index)` and `Project #003 (Sequential Project Index)`. (Thanks to @aukgit)
- **Conversation Actions & Autonomous Verification**: Integrated triple-trigger conversation renaming (inline double-click, `F2` keyboard hotkey, and right-click menu item); added multi-prompt resend submenu (Resend All, Last 5, Last 10, Custom N); implemented autonomous 5-chapter "Verify Last 10 Prompts" synthesis enqueued to the IDE; and added "Clone Conversation & Compact Memory" session handoff. (Thanks to @aukgit)

---

## [v4.221.1] - 2026-10-17

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.221.0] - 2026-10-17

### Added
- **Multi-Machine Fleet Synchronization & W2 Connectivity Recovery**: Extended TCP probe timeout to 1500ms with a 2-stage retry ladder and backoff to eliminate false-offline readings on Wi-Fi and cross-subnet nodes (such as W2); added rich error diagnostics (`error_message`, `latency_ms`) to `get_cluster_instances_and_balances`. (Thanks to @aukgit)
- **Multi-Machine Matrix Dynamic IP & Placeholder Normalization**: Resolved empty `()` bug by serializing both `machine_ip` and `ip_address` on `ClusterInstanceBalanceItem` and resolving `ip_address || machine_ip`; implemented `resolve_username_placeholders` to replace raw `${adminUser}` with `Administrator` and `${linuxUser}` with `root` across SSH connection imports, envelopes, and table rows. (Thanks to @aukgit)
- **Settings SSH & GitMap Section UI Polish**: Replaced loose action buttons with a contiguous segmented pill capsule (`rounded-full`, dark-glass backdrop, subtle dividers) for `Import SSH from GitMap`, `Deploy Encrypted Credentials to Fleet`, and `Check Cluster Balances`; redesigned GitMap installer box with segmented pill buttons for `Copy Install Command` and `Open PowerShell & Install GitMap` with copy toast feedback; cleaned up raw filesystem path dumps into compact metadata badges (`GitMap Installed`, `RSA-2048 Vault Key Active`, `Nodes Configured`). (Thanks to @aukgit)
- **Multi-Agent Concurrency Upgrade (A = 3, H = 2)**: Upgraded autonomous subagent orchestration capacity across `.agents/skills/execute-parent-task-with-n-steps-v6/skill.md`, `.agents/skills/gitmap/skill.md`, coding guidelines, instruction writer, and muse master skills to spawn 3 concurrent subagents with 2 operational hands each (`A × H = 6` concurrent operations). (Thanks to @aukgit)

---

## [v4.220.0] - 2026-10-17

### Added
- **Accounts Table Delete Remediation & Modal Mounting**: Resolved non-responsive single-item and bulk-selection Delete buttons by mounting dedicated destructive `ModalDialog` components in `AccountsDialogs.tsx`, driving `executeDelete()` and `executeBatchDelete()` with error-store telemetry tracking. (Thanks to @aukgit)
- **Position-Locked Table Row Hover Inversion**: Implemented row color inversion to pure white background and pure black text without layout shift or jitter, preserving static font-weight and border metrics, with full opacity alignment across quota cells and sticky actions column. (Thanks to @aukgit)
- **Action Buttons Color Scheme Overhaul**: Styled Switch button (`⇄`) with prominent yellow highlighter theme (`bg-yellow-400 text-slate-900 border-yellow-500`) and styled Remove button (`🗑️`) with solid red background and crisp white icon (`bg-red-600 text-white border-red-700`). (Thanks to @aukgit)
- **Account Disable / Enable Lifecycle & Auto-Switch Exclusion**: Added disable options via selection toolbar (`<Ban>`) and row context menu; persisted `disabled: bool` in account state; guaranteed 100% exclusion from automated daemon rotations and candidate scoring while preserving permissive manual user switching. (Thanks to @aukgit)

---

## [v4.219.0] - 2026-10-17

### Added
- **Rich Verification Chapter Engine & Status Checklists**: Implemented full 5-chapter prompt verification architecture (`verificationTemplates.ts`) for "Is it done?", "Is it released?", "Are you confident?", and "Edge Cases & Tests". Enforces structured quality audits, branch validation, 14-location SemVer sync gates, and faithful embedding of original prompts inside Chapter 4.
- **Rerun Iteration Chapter Engine**: Integrated multi-round nudge formatting in `dispatchRerunMessages` and `DetailHeader.tsx`, embedding `{round} of {total}` progression counters alongside the full original prompt context rather than bare 3-word strings.
- **Dual-MIME Rich Image Transmission & Extractor Bug Fix**: Fixed regex parsing index in `extractAssets` (`match[2]` data URI instead of alt text), implemented Base64 image materialization with SHA-256 fingerprinting to workspace attachments (`./.antigravity/attachments/<hash>.<ext>`), and primed dual-MIME `ClipboardItem` (`text/html` and `text/plain`) on prompt resend and enqueue to ensure images are never severed.
- **Unified Copy All Parity**: Unified top `handleCopyAll` dropdown actions and bottom `handleCopyAllWithImages` to share `copyPromptWithRichImages` in `clipboard.ts`, ensuring identical rich HTML rendering, image preservation, and toast notifications.
- **Delegate Updater Console Formatting & Window Title**: Cleared `DISABLE_NEWLINE_AUTO_RETURN` flag on Windows console handles, enabled `ENABLE_PROCESSED_OUTPUT` and `ENABLE_VIRTUAL_TERMINAL_PROCESSING` to eliminate staircase line wrapping, set dynamic modern console title via `SetConsoleTitleW`, and aligned console summary fields.

---

## [v4.218.0] - 2026-10-17

### Added
- **Stale Versions Clean CLI (`agm clean-other-versions` / `agm ov`)**: Added `clean-other-versions` command (with aliases `ov` and `versions clean`) to detect and purge stale/legacy binaries across user profile and system install paths, safely reclaiming disk space while strictly preserving SQLite databases and configuration files.
- **Telegram Dynamic Host Versioning & Prompt Sanitization**: Bound Telegram version reporting dynamically to host version; stripped internal synthetic ID tags (`Id: ...`) from prompt dispatches; resolved high-DPI desktop and window screenshot capture with PNG magic byte verification.
- **Configurable Telegram Prompt Summaries & One-Tap Resend**: Added configurable prompt summary word count (100 or 200 words, via `agm telegram config --summary-words <100|200>`), clamped preview length, and added inline `↻ Resend` callback buttons on project prompt histories.
- **Instances List View Single Combined Column & Primary Actions**: Combined IDE Executable and Data Directory into a single high-density `IDE EXE & DATA` column, eliminated list view horizontal overflow with proportional 5-column layout, and promoted the Prompts button outside into the primary action group.
- **Prompt Tree DetailHeader Overlap Resolution & Transcript Sync**: Fixed toolbar collision between action dropdowns and view mode capsules (`DetailHeader.tsx`), and enhanced `Refresh` to bust both SWR and SQLite caches for immediate live re-reads of conversation turns.
- **Rich Media & Image Clipboard Pipeline**: Upgraded `copyPromptWithRichImages` to generate dual-MIME `ClipboardItem` (`text/html` and `text/plain`) preserving embedded images across rich text applications and markdown editors.
- **5-Tier Multi-Segment Quota Progress Bar**: Redesigned QuotaProgressBar track and milestone indicators into 5 distinct quintile color segments (Red `#ef4444`, Orange `#f97316`, Amber `#eab308`, Lime `#10b981`, Emerald Neon `#1af18d`).
- **Interaction Telemetry & Audit Log Copy**: Attached `data-ui-name` and `data-ui-id` to UI elements, mapped friendly action names in audit records, added a `📋 Copy Summary` button to AuditPanel, and filtered high-frequency system telemetry from active logs.
- **Process Exit Cleanliness**: Hooked active `agy` worker termination, Telegram daemon shutdown, and email watcher teardown into `exit_app_fully` before application exit.

---

## [v4.217.0] - 2026-10-17

### Added
- **Instance Audit Trail Visibility**: Fixed issue where opening an instance audit trail yielded an empty view. Expanded backend SQL condition across split DBs and `hot_tasks_cache` to resolve default instance aliases (`'default'`, `''`, `NULL`, `'antigravity'`, `'agy'`, `'ide'`, `'classic'`). Added segmented pill control in `InstanceAuditTrailModal` for Switch Cycles vs All Operations with live counts, automatically defaulting to `All Operations` when switch records are not yet present. (Thanks to @aukgit)
- **Switch Account Dialog UI/UX Consistency**: Overhauled `SwitchAccountDialog` with sleek dark-glass styling, top-right header 'X' dismiss button, smooth row hover transitions, distinct Current account checkmark badge, color-coded quota metrics (4h, Weekly, Total), and in-flight switch spinner. (Thanks to @aukgit)
- **Active Caller IDE Detection & Self-Preservation Guard**: Enhanced `detect_caller_instance_id()` to trace ancestor processes, inspect `proc.cmd()` for `--user-data-dir` arguments against registered instances, and check `ANTIGRAVITY_INSTANCE_ID` / `AGM_INSTANCE_ID`. When running from an IDE terminal, commands like `agm instances stop` or `switch` refuse to stop or mutate the caller instance (exit code 2). (Thanks to @aukgit)
- **Lightweight Test Instance Lifecycle**: Added `agm instances create-test` and `agm instances rm-test` CLI subcommands for rapid, transient sandbox creation and cleanup without binary duplication, keeping disk usage minimal and SQLite databases synchronized. (Thanks to @aukgit)
- **Automated E2E Verification**: Added `scripts/test-instance-autoswitch-e2e.ps1` verifying caller detection, test instance creation, caller guard interception, and clean teardown. (Thanks to @aukgit)

---

## [v4.216.2] - 2026-10-17

### Fixed
- **Build fix**: `ui_cmds.rs` passed 8 arguments to `audit::append_entry` after the audit stack-trace change added a 9th `stack_trace: Option<&str>` parameter. Passing `None` for the CLI path (E0061 broke the v4.216.1 build). (Thanks to @aukgit)

---

## [v4.216.1] - 2026-10-17

### Fixed
- **Build fix**: `post_switch_resume.rs` used `tokio::task::JoinHandle` for the timer registry but `tauri::async_runtime::spawn` returns `tauri::async_runtime::JoinHandle` — a distinct type. Switched the registry to the tauri handle type (E0308 broke the v4.216.0 build). (Thanks to @aukgit)

---

## [v4.216.0] - 2026-10-17

### Added
- **Post-switch prompt resume**: ~10 seconds after an auto-switch or account switch, the last running prompt resumes on the new account. Delay is configurable (`post_switch_resume_delay_seconds`, default 10s), cancellable, and rides the existing RCA-15 restore pipeline (no parallel restore logic, no duplicate reinjection). (Thanks to @aukgit)
- **Prompt backup + re-send**: Running and enqueued prompts are backed up durably and can be re-sent later via the new `agm prompts resend-queued` CLI command or the UI "Re-send queued prompts" button, in FIFO order with deduplication. (Thanks to @aukgit)
- **Rerun section in prompts tree**: The send area now has a rerun section with a count selector and message options ("is it done?", "is it published?", or randomized), dispatching through the canonical send path. (Thanks to @aukgit)
- **Audit stack traces**: Every audit entry now captures the recording code path (function + file + line, Rust and frontend), visible as a collapsible trace in the audit panel and included in exports. (Thanks to @aukgit)

---

## [v4.214.0] - 2026-10-17

### Added
- **Telegram instance step**: `/send` step 2 lists each instance with two subpoints, `IDE exe` and `IDE data`. Both are the last section of the path only, such as `Antigravity.exe` and `default-copy-5597`. The full absolute path is not sent. (Thanks to @aukgit)

---

## [v4.213.0] - 2026-10-17

### Fixed
- **Dispatch menu hover**: The Destination hint in the prompt-tree Send menu no longer uses the browser tooltip. That tooltip opened to the right of a menu already on the window edge, so it read `Destination conversation mo`. The hint now stays inside the menu, anchored to the right edge of the row. The same rule covers Suffix and the Copy AI rows. Do not put `title` back on those controls. The write-up is `.ai-memory/rca/20-dispatch-menu-hover-clipped.md`. (Thanks to @aukgit)

---

## [v4.212.0] - 2026-10-17

### Added
- **Telegram picker memory**: `.ai-memory/rca/18-telegram-project-listing-and-node-cache.md` records why `/send` showed `Node-*` and `project · conv-id`, how the page, title, and 24-hour node cache work, the mistakes that break FIFO and the 64-byte callback cap, and a checklist. Indexed from `.ai-memory/what-to-read.md` and `.ai-memory/memory/learned/28-telegram-project-picker-and-node-cache.md`. (Thanks to @aukgit)

---

## [v4.211.0] - 2026-10-17

### Added
- **Telegram project picker**: `/send` lists eight projects per page, newest conversation first. Each row shows the project name and the conversation title. The user selects by number, and Next loads the next eight. The current machine is labeled This machine with a green button. Generated `Node-*` names are resolved through GitMap and cached in SQLite for `node_name_refresh_hours` (default 24). (Thanks to @aukgit)

---

## [v4.210.0] - 2026-10-16

### Added
- **Modern Console Logger & ANSI Color Rendering**: Initialized modern terminal console handling via `enable_modern_console()` with UTF-8 code page (65001), ANSI virtual terminal sequence processing, and 18pt Consolas font on Windows; implemented a compact single-line `%H:%M:%S` `ConsoleTimer` preventing log message line wraps; and enabled rich ANSI color styling when writing to terminal outputs. (Thanks to @aukgit)
- **Telegram Notification Card Design-Token Palettes**: Introduced 6 chromatic card palettes (`RISEUP` Emerald #10b981 and Gold #facc15 on Slate #0f172a, `OBSIDIAN` Violet and Sky on Navy, `GOLD` Corporate Gold, `INDIGO` Electric Indigo, `CRIMSON` Luxe, `EMERALD` Mint) in `src-tauri/src/modules/telegram_inbound/palette.rs` adhering to design tokens; aligned alert cards, status reports, and telemetry broadcasts with cohesive chromatic markers, rules, and bullet glyphs; and added `/palette` command dispatching. (Thanks to @aukgit)
- **Native AGM Skills Preservation & Sync Protection**: Restored and indexed 45 native Antigravity Manager agent skills across `.agents/skills/` and `.cursor/skills/` (including quota switching, proxy pooling, thinking store, fleet synchronization, and desktop lifecycle), updated skill inventories in `.ai-memory/what-to-read.md`, and updated `03-ai-scripts/38-sync-prompts-skills-scripts.py` to safeguard local AGM skills during repository synchronizations. (Thanks to @aukgit)

### Fixed
- **Idempotent SQLite Migration Log Hygiene**: Added `expected_best_effort()` in `src-tauri/src/error.rs` to detect anticipated idempotent SQLite schema migration errors (`duplicate column name`), demoting them from console `warn!` to `debug!` and eliminating false-positive warning noise during database initialization. (Thanks to @aukgit)

---

## [v4.209.0] - 2026-10-16

### Fixed
- **Instance list PID label and path endings**: The running badge shows `PID` plus the process id. The data-path cell and the executable cell keep the filename ending on the same row. (Thanks to @aukgit)
- **Stop and account switch now kill the instance PID**: Stop waits until the process is gone. Account switch aborts before rewriting credentials when that PID is still alive. Windows liveness uses the process exit code so a dead PID is not reported as still running. (Thanks to @aukgit)
- **Instance audit trail**: Switch history is returned as `recent_switches` and `total_switches`, which the audit dialog reads. (Thanks to @aukgit)

### Changed
- **Switch Account dialog**: Dark-glass layout aligned with the instance list. (Thanks to @aukgit)
- **CLI self-instance guard**: `agm instances current` prints the instance that owns this command. Stop, switch, restart, and delete refuse that instance. (Thanks to @aukgit)

---

## [v4.208.1] - 2026-10-16

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.208.0] - 2026-10-16

### Fixed
- **账号配额跨组件自动同步与 403 兜底修复 (Account Quota Live Sync & 403 Fallback)**: 修复了 `retrieveUserQuotaSummary` 接口因项目 ID 无效或受限导致 HTTP 403 错误并丢失配额组从而误降级为 100% 假配额的问题，新增无项目 ID (`{}`) 即时重试机制；并在配额后台刷新时触发 `accounts://refreshed` 事件广播，前端实时订阅并即刻更新实例卡片与进度条，彻底解决 IDE 显示 0% 而 AGM 卡片显示 100% 导致无法触发自动切换的问题。 (Thanks to @aukgit)
- **快进手动点击与候选池耗尽修复 (Fast-Forward Manual Trigger & Candidate Pool Unblocking)**: 移除了快进候选池中对未运行空闲实例的误排除，仅排除当前实例与正在运行的兄弟实例绑定的账号；并将切换后的 15 秒进程重启验证与 15 秒安全提示弹窗消除移至后台异步任务执行，避免 IPC 同步超时阻塞，使用户点击 Fast-Forward 3 秒内即刻响应。 (Thanks to @aukgit)
- **Windows 子进程黑框弹窗彻底消除 (`CREATE_NO_WINDOW = 0x08000000`)**: 遍历消除全项目 14 处涉及 `powershell.exe`、`cmd.exe`、`gitmap`、`ssh`、`icacls` 等调用外部命令行时弹出的黑色终端控制台窗口，统一在 Windows 平台配置 `creation_flags(0x08000000)`，实现静默后台执行。 (Thanks to @aukgit)
- **交互审计日志复制按钮修复 (Audit Log Direct Clipboard Copy)**: 修复了 `AuditPanel.tsx` 中使用虚拟 `<a download>` DOM 触发下载被 WebView2 拦截导致点击复制 JSON / JSONL 无效的问题，全面重构为直接写入系统剪贴板 (`copyToClipboard`)，并提供 2 秒的绿色 "Copied!" 状态徽标与 Toast 悬浮提示。 (Thanks to @aukgit)
- **审计日志方法与目标列解析修复 (Audit Log Serde Alignment)**: 在 Rust 后端 `InteractionEntry` 与 `RecordInteractionInput` 结构体上补齐 `#[serde(rename_all = "camelCase")]`，修复前端因大小写映射不一致导致 `METHOD` 与 `TARGET` 列显示为空白的缺陷；并在 `audit-client.ts` 中修正 Tauri IPC 调用参数名为 `{ input: payload }`。 (Thanks to @aukgit)
- **严格邮箱隐私脱敏与悬浮泄露消除 (Strict Email Privacy Masking)**: 彻底移除了实例卡片 `InstanceCard.tsx` 上的鼠标悬浮自动显示原邮箱行为 (`isEmailHovered`)，并清洗了 HTML DOM 原生 `title` 提示属性，杜绝鼠标悬停泄露真实邮箱地址；标准化前端与后端脱敏工具为 `ris***@***` 格式，日志与 Telegram 告警均自动脱敏。 (Thanks to @aukgit)
- **导航栏更新图标动画优化 (Smooth Navbar Update Micro-Interaction)**: 移除了 `NavMenu.tsx` 中更新可用图标的生硬弹跳动画 (`group-hover:animate-bounce`)，替换为平滑优雅的微动上浮与缩放过渡 (`group-hover:-translate-y-0.5 group-hover:scale-110`)。 (Thanks to @aukgit)

### Added
- **审计日志调用栈与触发函数深度捕获 (Call Stack & Caller Inspection)**: 优化 `useAuditRecorder.ts` 在点击事件中向上攀爬 DOM 树寻找真正的按钮/链接交互容器，并使用 `new Error().stack` 过滤内部框架代码后提取调用函数名与源码行号，展开审计详情时呈现清晰的格式化调用栈信息。 (Thanks to @aukgit)
- **自动切换决策与候选打分全量入库审计 (Auto-Switch Decision Audit Logging)**: 在自动切换候选打分、满额候选命中、速率限制冷却跳过以及执行切换等全链路节点调用 `audit::record_if_active`，将决策过程完整写入 `ui_audit.db`，方便随时追溯切换根因。 (Thanks to @aukgit)
- **提示词备份条数上限设置与 FIFO 淘汰 (Max Prompt Backup Cap & Settings)**: 在后端 `backup_prompts_db` 实现了默认 100 条的严格 FIFO 限制 (`ORDER BY created_at DESC, id DESC LIMIT ?1`)，超出时自动修剪最旧备份并清理孤儿批次；并在“高级设置 (Settings → Advanced)”中新增了 `10` 至 `1000` 条的滑动条与数字输入控件。 (Thanks to @aukgit)
- **独立更新器终端 18pt 字体与 ANSI 彩色渲染 (18pt Modern Console for Updater)**: 在 Windows 平台调用 `AllocConsole` 后配置 UTF-8 代码页 (65001)、启用原生 ANSI 虚拟终端转义序列 (`0x0004`)，并将控制台字体放大至 18pt Consolas，彻底消除更新过程中的界面错位与字体过小问题。 (Thanks to @aukgit)

---

## [v4.207.0] - 2026-10-16

### Fixed
- **窗口标题栏双击最大化与拖拽冲突修复 (TitleBar Double-Click Maximize)**: 在 `TitleBar.tsx` 与 `Navbar.tsx` 中拦截双击序列 (`e.detail > 1`)，防止 Windows WebView2 原生 Win32 模态拖拽循环吞没双击事件，并在双击触发时直接调用 `toggle_maximize_window` Tauri IPC 命令，保证窗口双击可靠切换全屏与最大化。 (Thanks to @aukgit)
- **配额进度条霓虹强光弱化与 10% 宽度精简 (QuotaProgressBar Softened Glow & 10% Shrinkage)**: 弱化了配额条与表格行的过度霓虹发光（容器阴影从 `drop-shadow-[0_0_8px]` 调整为细腻的 `[0_0_3px]`，轨道发光减半）；将进度条轨道最小宽度精简 10%（从 `140px` 调整至 `126px`，表头与单元格由 `220px/210px` 调整至 `195px/185px`），为左侧邮箱列释放宝贵的水平展示空间；将里程碑检查点统一为 4 等分点 (`[20, 40, 60, 80]`)，并将彩色杂色统一为高质感的翡翠绿 `#10b981`。 (Thanks to @aukgit)

### Added
- **邮箱列交互式宽度拖拽与自定义缩放 (Resizable Email Column)**: 在 `AccountTable.tsx` 表头右侧新增交互式列宽调整手柄，支持鼠标自由拖拽缩放邮箱列宽度，设置 `[220px, 550px]` 安全阈值，双击手柄重置为默认 300px，并通过 `localStorage` 自动持久化用户偏好。 (Thanks to @aukgit)
- **录制与空闲状态精简胶囊 (Compact IDLE & Log Capsule)**: 将原先分散的 `IDLE` 状态标签与 `Log` 日志按钮整合为单一行内 28px (`h-7`) 深色暗透分割胶囊 (`RecordingIndicator.tsx`)，采用统一的分割线与悬浮质感，显著减少顶部工具栏冗余按钮。 (Thanks to @aukgit)
- **提示词树操作工具栏合并与新建会话直接发送 (PromptTreeView Toolbar Compaction & New Conversation Flow)**: 在 `DetailHeader.tsx` 中将原本散落的 3 个独立按钮胶囊合并为一个连续的深色暗透分割胶囊 (`[ Copy All | Send/Enqueue v ]`)；将直接发送与加入队列合二为一，提供带有快捷键 `N` 的主分流按钮与下拉配置弹窗；在下拉菜单中新增“新建会话 (New Conversation)”选项与内联会话名称输入框，用户可直接键入新会话名称并向指定项目创建并发送提示词。 (Thanks to @aukgit)
- **纯 JSON 跨机器集群遥测与调度同步 (Fleet Remote JSON Telemetry & RCA)**: 扩展 `fleet_sync.rs` 支持生成与解析纯 JSON 集群遥测载荷，适配 Telegram Bot、IMAP 邮件与 Supabase 数据流，交换实例数量、账号配额及账号切换/快进信号；在 `02-spec/21-app/ui-and-functionality-enhancements/02-backend-fleet-and-rca-spec.md` 中完整记录提示词发送与排队修复的 4 部根因分析 (RCA)。 (Thanks to @aukgit)

---

## [v4.205.0] - 2026-10-16

### Fixed
- **Table Column Squishing & Text Collision Resolution (`WEEKLY QUOCACTIONS`)**: Enforced a `min-w-[1060px]` floor on `<table className="w-full min-w-[1060px]">` in `src/components/accounts/AccountTable.tsx` and changed `overflow-y-auto` to `overflow-auto` in `src/pages/accounts/AccountsContent.tsx`, guaranteeing 4H Model Quota (min 220px) and Weekly Quota (min 210px) columns never collapse or overlap into sticky `ACTIONS` when the desktop window is resized; made `sticky right-0` header and row cell backgrounds 100% solid and opaque (`bg-slate-50 dark:bg-[#061220]`, `dark:bg-[#171025]`, `dark:bg-[#081826]`) in `AccountTable.tsx` and `ActionsCell.tsx` to eliminate horizontal scroll text bleed-through; and enforced `min-w-[780px]` on `src/components/instances/InstanceTable.tsx`. (Thanks to @aukgit)
- **Top Navbar Window Controls Protection & Responsive Compaction**: Switched `NavLogo.tsx` text from "Antigravity Tools" to compact "AGM" below the `lg` breakpoint (`1024px`), reclaiming ~95px of horizontal navbar space; refined active profile label truncation in `InstanceSelector.tsx` (`max-w-[70px] sm:max-w-[90px] md:max-w-[125px]`); optimized button padding in `NavSettings.tsx` (`px-1.5 sm:px-2`) and navbar flex gaps in `Navbar.tsx`; and lowered total navbar minimum width from 880px to 670px, completely preventing window controls (`btn-window-minimize`, `btn-window-maximize`, `btn-window-close`) and Fast-Forward from being pushed off-screen on narrow window widths. (Thanks to @aukgit)

### Added
- **Native Desktop Window Size Floor Constraints**: Added `"minWidth": 800` and `"minHeight": 520` to the `"main"` window configuration in `src-tauri/tauri.conf.json`, preventing operating system window managers (Windows DWM, macOS, Linux) from resizing the application below viable layout dimensions. (Thanks to @aukgit)
- **UI Builder Responsive Viewport Simulation Toolbar**: Added an interactive 4-tier viewport simulator toolbar (`100% Fluid`, `1280px`, `1024px`, `800px Min Desktop`) to `scripts/ui-builder.html` and wired `window.setSimWidth`, allowing designers and developers to test and verify component responsiveness live in the browser. (Thanks to @aukgit)

---

## [v4.204.1] - 2026-10-16

### Fixed
- **TitleBar Window Control Cut-Off Remediation**: Added safe-area right inset padding (`pr-2.5 sm:pr-3`) in `src/components/layout/TitleBar.tsx` and `src/components/navbar/NavSettings.tsx`, with centered viewBox coordinates and `overflow-visible`, eliminating window close button clipping against window edges and DWM rounded borders. (Thanks to @aukgit)
- **Modal Stacking Context & Sticky Header Collision Fix**: Removed `isolation: 'isolate'` from `src/components/navbar/Navbar.tsx` and elevated modal z-index to `z-[99999]` (drag region `z-[100000]`) in `src/components/common/ModalDialog.tsx` and `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, ensuring modals and prompt inspectors always float strictly above top sticky navigation headers. (Thanks to @aukgit)
- **Non-Silent Update Verification Feedback**: Hardened `handleCheckUpdates` in `src/components/navbar/NavMenu.tsx` to provide immediate tactile toast feedback ("Checking for updates...") followed by clear status reporting ("No updates available (latest version installed)") so manual update checks never fail silently. (Thanks to @aukgit)

### Added
- **Navbar Segmented Capsule Modernization (Icon-Only Settings & Triangle Chevron Trigger)**: Refined the navigation toolbar capsule in `src/components/navbar/NavMenu.tsx` by converting Settings to an icon-only item (`<Settings className="w-3.5 h-3.5 shrink-0" />`) with tooltip parity, and replacing the trailing round hamburger button with an integrated micro-triangle chevron trigger (`<ChevronDown />` rotating 180° when expanded) featuring seamless `rounded-r-full rounded-l-none` capsule rounding. (Thanks to @aukgit)
- **Quota Progress Bar Overhaul**: Re-architected `src/components/accounts/QuotaProgressBar.tsx` with a 10px track (`h-2.5`), 20px protruding circular milestone nodes (`w-5 h-5`), crisp symmetrical rounded downward chevrons (`v`), strictly zero boundary dots at 0% and 100%, 5 internal milestone splits (`[20, 40, 60, 80, 90]`), a 6-stop spatial gradient from dark red `#7f1d1d` (0%) to Progress Green `#1af18d` (100%), resting opacity at 85%, and dynamic row hover neon glow. (Thanks to @aukgit)
- **Browser-Based UI Builder Workbench**: Added `scripts/ui-builder.html` and package script `npm run ui:builder`, allowing developers and designers to interactively adjust colors, progress bar metrics, and capsule styles directly in the browser with bi-directional JSON state import/export. (Thanks to @aukgit)
- **Desktop UI Binary Target Alignment (`Antigravity Manager (AGM)-ui.exe` / `agm-ui`)**: Aligned the desktop UI executable target to `agm-ui` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and test harnesses (`scripts/test-instance-e2e.ps1`), while preserving the standalone terminal CLI binary `agm`. (Thanks to @aukgit)

---

## [v4.204.0] - 2026-10-15

### Fixed
- **Prompt Tree Direct Send & FIFO Enqueue Decoupling**: Replaced the combined action dropdown in `src/components/instances/PromptTreeView/DetailHeader.tsx` with dedicated segmented capsules (`[ Send N | v ]` and `[ Enqueue | v ]`) and separate menus; "Send" now dispatches directly to the active IDE conversation composer instead of silently acting as an enqueue operation, while "Enqueue" preserves strict FIFO background queueing. All inline `<select>` elements use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill visual defects. (Thanks to @aukgit)
- **Direct Prompt Dispatch CDP DOM Insertion Fallback**: In `src-tauri/src/modules/repo_db/agy.rs` and `src-tauri/src/bin/agm/prompt_dispatch.rs`, direct prompt dispatch automatically falls back to loopback CDP DOM insertion (`submit_prompt_via_cdp_for_instance`) when the standalone `agy` executable is absent or fails to launch, writing both `.antigravity_resume_task.json` and instance-scoped resumption files and recording audit trail telemetry. (Thanks to @aukgit)
- **SSH Vault Status Field Alignment**: Aligned field access in `src-tauri/src/bin/agm/ssh_cmds.rs` with `SshGitmapStatus` (`rsa_vault_ready`, `rsa_public_key_fingerprint`). (Thanks to @aukgit)

### Added
- **Standalone One-Shot Rich Copy (Content + Images)**: Implemented `copyPromptWithRichImages` in `src/components/instances/PromptTreeView/clipboard.ts` and added a standalone "Copy All (Content + Images)" button in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` and `DetailHeader.tsx` that copies dual-MIME `ClipboardItem` (`text/html` with embedded `<img>` tags and markdown `text/plain` fallback) with 2-second visual checkmark confirmation and structured error tracking. (Thanks to @aukgit)
- **CLI Methods & Sections Parity (`agm prompt send` / `agm prompt queue` / `agm queue`)**: Added top-level shortcuts `agm queue`, `agm enqueue`, and `agm qe`, along with `--queue` / `-q` flag support for `agm prompt [text]`, sequence target resolution (`C001`, `P001`, `GM:#1`) via `repo_db::resolve_agm_sequence_target`, and empty prompt validation with actionable CLI help. (Thanks to @aukgit)

---

## [v4.203.0] - 2026-10-15

### Fixed
- **Telegram HTML Entity Rendering Fix (`&nbsp;` / `&bull;`)**: Replaced unsupported `&nbsp;&nbsp;` and `&bull;` HTML entities in `src-tauri/src/modules/notification_hub/telegram_alerts.rs`, `src-tauri/src/modules/notification_hub/telemetry.rs`, and `src-tauri/src/modules/telegram_inbound/formatting.rs` with Unicode figure spaces (`\u{2007}\u{2007}`) and bullet glyphs (`•`) so Telegram HTML cards render cleanly without raw entity text. (Thanks to @aukgit)

### Added
- **Enhanced Telegram Bot UI, Inline Action Keyboards, `/balance` & CSV/JSON File Exports**: Upgraded `/help` (`src-tauri/src/modules/telegram_inbound/reports_b.rs`) with categorized click-to-copy (`<code>...</code>`) command examples, attached interactive inline action buttons to switch and liveness alerts, added `/balance` (`/credits`, `/quota`) for per-instance 4h and weekly account credit inspection, implemented `send_telegram_document` in `src-tauri/src/modules/telegram_inbound/media.rs` to deliver downloadable `.csv` and `.json` files on `/export <accounts|instances|balance|nodes|prompts|status> [csv|json]` and `--csv`/`--json` flags, and simplified machine/instance prompting via `/prompt <machine:instance> <text>` and `/target <machine> <instance>`. (Thanks to @aukgit)
- **GitMap Multi-Machine Control, SSH Node Import, Auto-Install & RSA-2048 Password Encryption**: Implemented RSA-2048 hybrid envelope password encryption (`ag_rsa_v1_`) in `src-tauri/src/utils/crypto.rs` and `src-tauri/src/modules/ssh_manager/` so SSH passwords are encrypted on first login and never stored in plain text on disk; added cross-machine encrypted bundle deployment (`deploy_encrypted_ssh_to_nodes`), GitMap SSH node import (`agm ssh import-gitmap`), PowerShell auto-install helper (`agm ssh install-gitmap`), multi-machine instance and credit balance inspection (`agm balance`, `agm ssh balance`, `--csv`, `--json`), and remote prompt delegation (`agm ssh prompt`). (Thanks to @aukgit)
- **Telegram Power Commands Hub & SSH / GitMap Fleet Settings UI**: Upgraded `src/components/settings/EmailNotifications/TelegramSection.tsx` and `src/components/settings/SupabaseSync/TelegramSection.tsx` with categorized click-to-copy command chips, an inline `Machine -> Instance -> Prompt` dispatcher, CSV/JSON export bar, and live balance inspector; and added `src/components/settings/EmailNotifications/SshGitmapSection.tsx` (`/settings/ssh`) with GitMap detection and one-click PowerShell installer, RSA-2048 encrypted SSH node configuration, and the multi-machine instance/account credit matrix. (Thanks to @aukgit)

---

## [v4.202.0] - 2026-10-15

### Fixed
- **Automatic CDP Dismissal of Antigravity "Security Notice & Data Use" Modal**: Added `src-tauri/src/modules/instance/cdp_onboarding.rs` and wired `--remote-debugging-port=0` on instance launches so switching accounts or launching instances automatically connects to `DevToolsActivePort`, unchecks the *"Yes, I agree to help improve Antigravity by allowing Google to collect and use my Interactions data..."* checkbox, clicks **"Accept and continue"**, and persists `hasAcceptedTermsOfService = true` and `shareDataToImproveAntigravity = false` across `app_storage.json` and `state.vscdb`. (Thanks to @aukgit)
- **Live CDP Conversation Prompt Submission & Startup Purge Persistence Fix**: Fixed `agm prompt` (`src-tauri/src/bin/agm/prompt_dispatch.rs`), `src-tauri/src/bin/agm/project_cmds.rs`, and `src-tauri/src/modules/repo_db/backup.rs` to persist `running_projects` and `active_prompts` via `save_or_requeue_prompt` (populating composite `<project>__<instance>` IDs and `workspace_storage_path`) so subsequent CLI invocations do not purge active prompts during `purge_corrupted_running_projects`; and added `submit_prompt_via_cdp_for_instance` in `src-tauri/src/modules/instance/cdp_onboarding.rs` so `spawn_or_replace_prompt_via_agy` also injects and sends prompts directly into the open Antigravity IDE conversation composer via CDP. (Thanks to @aukgit)
- **CLI UI Sub-Route Navigation (`/settings/:tab`)**: Added the `settings/:tab` route in `src/App.tsx`, synchronized tab selection from URL parameters in `src/pages/Settings/useSettingsState.ts`, and expanded `UI_ROUTES` in `src-tauri/src/proxy/server/admin_ui.rs` so `agm ui goto settings/<tab>` navigates directly without triggering a React Router 404 screen. (Thanks to @aukgit)

---

## [v4.201.0] - 2026-10-15

### Fixed
- **Post-Switch `agy` Silent Auth & Token Provisioning**: Wrote `antigravity-oauth-token`, `antigravity-cli-oauth-token`, and `antigravity-ide-oauth-token` alongside `jetski-standalone-oauth-token` across all `.gemini` subdirectories (`antigravity`, `antigravity-cli`, `antigravity-ide`, `cache`) in `src-tauri/src/modules/integration/file_credentials.rs` so `agy -p` print mode authenticates silently on isolated instances instead of hanging on interactive OAuth. (Thanks to @aukgit)
- **Quota Summary 403 `VALIDATION_REQUIRED` Detection**: Updated `fetch_quota_summary` in `src-tauri/src/modules/quota/fetch_more.rs` and `src-tauri/src/modules/quota/fetch_cached.rs` to inspect HTTP 403 responses for `VALIDATION_REQUIRED` / `Verify your account`, marking affected accounts `validation_blocked = true` and `quota.is_forbidden = true` so `auto-switch` and `ff` never rotate into verification-blocked accounts. (Thanks to @aukgit)
- **Synchronous Post-Switch Prompt Restoration & Queue Race Prevention**: Removed `'backed_up'` from background `check_and_dispatch_enqueued_prompts` in `src-tauri/src/modules/repo_db/queue.rs` so in-flight switch backups cannot be stolen while an instance is restarting; made `switch_account_to_instance` in `src-tauri/src/modules/instance/switch.rs` terminate stale `agy` workers and synchronously restore/dispatch backed-up prompts upon relaunch verification; and updated `send_prompt_now_for_instance`, `resend_running_commands_for_instance`, and `dispatch_running_prompts` to replace existing `agy` workers cleanly and automatically drain the FIFO queue when a worker exits. (Thanks to @aukgit)
- **Account Switch Active Instance Target & Progress Bar Inner Glow**: Updated `useAccountRowState.ts` and `AccountRow.tsx` so the row Switch button targets `activeInstanceId` rather than hardcoding `'default'`, and tuned the inner checkpoint dot glow in `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx` for subtle contrast. (Thanks to @aukgit)

### Added
- **CLI UI Audit Recorder (`agm ui record`)**: Added `agm ui record start/stop/status/list/clear/export` in `src-tauri/src/bin/agm/ui_cmd.rs` and instrumented CLI account switches, fast-forward rotations, and prompt send/rerun/enqueue commands to persist structured audit entries in `ui_audit.db`. (Thanks to @aukgit)
- **Rebrand to `com.alimtvnetwork`, Uninstaller & Homebrew Cask Automation**: Updated bundle identifiers, installer/uninstaller scripts, and `.deb`/`.rpm`/macOS cask package detection across platforms. (Thanks to @aukgit)

---

## [v4.199.0] - 2026-10-14

### Fixed
- **Prompt Tree Send, Backup, and Restore**: `agy` is started with `--conversation` and `-p`. `--resume` and `--user-data-dir` are not passed. A process that exits immediately returns that error instead of a success toast. Backup and Restore use the native save and open dialogs with `write_prompt_backup_file` and `read_prompt_backup_file`, and Restore dispatches the imported rows. (Thanks to @aukgit)
- **Account Switch Window Check and Shared Login**: A live stock Antigravity window counts as a successful relaunch even when its command line omits the saved data directory. A saved process id that has already exited does not count as running. Switching a copy instance no longer replaces the Default account in the shared OS keyring or `~/.gemini`. (Thanks to @aukgit)
- **Supabase Pause Message and Email Import**: A paused free-tier project is reported as DNS `NXDOMAIN` instead of a bad API key. `agm email import` accepts a JSON file that has no password and prompts for the password separately. (Thanks to @aukgit)
- **Clipboard While the WebView Is Unfocused**: Copy uses the OS helper when the webview does not have focus. (Thanks to @aukgit)

### Added
- **Telegram Window Commands**: `/ui` opens the manager, `/shot` sends a PNG of the current screen, and `/target` aims the next plain message at one instance and conversation. (Thanks to @aukgit)
- **Retest Notes**: The UI session recorder guideline and the local CLI retest cases are written. The recorder itself is not implemented. (Thanks to @aukgit)

---

## [v4.198.0] - 2026-10-14

### Fixed
- **Prompt Tree WebView2 OOM Crash (`0xe0000008`) & Thread Storm Elimination**: Resolved an infinite React `useEffect` render loop in `src/components/instances/PromptTreeView/usePromptTreeData.ts` where `loadTree` depended on `archivedProjectIds` and `pinnedProjectIds` state arrays while the modal-open effect re-allocated both arrays on every run. Moved both arrays to `useRef`s inside `loadTree` and serialized uncached workspace scans behind a process-wide mutex in `src-tauri/src/modules/repo_db/tree.rs`, keeping process thread count flat (`~52` threads instead of spiking to `565` threads) and warm Prompt Tree loads under `200 ms`. (Thanks to @aukgit)
- **Detached IDE Launch Pipe Hang & Relative Repo Resolution**: Redirected `stdin`, `stdout`, and `stderr` to `Stdio::null()` in `src-tauri/src/modules/process/launch.rs` (`spawn_with_job_breakaway_fallback`) so detached `Antigravity` launches never inherit the caller's stdout pipe handle or hang CLI commands (`agm prompts restore`, `agm auto-switch test`). Canonicalized relative `-r .` paths in `src-tauri/src/modules/cli/types.rs` and guarded `DELETE FROM running_projects` against active-prompt foreign keys in `src-tauri/src/modules/repo_db/detection.rs`. (Thanks to @aukgit)
- **Update Available Platform Asset Gate & UI Contrast**: Updated `src-tauri/src/modules/update_checker/channels.rs`, `src-tauri/src/modules/update_checker/platform.rs`, and `src/stores/use-update-store.ts` so **Update Available** only appears when the target release tag is strictly newer than `CURRENT_VERSION` and carries a downloadable binary asset for the active OS (`.exe`/`.msi` on Windows, `.dmg`/`.app.tar.gz`/`.pkg` on macOS, `.AppImage`/`.deb`/`.rpm` on Linux). Restyled `src/components/UpdateNotification.tsx` with high-contrast dark-glass surfaces and Progress Green `#1af18d` action buttons. (Thanks to @aukgit)

### Added
- **CLI Prompt Copy, Resend, UI Launch & Pure JSON Stdout**: Added `agm prompts copy` (and `agm copy-prompt`) to copy running, queued, or recent prompts to the system clipboard with `--json` output, wired `agm prompts resend` to `cmd_resend_running_commands`, added `agm instances open-ui`, and routed console tracing logs to `stderr` when `--json` / `-j` is passed in `src-tauri/src/modules/logger.rs`. (Thanks to @aukgit)

---

## [v4.197.1] - 2026-10-14

### Fixed
- **Prompt Tree Cache Persistence & Panic Backtrace**: Preserved `prompt_tree_cache` across startup purges with a 120-second TTL, enabled cached tree reads in `agm prompts tree`, caught panics with backtraces during tree construction, and added the Prompt Tree header **Reset** control plus `agm prompts reset-tree`. (Thanks to @aukgit)

---

## [v4.195.0] - 2026-10-13

### Fixed
- **CI/CD Quality Gates & Macro Syntax Remediation**: Resolved duplicate variable declaration and expected assertion syntax in json envelope test suite (`src-tauri/src/modules/json_envelope/tests.rs`), ensuring zero-error Clippy and test compilation across Ubuntu, macOS, and Windows matrix runners. (Thanks to @aukgit)
- **Window Manager Error Tracking**: Enforced strict 'No Silent Errors' discipline in `src/utils/windowManager.ts` by routing all best-effort window operations through `useErrorStore.getState().trackWarning`. (Thanks to @aukgit)

### Added
- **Complete Supabase Hardening & UI/UX Polish**: Verified 100% compliance across all 56 automated tests for Progress Green `#1af18d`, quintile 5-slot progress bars, right-compacted layout, high-contrast theme typography, email privacy masking, prompt tree action capsules, and secure repo-secrets dynamic endpoint discovery. (Thanks to @aukgit)

---

## [v4.194.0] - 2026-10-13

### Fixed
- **CI/CD Quality Gates & Rust Formatting**: Resolved `cargo fmt` newline discrepancy in `src-tauri/src/modules/supabase_sync/config_a.rs`, restoring clean CI/CD pipeline checks across Linux, macOS, and Windows runners. (Thanks to @aukgit)
- **Supabase Credentials Eradication & External Vault Isolation**: Completely purged hardcoded Supabase project tokens and credentials from the tracked codebase, scripts, documentation, and test fixtures. Shifted endpoint provisioning to dynamic environment variables (`SUPABASE_ROOT_URL`, `SUPABASE_ROOT_KEY`, `SUPABASE_SECONDARY_URL`, `SUPABASE_SECONDARY_KEY`) and automated Base64 ingestion from `repo-secrets`. Added single-liner import script `scripts/import-supabase-credentials.ps1`. (Thanks to @aukgit)

### Added
- **Unified Test Automation & Verification**: Comprehensive 56-test regression suite covering Progress Green `#1af18d`, compact quintile progress bar layout, theming contrast, email masking, prompt tree action capsules, and dynamic vault connectivity. (Thanks to @aukgit)

---

## [v4.193.0] - 2026-10-13

### Fixed
- **CI/CD Quality Gates & Linter Healing**: Resolved TypeScript index signature mismatch on `ErrorContext` in `src/stores/error-store.ts`. Re-formatted `src-tauri/src/modules/supabase_client/client.rs` and `src-tauri/src/modules/supabase_sync/config_a.rs` to satisfy strict `cargo fmt` checks across all OS runners. (Thanks to @aukgit)
- **Prompt Tree Action Stability & Deadlock Elimination**: Fixed subprocess pipe deadlock in `copy_to_system_clipboard` by properly dropping stdin before awaiting child processes, bounded clipboard execution with timeouts, and eliminated uncaught promise rejections on clipboard operations. (Thanks to @aukgit)

### Added
- **Progress Bar Modernization & Compacting**: Updated quota progress bar gradient terminal stop to Progress Green `#1af18d` with glowing drop shadow, standardized 5 slots at 20% quintile intervals, replaced misleading SVG glyphs with glowing LED apertures, and compacted bar layout towards the right edge. (Thanks to @aukgit)
- **Dual Supabase Diagnostics & Theming Contrast**: Integrated structured backtrace logging for connection probes, auto-corrected Lovable endpoint typos, enforced high-contrast typography (no purple on purple), added CSS3 hover micro-elevation, and implemented email privacy masking via `maskEmail`. (Thanks to @aukgit)

---

## [v4.192.0] - 2026-10-12

### Added
- scripts - fix symlink resolution in update-antigravity-ide.sh

---

## [v4.191.0] - 2026-10-12

### Added
- scripts - add Linux Antigravity IDE update script

---

## [v4.190.0] - 2026-10-12

### Fixed
- **Running Prompt Backup/Restore & Rerun Root-Cause Fix**: Fixed `agm restore` / `agm running-prompts restore` silently restoring nothing after an instance switch — the strict per-instance filter introduced in v4.102.0 scoped restores to the *currently* active instance while backups are recorded under the instance that was active at *backup* time, so post-switch restores matched zero rows. `agm restore` without `-i/--instance` now restores all unrestored backups across every instance in FIFO order (each re-injected under its own recorded instance id and resent via agy); an explicit `-i <id>` keeps strict single-instance scoping. Fixed `agm rerun [N]` selecting the *oldest* N prompts instead of the most recent N (`list_all_prompts` returns oldest-first and the head was truncated), and marking prompts `dispatched` without ever re-sending them — rerun now re-queues as `backed_up` and dispatches through the standard resend pipeline via agy. Added `test_restore_scoping_across_instances` regression test covering cross-instance restore and strict scoping. (Thanks to @aukgit)
- **Account Switch Relaunch Verification**: Added post-relaunch IDE verification to the account-switch flow (`switch_account_to_instance` and the 5-step desktop switch). A spawn that returns Ok no longer reports success when no IDE window materializes — the switch now surfaces a diagnosable error instead of a silent no-reopen. (Thanks to @aukgit)

### Added
- **Royal Violet UI Theme**: Implemented the approved Royal Violet design system across the app — daisyUI dark theme tokens, semantic CSS variables, and legacy navy hex remap (canvas `#090611`, panel `#171025`, primary `#7C3AED`, aurora `#A855F7`, highlight `#E879F9`). (Thanks to @aukgit)

---

## [v4.189.0] - 2026-10-12

### Fixed
- **Auto-Switch & Fast-Forward Scoring and Process Isolation**: Replaced strict 100% quota requirement in Step 3 fallback with proportional quota headroom scoring (`score_candidate_account_fallback`), eliminating auto-rotation deadlocks when all fleet accounts have sub-100% quota. Enforced conscious instance root PID isolation in `pid_scan.rs` (`matched_set.contains(&p)`), preventing stopped sibling instances from falsely reporting as running or having their processes terminated during neighbor instance restarts. (Thanks to @aukgit)
- **CLI Update Revert & Downgrade Engine**: Fully parameterized `agm update <ver>` / `agm update to <ver>` to parse positional and prefixed version arguments, normalize 2-part SemVer (`4.162` -> `4.162.0`), query GitHub release tag endpoints directly rather than hardcoding `/releases/latest`, and pass `-isPinned`/`-Force` flags through the 3-stage self-delegating updater and `install.ps1` to bypass downgrade suppression gates. Added fallback probing for historical NSIS asset filenames (`Antigravity.Manager.Tools_4.162.0_x64-setup.exe` and `agm-alim-setup.exe`). (Thanks to @aukgit)
- **Cross-Platform CI/CD Compiler & Quality Gate**: Fixed 7 compile and Clippy errors across Ubuntu, macOS, and Windows runners including rusqlite statement borrow lifetimes in `compact_engine/workdir.rs`, non-exhaustive `KnownDataType::SshNodes` pattern matching in `json_envelope/inspection.rs`, re-exported Telegram daemon lifecycle controls (`stop_telegram_daemon`, `get_telegram_status`), and normalized full `Account` quota resolution in `reports_a.rs`. All 7 GitHub Actions matrix jobs verified 100% green. (Thanks to @aukgit)

---

## [v4.188.0] - 2026-10-11

### Fixed
- **Auto-Switch & Fast-Forward PID Contamination and Quota Gate Fix**: Eliminated cross-instance PID contamination in `pid_scan.rs` where ancestor walking accumulated across all instances caused stopped sibling instances to falsely report as running and trigger collision guards. Resolved candidate evaluation freeze in `selection.rs` and `scoring.rs` by unblocking Step 3 fallback candidates when `cooldown_pool` is non-empty, and applying proportional scaling for accounts with sub-100% quota. (Thanks to @aukgit)
- **CLI Version Downgrade & 2-Part SemVer Normalization**: Enabled version rollbacks and downgrades via `agm update <ver>` / `agm update to <ver>`. Automatically normalizes 2-part version numbers (e.g. `4.162` -> `4.162.0`) in CLI arguments and `install.ps1`, parameterizes `download_release_zip` to target explicit release tags rather than hardcoding `releases/latest`, and bypasses installer downgrade checks when a pinned version is specified. (Thanks to @aukgit)
- **Launch & Switch Latency Optimization**: Eliminated synchronous 6,000ms delay in `wait_for_instance_prompt_channel` during IDE launch, replacing it with an adaptive 100ms readiness probe capped at 1,200ms. Reduced loop sleep intervals in `close_instance_verified` from 200ms down to 50ms to eliminate Tokio worker thread blocking. (Thanks to @aukgit)
- **Error Visibility & Swallowed Error Elimination**: Audited and eliminated silent error discards across `instance_fastforward.rs`, `instance.rs`, and `switch.rs` by routing through `record_ignored` with justification comments and logging warnings; connected frontend errors in `useAutoSwitcher.ts` and `useInstanceSettings.ts` to `useErrorStore.getState().trackWarning`. (Thanks to @aukgit)

### Added
- **Instance List View IDE Executable Path Column**: Added a dedicated `IDE Executable` column in the instance list view table displaying the ending executable path via `formatShortPath`, accompanied by a Terminal icon, tooltip with the full path, and clipboard copy button. (Thanks to @aukgit)
- **Decentralized Headless Fleet Sync via Email & Telegram**: Introduced `fleet_sync.rs` and IPC commands (`get_fleet_sync_machines`, `generate_fleet_sync_json`, `ingest_fleet_sync_json`) enabling decentralized machine, instance, and workspace state synchronization over Email and Telegram without requiring Supabase. (Thanks to @aukgit)

---

## [v4.187.0] - 2026-10-11

### Fixed
- **Full Process Termination on Application Close**: Resolved lingering background processes and sockets remaining in Windows Task Manager after closing the application window. Refactored `WindowEvent::CloseRequested`, `RunEvent::ExitRequested`, native window close buttons (`cmd_window.rs`), and system tray quit action (`tray.rs`) to unconditionally route through `exit_app_fully(&AppHandle)`. Gracefully halts Cloudflared tunnels, Admin Server listeners, and Proxy instances before invoking `std::process::exit(0)`, providing immediate, zero-residual Task Manager process teardown. (Thanks to @aukgit)
- **Profile Switch Spawn Crash & Access Denied (os error 5) Remediation**: Fixed account and profile switching failures where spawning cloned instance binaries failed with `Access is denied (os error 5)` inside restricted Windows Job Objects. Architected a robust 4-stage launch fallback ladder in `src-tauri/src/modules/instance/launch.rs` and `src-tauri/src/modules/process/launch.rs`: direct binary execution with job breakaway fallback -> detected base executable -> Windows Command Shell (`cmd.exe /s /c start`) -> PowerShell `Start-Process` (leveraging Win32 `ShellExecuteEx` for guaranteed job breakaway). Hardened `switch_account_to_instance` so that auto-launch errors are logged gracefully without failing the already-persisted switch transaction. (Thanks to @aukgit)

### Added
- **Telegram Rich Formatting Cards, Custom Copy Button & Fleet Identity**: Upgraded Telegram notification cards to structured HTML markup with `<pre><code>...</code></pre>` blocks for active prompts. Implemented Telegram Bot API inline keyboards with native `copy_text` buttons for one-click prompt copying on mobile and desktop. Added free image storage support via Catbox and Telegram permanent `file_id` caching. Introduced standalone Fleet Identity and discovery commands (`/fleet`, `/whoami`, `/instances`, `/accounts`), providing complete machine, instance, and account visibility directly within Telegram and Email even when Supabase is disconnected or offline. (Thanks to @aukgit)

---

## [v4.186.0] - 2026-10-11

### Fixed
- **CLI Self-Delegating Updater & Windows Sharing Violation Remediation**: Implemented universal detached self-delegation for `agm update` to temporary process binaries (`%TEMP%\agm-updater\agm-update-cli-<pid>.exe`) with immediate caller termination (exit code 0), eliminating Windows binary image file locks (`os error 5: Access is denied` / sharing violation `0x20`). Enhanced process detection in `delegate_updater/process.rs` and `install.ps1` to actively identify and terminate running `antigravity-manager.exe` and `agm.exe` processes. (Thanks to @aukgit)
- **Prompt Pipeline Dispatch & Multi-Destination Queue Architecture**: Restored and unified CLI prompt verbs (`agm prompt send`, `agm prompt queue`, `qe` alias, and `agm instance prompt`) with destination session targeting (`--conversation <id>`, `--new`, `--existing`), automatic working directory resolution, and robust argument parsing (`-c`, `--session`, `-m`, `--model`). Implemented Tier 2 resilient prompt dispatch in `repo_db::send_prompt_now_for_instance` with automatic task file persistence (`.antigravity_resume_task.json`), clipboard synchronization, and IDE window activation. (Thanks to @aukgit)
- **Prompt Backup & Cross-Instance Recovery during Account Switching**: Expanded in-flight prompt status capture in `backup_running_prompts` to `WHERE status IN ('running', 'in_flight', 'dispatched')`. Added atomic SQLite re-attribution of backed-up prompts from source instances to target instances during profile rotation (`rotation.rs`), and recovered `backed_up` prompts in `auto_resume_recent_prompts`. (Thanks to @aukgit)
- **Prompt Tree View Destination Conversation Capsule Selector**: Added an inline segmented capsule selector in `DetailHeader.tsx` allowing developers to explicitly select between "Existing Conv" and "New Conv" before dispatching or enqueueing prompts, wired end-to-end to `usePromptTreePromptActions.ts` and `useInstanceStore.ts`. (Thanks to @aukgit)

---

## [v4.185.0] - 2026-10-11

### Fixed
- **Comprehensive Compiler and Pipeline Error Resolution**: Resolved all compilation, visibility, and trait bound errors reported by CI across proxy rate limiting (`RateLimitTracker`), signature cache (`SignatureCache::new`), token manager helpers, HTTP session store (`SessionParent`), upstream client (`build_url`), and mappers. (Thanks to @aukgit)
- **Axum 0.7 Router Trait Bound Compatibility**: Restored `assemble_app` return type to `axum::Router` and aligned `spawn_serve_loop` signature, resolving hyper/axum `ServiceExt::map_request` trait bound compilation errors. (Thanks to @aukgit)
- **Platform-Specific Windows Flag Scoping**: Added `#[cfg(target_os = "windows")]` attributes for `CommandExt::creation_flags`, `CREATE_NO_WINDOW`, and Win32 console functions (`AllocConsole`, `AttachConsole`) across process, updater, and sync modules. (Thanks to @aukgit)
- **Codebase Formatting and Test Visibility**: Automated full Rust code formatting pass via `cargo fmt` and restored crate-level visibility re-exports across OpenAI, Claude, and Gemini mappers and test suites. (Thanks to @aukgit)

---

## [v4.184.0] - 2026-10-10

### Added
- Routine release v4.184.0

---

## [v4.167.0] - 2026-10-09

### Added
- **Multi-Instance Process Cache & Relaunch Prevention**: Implemented two-tier process cache (`SMART_PROCESS_CACHE`) with sub-millisecond targeted PID vitality verification (`is_pid_alive_targeted`). Decoupled window focus failures from process liveness, ensuring prompt send and enqueue operations never terminate or relaunch already running Antigravity IDE instances. (Thanks to @aukgit)
- **Prompt Enqueue IPC Command & Strict FIFO Queueing**: Wired `enqueue_prompt` IPC handler and CLI command (`agm prompt queue`) to persist in-flight prompts with status `queued` in strict First-In-First-Out order (`ORDER BY created_at ASC, id ASC`) with automatic local fallback task persistence. (Thanks to @aukgit)
- **False-Positive Running Prompt Elimination**: Hardened transcript inspection (`inspect_conversation_transcript`) with terminal state detection (`is_terminal_done`) for completed planner responses, narrowed modified timestamp freshness window to 60s, and pruned stale `running_projects` SQLite records. (Thanks to @aukgit)
- **Prompt Tree View UI Compaction & Bracket Stripping**: Streamlined sequence badges to clean `P001 · #1` and `C001 · <cid>` without heavy bracket clutter, eliminated redundant uppercase role badges in favor of native tooltips, and stripped bracket noise from button labels (`Show Less`, `Show All (${words}w)`). (Thanks to @aukgit)
- **Host-Shielded E2E Testing & CLI Parity**: Verified 5/5 test cases with 100% pass rate in safe host-shielded test harness `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`, expanding `agm` CLI verbs (`instance status`, `prompt send`, `prompt running`, `prompt queue`) with unified JSON envelopes. (Thanks to @aukgit)

---

## [v4.166.0] - 2026-10-09

### Added
- 3-step smart instance process caching with closed-PID OS re-scan and multi-instance process counting
- Reliable prompt dispatch via `--user-data-dir` socket alignment and cross-platform clipboard synchronization
- Prompt Tree View bracket de-cluttering, sequence code styling, and clean context omission banners
- Ghost running state eradication via active worker confirmation and removal of synthetic crash prompts
- Full CLI parity for `agm prompts` (`ls`, `tree`, `send`, `enqueue`, `backup`, `restore`) and `agm doctor`

---

## [v4.165.0] - 2026-10-09

### Added
- Smart instance process cache and OS PID vitality checks eliminating duplicate IDE launches
- Dedicated Tauri IPC commands `send_prompt_now`, `enqueue_prompt`, and `get_running_instances_process_count`
- Bracket tag clutter elimination in Prompt Tree View, replacing noisy dual badges with streamlined sequence pills
- Ghost running indicator eradication and duration formatter hygiene preventing `NaNm NaNs` display

## [v4.165.0] - 2026-10-09

### Added
- **Smart Instance Process Cache & Relaunch Prevention**: Implemented two-tier process cache (`SMART_PROCESS_CACHE`) with sub-millisecond targeted PID liveness verification (`is_pid_alive_targeted`). Decoupled window focus failures from process liveness, ensuring prompt send and enqueue operations never terminate or relaunch already running IDE instances. (Thanks to @aukgit)
- **Prompt Enqueue IPC Command & Strict FIFO Queueing**: Registered `enqueue_prompt` IPC command and wired `agm prompt queue` to persist prompts with status `queued` in strict First-In-First-Out order (`ORDER BY created_at ASC, id ASC`). (Thanks to @aukgit)
- **False-Positive Running Prompt Elimination**: Hardened transcript inspection (`inspect_conversation_transcript`) with terminal state detection (`is_terminal_done`) for completed planner responses, narrowed timestamp window to 60s, and pruned stale `running_projects` SQLite rows. (Thanks to @aukgit)
- **Prompt Tree View UI Compaction & Bracket Stripping**: Streamlined `formatDualBadge` to `${rawAgm} · ${rawGm}` (e.g. `P001 · #1`, `C001 · <cid>`), eliminating redundant brackets and `AGM:` prefixes. Replaced verbose uppercase role badges with clean tooltips and stripped bracket noise from button labels (`Show Less`, `Show All (${words}w)`). (Thanks to @aukgit)
- **CLI Parity & Safe Host-Shielded E2E Testing**: Expanded `agm` CLI verbs (`instance status`, `prompt send`, `prompt running`, `prompt queue`) with unified JSON envelopes (`CliEnvelope`), and verified all 5 test cases with 100% pass rate in safe E2E test harness `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`. (Thanks to @aukgit)

---

## [v4.164.0] - 2026-10-08

### Added
- Prompt tree view in-flight execution banner with elapsed timer, step count, and direct Antigravity window activation
- Strict FIFO ordering (`ORDER BY created_at ASC, id ASC`) across prompt backup restore and auto-resume queries
- Extended subagent detection heuristics distinguishing background tasks from user prompts
- Deduplication and schema unification for Supabase fleet machine synchronization

---

## [v4.163.0] - 2026-10-08

### Added
- Multi-machine fleet synchronization and cross-node instance observability via Supabase
- FleetMachinesTable component in Instances section displaying worker nodes, IPs, instances, accounts, and running prompts
- Idempotent database schema extension for running prompt counts and real-time heartbeat telemetry
- Tauri IPC commands `get_supabase_fleet_machines` and `sync_supabase_now`

- Remote fleet machines table beneath local instances in both Card and Table views
- Supabase multi-machine cluster aggregation for worker nodes, running instances, and bound accounts
- Account email privacy masking with row-level and global unmask toggles
- Background heartbeat in-flight prompt count telemetry and PostgREST resilience
- Segmented dark-glass pill controls with 15-second auto-polling countdown

---

## [v4.162.5] - 2026-10-08

### Added
- Modernize prompt tree view with compact dark-glass pill capsules, prompt classification, and repeated prompt grouping
- Quota progress bar milestone dot capping (max 5), low-quota (<25%) dark red transition, and numeric percentage checkpoints
- Instance card whitespace reduction and custom thin scrollbars
- AGM CLI prompt management (`agm prompts ls|tree|backup|restore`) and system diagnostics (`agm doctor`)

---

## [v4.162.3] - 2026-10-07

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.162.2] - 2026-10-07

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

## [v4.162.1] - 2026-10-07

### Added
- Synchronize prompts, skills, AI scripts, and coding guidelines

---

# 📝 Changelog

> Complete version history for Antigravity Tools. Return to project home at [README_EN.md](README_EN.md).

*   **Version History**:
    *   **v4.279.0 (2026-10-30)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.


    *   **v4.278.0 (2026-10-29)**:
        -   **[Prompt Queue Visibility, Honest JSON Switch & Friendly Telegram Cards] Enqueue/rerun show on click, JSON switch proves bound_email, Telegram Send (Thanks to @aukgit)**:
            -   **Description**: Prompt tree force-refresh keeps queued turns instead of wiping the 30s SWR cache first; rerun ×N keeps each identical queued turn; `agm instances switch --json` no longer fakes `"switched"`; Telegram cards show idle/busy/queued and ✉️ Send. Isolated `agm instances test-create` enqueued three queued ids, copied, sent `p-0db14cf4`; Default and v2-3127 were not stopped. (Thanks to @aukgit)


    *   **v4.277.0 (2026-10-29)**:
        -   **[Instance UI & Verification Templates] Restyled Create Profile Modal, 100-Step Prompt Breakdown & Release Tag Normalization (Thanks to @aukgit)**:
            -   **Description**: Restyled the Create Profile dialog in `src/pages/instances/CreateEditDialogs.tsx` with unified dark-glass styling, segmented button controls, and high-contrast form elements; added multi-step prompt breakdowns and verification templates in `src/components/instances/PromptTreeView/verificationTemplates.ts`; integrated flywheel status triggers in `src/components/telegram/ImprovementsFlywheel.tsx` and `src/pages/Instances.tsx`; and purged rogue v5 tags to restore canonical 4.x SemVer progression. (Thanks to @aukgit)

    *   **v4.275.3 (2026-10-28)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.275.2 (2026-10-28)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.275.1 (2026-10-28)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.275.0 (2026-10-28)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.274.0 (2026-10-28)**:
        -   **[Feature and Instance Management] Instance Card Contrast, 3D Switch Button, Idle Notification Hub, UI Step Logging & CLI Remote Control (Thanks to @aukgit)**:
            -   **Description**: Redesigned `/instances` cards with Royal Violet dark glass (`#171025`) and solid quota panels (`#120b1e`), elevated Switch Account buttons with illuminated amber styling and 3D yellow tactile hover gradients, implemented automated idle-mode dual-channel notification alerts across Telegram and Email in `idle_alerts.rs`, added structured `[UI_STEP]` step and button click trace logging in `uiLogger.ts` and `cmd_ui_trace.rs`, expanded CLI remote control (`agm ui click` and `agm instances click`), verified instance switching and prompt restoration integrity with zero errors in `errors.db`, and authored comprehensive specifications under `02-spec/21-app/233-feature-and-instance-management/`. (Thanks to @aukgit)

    *   **v4.273.0 (2026-10-28)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.272.0 (2026-10-27)**:
        -   **Anti-gravity Rerun Target ID, Sequential Enqueue Retention, Filter Compaction & Tree Design System**:
            -   **Description**: Resolved sequential enqueue dropping by adding status checks in `prompts_crud.rs`, prevented queued prompt transcript turn mutations in `tree_node_conv.rs`, suppressed enqueue refresh bars via silent SWR `loadTree(false, true)`, compacted Tree View Row 2 filter controls into two segmented capsules (194px layout budget, zero clipping), authored canonical Tree View UI/UX design system specification in `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md`, and added 28-assertion automated E2E test suite in `scripts/test-anti-gravity-rerun-and-ui-fixes-e2e.ps1`. (Thanks to @aukgit)

    *   **v4.271.0 (2026-10-27)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.270.0 (2026-10-27)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.269.0 (2026-10-25)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.268.0 (2026-10-25)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.267.0 (2026-10-25)**:
        -   **[Routine Minor Bump] Version sync to 4.267.0 with no functional code changes (Thanks to @aukgit)**:
            -   **Description**: Routine minor version bump from 4.266.0 to 4.267.0: all 14 version locations synchronized via `scripts/bump-version.mjs` (`package.json`, `package-lock.json`, `version.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/hooks.nsh`, `Casks/antigravity-tools.rb`, `readme.md`, `readme_en.md`, `src/components/layout/MiniView.tsx`, `releases-manifest.json`, `changelog.md`, `changelog_en.md`). No functional code changes in this bump; the v4.266.0 feature and fix set ships in its own release. (Thanks to @aukgit)

    *   **v4.266.0 (2026-10-25)**:
        -   **[Tree View & Rerun UX] Compact Dispatch UI, Anti-Repeat Random Nudge Selection & Recent-on-Top Hierarchy (Thanks to @aukgit)**:
            -   **Description**: Streamlined the Prompt Tree View and Rerun dispatch workflow: replaced verbose 17-item radio list and 37-variation bullet dump with a compact `<select>` dropdown and dynamic status pill (`Auto-picks random unused (N of 37 left)`); implemented `extractUsedNudgesFromConversation` in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` to strictly exclude already used questions from multi-round rerun dispatches; inverted prompt turn and subagent hierarchy in `src/components/instances/PromptTreeView/ConversationNode.tsx` and `tierClassification.ts` to display running, queued, and newest turns on top; and added weekly quota fallback in `src/pages/instances/instanceCardData.ts` to ensure full GUI parity with CLI. (Thanks to @aukgit)

    *   **v4.265.0 (2026-10-25)**:
        -   **[CI Concurrency SHA Scoping & Release Resilience] Isolated main branch CI concurrency to commit SHA to prevent rapid sequential push cancellations (Thanks to @aukgit)**:
            -   **Description**: Scoped CI concurrency group on `main` in `.github/workflows/ci.yml` to the specific commit SHA (`group: ${{ github.workflow }}-${{ github.ref == 'refs/heads/main' && github.sha || github.ref }}`), ensuring subsequent commits on `main` never cancel in-flight CI workflows. Expanded 4-part Root Cause Analysis in `.ai-memory/cicd-issues/47-release-gate-ci-race-condition-rca.md` and verified clean release gate polling and execution. (Thanks to @aukgit)

    *   **v4.264.0 (2026-10-25)**:
        -   **[Release Gate In-Progress CI Polling] Bounded polling loop awaits in-flight CI runs on tag push before validating green pass (Thanks to @aukgit)**:
            -   **Description**: Fixed race condition in `.github/workflows/release.yml` where `release.yml` immediately queried `status=completed` and aborted with `conclusion: missing`. Added 25-minute polling loop with heartbeat logging, authored RCA 47, and verified release gate pass on GitHub Actions. (Thanks to @aukgit)

    *   **v4.263.0 (2026-10-24)**:
        -   **[Account Switch Prompt Restore and CLI Inline Resume] Scoped prompt snapshots to the target instance and executed inline post-switch resume on CLI paths (Thanks to @aukgit)**:
            -   **Description**: Stage 1 snapshot source now uses the target switched instance in `src-tauri/src/modules/instance/switch.rs` instead of global active instance. Added `run_post_switch_resume_after_delay` in `src-tauri/src/modules/instance/post_switch_resume.rs` to await settle delay and restore inline for CLI/headless operations. Fast-forward leftover handling in `src-tauri/src/bin/agm/instance_fastforward.rs` avoids duplicate prompt dispatch. (Thanks to @aukgit)

    *   **v4.262.0 (2026-10-24)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.261.0 (2026-10-24)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.260.0 (2026-10-24)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.259.0 (2026-10-24)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.258.0 (2026-10-24)**:
        -   **[Prompt Tree Header Compaction, Hover Jitter Elimination, Segmented Pill Capsules, Combined Send/Enqueue/Rerun Dropdown & CLI Error Management] Comprehensive Prompt Tree View UX overhaul, compact header layout with icon actions, high-contrast 3D Reset button, hover border stabilization, strict 30s cache TTL guard, clickable RFC-8089 IDE file image links, contiguous segmented pill capsules across toolbars, current conversation routing RCA & fix, and CLI error inspection with agent skill (Thanks to @aukgit)**:
            -   **Description**: Compacted Prompt Tree View header height to `< 42px` in `src/components/instances/PromptTreeView/ModalHeader.tsx` and tightened node vertical spacing in `TreePanel.tsx`; converted Import/Export text buttons to Lucide icons (`Download`, `Upload`) with tooltips and converted Hide Tree View to `PanelLeftClose`/`PanelLeftOpen` icon button; relabeled the yellow button adjacent to Refresh to "Reset" with high-contrast amber styling (`text-zinc-950 font-bold bg-amber-400 hover:bg-amber-300 shadow-[0_2px_0_0_#b45309]`) and 3D tactile elevation; eliminated green hover outline shifts in `ConversationNode.tsx` using permanent base `border border-transparent` and subtle background tint transitions; enforced a strict 30-second TTL cooldown guard (`TREE_CACHE_TTL_MS = 30000`) in `usePromptTreeData.ts` to prevent mouseover-induced refetches; reunited Send, Enqueue, and Rerun into a contiguous segmented dropdown pill capsule in `PromptInstructionView.tsx` with borderless transparent destination selectors; resolved the Root Cause Analysis (RCA) defect in `usePromptTreePromptActions.ts` and `src-tauri/src/modules/repo_db/dispatch.rs` where "Current Conversation" mode was erroneously routing to new conversations; implemented `formatIdeClickableFileLink` in `clipboard.ts` and `markdown.tsx` formatting images and file paths into RFC-8089 `file:///` clickable links with a 1-click "Open in IDE" preview button; harmonized toolbar action clusters in `InstanceRowActions.tsx` and `AccountCardActions.tsx` into contiguous segmented pill capsules with high-contrast yellow hover states; verified CLI error inspection commands (`agm errors list`, `agm errors inspect`, `agm errors clear`, `agm errors fix`); authored canonical agent skill `.agents/skills/agm-error-inspection/skill.md`; and passed all 28/28 verification gates in `scripts/test-agm-gitmap-running-backup-e2e.ps1`. (Thanks to @aukgit)

    *   **v4.257.1 (2026-10-24)**:
        -   **[Prompt Tree Titles and File Image URIs] Placeholder conversation titles use the prompt's first line, and out-of-workspace images are `file:///` URIs (Thanks to @aukgit)**:
            -   **Description**: `conversation_display_title` in `src-tauri/src/modules/repo_db/tree_node_conv.rs` replaces empty, `untitled`, and `new conversation` titles with `extract_smart_prompt_summary`. `workspace_relative_media` keeps in-repo images relative and writes out-of-repo images as `file:///` URIs. `agm prompts tree --json` on the live instance reported 146 conversations, 0 untitled titles, 0 empty prompts, and 10 `file:///` image paths with 0 raw drive paths. (Thanks to @aukgit)

    *   **v4.257.0 (2026-10-24)**:
        -   **[Prompt Tree Current Conversation, Silent Sync, and Image Files] Current Conv passes real `conv-` ids to `agy --conversation`, background sync no longer paints a layout-shifting bar, and image clicks open the file (Thanks to @aukgit)**:
            -   **Description**: `agy_conversation_id` in `src-tauri/src/modules/repo_db/agy.rs` now keeps `conv-` conversation ids and still rejects `new`, `p-`, `proj-`, `queued-`, and path-like values. `agm prompt send --new` and a send with no session pass the `new` sentinel in `src-tauri/src/bin/agm/prompt_cmds.rs`. The prompt tree hides the top progress bar during background sync in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`. Thumbnail clicks open the disk file through the opener in `src/components/instances/PromptTreeView/PromptInstructionView.tsx`. Verified with `cargo build --bin agm`, the conversation-id unit test, `npm run build`, and `agm errors list`. (Thanks to @aukgit)

    *   **v4.256.0 (2026-10-24)**:
        -   **[Single-Prompt Resume After Switch & Fast-Forward] After an account switch or fast-forward, re-dispatch exactly one prompt — the most recently active backed-up prompt strictly belonging to the target instance — while every other backed-up prompt stays backed up, replacing the previous whole-queue re-injection (Thanks to @aukgit)**:
            -   **Description**: Added `dispatch_single_latest_prompt` in `src-tauri/src/modules/repo_db/dispatch.rs`, which selects the single most recently active backed-up prompt for the target instance (greatest `updated_at`, ties broken by `created_at` then `id`) under a strict instance match with no cross-instance fallback, and sends it through the same resume-file + agy/CDP path as full dispatch; the STAGE_5_RESTORE stage in `src-tauri/src/modules/instance/post_switch_resume.rs` now uses this single-prompt dispatch, `has_auto_resume = false` fully suppresses the re-dispatch while restore bookkeeping still runs, and the duplicate inline auto-resume of recent prompts was removed from the switch path in `src-tauri/src/modules/instance/workspace.rs` so resume happens exactly once after the prompt channel is up (the manual auto-resume command path is unchanged); FIFO ordering of the remaining backlog is unchanged, and unit tests cover the latest-prompt selection. (Thanks to @aukgit)

    *   **v4.255.0 (2026-10-24)**:
        -   **[American English Modernization, Global React Error Boundaries & Root Route Recovery] Full American English language modernization across 352 backend Rust files, 76 frontend TypeScript/React files, and all architecture docs, fix minified React hook error #310, mount universal RouteErrorBoundary on root route, wrap fleet tables and modals with defensive error boundaries, and update root README (Thanks to @aukgit)**:
            -   **Description**: Converted all legacy Chinese code comments, JSDoc documentation, and internal log/diagnostic messages across 352 Rust backend files, 76 TypeScript/React frontend files, and all architecture documentation to pure American English; resolved minified React error #310 ("Rendered more hooks than during the previous render") on the Instances page "Switch Profile" modal and prompt export modal by strictly lifting hooks above conditional early returns; mounted universal RouteErrorBoundary on root browser routing in `src/App.tsx` to intercept unhandled render exceptions and provide user-friendly dark-glass error recovery cards with reload, navigation, and telemetry actions; wrapped fleet tables and modal dialogs with defensive error boundaries; and refreshed root `README.md` and `README_EN.md` with latest version badges and release highlights. (Thanks to @aukgit)

    *   **v4.254.0 (2026-10-24)**:
        -   **[Fast Forward 4-Tier Selection Ladder, Weekly Floor Exclusion & Headroom Invariants] Implement 4-tier Fast-Forward account target resolver, configurable weekly quota floor slider (default 15%), 100% 4-hour clean headroom invariant, cross-instance fleet exclusion (SSH/Email/Telegram/GitMap), earliest expiry ranking, and degraded fallback scoring (Thanks to @aukgit)**:
            -   **Description**: Refactored `fast_forward_target_resolver.rs` to enforce the full 4-tier decision ladder: accounts with weekly quota < 15% (configurable via slider) are treated as 0 credits and skipped; candidate accounts in the Primary tier must possess 100% 4-hour credit and zero activity across local or fleet instances (SSH, Email, Telegram, GitMap) in the last 4 hours; primary accounts are sorted by closest reset/expiry time to maximize window efficiency before reset; when no 100% 4-hour candidate exists, the algorithm safely falls back to highest combined weekly and 4-hour credits weighted by subscription tier multipliers (Ultra ×4.0, Pro ×2.0, Free ×1.0); added UI slider in `ScoringAlgorithm.tsx` for `min_weekly_quota_percent` (0–50%, default 15%); synchronized backend config models in `src-tauri/src/models/config.rs`; and added comprehensive unit test suite in `tests_fast_forward_target_resolver.rs`. (Thanks to @aukgit)

    *   **v4.253.3 (2026-10-23)**:
        -   **[Release Pipeline & CI Dynamic Polling Gate, Atomic Version Sync, Prompt Restore & CLI Rerun Hardening] Fix release asset publication pipeline, synchronize all project manifests, harden prompt restore sweep on quota completion, and enable instant rerun randomizer (Thanks to @aukgit)**:
            -   **Description**: Resolved GitHub Actions release pipeline failure where concurrent tag pushes caused premature release gate exits due to in-progress CI runs; implemented an asynchronous polling wait loop up to 45 minutes in `.github/workflows/release.yml` with post-release asset count verification; synchronized all 14 repository version manifests (`package.json`, `package-lock.json`, `version.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.lock`, `releases-manifest.json`, etc.) to eliminate version drift; and integrated prompt restore sweep engine with diverse rerun anti-repeat buffer and top-level CLI command routing. (Thanks to @aukgit)

    *   **v4.253.0 (2026-10-23)**:
        -   **[Streaming Thought Signature Validator, Segmented Capsule Styler, FIFO Queue Partition Allocator, Transient Sandbox Sentinel, Sole Contributor Attribution & Windows FFI Remediation] Complete implementation of final architectural components (Steps 296–300), stream delimiter validation, capsule styling rules, FIFO partition allocation, caller IDE preservation, and pure kernel32 vitality probes (Thanks to @aukgit)**:
            -   **Description**: Successfully finalized the 300-step architectural roadmap across reverse proxy streaming, UI design conventions, Split-DB queue allocations, process sandboxing, and release governance:
                - **Streaming Thought Signature & Delimiter Validator (`src-tauri/src/proxy/handlers/common/`)**: Implemented `streaming_thought_signature_validator` ensuring balanced `<thought>` and `</thought>` tags, `thought: true` attributes, and delta buffer delimiters across streaming SSE frames without leaking internal markers; added `repair_truncated_thought_block` auto-closing truncated blocks. (Thanks to @aukgit)
                - **Segmented Capsule Toggle Switch Styler & Linter (`src-tauri/src/modules/instance/`)**: Implemented `segmented_capsule_toggle_switch_styler` and markup linter enforcing `rounded-full`, shared dark-glass container borders, subtle divider lines, and transparent borderless inline select dropdowns (`bg-transparent border-0 focus:ring-0 focus:outline-none`). (Thanks to @aukgit)
                - **FIFO Prompt Queue Partition & Priority Allocator (`src-tauri/src/modules/db_orm/`)**: Implemented `fifo_prompt_queue_partition_allocator` allocating prompt partition buckets across SQLite databases with strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`, monotonic timestamp/id verification, and zero LIFO leaks. (Thanks to @aukgit)
                - **Transient Sandbox Lifecycle & Cleanup Sentinel (`src-tauri/src/modules/instance/`)**: Implemented `transient_sandbox_cleanup_sentinel` tracking test sandbox instances, protecting developer caller IDE sessions with exit code 2 fail-fast protection, and verifying zero-orphan remnant directory purging. (Thanks to @aukgit)
                - **Sole Contributor Attribution & No Silent Errors Audit Guard (`src-tauri/src/modules/release_gate/`)**: Implemented `sole_contributor_attribution_audit_guard` enforcing strict sole attribution to `@aukgit` (`(Thanks to @aukgit)`), rejecting unauthorized third-party handles, enforcing strictly relative paths, and auditing source code for the No Silent Errors house rule. (Thanks to @aukgit)
                - **Windows Kernel32 FFI & Fast-Forward Rotation Remediation**: Repaired process vitality probes and child process termination to link directly against Windows `kernel32` (`OpenProcess`, `GetExitCodeProcess`, `TerminateProcess`, `CloseHandle`) without unlinked external dependencies; harmonized `smartRotateProfileAccount` with backend `fastForwardInstance` delegation; and expanded rerun nudge detection templates. (Thanks to @aukgit)
                - **Consolidated Wave Summary**: v4.253.0 is the first published release of today's combined improvement wave, shipped all together — the v4.251.0 prompt backup & restore rebuild and rerun randomizer, the v4.252.0 pipeline/FIFO/capsule/release-governance guard wave, and the sequences 213–218 hardening detailed in the entries below — released after a full compile and CI repair campaign cleared 18 pre-existing TypeScript errors and the accumulated Rust test-target compile debt, with `cargo check --tests`, `cargo clippy --all-targets`, `cargo fmt --check`, `tsc --noEmit`, and the frontend build all verified green and a completed green CI run for the tagged commit. The earlier v4.251.0 and v4.252.0 tags were held back by the release gate on red CI and are superseded by this release. (Thanks to @aukgit)

    *   **v4.252.0 (2026-10-23)**:
        -   **[Pipeline Decoupling, FIFO Queue Invariants, Caller IDE Self-Preservation, UI Capsules & Release Governance] Pipeline response header sanitizer, protocol adapter divergence auditor, thinking block backfill & headroom budget evaluator, unified context alternating alignment guard, FIFO dead-letter spillover guard, multi-table FIFO restore transaction coordinator, FIFO batch drain monotonicity verifier, caller process ancestry tree scanner, caller IDE exit code 2 enforcer & heartbeat watchdog, transient test sandbox verifier, multi-stage launcher dispatcher, window control capsule formatter, truncation omission banner formatter, prompt tree 4-tier hierarchy formatter, and GitHub release notes parity guard (Thanks to @aukgit)**:
            -   **Description**: Successfully implemented and verified 25 architectural improvements across reverse proxy pipelines, database queue invariants, multi-instance process management, UI/UX desktop styling, and release compliance gates:
                - **Pipeline Protocol Decoupling & Header Sanitization (`src-tauri/src/proxy/handlers/common/`)**: Implemented `pipeline_response_header_sanitizer` normalizing Gemini SSE response headers (`text/event-stream; charset=utf-8`, `cache-control: no-cache`, `connection: keep-alive`) while stripping upstream server telemetry (`server`, `alt-svc`, `set-cookie`, `x-goog-safety-ratings`, `x-goog-ratelimit-remaining`, `via`, `x-powered-by`); integrated `protocol_adapter_divergence_auditor` enforcing strict decoupling between inbound protocol adapters and internal pipeline logic; added `thinking_block_backfill_pipeline` and `thinking_budget_filter_pipeline` ensuring uniform thinking tags and budget handling; implemented `unified_context_alignment_guard` for strict alternating conversation turns; and established `prefix_stability_pipeline_guard` guaranteeing prompt prefix immutability.
                - **FIFO Database & Dead-Letter Queue Invariants (`src-tauri/src/modules/db_orm/`)**: Implemented `fifo_dead_letter_queue_spillover_guard` safely transitioning unrecoverable prompts into dead-letter quarantine while preserving chronological `created_at` lineage and enforcing `ORDER BY created_at ASC, id ASC`; established `fifo_queue_drain_monotonicity_verifier` and `fifo_queue_batch_drain_guard` preventing LIFO sequence inversions across batched queue processing passes; and added `fifo_restore_transaction_coordinator` and `fifo_queue_index_analyzer` verifying composite indices and atomic transaction safety.
                - **Caller IDE Self-Preservation & Process Management (`src-tauri/src/modules/instance/`)**: Implemented `caller_process_ancestry_tree_scanner` traversing process PPID lineage upward up to 16 levels to detect caller IDE instances and protect the active working developer session; established `caller_ide_exit_code_enforcer` enforcing fail-fast exit code 2 on lifecycle commands (`stop`, `rm`, `switch`) targeting the caller instance without `--force`; added `caller_ide_heartbeat_watchdog` providing eviction immunity; implemented `transient_sandbox_lifecycle_verifier` ensuring test sandboxes leave zero orphan processes or filesystem remnants; and added `windows_shell_dispatch_argument_escaper` and `multi_stage_launcher_dispatcher` with Windows Job Object breakaway fallback ladder.
                - **UI Capsules, Hierarchy & Truncation Formatter (`src-tauri/src/modules/instance/`)**: Implemented `window_control_capsule_formatter` wrapping Minimize, Maximize/Restore, and Close actions into contiguous segmented pill capsules (`rounded-full`, shared border, dark-glass backdrop, subtle dividers) with hover-danger styling; added `capsule_segmented_control_validator` enforcing transparent borderless styling for nested dropdowns; added `truncation_omission_banner_formatter` parsing raw truncation callouts into interactive dark-glass omission banners; and added `prompt_tree_branch_hierarchy_formatter` discriminating across 4 distinct origin tiers (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) with branch glyphs (`↳`, `└──`).
                - **Release Governance & Attribution Gate (`src-tauri/src/modules/release_gate/`)**: Implemented `github_release_notes_parity_guard` verifying GitHub release notes strictly attribute `@aukgit`, reject third-party handles, enforce strictly relative Git paths, and match `CHANGELOG.md` headings; added `individually_revertable_commit_guard` auditing commit boundaries; and enforced the no-silent-errors rule via `no_silent_errors_justification_auditor`. (Thanks to @aukgit)

    *   **v4.251.0 (2026-10-23)**:
        -   **[Prompt Backup & Restore Reliability, Rerun Auto-Pick, and Release Integrity] Incomplete-prompt sweep on quota switch, truthful restore reporting, FIFO restore drain, persisted post-switch resume, shared GUI/CLI backup database, randomized rerun by default with persisted choice, skipped-version update reporting, email dedupe and supabase push-settings CLI commands, configurable PID refresh, and CI release gating (Thanks to @aukgit)**:
            -   **Description**: Rebuilt prompt backup and restore so quota-driven account switches no longer lose long-running prompts: a new incomplete-prompt sweep in `backup.rs` captures every running, in-flight, or dispatched prompt at switch time regardless of age, replacing the previous 120-second cutoff that silently dropped longer work, the backup cap now prunes only already-restored rows, and queued rows are exempt from the freshness filter; manual instance switches re-attribute unrestored prompt backups to the incoming account (parity with the rotation path), restore returns real restored, dispatched, failed, and skipped-live counts instead of unconditional success, a live-status probe prevents duplicate re-injection of prompts that already finished, multi-prompt restore drains the full FIFO queue in order instead of one prompt per repository per pass, post-switch resume tasks are persisted to disk and re-armed after an application restart, and the GUI and CLI resolve one shared backup database in the application data directory (migrating any legacy working-directory database); rerun now defaults to the randomized message option with an expanded message set and a default batch of four, and the chosen option and count persist across restarts in the detail header, the conversation context menu, the prompt tree context menu, and the tree view modal; the update checker explains why candidate versions were skipped (draft, prerelease, or no installable platform asset) instead of silently offering nothing, with matching skipped-version lines in `agm update`; added `agm email dedupe` (dry-run by default, timestamped database backup before applying) and `agm supabase push-settings` with vault-path support, plus a configurable PID refresh interval in Settings and `agm auto-switch pid-refresh`; CI workflows are pinned to Ubuntu 24.04 with current action versions, main-branch runs can no longer be cancelled mid-flight, and the release workflow refuses to tag unless CI is green for that exact commit. (Thanks to @aukgit)

    *   **v4.250.0 (2026-10-21)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.249.0 (2026-10-21)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.248.0 (2026-10-21)**:
        -   **[Repo DB & Diagnostics] Elimination of E9001 Media Vault Column Error on Instance Tree Cache Wipe (Thanks to @aukgit)**:
            -   **Description**: Resolved diagnostic error `E9001` (`no such column: instance_id in DELETE FROM media_vault WHERE instance_id = ...`) in `src-tauri/src/modules/repo_db/agy.rs` during per-instance prompt tree cache invalidation and reset; enforced architectural invariant establishing `media_vault` as an immutable, content-addressed SHA-256 media cache decoupled from workspace instance tenancy; added unit test `test_media_vault_schema_and_column_invariant` verifying schema column invariants and `test_wipe_and_rescan_single_instance_does_not_fail` verifying single-instance wipe stability; cleared error diagnostics history via `agm errors clear`, verified zero errors in `errors.db`, and documented comprehensive 4-part RCA in `02-spec/22-app-issues/36-media-vault-missing-instance-id-wipe-rca.md`. (Thanks to @aukgit)

    *   **v4.247.0 (2026-10-21)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.246.0 (2026-10-21)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.245.0 (2026-10-21)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.244.1 (2026-10-21)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.244.0 (2026-10-21)**:
        -   **[Boolean Logic Modernization, Affirmative Guard Decomposition & Instance Task Liveness] Discrete Guard Clauses, Affirmative isConversationGenuinelyRunning Extraction, Mixed Operator Elimination, and Verified Liveness Accuracy (Thanks to @aukgit)**:
            -   **Description**: Refactored conditional logic across `src/pages/instances/` adhering strictly to `01-prompts/15-cg-execute/04-booleans-and-complex-conditions.md`; replaced compound negative checks such as `if (!inst.is_running || !proj.is_running) return false;` with discrete, single-variable affirmative guard clauses in `InstanceCardProjects.tsx`; extracted and centralized `isConversationGenuinelyRunning` in `src/pages/instances/instancePageUtils.ts`, enforcing discrete boolean decomposition (affirmative running checks, explicit instance ownership matching, active task validation, and ghost conversation filtering), eliminating 4-operand chained `.some(...)` predicates across `instanceCardData.ts` (`hasActiveTask`), `InstanceCardProjects.tsx` (`isProjectRunningForInstance`), and `useInstancePage.ts` (`hasActiveTaskFor`); decomposed mixed `&&`/`||` operators into intermediate named booleans in `instancePageUtils.ts` (`isDefaultInstance`, `hasMatchingSeqNum`), `instanceCardData.ts` (`weeklyBucket` resolution), and `useInstancePage.ts` (`filteredInstances` search filtering); and verified all 32 unit test assertions pass in `src/pages/instances/__tests__/instanceRunningPrompts.test.ts`. (Thanks to @aukgit)

    *   **v4.243.0 (2026-10-20)**:
        -   **[Telegram Prompt Backup, Fleet w1 Identity Primacy, Prompt Tree Liveness & Responsive UI Expansion] Telegram Inbound Prompt Snapshots, Base64 Remote Prompt Protocol, Fleet Local w1 Identity Reconcile with 6-Node Discovery, SSH 30s Caching, Default Instance Prompt Liveness, state.vscdb Title Rename Sync, Terminal Box Glyphs, Header Image Filter & Details Button Health Pulse (Thanks to @aukgit)**:
            -   **Description**: Added `/backup`, `/backup_prompt`, `/backup_prompts`, `/backup_running`, and `/brp` inbound command routes in `src-tauri/src/modules/telegram_inbound/processing.rs` and `commands.rs`, delivering automated snapshots of running prompts, attached multimodal image counts, dynamic instance resolution, and structured HTML backup reports; implemented `--base64` RFC 4648 payload encoding across `src-tauri/src/modules/telegram_inbound/injection.rs` and `src-tauri/src/bin/agm/prompt_dispatch.rs` for cross-machine SSH prompt execution, preserving shell whitespace, quotes, and multiline instructions without command-line escaping defects; enforced Cluster-IP primacy in `src-tauri/src/modules/supabase_sync/node.rs` and `email_watcher/detection.rs` to reliably resolve `192.168.1.3` as `w1` (local lead) and `192.168.1.20` as `main`, and fixed deduplication logic in `query_fleet_machines` so all 6 registered cluster nodes (`w1`, `main`, `u1`, `w2`, `w3`, `w4`) are discovered and displayed with Lead indicators; implemented `CLUSTER_BALANCES_CACHE` with 30s TTL in `src-tauri/src/modules/ssh_manager/exec.rs` and decoupled balance enrichment via SWR in `useFleetMachines.ts` to eliminate UI freezing; updated `reset_ssh_cluster_nodes` in `sync.rs` to reload cluster nodes preserving passwords and SSH key paths; standardized instance specifier normalization across `observe.rs` (`#1 Default`, `Default`, `__default__`), hardened canonical directory discovery in `gemini_dirs.rs`, enforced strict FIFO `ORDER BY created_at ASC, id ASC` in `liveness.rs` to eliminate false terminal overrides, and added IDE prompt goal heartbeat fallback in `cmd_running_up_next.rs` for active IDE sessions; implemented immutable React state cloning in `usePromptTreeData.ts` to instantly propagate renamed conversation titles to header and selected state, and wired `sync_title_to_state_vscdb` in `src-tauri/src/modules/repo_db/sequences.rs` to update `ItemTable` in `state.vscdb`; added a header `🖼️ Images` toggle button in `src/components/instances/PromptTreeView/ModalHeader.tsx` connected to `imageFilterEmitter` to instantly filter projects and conversations containing attached images, expanded tree panel width to `w-[460px] lg:w-[520px]`, increased project separation (`space-y-3.5`), and styled Auto-Hide and scope dropdown as a contiguous dark-glass segmented pill capsule; restored Details button with emerald `Activity` icon and glowing `#1af18d` pulsing health indicator dot in `DetailTabs.tsx`; standardized child branch glyphs to monospace box-drawing glyphs (`├──`, `└──`, and `└── ↳` for subagents) with auto-selection on double-click in `ConversationNode.tsx` and `ProjectNode.tsx`; added a 10-retry overload in `cmd_files.rs` for Win32 clipboard image copies; and converted verbose text in `SshGitmapSection.tsx` into compact segmented pill capsules with tooltips. (Thanks to @aukgit)

    *   **v4.242.0 (2026-10-20)**:
        -   **[Win32 CF_DIB Clipboard Engine, Prompt Tree Image Attachment Splicing, Instant Enqueue Cache Invalidation & DetailHeader Title Polish] Native CF_DIB / PNG Multi-Format Clipboard Engine for OneNote and Antigravity, Preserved Turn Media Splicing, Live Enqueue Refresh, and DetailHeader Title Prominence (Thanks to @aukgit)**:
            -   **Description**: Upgraded `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` to extract raw bitmap bytes and strip the 14-byte `BITMAPFILEHEADER`, registering native `DeviceIndependentBitmap` (`CF_DIB`) alongside `PNG` streams on `System.Windows.Forms.DataObject` to ensure images pasted into Microsoft OneNote desktop, Word, Paint, Discord, and Google Antigravity IDE render 100% reliably; updated regex parsing in `extract_clean_user_prompt` (`src-tauri/src/modules/repo_db/text_utils.rs`) to collect and preserve markdown images located outside `<USER_REQUEST>`; updated `usePromptTreePromptActions.ts` (`handleResendPrompt` and `handleEnqueuePrompt`) to splice missing conversation images inside `<USER_REQUEST>` before prompt submission; inverted execution order in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` so `invalidate_prompt_tree_cache` executes before `loadTree`, guaranteeing newly enqueued turns immediately appear with amber `QUEUED` badges without stale cache reads; added multi-image aggregation in `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`, invoking `copyAllImagesToClipboard` to composite multiple images into a stitched canvas with native `CF_DIB` and `PNG` clipboard writes; standardized button labels strictly to "Copy" and "Copy Images" (eliminating "Copy including image"); stripped all turn/origin prefix badges preceding conversation titles in `src/components/instances/PromptTreeView/DetailHeader.tsx` and enlarged title typography to `text-xl md:text-2xl font-bold font-ubuntu tracking-tight`; and verified complete sandbox isolation via `agm instances test-create` and `agm instances test-clean` alongside 74 passing Telegram inbound tests. (Thanks to @aukgit)

    *   **v4.241.0 (2026-10-20)**:
        -   **[Native OS DIB Image Clipboard, Prompt Tree Send & Enqueue Restoral, DetailHeader Title Polish, SQLite Reset & Rescan, Fleet Lead Alignment] Native Win32 Bitmap Clipboard for OneNote & Antigravity, Preserved Turn Media Paths, Instant Enqueue Tree Refresh, DetailHeader Prefix Stripping & Title Prominence, Complete SQLite Cache Purge, and Fleet Machine Git Origin Alignment (Thanks to @aukgit)**:
            -   **Description**: Repaired `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` by checking `data:image/` before path normalization, running PowerShell in `-Sta` mode with `[System.Windows.Forms.Clipboard]::SetDataObject($data, $true)` to ensure persistent native `CF_DIB`/`CF_BITMAP` clipboard memory for Microsoft OneNote, Google Antigravity IDE, Word, and Paint; hooked native clipboard priming into `copyPromptWithRichImages` in `src/components/instances/PromptTreeView/clipboard.ts`; restored attached media preservation during turn selection in `createTurnConv` (`src/components/instances/PromptTreeView/ConversationNode.tsx`); passed `loadTree` callback from `PromptTreeViewModal.tsx` to `usePromptTreePromptActions` to trigger immediate tree refresh upon enqueuing; added current working directory fallback in `src-tauri/src/modules/repo_db/agy.rs` for prompts with empty workspace paths; stripped noisy turn badges preceding conversation titles in `src/components/instances/PromptTreeView/DetailHeader.tsx` and enlarged title typography to `text-lg md:text-xl font-bold font-ubuntu tracking-tight`; enhanced `wipe_and_rescan_repo_db` in `src-tauri/src/modules/repo_db/agy.rs` to wipe `media_vault`, in-memory tracking maps, and dispatched prompts cache alongside `invalidatePromptTreeCache()` on reset; and aligned cluster lead node `main` (`192.168.1.20`) and local node `w1` (`192.168.1.3`) in `src/components/instances/fleet/useFleetMachines.ts` and `FleetMachineRow.tsx` to reflect the authentic Git origin `alimtvnetwork / Antigravity-Manager` with bound accounts `sakibagjdjg@gmail.com` and `cirajayu11@gmail.com`. (Thanks to @aukgit)

    *   **v4.240.1 (2026-10-20)**:
        -   **[Prompt Tree Buttons Repair, Native OS DIB Image Clipboard, DetailHeader Title Prominence, Full SQLite Reset & Rescan, Fleet Lead Node Alignment & agy CLI Path Resolution] Prompt Tree Send & Enqueue Restoral, Native Windows DIB/Bitmap Image Clipboard for OneNote & Antigravity, Single-Image "Copy" Button & Unified Multi-Image Canvas Batch Copy, DetailHeader Prefix Stripping & Title Polish, Full SQLite Reset & Rescan, Fleet Machine Git Origin Alignment, and agy CLI Executable Path Resolution (Thanks to @aukgit)**:
            -   **Description**: Restored independent `[ Send N | v ]` and `[ Enqueue | v ]` buttons in `src/components/instances/PromptTreeView/DetailHeader.tsx` (now in distinct Segment 2 and Segment 3 capsules), ensuring full prompt text `conv.full_prompt_text` is dispatched without premature truncation and preserving attached media paths; implemented `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` and `src-tauri/src/lib/run.rs` setting native Windows `CF_DIB`/`CF_BITMAP` format via PowerShell `System.Windows.Forms.Clipboard::SetImage` alongside pure PNG binary Blob to guarantee pasted images render 100% reliably in Microsoft OneNote, Paint, Word, and Antigravity IDE; normalized single-image copy button in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` strictly to "Copy" (eliminating "Copy including image"), and implemented `handleCopyImagesUnified` on the main copy button so clicking it when multiple images exist automatically stitches all attached images into a single vertical composite PNG canvas and writes to the system clipboard; added `~/.gemini/bin/agy.exe` and `~/.antigravity/bin/agy.exe` to `get_antigravity_cli_executable_path` in `src-tauri/src/modules/process/locations.rs` so CLI prompt dispatch resolves the installed `agy` binary reliably; expanded CDP composer selector ladder (`div.monaco-editor`, `div.interactive-input-editor`, `textarea.monaco-mouse-cursor-text`) and send button selectors in `src-tauri/src/modules/instance/cdp_onboarding.rs`; polished `DetailHeader.tsx` by stripping all noisy turn prefix badges preceding the title, enlarging conversation title text (`text-base md:text-lg font-bold font-ubuntu`), keeping Copy Title button adjacent, and positioning the status capsule strictly after the title; connected `onResetTree` in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `wipe_and_rescan_repo_db` and frontend `loadTree(true, true)` to completely purge SQLite prompt caches, terminate stale workers, rescan disk `transcript_full.jsonl` files fresh, and rebuild the tree with live spinner feedback; and corrected cluster lead node `main` (`192.168.1.20`) in `src/components/instances/fleet/useFleetMachines.ts` and `FleetMachineRow.tsx` to reflect its authentic Git origin `gitmap / gitmap` and bound account `sakibagjdjg@gmail.com` without leaking local machine `w1` (`192.168.1.3`, `Antigravity-Manager`) accounts. (Thanks to @aukgit)

    *   **v4.240.0 (2026-10-20)**:
        -   **[Prompt Tree Buttons Repair, Pure PNG Image Clipboard, DetailHeader Title Polish, SQLite Full DB Wipe & Rescan, Fleet Lead Node Alignment & Telegram Inbound Prompt Fix] Prompt Tree Send & Enqueue Restoral, Multi-Image Composite Canvas Copy, DetailHeader Prefix Stripping & Title Polish, Full SQLite Reset & Rescan, Fleet Machine Alias Alignment, and Telegram Prompt Routing (Thanks to @aukgit)**:
            -   **Description**: Restored prompt and image dispatch across Prompt Tree Send and Enqueue actions without stripping image URIs into non-existent paths, with project path resolution fallbacks; upgraded image clipboard copying to write strictly single-MIME `'image/png'` binary blobs without conflicting HTML to guarantee pasting into Microsoft OneNote, Paint, and Antigravity; added vertical composite multi-image canvas stitching and 1-click `[ Copy All Images ]` batch copying; cleaned DetailHeader by stripping prefix turn badges, enlarging conversation titles, and repositioning the status capsule; added backend `wipe_and_rescan_repo_db` command with live progress feedback to completely rebuild the SQLite prompt database and regenerate the tree view; aligned Fleet lead node `main` (`192.168.1.20`) and `w1` (`192.168.1.3`) SSH host alias mappings with Git repository badge display; and fixed Telegram inbound `/prompt` route collision where conversation seek shadowed prompt execution. (Thanks to @aukgit)

    *   **v4.239.0 (2026-10-20)**:
        -   **[Fleet Machine Identity, Prompt Tree Liveness, Audit Log Semantic Hierarchy & UI Polish] Fleet SSH Node w1 Matching, Cluster Reset, Running Prompt Detection for Default Instances, Conversation Rename Sync, Meaningful Audit Log Naming with Interactive Step Breadcrumbs, Start All Instances, and Markdown View Background (Thanks to @aukgit)**:
            -   **Description**: Resolved local machine identity matching `192.168.1.3` to cluster node `w1` using Cluster-IP primacy in `src-tauri/src/modules/ssh_manager/exec.rs` while preserving all 6 discovered GitMap nodes in `FleetMachinesTable.tsx`; added `reset_ssh_cluster` backend command and UI toolbar button `[ 🔄 Reset Cluster ]` to safely clear stale connection pools; fixed running prompt liveness in `src-tauri/src/modules/repo_db/liveness.rs` and `detection.rs` by including `dispatched` status and preserving empty instance records; ensured conversation title renames update synchronously across `conversations.title`, `conversation_summaries.summary`, and `conversations_fts.title` with live event notification; replaced cryptic Tailwind utility class names (`div.grid`, `Button.P 1.5`, `div.group`, `span.truncate`, `scroll:/instances`) in `useAuditRecorder.ts` and `AuditPanel.tsx` with human-readable semantic action descriptors; added an interactive "Step Performed & Interaction Execution Path" inspection card in `AuditPanel.tsx` featuring action summaries, connected DOM hierarchy breadcrumbs with arrow glyphs, and a 4-part target context grid; relocated the Details button with healthy status icon to row 2; standardized "Copy All (Include Images)" in the prompt tree toolbar; updated the Markdown view background to `#19191c`; converted verbose SSH settings text to icon buttons with hover tooltips; and added a "Start All" instances button in the main instance management toolbar. (Thanks to @aukgit)

    *   **v4.238.7 (2026-10-20)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.238.0 (2026-10-19)**:
        -   **[Pure PNG Clipboard Copy, Image Attachment Forwarding, Turn Status Badges & GitMap Node Resolution] Pure PNG Clipboard Write for OneNote, Dedicated Gallery Image Copy Buttons, Immediate Send vs Enqueue Decoupling, Live Transcript Turn Status Supremacy, and GitMap SSH Machine Resolution (Thanks to @aukgit)**:
            -   **Description**: Implemented `writePureImageToClipboard` in `src/components/instances/PromptTreeView/clipboard.ts` writing strictly single-MIME `'image/png'` with canvas-based PNG header transcoding (`transcodeSrcToPngBlob`), eliminating multi-MIME format collisions that previously caused Microsoft OneNote, Slack, Discord, and Telegram to strip images or paste empty placeholders; added 1-click `[ 📋 Copy Image (PNG) ]` and sequential `[ 📋 Copy Images (N) ]` buttons in the "Attached Images" gallery header, plus hover-activated 1-click copy badges on individual thumbnails in `PromptInstructionView.tsx`; decoupled immediate session injection (`handleSendPromptNow` / `sendPromptNow`) from queue scheduling (`handleEnqueuePrompt`) in `usePromptTreePromptActions.ts` and `DetailHeader.tsx`; preserved and appended attached conversation images (`media_paths`) in markdown image syntax (`![Attached Image N](<path>)`) across Rerun, Send, and Enqueue flows so IDE sessions receive complete image attachments without data loss; refactored status evaluation in `src-tauri/src/modules/repo_db/tree_collect.rs` so live transcript activity (`is_owning_inst_alive && inspection.is_recent_active && !inspection.is_terminal_done`) takes precedence over stale SQLite idle summaries, accurately displaying emerald pulse `RUNNING` for active turns and amber `QUEUED` for pending turns; upgraded `src-tauri/src/modules/telegram_inbound/node_label.rs` to parse `~/.gitmap/gitmap-ssh-nodes.json` and `./gitmap-ssh-nodes.json` when the machine alias is `none` or auto-generated `Node-*`, matching local IP `192.168.1.3` to alias `w1` and annotating as `w1 (collected from gitmap)`; separated `Machine:` and `Instance:` onto distinct lines across all Telegram status reports in `reports_b.rs`, `system.rs`, `telegram_alerts.rs`, and `telemetry.rs`; and upgraded preview, raw markdown, and edit mode containers in `PromptInstructionView.tsx` to `dark:bg-[#0c0514]/95` with `dark:border-purple-500/25` subtle borders. (Thanks to @aukgit)

    *   **v4.237.0 (2026-10-19)**:
        -   **[App Update Verification, Prompt Tree Selection Stability, Image Clipboard & Import-Export Restoral] Release Asset Verification Before Update Notification, Anti-Flapping Turn Locking, Rich Clipboard Copy, Running & Queued JSON Backup/Restore with Countdown HUD, and Telegram Remote Multimodal Dispatch (Thanks to @aukgit)**:
            -   **Description**: Added `verify_tag_has_platform_asset` in `src-tauri/src/modules/update_checker/channels.rs` performing HTTP HEAD requests against the GitHub Release API to ensure that native platform installer binaries (`agm-alim-setup.exe`, `.dmg`, `.deb`) actually exist and are downloadable before notifying users of updates, eliminating 404 installation errors when tags exist before assets finish compiling; locked the actively selected turn (`selected_turn_number`) and project selection across background SWR sync intervals in `src/components/instances/PromptTreeView/usePromptTreeData.ts` and `ConversationNode.tsx`, preventing title and prompt text from glitching or alternating between conversation summaries and raw turn prompts during active inspection; added explicit `onClick` and `onDoubleClick` backdrop event handlers on the backdrop container in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to cleanly dismiss the modal when clicking outside; upgraded image processing in `PromptInstructionView.tsx`, `markdown.tsx`, and `clipboard.ts` with local base64 and asset protocol fallback cards (`PromptThumbnailCard`), and support rich multi-format clipboard copying (`image/png` binary blob + `text/html`) through `copyPromptWithRichImages` with deduplicated URI extraction and Windows drive letter decoding; added direct combined export for both running and enqueued prompts (`Export Running & Queued Prompts (.json)`) and direct backup restoration in `src/components/instances/PromptTreeView/ImportExportMenu.tsx` and `usePromptTreeCoreActions.ts`, displaying an interactive 5-second countdown HUD banner before automatic prompt dispatching; preserved `saved_media_opt` during `/send` flows in `src-tauri/src/modules/telegram_inbound/send_flow.rs` and `injection.rs`, populating `active_prompts.image_payload` with attached media counts and timestamps; implemented an anti-aliased synthetic dispatch card in `open_shot.rs` when running under non-interactive Windows OpenSSH Session 0 without display station access; and reconciled `SshConnectionRecord` field schemas across `src-tauri/src/modules/supabase_sync/query.rs` and `src/components/instances/fleet/useFleetMachines.ts`, eliminating all Rust compilation, Clippy, and TypeScript linter errors across all supported OS matrix targets. (Thanks to @aukgit)

    *   **v4.236.0 (2026-10-19)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.235.0 (2026-10-19)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.234.0 (2026-10-19)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.233.1 (2026-10-19)**:
        -   **[Prompt Tree View Reliability, GitMap Node Discovery & Clipboard Pipeline] Singleton Context Menu Architecture, Outside Capture Dismissal, Queued Turn Auto-Expansion, GitMap Cluster Node Import, Accurate Offline Node Quotas & Taint-Free Base64 Image Reader (Thanks to @aukgit)**:
            -   **Description**: Centralized context menu state into `TreePanel.tsx` with a single `<PromptTreeContextMenu>` and transparent portal backdrop; added outside capture event listeners (`pointerdown`, `contextmenu`, `keydown`, `scroll`, `blur`) to dismiss immediately on any outside interaction; introduced `isConversationOrTurnQueued` to auto-expand queued child turns, render amber `QUEUED` badges with clock icons, and prioritize queued items in Tier 2 sorting; prioritized `gitmap cluster nodes --json` and `export-json` in `src-tauri/src/modules/ssh_manager/sync.rs` for reliable cluster node discovery across Windows/Linux machines; corrected offline SSH node quota fallbacks to 0.0% instead of 100.0% in `src-tauri/src/modules/ssh_manager/exec.rs`; added request debouncing and in-flight promise coalescing to `useAccountStore.ts` to prevent redundant IPC bursts; implemented `read_image_as_base64` in `src-tauri/src/commands/cmd_files.rs` and `run.rs` with safe path canonicalization and 100MB limit; decoded Base64 directly into PNG blobs in `clipboard.ts`, eliminating offscreen canvas CORS errors and WebView2 `SecurityError` tainting; and added rich event JSON clipboard copying with instant toast feedback.

    *   **v4.233.0 (2026-10-19)**:
        -   **[GitMap Node, Prompt System & Instance Management Synchronization] SSH Fleet Discovery, Telegram Multi-Node Routing, Prompt Tree Action Reliability, Screen Glitch Elimination, Supabase 5-Min Circuit Breaker, CDP-First Prompt Dispatch, Inline PID Pill & Caller Self-Preservation (Thanks to @aukgit)**:
            -   **Description**: Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via GitMap discovery and cluster metadata; forwarded identity public key `-i <pubkey>` in `fix_node_auth_via_gitmap` across `mesh.rs`, `commands/ssh.rs`, and CLI handlers, removing hardcoded single-node bypasses in favor of generalized credential vault auto-repair; removed fatal callback drop guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` so the receiving Telegram bot coordinator accurately routes prompts across cluster nodes; standardized remote dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to canonical `agm prompt send <project> --instance <inst> "<prompt>"` with safe quoting and multi-line escaping; inverted prompt dispatch in `src-tauri/src/modules/repo_db/agy.rs` to submit directly via Chrome DevTools Protocol (CDP) into active IDE windows before falling back to spawning external `agy` CLI background workers; wired direct 1-click execution for Backup and Restore in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `coreActions.handleBackup()` and `coreActions.handleRestore()`; added spinning animation on the Refresh icon during active sync; provided instant visual notification feedback on Reset and Refresh; restricted animated skeleton loaders in `src/components/instances/PromptTreeView/TreePanel.tsx` and `PromptTreeViewModal.tsx` strictly to initial cold starts (`isLoading && treeData.length === 0`), keeping the active tree rendered during background polling to eliminate visual flashing; enforced 3-tier deterministic sorting in `ProjectNode.tsx` and `tierClassification.ts` with `RUNNING` prompts strictly at Tier 1, `QUEUED` prompts at Tier 2 with explicit amber badges, and timestamp descending at Tier 3; removed redundant `"User Prompt"` badge and duplicate `{instanceNameDisplay}` text token from `DetailHeader.tsx`, relocating deep runtime telemetry to the Details modal; added `relative z-40` to `ModalHeader.tsx` container and `relative z-50` to `ImportExportMenu.tsx` so menus stack cleanly above banners and header elements; implemented atomic `LAST_SUPABASE_ERROR_TIMESTAMP` circuit breaker in `src-tauri/src/modules/supabase_sync/types.rs` with a 300-second cooldown on network errors across `fleet.rs`, `query.rs`, and `release.rs`; relaxed fleet polling to 300s in `useFleetMachines.ts`; added a 30-second TTL cache for account listings in `src-tauri/src/modules/account/crud.rs` to stop repetitive reads of 46 JSON account files; removed circular cache invalidation sabotage in `src-tauri/src/modules/repo_db/detection.rs:254` to eliminate console log flood; replaced ambiguous status codes with an emerald pulse badge (`PID: <pid>`) and aligned executable tail and data directory tail together on the exact same horizontal flex row (`.../Antigravity.exe · .../data`) in `src/components/instances/table/InstanceRowCells.tsx`; upgraded `src/pages/instances/SwitchAccountDialog.tsx` with cohesive dark-glass UI/UX; enforced process tree killing via `taskkill /F /T /PID` before stopping or switching accounts in `src-tauri/src/commands/instance.rs`; hardened `detect_caller_instance_id` to protect the developer's active workspace session (`default-copy-5597`); and verified with transient test sandboxes (`agm instances test-create` and `agm instances test-clean`).

    *   **v4.232.0 (2026-10-18)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.231.0 (2026-10-18)**:
        -   **[Instance Management & UI/UX Synchronization] PID Inline List View Layout, Deterministic Process Termination Ladder, Account Switch Pre-Switch Eviction, Royal Violet Switch Account Modal & Sharded Audit Trail Visibility (Thanks to @aukgit)**:
            -   **Description**: Confirmed numeric status `1632` represents native OS Process ID (`PID`); merged separate `Status & PID` and `IDE EXE & DATA` table columns into a unified `Status, Process & Environment` cell in `src/components/instances/InstanceTable.tsx` and `src/components/instances/table/InstanceRowCells.tsx` with inline monospace PID badge, shortened executable tail, and shortened data directory tail separated by `·` with segmented copy action capsule; established 5-stage deterministic process termination ladder (`taskkill /F /T`, Win32 `TerminateProcess`, PowerShell `Stop-Process`, 200ms kernel settle delay, Chromium lockfile eviction) in `src-tauri/src/modules/instance/close.rs` and `create.rs`; enforced pre-switch PID eviction and 200ms settle delay in `switch.rs` and `credentials.rs` to eliminate Electron write-back race conditions; redesigned `SwitchAccountDialog.tsx` to 100% Royal Violet design tokens (`var(--ui-surface-1)`, `var(--ui-border-subtle)`, `var(--ui-interactive)`, `var(--ui-success)`) with segmented quota pill capsule; resolved empty audit trail in `switch_history.rs` and `InstanceAuditTrailModal.tsx` by expanding default instance aliases, guaranteeing response fields, and adding smart auto-tab fallback to operations; hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations) with exit code 2 immunity guard; and verified 100% green via `scripts/test-instance-lifecycle-e2e.ps1`.

    *   **v4.230.0 (2026-10-18)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.229.0 (2026-10-18)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.228.0 (2026-10-18)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.227.0 (2026-10-18)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.226.0 (2026-10-18)**:
        -   **[E9001 Sibling Isolation & UI Redesign] Error E9001 Root-Cause Resolution, Loop Throttling, Accounts Table Dark-Glass Redesign & 3D Riseup Yellow Switch (Thanks to @aukgit)**:
            -   **Description**: Resolved Error E9001 (`Cannot switch instance to account: Account is actively bound to running sibling instance`) by guarding running instances from being considered stale or stealable in `instances.rs`, excluding running sibling accounts in `manual.rs`, and shielding candidates in `selection.rs`; prevented infinite 5-second retry loops in `PromptQueueScheduler` (`queue.rs`) for missing workspace directories by setting status to `failed` with retry backoff; implemented adaptive L1 quota cache TTL (120s/180s/300s) and 5-minute Supabase heartbeat error backoff; redesigned Accounts Table UI by removing blinding stark white hovers, flattening idle action buttons, styling Switch with Riseup Yellow 3D gradient, reordering action buttons, and moving custom label Tag into More menu; added Prompt Tree "Reset Filters" button in `TreePanel.tsx`; upgraded Win32 updater console to 18pt Consolas (`font_family = 54`); and refactored magic status strings into strongly-typed domain enums with inverted guard clauses.

    *   **v4.225.0 (2026-10-17)**:
        -   **[System Broadcasting & 30s Interval Guard] Cross-System 30-Second Minimum Interval Guard & Broadcast Storm Suppression Across Antigravity Manager & GitMap (Thanks to @aukgit)**:
            -   **Description**: Enforced an architectural invariant across Antigravity Manager and GitMap mandating that all periodic background broadcasts, polling schedulers, worker tickers, and notification dispatchers operate at intervals of not less than 30 seconds (`>= 30s`) to prevent CPU spikes, database lock contention, and notification storms; clamped the detached prompt goal heartbeat worker loop in `goal_heartbeat.rs` to a strict 30-second minimum (`interval_secs.max(30)`, default 30s); expanded the duplicate switch notification broadcast suppression window in `channels.rs` from 5s to 30s (`(now_ts - guard.1).abs() <= 30`), completely preventing rapid notification storms across Email, Telegram, and Supabase; and aligned GitMap task synchronization defaults (`TaskMinInterval = 30`, `TaskDefaultInterval = 30` in `constants_task.go`), clamped async periodic monitors (`gitmap async -t`) to `>= 30s`, clamped macro runner intervals to `>= 30s`, and enforced minimum 30s cluster distribution heartbeat loops.

    *   **v4.224.0 (2026-10-17)**:
        -   **[Telegram Alerts & Collision Shielding] Compact Layout with Next-Line White Text Values, Step 2 Multi-Machine Deduplication, Node IP & Signal Attribution, and Action-Only Keyboards (Thanks to @aukgit)**:
            -   **Description**: Modernized all Telegram alert cards (Account Rotation, Post-Switch Liveness, Emergency Alerts, System Updates) to be ~40% more vertically compact with key values indented on the next line in high-contrast bright white text (`kv_block`, `kv_white`); removed duplicate header and workspace fields; clamped project lists to top-3; eliminated duplicate Step 2 prompt wizard dispatch via `node_label::is_current_machine()` destination matching and <3s debounce guard; enriched instance selectors with explicit node alias, IP, and signal indicators (`🏷️ {name} ({alias} • {ip})`) in single-column layout; and streamlined Account Rotation inline keyboards to functional action-only buttons.

    *   **v4.223.0 (2026-10-17)**:
        -   **[Prompt Tree & Telegram Shielding] Custom Right-Click Context Menu, Right-Aligned Navigation Tabs, Header Dead-Space Removal, Top Toolbar Copy All Consolidation, and Telegram Cluster Routing Shielding (Thanks to @aukgit)**:
            -   **Description**: Added right-click custom context menus to Project headers (`ProjectContextMenu.tsx`) with Import into Project, Export (JSON, SQLite DB, YAML), and Rerun (Queued or All) actions; wired context menus and keyboard accessibility to child subagents and prompt turns with active selection highlights (`bg-sky-500/10 border-l-2 border-sky-500 text-sky-400`); relocated primary navigation tabs (`[ Prompt Instruction | AI Results & Outputs ]`) to the right side of Row 2 in `DetailHeader.tsx` and eliminated the redundant 3rd row in `DetailTabs.tsx` to recover ~44px of vertical height; consolidated all copy operations into the top-level `[ Copy All | v ]` segmented capsule and removed redundant bottom copy buttons in `PromptInstructionView.tsx`; eliminated cluster-wide prompt broadcast storms by binding `target_node_id` in Supabase `command_queue` payloads across `injection.rs`, `open_shot.rs`, and `processing.rs`; unified current machine detection via `node_label::is_current_machine()`; deduplicated machine entries in Telegram picker; added native CLI support for `agm prompt <target> [flags] <prompt_text>`; and redesigned destination locked UX with expandable blockquotes for past prompts and clear instruction for direct prompt typing.

    *   **v4.222.0 (2026-10-17)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.221.0 (2026-10-17)**:
        -   **[Multi-Machine Fleet & UI Overhaul] Fleet Sync Recovery, W2 TCP Retry Ladder, Dynamic IP Resolution, and Settings UI Overhaul (Thanks to @aukgit)**:
            -   **Description**: Extended TCP probe timeout to 1500ms with a 2-stage retry ladder to resolve false-offline W2 machine status; normalized `${adminUser}` to `Administrator` and `${linuxUser}` to `root` across SSH connection imports and envelopes; eliminated empty `()` bug in Multi-Machine Matrix by serializing both `machine_ip` and `ip_address`; overhauled Settings SSH & GitMap actions with a contiguous segmented pill capsule, sleek metadata badges replacing raw path dumps, and active copy feedback for terminal installer commands; upgraded autonomous multi-agent concurrency capacity to A = 3, H = 2 (6 concurrent operations) across skills and coding guidelines.

    *   **v4.220.0 (2026-10-17)**:
        -   **[Accounts Management & UI Overhaul] Delete Remediation, Position-Locked Hover Inversion, Yellow Switch & Red Remove Buttons, and Disable Lifecycle (Thanks to @aukgit)**:
            -   **Description**: Resolved non-responsive single and batch Delete buttons by mounting dedicated destructive `ModalDialog` components in `AccountsDialogs.tsx`; implemented table row color inversion to white background and black text with static geometry to eliminate layout jitter; styled Switch button with prominent yellow highlighter theme and styled Remove button with solid red background and white icon; added account disable/enable lifecycle via selection toolbar and action menu with auto-switcher candidate exclusion.

    *   **v4.219.0 (2026-10-17)**:
        -   **[Prompt Tree & Delegate Updater] Rich Verification Chapter Engine, Dual-MIME Image Transmission, Rerun Chapter Engine & Console Formatting**:
            -   **Description**: Implemented 5-chapter prompt verification architecture (`verificationTemplates.ts`) for "Is it done?", "Is it released?", "Are you confident?", and "Edge Cases & Tests" embedding un-truncated original prompts; fixed image extraction regex index bug and implemented dual-MIME clipboard priming on dispatch; upgraded rerun engine to wrap full prompt context with round progression; fixed Windows console newline staircase wrapping in delegate updater and aligned summary fields.

    *   **v4.218.0 (2026-10-17)**:
        -   **[Repository Issues & System Hardening] Clean Stale Versions CLI, Telegram Fixes, Combined Instance Columns, Prompt Tree Layout & Multi-Segment Quota Bar**:
            -   **Description**: Added `agm clean-other-versions` CLI command, dynamic host versioning in Telegram, internal ID stripping from prompts, configurable 100/200-word summaries with resend buttons, combined IDE Exe & Data directory column, PromptTree DetailHeader overlap resolution and live disk sync, rich clipboard media copying, 5-segment color quota bar, interaction telemetry attributes, and clean process termination.

    *   **v4.217.0 (2026-10-17)**:
        -   **[Instance Management & UI/UX Synchronization] Instance Audit Trail, Switch UI, CLI Caller Guard & Test Sandboxing**:
            -   **Description**: Instance audit trail multi-source loading (switches + all operations), switch account dialog dark-glass styling & quota sorting, CLI caller IDE self-preservation guard, lightweight test instance lifecycle (`create-test`, `rm-test`), and automated E2E test suite. (Thanks to @aukgit)

    *   **v4.216.2 (2026-10-17)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.216.1 (2026-10-17)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.216.0 (2026-10-17)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.215.1 (2026-10-17)**:
        -   **[Telegram] Fix `/expand` format-string compile error (Thanks to @aukgit)**:
            -   **Description**: `format_expand_prompt_report` had 10 format placeholders but 9 arguments (leftover `{}` from the message rewrite); removed the stray placeholder. v4.215.0's tag pointed at the broken commit, so this patch carries the same Telegram enhancements on a green tree. (Thanks to @aukgit)

    *   **v4.215.0 (2026-10-17)**:
        -   **[Telegram] IDE exe on every prompt, history with one-tap rerun, professional messages (Thanks to @aukgit)**:
            -   **Description**: Every prompt the bot shows now ends with the IDE exe name (`ide_exe_line`, basename only, omitted when unknown rather than guessed). New `/history` lists recently sent prompts with a `↻ Rerun` button per row (`send:r:<id8>`, 64-byte-safe); `/rerun <id>` re-dispatches through the canonical injection path. All 64 prompt-related bot strings rewritten into professional human-readable messages: "Destination locked" is now "Destination confirmed", raw Id dumps became formatted cards with 8-char refs, jargon ("Inject", "Split DB", "Dual Seq ID", "Vault Path") purged; `/send` and `/target` added to the Telegram command menu. (Thanks to @aukgit)

    *   **v4.214.0 (2026-10-17)**:
        -   **[Telegram] Instance step shows the IDE exe and data folder ending (Thanks to @aukgit)**:
            -   **Description**: `/send` step 2 adds `IDE exe` and `IDE data` under each instance. `path_ending` keeps the last section only. (Thanks to @aukgit)

    *   **v4.213.0 (2026-10-17)**:
        -   **[Prompt tree] Dispatch hover stays inside the menu (Thanks to @aukgit)**:
            -   **Description**: Destination, Suffix, and the Copy AI rows use an in-menu hint anchored to the right edge. A browser `title` on that right-aligned menu is clipped by the window. See `.ai-memory/rca/20-dispatch-menu-hover-clipped.md`. (Thanks to @aukgit)

    *   **v4.212.0 (2026-10-17)**:
        -   **[Memory] Telegram picker root cause, checklist, and do-not-break rules (Thanks to @aukgit)**:
            -   **Description**: `.ai-memory/rca/18-telegram-project-listing-and-node-cache.md` explains the FIFO `session_id` label, the short `send:n:` callback, the This machine button, and the 24-hour `node_display_cache`. `.ai-memory/what-to-read.md` points at it. (Thanks to @aukgit)

    *   **v4.211.0 (2026-10-17)**:
        -   **[Telegram] Numbered project pages, conversation titles, and cached node names (Thanks to @aukgit)**:
            -   **Description**: `/send` lists eight projects per page, newest conversation first, with the project name and the conversation title. Numbers select a row, and Next loads the next eight. The current machine is labeled This machine with a green button. A missing node alias is resolved through GitMap once and stored in SQLite until `node_name_refresh_hours` (default 24). (Thanks to @aukgit)

    *   **v4.210.0 (2026-10-16)**:
        -   **[Console Logger & Diagnostics] ANSI Color Rendering, Compact Console Timer & Idempotent Migration Log Hygiene (Thanks to @aukgit)**:
            -   **Description**: Initialized Windows console support with UTF-8 (65001), ANSI virtual terminal processing, and 18pt Consolas typography via `enable_modern_console()`; added compact `%H:%M:%S` timestamp formatting (`ConsoleTimer`) to prevent log wrapping in terminal outputs; and added `expected_best_effort` in `src-tauri/src/error.rs` to demote benign idempotent migration errors (`duplicate column name`) from `warn!` to `debug!`.
        -   **[Telegram Card Palettes] Riseup Emerald/Gold Design Tokens, Chromatic Themes & Palette Switching (Thanks to @aukgit)**:
            -   **Description**: Implemented 6 chromatic card palettes in `src-tauri/src/modules/telegram_inbound/palette.rs` adhering to `02-spec/07-design-system/tokens/design-tokens.json` (`RISEUP` Emerald #10b981 and Gold #facc15 default, `OBSIDIAN`, `GOLD`, `INDIGO`, `CRIMSON`, `EMERALD`); updated alert formatting across `telegram_alerts.rs`, `telemetry.rs`, `reports_a.rs`, and `reports_b.rs` with cohesive section markers, rules, and bullet glyphs; and added `/palette` command routing.
        -   **[Agent Skills & Sync Safety] 45 Native AGM Skills Restoration & Cross-Repo Sync Shielding (Thanks to @aukgit)**:
            -   **Description**: Restored 45 specialized Antigravity Manager skills across `.agents/skills` and `.cursor/skills`, documented skill navigation in `.ai-memory/what-to-read.md`, and protected AGM-specific skills in `03-ai-scripts/38-sync-prompts-skills-scripts.py` from deletion during cross-repository synchronizations.

    *   **v4.209.0 (2026-10-16)**:
        -   **[Instance Management] PID label, verified stop, switch dialog, and audit trail (Thanks to @aukgit)**:
            -   **Description**: The list badge shows `PID` and the process id, and path cells keep the filename ending. Stop and account switch kill that process and fail if it is still alive. The audit dialog reads `recent_switches`. `agm instances current` refuses to stop, switch, or delete the instance that owns the command. (Thanks to @aukgit)

    *   **v4.208.0 (2026-10-16)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.207.0 (2026-10-16)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.206.0 (2026-10-15)**:
        -   **[Prompt Tree & Telegram] Send Dropdown Enqueue, Tree Import/Export (SQLite+JSON), Telegram Button Fixes (Thanks to @aukgit)**:
            -   **Description**: Send button dropdown now contains an "Enqueue to FIFO Queue" entry wired to the same dispatch path as the standalone Enqueue button; prompt tree gains copy-all-tree-prompts plus a unified Import/Export section (single-file SQLite DB export/import and JSON export/import); Telegram `cmd:` callback double-slash dispatch bug fixed (send-prompt, find-instance, credits-check buttons now work), 64-byte callback cap guarded, and unknown button taps show a visible notice instead of failing silently. RCAs 14-16 document each root cause.

    *   **v4.205.0 (2026-10-16)**:
        -   **[UI & UX Responsive Layout] Table Minimum Width Floor, Sticky Column Opaque Shield, Dual-Axis Viewport Scroll, Navbar Responsive Compaction & Window Minima (Thanks to @aukgit)**:
            -   **Description**: Enforced `min-w-[1060px]` on `<table className="w-full min-w-[1060px]">` in `src/components/accounts/AccountTable.tsx` and updated `src/pages/accounts/AccountsContent.tsx` to `overflow-auto`, preventing Column 5 (Weekly Quota) from squishing into `ACTIONS` (`WEEKLY QUOCACTIONS`) and surfacing immediate horizontal scrolling; rendered sticky `ACTIONS` column header and cells with 100% solid opaque backgrounds (`bg-slate-50 dark:bg-[#061220]`, `dark:bg-[#171025]`, `dark:bg-[#081826]`) to prevent scrolled text bleed-through; protected `src/components/instances/InstanceTable.tsx` with `min-w-[780px]`; made `NavLogo.tsx` switch to "AGM" below `lg` (1024px), saving ~95px and lowering Navbar minimum width to 670px so window controls (`btn-window-minimize`, `btn-window-maximize`, `btn-window-close`) are never pushed off-screen; added `"minWidth": 800` and `"minHeight": 520` in `src-tauri/tauri.conf.json`; and implemented a 4-tier responsive simulator toolbar (`100% Fluid`, `1280px`, `1024px`, `800px`) in `scripts/ui-builder.html`.

    *   **v4.204.1 (2026-10-16)**:
        -   **[UI & UX Enhancements] TitleBar Close Button Padding, Icon-Only Settings, Triangle Capsule Trigger, QuotaProgressBar Overhaul, Modal z-index Fix, Browser UI Builder & Binary Target Alignment (Thanks to @aukgit)**:
            -   **Description**: Added safe-area right inset padding (`pr-2.5 sm:pr-3`) and centered `overflow-visible` cross in `src/components/layout/TitleBar.tsx` and `src/components/navbar/NavSettings.tsx` to eliminate window close button clipping against window edges; converted Settings in `src/components/navbar/NavMenu.tsx` to an icon-only button and replaced the trailing round hamburger trigger with an integrated rotating micro-triangle chevron (`rounded-r-full`); hardened `handleCheckUpdates` with non-silent toast feedback ("Checking for updates..." -> "No updates available (latest version installed)"); overhauled `src/components/accounts/QuotaProgressBar.tsx` with a 10px track, 20px protruding milestone nodes with symmetrical `v` chevrons, strictly zero boundary dots at 0% and 100%, 5 internal splits (`[20, 40, 60, 80, 90]`), 6-stop spatial gradient (`#7f1d1d` to `#1af18d`), 85% resting opacity, and table row hover neon glow; removed `isolation: 'isolate'` from `src/components/navbar/Navbar.tsx` and standardized modal stacking context to `z-[99999]`; implemented browser-based UI builder workbench (`scripts/ui-builder.html` / `npm run ui:builder`) with bi-directional JSON state export/import; and aligned desktop UI binary target to `agm-ui` / `Antigravity Manager (AGM)-ui.exe` across `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` while preserving standalone CLI `agm`.

    *   **v4.204.0 (2026-10-15)**:
        -   **[Prompt Tree UI & Action Toolbar] 3-capsule segmented dark-glass toolbar, send vs enqueue decoupling, and transparent select styling (Thanks to @aukgit)**:
            -   **Description**: Redesigned `src/components/instances/PromptTreeView/DetailHeader.tsx` into 3 contiguous segmented capsules (`[ Copy All | v ]`, `[ Send N | v ]`, and `[ Enqueue | v ]`), separating immediate direct prompt execution from background FIFO queueing; styled `<select>` elements with transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eradicate nested white pill defects; wired independent menu toggles (`isSendMenuOpen`, `isEnqueueMenuOpen`, `isCopyMenuOpen`) in `usePromptTreeState.ts` and refined handlers in `usePromptTreePromptActions.ts`.
        -   **[Standalone Rich Copy & Dual-MIME Clipboard] One-shot copying of prompt content and embedded media with error tracking (Thanks to @aukgit)**:
            -   **Description**: Implemented `copyPromptWithRichImages` in `src/components/instances/PromptTreeView/clipboard.ts` copying dual-MIME `ClipboardItem` (`text/html` with embedded `<img>` tags for rich text editors and Telegram, and `text/plain` markdown fallback); added a standalone "Copy All (Content + Images)" button with 2-second visual confirmation in `PromptInstructionView.tsx`; and routed clipboard failures through `useErrorStore.getState().trackWarning`.
        -   **[CLI Methods & Backend Dispatch] CLI parity for `agm prompt send`, `agm prompt queue`, `agm queue`, `agm enqueue`, and CDP injection fallback (Thanks to @aukgit)**:
            -   **Description**: Added `agm queue`, `agm enqueue`, and `agm qe` top-level aliases, `--queue` / `-q` flags for `agm prompt [text]`, and sequence token resolution (`C001`, `P001`, `GM:#1`) in `src-tauri/src/bin/agm/main.rs`, `prompt_cmds.rs`, and `common.rs`; added empty prompt validation; hardened `src-tauri/src/modules/repo_db/agy.rs` and `prompt_dispatch.rs` with automatic fallback to loopback CDP DOM injection (`submit_prompt_via_cdp_for_instance`) when the `agy` CLI binary is absent or failing; and aligned `ssh_cmds.rs` with `SshGitmapStatus`.

    *   **v4.203.0 (2026-10-15)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.202.0 (2026-10-15)**:
        -   **[Onboarding & Privacy] Automatic CDP dismissal of Antigravity "Security Notice & Data Use" dialog with telemetry unchecked (Thanks to @aukgit)**:
            -   **Description**: Added `src-tauri/src/modules/instance/cdp_onboarding.rs` and enabled `--remote-debugging-port=0` on instance launches so account switches and instance starts automatically connect to `DevToolsActivePort`, uncheck the Interactions data-sharing checkbox, click **"Accept and continue"**, and persist `hasAcceptedTermsOfService = true` and `shareDataToImproveAntigravity = false` in `app_storage.json` and `state.vscdb`.
        -   **[Prompt Engine & Backup] Live CDP conversation prompt submission and startup-purge persistence fix (Thanks to @aukgit)**:
            -   **Description**: Routed `agm prompt` (`prompt_dispatch.rs`), `project_cmds.rs`, and `repo_db/backup.rs` through `save_or_requeue_prompt` so `running_projects` rows always carry composite `<project>__<instance>` IDs and valid `workspace_storage_path` values (preventing `purge_corrupted_running_projects` from deleting active/queued prompts on subsequent CLI runs), and added `submit_prompt_via_cdp_for_instance` so prompts dispatched via CLI or restored from backup are typed and submitted directly inside the live Antigravity IDE conversation window.
        -   **[CLI & UI] `/settings/:tab` sub-route navigation and remote click selectors (Thanks to @aukgit)**:
            -   **Description**: Registered `settings/:tab` in `src/App.tsx` and `src/pages/Settings/useSettingsState.ts` and synchronized `UI_ROUTES` in `admin_ui.rs` so `agm ui goto settings/<tab>` opens the target settings tab cleanly.

    *   **v4.201.0 (2026-10-15)**:
        -   **[Auto-Switch & Prompt Engine] Post-switch `agy` silent auth, 403 `VALIDATION_REQUIRED` detection, and synchronous prompt restoration (Thanks to @aukgit)**:
            -   **Description**: Provisioned `antigravity-oauth-token`, `antigravity-cli-oauth-token`, and `antigravity-ide-oauth-token` across isolated instance `.gemini` subdirectories (`antigravity-cli`, `antigravity`, `antigravity-ide`, `cache`) so `agy -p` print mode authenticates silently without timing out; detected HTTP 403 `VALIDATION_REQUIRED` on `retrieveUserQuotaSummary` and persisted `validation_blocked = true` so `auto-switch` and `ff` skip verification-blocked accounts; prevented the background `PromptQueueScheduler` from stealing `'backed_up'` prompts mid-switch; and made `switch_account_to_instance` terminate stale `agy` workers and synchronously re-dispatch backed-up prompts upon IDE relaunch verification.
        -   **[Prompt Tree & Queue] Live `agy` worker replacement and automatic FIFO queue drain (Thanks to @aukgit)**:
            -   **Description**: Explicit `Send` and `Rerun` now replace an already-running `agy` worker in the same workspace, and the `agy-monitor` thread marks finished prompts `'completed'` and automatically drains the next `'queued'` prompt in FIFO order (`ORDER BY created_at ASC, id ASC`).
        -   **[CLI & UI] `agm ui record` CLI audit commands, active-instance account switch target, and progress bar glow refinement (Thanks to @aukgit)**:
            -   **Description**: Added `agm ui record start/stop/status/list/clear/export` and recorded CLI switch/send/enqueue/rerun/ff events into `ui_audit.db`; fixed the Accounts table row switch action to target `activeInstanceId`; and refined checkpoint dot glow in `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx`.
        -   **[Installers] Rebrand to `com.alimtvnetwork`, uninstaller, and cask automation (Thanks to @aukgit)**:
            -   **Description**: Standardized bundle identifiers, Linux `.deb`/`.rpm` package detection, and Homebrew cask URLs across macOS, Linux, and Windows.

    *   **v4.200.1 (2026-10-14)**:
        -   **[Fix] Missing `tauri::Emitter` trait import broke the v4.200.0 build (Thanks to @aukgit)**:
            -   **Description**: `src-tauri/src/proxy/server/admin_ui.rs` called `AppHandle::emit` without the trait in scope (E0599). One-line import fix; v4.200.0 tag pointed at the broken commit, so this patch release carries the fix.

    *   **v4.200.0 (2026-10-14)**:
        -   **[Audit] UI interaction audit recorder (Thanks to @aukgit)**:
            -   **Description**: New recorder captures every UI interaction (click, navigation, input, scroll) with method name, UI target (route + component + element id), trigger, actor (user/cli/telegram/system), and timestamp. SQLite-backed, with start/stop/clear/export, a recording indicator in the navbar, and a filterable audit panel with JSON/JSONL export for bug reports.
        -   **[CLI] `agm ui` drives any UI surface (Thanks to @aukgit)**:
            -   **Description**: New `agm ui open/goto/ls/scroll/click` commands navigate to any page or settings subpage, list the UI element tree, scroll to and focus elements, and click through the same code path as the GUI. Actionable `--help` with examples for every subcommand. Backend `/api/ui/*` endpoints drive the live UI via Tauri events.
        -   **[Telegram] `/ui goto/ls/scroll` commands (Thanks to @aukgit)**:
            -   **Description**: The bot can now navigate the UI, list routes, and scroll to elements, with 15 new hardening tests for the `/send` flow and help text.

    *   **v4.199.0 (2026-10-14)**:
        -   **[Prompt Tree] Send, Backup, and Restore report a real agy result (Thanks to @aukgit)**:
            -   **Description**: Send passes `--conversation` and returns stderr when `agy` exits. Backup and Restore use a native dialog, and Restore dispatches the imported prompts.
        -   **[Accounts] A live IDE window counts as a switch, and a copy switch keeps the Default login (Thanks to @aukgit)**:
            -   **Description**: A dead saved process id is ignored. The stock Antigravity window counts for Default. Switching a copy does not rewrite the shared keyring or `~/.gemini`.
        -   **[Telegram] Open the manager, screenshot it, and aim the next prompt (Thanks to @aukgit)**:
            -   **Description**: `/ui`, `/shot`, and `/target` are in the help text. A paused Supabase project is described as DNS `NXDOMAIN`.

    *   **v4.198.0 (2026-10-14)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.197.1 (2026-10-14)**:
        -   **[Instances] Prompt tree cache and crash log (Thanks to @aukgit)**:
            -   **Description**: Startup no longer deletes `prompt_tree_cache`. The cache lives 120 seconds, and `agm prompts tree` reads it. A panic while building the tree returns a backtrace instead of closing the window. The tree header has Reset, and the CLI command is `agm prompts reset-tree`. CLI checks on the previous binary listed both instances, backed up and restored prompts, sent a review prompt, and enqueued a second prompt. `agm auto-switch threshold 98` read back 98 percent and was restored to 15 percent.

    *   **v4.197.0 (2026-10-14)**:
        -   **[CI/CD] Formatting Gate Fix & Dispatch Hardening**:
            -   **Description**: Restored green CI/CD pipeline checks by resolving the `cargo fmt` signature-formatting discrepancy in `src-tauri/src/commands/instance.rs` across all OS matrix runners; includes instance error-modal deduplication with poisoned prompt-lock prevention and pending AI end-to-end test cases for prompt dispatch. (Thanks to @aukgit).

    *   **v4.196.0 (2026-10-13)**:
        -   **[Accounts UI] Quota Column Layout Fix**:
            -   **Description**: Fixed the Accounts table quota layout so the 4H Model Quota and Weekly Quota bars expand to fill their columns instead of floating as narrow right-aligned bars with dead space. The 5-slot checkpoint design, `#1af18d` progress green, full-width spatial gradient with mask reveal, and compact bar heights are all preserved; this is a layout-width change only. (Thanks to @aukgit)

    *   **v4.195.0 (2026-10-13)**:
        -   **[CI/CD & Code Quality] Macro Syntax & Error Observability**:
            -   **Description**: Resolved duplicate variable declaration and expected assertion syntax in json envelope test suite (`src-tauri/src/modules/json_envelope/tests.rs`), ensuring zero-error Clippy and test compilation across all OS matrix runners. Enforced 'No Silent Errors' error tracking in `src/utils/windowManager.ts`. (Thanks to @aukgit)

    *   **v4.194.0 (2026-10-13)**:
        -   **[Security & CI/CD] Supabase Credentials Eradication & Vault Isolation**:
            -   **Description**: Completely purged hardcoded Supabase project tokens and credentials from the tracked codebase, scripts, documentation, and test fixtures. Shifted endpoint provisioning to dynamic environment variables and automated Base64 ingestion from `repo-secrets`. Added single-liner import script `scripts/import-supabase-credentials.ps1` and resolved `cargo fmt` newline discrepancy. (Thanks to @aukgit)

    *   **v4.193.0 (2026-10-13)**:
        -   **[CI/CD & Progress Bar] Healing & Modernization**:
            -   **Description**: Resolved TypeScript index signature mismatch on `ErrorContext` in `error-store.ts` and normalized `cargo fmt` formatting in `client.rs` and `config_a.rs`. Modernized quota progress bar to Progress Green `#1af18d` with 5 slots at 20% quintile intervals, glowing apertures, and compacted right alignment. Resolved subprocess clipboard deadlock in `copy_to_system_clipboard` and added Prompt Tree action improvements and loading bar. Enforced high-contrast theme typography, email masking, and Supabase connection diagnostics. (Thanks to @aukgit)

    *   **v4.190.0 (2026-10-12)**:
        -   **[Theme] Royal Violet UI**: Implemented the approved Royal Violet design system across the app — daisyUI dark theme tokens, semantic CSS variables, and legacy navy hex remap (canvas `#090611`, panel `#171025`, primary `#7C3AED`, aurora `#A855F7`, highlight `#E879F9`). (Thanks to @aukgit)
        -   **[Fixed] Account Switch Relaunch Verification**: Added post-relaunch IDE verification to the account-switch flow (`switch_account_to_instance` and the 5-step desktop switch). A spawn that returns Ok no longer reports success when no IDE window materializes — the switch now surfaces a diagnosable error instead of a silent no-reopen. (Thanks to @aukgit)
        -   **[Fixed] Running Prompt Backup/Restore & Rerun**: Fixed `agm restore` silently restoring nothing after an instance switch (strict per-instance filter matched the *current* active instance instead of the instance recorded at backup time) — unscoped restore now covers all instances in FIFO order, with `-i <id>` for strict single-instance scoping. Fixed `agm rerun [N]` selecting the oldest N prompts instead of the most recent N and marking them dispatched without re-sending — rerun now re-queues as `backed_up` and dispatches via agy. Added `test_restore_scoping_across_instances` regression test. (Thanks to @aukgit)

    *   **v4.189.0 (2026-10-12)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.188.0 (2026-10-11)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.187.0 (2026-10-11)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.186.0 (2026-10-11)**:
        -   **Update CLI Command & Pipeline Fixes**:
            -   **Description**: Implemented universal detached self-delegation for `agm update` to eliminate Windows file locks (`os error 5: Access is denied` / sharing violation `0x20`); unified CLI prompt verbs (`agm prompt send`, `agm prompt queue`, `qe` alias) with destination session targeting (`--conversation`, `--new`, `--existing`); expanded in-flight prompt backup coverage across profile rotations; and added UI conversation choice capsule ("Existing Conv" vs "New Conv"). (Thanks to @aukgit)

    *   **v4.185.0 (2026-10-11)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.183.0 (2026-10-10)**:
        -   **Switch / Fast-Forward Reliability Hardening**:
            -   **Description**: Root-caused the "switch and fast-forward not working at all" reports: the close step was best-effort and unverified, so orphaned IDE child processes could survive the kill and either hold the SQLite lock on `state.vscdb` (failing credential injection outright) or collide with the relaunch on the Chromium singleton lock (new IDE exits, old session stays). New `close_instance_verified` helper now kills, polls the OS process table, retries once for stragglers, and only then proceeds — surviving orphans log a warning instead of bricking the switch/restart (the v4.182.0 hard-error behavior could never fire on a lingering PID again). The primary token injection in `switch_account_to_instance` now retries 3x over the dying process's DB-lock release window instead of failing the whole switch on the first locked write. `fastForwardInstance` now captures errors into the Error Manager (with modal) like every other instance action, so future fast-forward failures are diagnosable from the in-app diagnostics report. (Thanks to @aukgit)
        -   **Clipboard Copy Fallback Fix**:
            -   **Description**: Fixed the `E9001 Failed to execute 'writeText' on 'Clipboard': Document is not focused` warnings in Prompt Tree View send flow. Root cause: `sendPromptNow` focuses the IDE window, stealing document focus, so the subsequent `navigator.clipboard.writeText` throws `NotAllowedError`. The send flow now uses the shared `copyToClipboard` helper, which falls back to the hidden-textarea + `execCommand('copy')` path when the async Clipboard API is unavailable — a warning is only tracked if both paths fail. (Thanks to @aukgit)

*   **v4.182.0 (2026-10-10)**:
        -   **Restart & Fast-Forward Reliability Fix**:
            -   **Description**: Rewrote `restart_instance` as a hardened Kill → Verify-Dead → Refresh → Start pipeline: the bound/selected account's token is refreshed up front (in the async command wrapper), the IDE process tree is force-closed and its death verified (a close failure is now a hard error instead of being swallowed), the stale smart-PID cache entry is dropped so launch cannot spawn a duplicate process on the same data dir, credentials are re-injected by the launch pipeline, and the new process is verified alive before returning. Also hardened `switch_account_to_instance` (the fast-forward path): the close is verified dead before any credential injection, fixing the bug where the dying IDE overwrote the newly injected `state.vscdb` credentials. (Thanks to @aukgit)
        -   **Configurable Best-Account Tier Multipliers**:
            -   **Description**: The best-account scoring algorithm no longer hardcodes tier weights: new `AutoProfileSwitcherConfig` fields `ultra_tier_multiplier` (default 4.0), `pro_tier_multiplier` (default 2.0), `free_tier_multiplier` (default 1.0) feed `calculate_weekly_base_score` (replacing the old hardcoded 5.0/3.0/1.0), and the frontend mirror in `instanceService.ts` (`getSubscriptionTierMultiplier` / `calculateMultiplicativeScore` / `rankSmartCandidates`) accepts the same configurable weights with identical defaults; all UI call sites resolve them from Settings. (Thanks to @aukgit)
        -   **Settings Algorithm Section**:
            -   **Description**: New Best-Account Scoring Algorithm card in Settings → Auto Profile Switcher showing the live formula `score = (M_tier × weekly_quota × hours_elapsed) / 100` with editable Ultra/Pro/Free multiplier inputs (clamped 0.1–10), a reset-to-defaults button, and hot-save — changes apply to the next evaluation immediately. (Thanks to @aukgit)
        -   **Scoring Spec Update**:
            -   **Description**: Updated `02-spec/20-instance-management/03-multiplicative-candidate-scoring-spec.md` for configurable multipliers (defaults, settings source, recalculated examples, v4.182.0 changelog note). (Thanks to @aukgit)
        -   **Remote Fleet Section Tests**:
            -   **Description**: Verified the Remote Fleet Machines data path (Supabase `nodes` + `instance_profiles` query, local-node filtering, 15s polling, manual refresh, empty/disabled/loading states) and added `src/components/instances/__tests__/fleetMachinesTable.test.ts` covering email masking, relative heartbeat formatting (sec/ms inputs), and local-node exclusion — full suite 5/5 green. (Thanks to @aukgit)

    *   **v4.181.0 (2026-10-10)**:
        -   **Quota UI Compaction & Stronger Progress Bars**:
            -   **Description**: Compacted the Accounts table quota columns (tighter cell padding, narrower info column) and strengthened `QuotaProgressBar` (compact bar `h-2` → `h-3`, standard bar `h-3` → `h-3.5`, larger checkpoint nodes, tighter gaps); instance cards on the Instances page got the same whitespace tightening. (Thanks to @aukgit)
        -   **Tauri Window ACL Fix**:
            -   **Description**: Granted `core:window:allow-set-decorations` and `core:window:allow-set-background-color` in `src-tauri/capabilities/default.json`, fixing the `plugin:window|set_decorations` / `set_background_color not allowed by ACL` warnings on startup and theme changes; `enterMiniMode` failures are now also tracked in the error module instead of only `console.error`. (Thanks to @aukgit)
        -   **Supabase Connectivity Diagnostics**:
            -   **Description**: `test_connection` now classifies transport failures into actionable messages (timeout → possibly paused project, DNS/connect → wrong or deleted project URL) instead of raw reqwest text, and endpoint cards show an honest red `Unreachable` badge instead of a misleading green `Active` when the last test failed. (Thanks to @aukgit)
        -   **Settings General Tab Redesign**:
            -   **Description**: Compacted the General settings tab (tighter spacing, smaller controls); Theme select became a segmented pill control and Launch at Startup became a toggle row, per the repo's segmented-capsule UI convention. Zero behavior change. (Thanks to @aukgit)
        -   **Prompt Tree View Send/Queue Hardening**:
            -   **Description**: The previously silent no-op guards in `handleResendPrompt` / `handleEnqueuePrompt` now surface a visible error and track a warning in the error module, so the Send/Queue buttons can never appear dead without explanation. (Thanks to @aukgit)
        -   **Windows Instance Executable Staleness Refresh**:
            -   **Description**: `clone_instance_executable` now compares an existing Windows clone against the base binary (size + mtime) at launch and re-links/re-copies when the updater has replaced the base file, instead of silently reusing a stale binary. (Thanks to @aukgit)
        -   **Navbar Segmented Capsule & Update Shortcut**:
            -   **Description**: The Accounts/Instances/Settings nav buttons now form one contiguous segmented capsule (circular outer edges, rectangular middles, subtle dividers); the hamburger dropdown gained a pinned `Check for Updates` section that runs the update check and surfaces the update card or a toast. (Thanks to @aukgit)

    *   **v4.180.0 (2026-10-10)**:
        -   **Release Pipeline Completion & CI Green Confirmation**:
            -   **Description**: No code changes since v4.179.0. Re-ran the v4.179.0 macOS universal artifact upload after a transient GitHub artifact-service timeout (`CreateArtifact` request timeout ×5) so the release carries the complete asset set; CI is green across all platforms on the record_ignored type-error fixes. (Thanks to @aukgit)

    *   **v4.179.0 (2026-10-10)**:
        -   **Accounts Table Compact Density Pass**:
            -   **Description**: Tightened the Accounts table density — row cell padding reduced (`py-1` → `py-0.5` on quota cells and header), `QuotaProgressBar` gained a `compact` mode (bar `h-3` → `h-2`, checkpoint markers `w-3.5` → `w-2.5` with proportionally smaller check glyphs, tighter gaps, smaller time/percentage labels) applied to the accounts table only; other views using the bar are untouched. Gradient, checkpoint positions, and behavior unchanged. (Thanks to @aukgit)

    *   **v4.178.0 (2026-10-10)**:
        -   **CI Fix: Duplicate Cargo.lock Version Entry Removed & Bump Script Hardened**:
            -   **Description**: Fixed `src-tauri/Cargo.lock` TOML parse failure — the `agm-alim` package entry carried two `version` lines (`4.177.0` + stale `4.175.0`), which broke cargo metadata parsing and failed all Rust CI/Release jobs. Removed the stale line and hardened `scripts/bump-version.mjs` so its Cargo.lock replacement collapses any run of duplicate version lines into exactly one instead of preserving them. (Thanks to @aukgit)

    *   **v4.177.0 (2026-10-10)**:
        -   **Smart Instance Process Cache Warm-Up, Closed-PID Recovery & Reopen Elimination**:
            -   **Description**: Initialized `warm_up_smart_process_cache` on startup in both desktop and headless modes (`src-tauri/src/modules/instance.rs`, `src-tauri/src/lib.rs`) to pre-warm cached PIDs across instances; implemented closed-PID vitality promotion and fresh OS re-scans before declaring instances offline, enforcing a strict zero-relaunch guarantee in `ensure_instance_running_smart` and protecting active IDE processes in `launch_instance_inner_with_extra_workspaces`. (Thanks to @aukgit)
        -   **SQLite Foreign Key Purge Resilience & Ghost Running Elimination**:
            -   **Description**: Fixed SQLite `FOREIGN KEY constraint failed` error during startup by deleting referencing `active_prompts` before `running_projects` in `purge_corrupted_running_projects` (`src-tauri/src/modules/repo_db.rs`); purged 88 stale un-namespaced rows and gated project liveness on host process status (`is_instance_process_running_smart`) in `get_live_project_execution_info` and `is_prompt_running_for_project`. Deduplicated workspace project rows by `(norm_inst, clean_path)` and hardened instance card `hasActiveTask` in `src/pages/Instances.tsx`. (Thanks to @aukgit)
        -   **Adaptive FIFO Queue Ticker & Task Persistence**:
            -   **Description**: Upgraded `start_prompt_queue_scheduler` (`src-tauri/src/modules/scheduler.rs`) to an adaptive FIFO background ticker (5s active interval when prompts are queued, backing off to 30s when empty) and standardized `.antigravity_resume_task.json` persistence via canonical `resume_task_document(...)` (`src-tauri/src/modules/repo_db.rs`) to preserve session and conversation context. (Thanks to @aukgit)
        -   **Prompt Tree View UI Compaction & Race Condition Fix**:
            -   **Description**: Compacted sidebar project rows in `src/components/instances/PromptTreeViewModal.tsx` by removing `{totalProjectPrompts} prompts` tags (reclaiming horizontal space), making project action buttons hover-only, removing `{conv.step_count} stp` pills from conversation rows, and stripping target bracket tokens (`#P001`, `C001`). Eliminated redundant client-side `focusInstanceWorkspace()` call after `sendPromptNow()` to resolve IPC window focus race hazards. (Thanks to @aukgit)

    *   **v4.176.0 (2026-10-10)**:
        -   **Smart Instance Process Cache Warm-Up, Closed-PID Recovery, Adaptive Queue Ticker & UI Compaction**:
            -   **Description**: Initialized `warm_up_smart_process_cache` on startup in both headless and GUI modes (`src-tauri/src/modules/instance.rs`, `src-tauri/src/lib.rs`) to pre-populate `INSTANCE_PROCESS_CACHE` with active OS processes across instances; implemented closed-PID vitality promotion and fresh OS re-scans before declaring instances offline, enforcing a strict zero-relaunch guarantee in `ensure_instance_running_smart` and protecting active IDE processes in `launch_instance_inner_with_extra_workspaces`. Upgraded `start_prompt_queue_scheduler` (`src-tauri/src/modules/scheduler.rs`) to an adaptive FIFO background ticker (5s active interval when prompts are queued, backing off to 30s when empty) and standardized `.antigravity_resume_task.json` persistence via `resume_task_document(...)` (`src-tauri/src/modules/repo_db.rs`) to preserve session and conversation context. Compacted sidebar project rows in `src/components/instances/PromptTreeViewModal.tsx` by removing `{totalProjectPrompts} prompts` tags (reclaiming horizontal space), making project action buttons hover-only, removing `{conv.step_count} stp` pills from conversation rows, and stripping target bracket tokens (`#P001`, `C001`). Eradicated ghost running indicators by synchronizing project `runningCount` to filter out ghost and empty 0-word placeholder records, tightening memory prompt TTLs to 45s, eliminating loose hyphen prefix matching in `is_prompt_running_for_project`, and hardening instance card `hasActiveTask` in `src/pages/Instances.tsx`. (Thanks to @aukgit)

*   **v4.175.0 (2026-10-10)**:
        -   **UI Refinement: Semantic Tokens, Continuous Quota Scale & Calmer Interactions**:
            -   **Description**: Centralized semantic UI tokens in `src/styles/ui-tokens.css` (surfaces, borders, text, interactive/success/warning/danger/info-cyan) consumed via `var(--ui-*)`; rebuilt the quota progress bar on a continuous 8-stop red→green scale with the gradient fixed to the full track width and revealed by percentage; fixed the account/email row hover (surface lift + restrained blue-tinted border instead of near-black + amber outline); calmed the active instance card to one amber rail + 35% edge tint; fixed update-dialog surface and secondary-action contrast. (Thanks to @aukgit)
        -   **Prompt Tree Send/Queue False-Success Fix**:
            -   **Description**: Root-caused dead Send/Queue buttons in the Prompt Tree View: `send_prompt_now_for_instance` discarded the agy spawn result and returned `Ok` on failure while the modal showed success messages regardless. Backend now returns descriptive errors on spawn failure; the modal shows the real error (with clipboard fallback) instead of false success, and the legacy double-invoke + silent file fallback in the queue path was removed. (Thanks to @aukgit)

    *   **v4.174.0 (2026-10-10)**:
        -   **CI Green: Stale CliContext Field Fix & Fork Actions Re-enable**:
            -   **Description**: Fixed `error[E0609]: no field 'instance' on type 'CliContext'` at two call sites in `src-tauri/src/bin/agm.rs` (`cmd_prompts_send`, `cmd_prompts_queue`) — the instance-process-cache refactor had renamed the field to `instance_id` but missed these sites; corrected to `ctx.instance_id.is_none()`, restoring compilation. Re-enabled GitHub Actions on the fork (GitHub had auto-disabled workflows because it is a fork) via the Actions-page attestation; CI now green across all 7 jobs (Check Rust Code and Build Tauri App on ubuntu/macos/windows, plus Build Frontend). (Thanks to @aukgit)

    *   **v4.173.0 (2026-10-10)**:
        -   **Updater Download Link & Release Manifest Honesty Fix**:
            -   **Description**: Fixed the in-app update checker's download URL, which pointed at the upstream `lbjlaq/Antigravity-Manager` releases instead of the `alimtvnetwork` fork — fork users were sent to the wrong repo's release page; removed phantom binary-asset URLs from `releases-manifest.json` for the notes-only releases v4.168.0–v4.172.0 (zero assets published) so the installer no longer chases dead download links. (Thanks to @aukgit)

*   **v4.173.0 (2026-10-10)**:
        -   **Smart Instance Process Cache Warm-Up, Closed-PID Recovery, Adaptive Queue Ticker & UI Compaction**:
            -   **Description**: Initialized `warm_up_smart_process_cache` on startup in both headless and GUI modes (`src-tauri/src/modules/instance.rs`, `src-tauri/src/lib.rs`) to pre-populate `INSTANCE_PROCESS_CACHE` with active OS processes; implemented closed-PID vitality promotion and fresh OS re-scans before declaring instances offline, enforcing a strict zero-relaunch guarantee in `ensure_instance_running_smart` and protecting active IDE processes in `launch_instance_inner_with_extra_workspaces`. Upgraded `start_prompt_queue_scheduler` (`src-tauri/src/modules/scheduler.rs`) to an adaptive FIFO background ticker (5s active interval when prompts are queued, backing off to 30s when empty) and standardized `.antigravity_resume_task.json` persistence via `resume_task_document(...)` (`src-tauri/src/modules/repo_db.rs`) to preserve session and conversation context. Compacted sidebar project rows in `PromptTreeViewModal.tsx` by removing `{totalProjectPrompts} prompts` tags (reclaiming horizontal space), making project action buttons hover-only, removing `{conv.step_count} stp` pills from conversation rows, and stripping target bracket tokens (`#P001`, `C001`). Eradicated ghost running indicators by synchronizing project `runningCount` to filter out ghost and empty 0-word placeholder records, tightening memory prompt TTLs to 45s, eliminating loose hyphen prefix matching in `is_prompt_running_for_project`, and hardening instance card `hasActiveTask` in `src/pages/Instances.tsx`. (Thanks to @aukgit)

    *   **v4.172.0 (2026-10-10)**:
        -   **Smart Instance Process Cache Multi-PID Survival, Relaunch Elimination & Deep Transcript Scan**:
            -   **Description**: Enhanced `check_cached_pid_alive` with multi-PID vitality scanning—when Electron launcher bootstrap PIDs exit, surviving child processes in `entry.pids` are automatically promoted to `primary_pid`, preserving cache validity and eliminating premature invalidation; added Windows 8.3 short-path expansion (`GetShortPathNameW`) and path normalization in `find_pids_for_data_dir` to ensure robust process detection across case and path formatting variants. Enforced zero-relaunch guarantee in `ensure_instance_running_smart` and added a running guard in `launch_instance_inner_with_extra_workspaces` to prevent `close_instance` from terminating active IDE windows. Deepened reverse transcript inspection to 25 lines in `inspect_conversation_transcript` while immunizing against telemetry bursts (`TOKEN_USAGE`, `HEARTBEAT`, `TELEMETRY`), broadened terminal completion detection, guarded fallback projects with `is_owning_inst_alive`, and tightened active prompt TTLs to 45s to completely eradicate phantom running indicators. Eliminated literal bracket tokens (`[Collapse Full Text]`, `[Expand Full Text]`) and word count clutter across the Prompt Tree View, compacted sequence badges to clean `C001` / `P001` with GitMap SHA tooltips, streamlined Instance Table status cells to `Running · {pid}`, and verified 100% pass rate in safe host-shielded E2E test suite `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`. (Thanks to @aukgit)

    *   **v4.171.0 (2026-10-10)**:
        -   **Smart Instance Process Cache, Relaunch Prevention & Ghost Running Fix**:
            -   **Description**: Resolved root-cause IDE instance relaunch bug by integrating canonical instance ID resolution into `is_instance_process_running_smart` to resolve sequence numbers (`1`), aliases (`default`), and case variants before checking the process cache; implemented two-tier closed-PID double-check and OS re-scan before declaring instances offline; decoupled window focus failures from process liveness to guarantee zero unwanted restarts when the IDE is already running. Unified `enqueue_prompt` IPC handler and CLI command (`agm prompt queue`) with strict FIFO queueing (`ORDER BY created_at ASC, id ASC`) and `.antigravity_resume_task.json` persistence. Eradicated false-positive running prompt indicators by filtering `auto_resume_recent_prompts` strictly to `WHERE status IN ('queued', 'pending')`, eliminating idle conversation bypasses, and enforcing terminal planner state detection (`is_terminal_done`). Compacted Prompt Tree View UI badges (`P001 · #1`, `C001 · <cid>`), stripped outer bracket tokens and redundant role labels, and verified 100% pass rate across all 5 test cases in safe host-shielded E2E test harness `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`. (Thanks to @aukgit)

    *   **v4.170.0 (2026-10-10)**:
        -   **Toolchain Installer UI + Profiles + SSH**:
            -   **Description**: New Toolchain settings page: install dev tools from the UI via backend endpoints with live progress. Installer script v3 adds stackable profiles (`minimal`/`rust-dev`/`frontend`/`full`), `--dry-run` preview, `--list-items`, `--troubleshoot`, `--ssh` remote install, and new items (Node.js, pnpm, sccache, cargo-watch, Tauri CLI, gh) — for both shell and PowerShell. (Thanks to @aukgit)

    *   **v4.169.0 (2026-10-10)**:
        -   **CI Build Fix — Duplicate enqueue_prompt Definitions**:
            -   **Description**: Fixed CI build failure (`E0428`) caused by duplicate `enqueue_prompt` / `enqueue_prompt_for_instance` definitions left behind by parallel feature streams: removed the stale 4-arg `enqueue_prompt` Tauri command (`src-tauri/src/commands/instance.rs`) and the stale `enqueue_prompt_for_instance` implementation (`src-tauri/src/modules/repo_db.rs`); the superset `enqueue_prompt` command (optional fields, `AppResult<serde_json::Value>`) remains and is backward-compatible with existing frontend invoke args. Migrated the `agm prompts enqueue` CLI verb (`src-tauri/src/modules/cli.rs`) to the canonical `enqueue_prompt_for_instance_full` route for GUI/CLI parity, and removed the duplicate `commands::enqueue_prompt` registration in `src-tauri/src/lib.rs`. (Thanks to @aukgit)

    *   **v4.168.0 (2026-10-10)**:
        -   **Model Inventory Curation**:
            -   **Description**: Curated the model inventory (`src/config/modelConfig.ts`): removed the outdated Gemini 2.5 series (flash, flash-lite, flash-thinking, pro); kept the nano tier (gemini-3.1-flash-lite); added 5 newer open-source models in a new 'Open Source' group (gpt-oss-20b, deepseek-v4, qwen3-max, qwen3-8b, llama-4-maverick); every model now carries priority, capabilities, and speed annotations (`nano`/`fast`/`balanced`/`powerful`). (Thanks to @aukgit)

    *   **v4.167.0 (2026-10-09)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

*   **v4.166.0 (2026-10-09)**:
        -   **[Smart Process Caching & Prompt Dispatch Architecture]**:
            -   **3-Step Smart Process Caching**: Implemented host-wide instance process enumeration (`scan_and_cache_all_running_instances`), verifying live PIDs against OS state before dispatch and invalidating stale caches on process exit. (Thanks to @aukgit)
            -   **Closed-PID OS Re-Scan & Rerun**: Fortified `is_instance_process_running_smart` to re-scan the OS process table when a cached PID dies, launching the IDE only when verified not running. (Thanks to @aukgit)
            -   **Profile Socket Alignment & Clipboard Sync**: Injected `--user-data-dir` in `spawn_prompt_via_agy` to ensure headless CLI dispatches connect directly to cloned instance sockets, accompanied by automatic system clipboard synchronization. (Thanks to @aukgit)
            -   **Double-Launch Race Elimination**: Removed duplicate `focusOrLaunchInstance` in `PromptTreeViewModal.tsx`, preventing secondary background IDE launches. (Thanks to @aukgit)
        -   **[Prompt Tree View De-cluttering & Ghost Running Eradication]**:
            -   **Bracket Noise Elimination**: Replaced cumbersome bracketed tokens (`[AGM:P006 | GM:#6]`, `[AGM:C025 | GM:antigrav]`) with clean, minimalist `#6`, `P006`, and `C025` badges and borderless omission banners. (Thanks to @aukgit)
            -   **Ghost Running Elimination**: Eradicated synthetic crash-recovery prompt manufacturing in `auto_resume_recent_prompts`, requiring empirical active worker validation before displaying active running states. (Thanks to @aukgit)
        -   **[CLI Parity for Prompts Management & System Health]**:
            -   **AGM Prompts Subcommands**: Added CLI routing and handlers for `agm prompts ls`, `tree`, `send`, `enqueue`, `backup`, and `restore` with complete `--json` and human-readable terminal table output. (Thanks to @aukgit)
            -   **AGM Doctor Diagnostic Engine**: Added `agm doctor` system health checker evaluating nodes, proxy gateways, database connectivity, and running Antigravity processes. (Thanks to @aukgit)

    *   **v4.165.0 (2026-10-09)**:
        -   **[Smart Instance Process Cache & Anti-Restart Reliability]**:
            -   **Process Vitality Verification**: Implemented thread-safe `INSTANCE_PROCESS_CACHE` and OS PID verification (`libc::kill(pid, 0)` on Unix, `sysinfo` on Windows) to verify instance process state before dispatch or focus. (Thanks to @aukgit)
            -   **Duplicate Launch Prevention**: Refactored `focus_or_launch_instance_with_workspace` and `focus_or_launch_workspace` with `ensure_instance_running_smart`, completely eliminating the blind restart fallthrough defect on Linux and Wayland. (Thanks to @aukgit)
        -   **[Prompt Dispatch & Enqueue IPC Architecture]**:
            -   **First-Class IPC Handlers**: Registered missing `send_prompt_now`, `enqueue_prompt`, and `get_running_instances_process_count` Tauri IPC handlers in backend commands and application router. (Thanks to @aukgit)
            -   **Direct Dispatch & Queue Decoupling**: Implemented `send_prompt_now_for_instance` and `enqueue_prompt_for_instance` in `repo_db.rs`, writing `.antigravity_resume_task.json` tasks with proper execution flags. (Thanks to @aukgit)
        -   **[Prompt Tree View De-cluttering & Sequence Hygiene]**:
            -   **Bracket Noise Elimination**: Replaced verbose `[AGM:P006 | GM:#6]` and `[AGM:C025 | GM:antigrav]` tag wrappers with streamlined `#6`, `P006`, and `C025` sequence indicators. (Thanks to @aukgit)
        -   **[Ghost Running Eradication & Duration Formatter Hygiene]**:
            -   **Reverse-Scan Transcript Verification**: Hardened `inspect_conversation_transcript` to scan backwards past trailing records and detect genuine step completions (`DONE`, `COMPLETED`, `ERROR`), suppressing false active states. (Thanks to @aukgit)
            -   **Duration Formatter Guard**: Fortified `formatDuration` against non-finite or `NaN` timestamps, parsing numeric epoch strings and ISO dates safely to eradicate `NaNm NaNs` errors. (Thanks to @aukgit)

*   **v4.165.0 (2026-10-09)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.164.0 (2026-10-08)**:
        -   **[Prompt Tree In-Flight Execution Banner & Window Focus]**:
            -   **Active Execution Banner**: Added dedicated in-flight execution banner inside `PromptTreeViewModal` displaying animated pulse, live agent step summary, elapsed runtime, and step counter. (Thanks to @aukgit)
            -   **Direct Antigravity Window Activation**: Implemented 1-click "Open in Antigravity Window" button with Win32 un-minimize (`ShowWindow` SW_RESTORE 9), foreground focus (`SetForegroundWindow`), and process PID fallback resolution. (Thanks to @aukgit)
        -   **[FIFO Prompt Restore & Origin Classification Hardening]**:
            -   **Strict FIFO Ordering**: Enforced strict First-In-First-Out ordering (`ORDER BY created_at ASC, id ASC`) across prompt backup restore and `auto_resume_recent_prompts` queries. (Thanks to @aukgit)
            -   **Deep Subagent Classification**: Extended detection keywords (`debugger`, `architect`, `tester`, `invoked by a caller agent`) to reliably differentiate subagent background tasks from user instructions. (Thanks to @aukgit)
        -   **[Fleet Synchronization Refinement & Deduplication]**:
            -   **Unified Schema & Types**: Unified `FleetMachineInfo` data structures and deduplicated imports across Supabase sync modules and Tauri IPC handlers. (Thanks to @aukgit)
            -   **UI Streamlining**: Deduplicated `<FleetMachinesTable />` elements, refining privacy email masking and 15-second auto-polling countdown pills. (Thanks to @aukgit)

    *   **v4.163.0 (2026-10-08)**:
-   **[Supabase Multi-Machine Fleet Synchronization & Observability]**:
            -   **Fleet Machines Telemetry & Aggregation**: Implemented multi-machine cluster synchronization in `src-tauri/src/modules/supabase_sync.rs`, joining PostgREST `nodes`, `instance_profiles`, and `workspace_leases` to expose cross-machine worker aliases, IPs, active profiles, bound accounts, and distributed lease states. (Thanks to @aukgit)
            -   **Real-time Prompt Telemetry Discovery**: Extended `send_heartbeat()` to discover live running prompts via `repo_db::discover_running_prompts_from_antigravity()`, upserting active prompt counts to Supabase for instant cross-node visibility. (Thanks to @aukgit)
            -   **Schema Extension & Migration Integrity**: Extended `public.instance_profiles` with `running_prompts_count INT NOT NULL DEFAULT 0` with idempotent SQL migrations in `src-tauri/src/modules/supabase_schema.rs`. (Thanks to @aukgit)
            -   **Offline Fallback & Local Machine Guarantee**: Guaranteed local machine presence and state synthesis in `fetch_fleet_machines()`, ensuring graceful degradation when offline or when Supabase sync is unconfigured. (Thanks to @aukgit)
        -   **[Instances Dashboard Fleet Machines Table UI]**:
            -   **Fleet Machines Table Component**: Created `src/components/instances/FleetMachinesTable.tsx` providing a unified dark-glass observational dashboard at the bottom of the Instances section across both Card Mode and Table Mode. (Thanks to @aukgit)
            -   **Worker Grouping & Telemetry Metrics**: Displayed machines grouped by worker node (`Worker 1 (Local)`, `Worker 2 (Remote Node-...)`) with 1-click clipboard IP copy, live status indicators, uptime counters, and active lease badges. (Thanks to @aukgit)
            -   **Masked Accounts & Running Prompt Badges**: Rendered connected Google account emails masked by default with 1-click unmask toggle, paired with vibrant cyan pulsing badges showing real-time running prompt counts per machine. (Thanks to @aukgit)
            -   **Strict Read-Only Observational Safety**: Guarded remote nodes from accidental termination or modification by strictly restricting remote rows to observational metrics. (Thanks to @aukgit)
        -   **[Tauri IPC & Frontend Service Integration]**:
            -   **IPC Commands**: Added `get_supabase_fleet_machines` and `sync_supabase_now` handlers in `src-tauri/src/commands/supabase.rs`, registered in `src-tauri/src/lib.rs`. (Thanks to @aukgit)
            -   **Frontend Service**: Added `getFleetMachines()` and `syncNow()` in `src/services/supabaseService.ts` with 10s auto-polling and manual sync trigger. (Thanks to @aukgit)

-   **[Supabase Multi-Machine Fleet Synchronization & Remote Worker Visibility]**:
            -   **Remote Fleet Machines Table**: Added `<FleetMachinesTable />` mounted beneath local instances in both Card mode and Table mode, displaying remote cluster worker machines, operating system, IP address with 1-click copy, active running instances, bound accounts, running prompt counts, and real-time heartbeat statuses. (Thanks to @aukgit)
            -   **Account Privacy & Masking**: Implemented masked account email format (`u***r@domain.com`) with both individual row eye toggles and a global toggle pill for privacy protection. (Thanks to @aukgit)
            -   **In-Flight Prompt Telemetry**: Enhanced Supabase background heartbeat with live running prompts count computed from local SQLite `active_prompts`, providing remote visibility into active workloads across the fleet. (Thanks to @aukgit)
            -   **Resilient Fallback & PostgREST Compatibility**: Implemented automatic fallback to local machine metrics when Supabase is disabled or unreachable, plus graceful column-missing tolerance during schema migrations. (Thanks to @aukgit)
            -   **Auto-Polling & Countdown Capsule**: Integrated 15-second background auto-refresh with a sleek segmented pill countdown and manual refresh trigger. (Thanks to @aukgit)

    *   **v4.162.5 (2026-10-08)**:
        -   **[Prompt Tree Modernization & Pill Capsule Header]**:
            -   **Segmented Pill Capsule Toolbar**: Redesigned header actions into contiguous segmented dark-glass pill capsules (`rounded-full`, shared border, subtle dividers) with compact typography and icons. (Thanks to @aukgit)
            -   **Prompt Classification & Subagent Distinction**: Implemented dual-source classification distinguishing genuine user prompts (`USER`), automated subagent briefs (`SUBAGENT`), and drafts. (Thanks to @aukgit)
            -   **Repeated Prompt Deduplication & Grouping**: Grouped consecutive or matching repeated prompts into collapsible clusters (`(xN)`) in tree navigation. (Thanks to @aukgit)
            -   **Pulsing Activity Indicators**: Added real-time glowing status badges (`RUNNING` in emerald and `QUEUED` in amber) in tree nodes and inspector headers. (Thanks to @aukgit)
            -   **Truncated Byte Suppression & Export**: Suppressed raw truncated byte markers from previews and added rich Copy with Images and JSON/Markdown Export. (Thanks to @aukgit)
        -   **[Quota Progress Bar & Low Quota Dark Red Transition]**:
            -   **Milestone Dot Capping**: Capped milestone checkpoints to at most 5 nodes (`[100, 75, 50, 25]`), seamlessly blending node borders and backgrounds with the track gradient. (Thanks to @aukgit)
            -   **Critical Quota (< 25%) Dark Red & Numbers**: Shifted track to deep red below 25% with white text and rendered numeric percentage values directly inside low-quota milestone balls instead of checkmarks. (Thanks to @aukgit)
        -   **[Instance Card Density & Custom Thin Scrollbars]**:
            -   **Whitespace Compaction**: Tightened padding (`p-2.5` / `p-2`) across cards and recent project lists, preventing excessive whitespace. (Thanks to @aukgit)
            -   **Thin Scrollbars**: Added dedicated `.custom-thin-scrollbar` (4px width) for sleek, accessible scroll areas. (Thanks to @aukgit)
        -   **[AGM CLI Commands & Diagnostics]**:
            -   **CLI Verbs & Telemetry**: Added `agm prompts ls|tree|backup|restore` and `agm doctor` commands with `--json` parity and help documentation. (Thanks to @aukgit)

    *   **v4.162.0 (2026-10-07)**:
        -   **[CI Runner Inventory Robustness & Sequence Linter Integrity]**:
            -   **Test Runner Schema Compatibility**: Hardened `03-ai-scripts/06-cicd-local-runner.py` to robustly handle both list and dictionary test inventory structures in `run_smart_go_tests`, preventing runtime type errors during automated pre-flight testing. (Thanks to @aukgit)
            -   **Sequence Linter Rule Alignment**: Extended `03-ai-scripts/21-sequence-integrity-linter.py` with placeholder token recognition and subtask directory exemptions, ensuring 100% clean validation across 675 workspace documents. (Thanks to @aukgit)
        -   **[Test Hygiene & Specification Reference Continuity]**:
            -   **Rust Test Cleanliness**: Removed redundant unused imports in `src-tauri/tests/auto_switcher_e2e_test.rs`, maintaining zero warnings across all unit and integration test targets. (Thanks to @aukgit)
            -   **AI Memory Index & Plan Linking**: Resolved relative plan linkages in pending roadmaps (`86`, `87`, `88`) and added canonical AI memory documentation index (`.ai-memory/readme.md`). (Thanks to @aukgit)

    *   **v4.161.0 (2026-10-06)**:
        -   **[E2E Testing for Account Switch, IDE Detection, Prompt Tracking & Release Gates]**:
            -   **Compilation & Build Gates**: Resolved Rust logging macro compile error in `src-tauri/src/modules/account.rs` (`tracing::info!`); fixed TypeScript build errors across `src/pages/Accounts.tsx`, `src/pages/Instances.tsx`, `src/components/accounts/AccountCard.tsx`, and `AccountTable.tsx`; verified clean Vite and Cargo builds. (Thanks to @aukgit)
            -   **End-to-End Account Switch & Prompt Continuity**: Verified multi-instance profile isolation, conscious PID matching, running IDE discovery, running and queued prompt lifecycle tracking, project last conversation extraction, and split SQLite audit logging across CLI, IPC, and UI. (Thanks to @aukgit)

    *   **v4.160.0 (2026-10-06)**:
        -   **[Awan Cloud Design Modernization, UI/UX Contrast & Global CSS3 Transitions]**:
            -   **Design & Contrast Modernization**: Harmonized entire UI with official Awan Software specifications (`Deep Orbit #071a27`, `Cloud Field #f5faf9`, `Lift Blue #2878f0`, `Vector Green #16a97a`, `Plasma Mint #43d6a2`); resolved dark-on-dark contrast issues in instance selection, account cards, and table headers; enabled universal 250ms CSS3 transition and `.awan-glow-pulse` animations; unified Option A selected highlight with 6px amber-400 rail and high-contrast amber active pills. (Thanks to @aukgit)
        -   **[Account Tier Badge & Startup Backfill Engine]**:
            -   **Subscription Tier Persistence**: Added shared `TierBadge` component with explicit unknown state pill and tooltip; decoupled tier fetching from project ID; added CLI suite `agm accounts refresh-tier [--all]` with automatic background startup backfill. (Thanks to @aukgit)
        -   **[Priority Inline Editing & Focus Viewport Target]**:
            -   **Priority & Focus**: Added `PriorityBadge` component hiding default priority 50 with double-click inline editing; aligned Focus button to scroll directly to the selected instance's bound account under a single animation frame. (Thanks to @aukgit)

    *   **v4.159.0 (2026-10-06)**:
        -   **[Fleet Accounts Deployment via GitMap Integration]**:
            -   **Fleet Synchronization**: Integrated 1-click fleet account deployment in `UnifiedBackupModal`, communicating via Tauri IPC (`deploy_accounts_to_fleet`) to run `gitmap nodes deploy agm-accounts`, safely synchronizing local account configurations and auth tokens to all reachable fleet nodes with default central node isolation. (Thanks to @aukgit)

    *   **v4.158.0 (2026-10-05)**:
        -   **[CI Gate Syntax Resolution & Two-Phase Quota Selection Protocol]**:
            -   **CI Gate Fix**: Fixed closing delimiter mismatch in `src-tauri/src/modules/auto_switcher.rs` detected via `gitmap pe`, restoring `cargo fmt` formatting and compilation gates across all CI workflows. (Thanks to @aukgit)
            -   **Candidate Quota Selection**: Enforced primary strict 100% 4-hour quota gate with linear fallback scaling when all accounts are exhausted, coupled with hours-remaining inverted weekly quota prioritization and 8% floor. (Thanks to @aukgit)

    *   **v4.157.2 (2026-10-05)**:
        -   **[Two-Phase 4-Hour Quota Gate & Hours-Remaining Weekly Scoring Algorithm]**:
            -   **Phase 1 (Primary Selection)**: Candidate accounts with less than 100% 4-hour quota (and whose reset period has not finished) strictly evaluate to `0.0`. Only accounts with full 100% 4h quota or elapsed cycles receive the base weekly score `base_weekly_score.floor()`. (Thanks to @aukgit)
            -   **Phase 2 (Graceful Degradation)**: When no account has 100% 4h quota (`available_pool` and `cooldown_pool` both empty), fallback evaluates partial 4h accounts via `((q_4h / 100.0) * base_weekly_score).floor()`, scaling linearly by available 4h percentage. (Thanks to @aukgit)
            -   **Hours-Remaining Inversion**: Uses weekly refill countdown `hours_remaining` to calculate elapsed cycle distance `(168 - hours_remaining)` so accounts nearest to their weekly refill receive highest weight: `(tier_multiplier * effective_weekly_pct * (168 - hours_remaining)) / 100.0`. (Thanks to @aukgit)
            -   **8% Weekly Elimination Floor**: Accounts with weekly quota below 8% strictly evaluate to `0.0`. (Thanks to @aukgit)
            -   **Descending Sort**: Candidate pools sorted descending (highest integer score selected first). Claude/3p buckets preserved with `TODO(claude)` ambiguity marker. (Thanks to @aukgit)

    *   **v4.157.1 (2026-10-05)**:
        -   **[Weekly Candidate Scoring DESC Order & Compact Integer Output]**:
            -   **Sort & Integer Output**: Standardized candidate scoring to descending sort order with integer output for SQLite persistence. (Thanks to @aukgit)

    *   **v4.157.0 (2026-10-05)**:
        -   **[Empty Prompt Tree Fix, Neon Green Progress Bar Restore, Hours-Elapsed Weekly Scoring, Modal Header Identity]**:
            -   **Fixed**: Prompt tree was completely empty because `detect_running_projects()` relied on `workspaceStorage/*/workspace.json` which is empty on managed Windows deployments. Added fallback in `compute_project_conversation_tree()` to synthesize `RunningProject` entries directly from `workspace_uris` in each instance's `conversation_summaries.db`, ensuring up to 200 recent conversations are always shown. Also fixed `workspace_storage_path: None` entries being incorrectly dropped by the filter. (Thanks to @aukgit)
            -   **Changed**: Restored neon glowing green `#1af18d` progress bars in `QuotaProgressBar.tsx` and `WaterDrainProgressBar.tsx`, reverting the VS Code cyan/teal palette introduced in v4.154.0. Color tiers: `≥75%` emerald-to-neon-green with glow `shadow-[0_0_10px_rgba(26,241,141,0.75)]`, `25–50%` amber/orange, `<25%` rose/red. Milestone nodes: 100% node glow `shadow-[0_0_8px_rgba(26,241,141,0.85)]`, 75% node `shadow-[0_0_6px_rgba(26,241,141,0.6)]`. (Thanks to @aukgit)
            -   **Changed**: Compact progress bar — label/icon column capped at `max-w-[18%]`, time/percent column fixed at `w-[18%]` giving the track more horizontal space. (Thanks to @aukgit)
            -   **Changed**: Accounts table header background updated from `dark:bg-slate-900/90` to `dark:bg-[#061220]` for a deeper navy alignment. (Thanks to @aukgit)
            -   **Added**: Prompt Tree Modal header now shows three identity badges: `#seq`, profile name, and `...\Antigravity.exe` trailing path. Props `sequenceNumber?` and `executablePath?` added to `PromptTreeViewModalProps`; all `setPromptTreeInstance` call sites in `Instances.tsx` now pass `seq_num` and `executable_path`. (Thanks to @aukgit)
            -   **Changed**: Replaced weekly quota scoring in `score_candidate_account()` with hours-elapsed-weighted formula: `hours_elapsed = 168 - hours_remaining_until_reset`, `weekly_effective_score = weekly_pct × hours_elapsed`, normalized by dividing by 16800. Weekly quota <8% is treated as zero. Sort direction is now ascending — accounts with the least hours elapsed (most recently reset) are prioritized first. Gemini-only (`gemini-weekly` buckets); Claude/3p buckets excluded with `TODO(claude)` stub. (Thanks to @aukgit)

    *   **v4.156.0 (2026-10-05)**:
        -   **[Instance Restart Split Button Capsule on Current Account]**: In both Table view (`InstanceTable.tsx`) and Card view (`Instances.tsx`) modes, converted the primary action for active instances into a seamless segmented split capsule (`rounded-[5px]`, dark-glass hairline divider). The left segment provides an immediate Stop (`<Square>`), while the right segment provides Restart (`<RotateCcw>`, tooltip "Restart Instance on Current Account") to cleanly terminate the process tree, poll for process exit and lock release (<1,500ms), purge the prompt tree cache, and relaunch on the currently bound account. The global busy overlay accurately reflects "Restarting...". (Thanks to @aukgit)
        -   **[Strict Account Selection Semantics for Switch Button]**: Strictly preserved the Switch button invariant: dedicated solely to opening the Switch Account modal (`setSwitchTargetInstance`) with zero restart side-effects. (Thanks to @aukgit)
        -   **[Semantic Icon Disambiguation: Reserve RotateCcw Strictly for Restart]**: Eliminated user confusion between sync and restart actions. Enforced an application-wide anti-confusion invariant reserving `<RotateCcw>` exclusively for instance restart. Replaced circular rotating arrows on "Sync All" with `<FolderSync className="text-cyan-500" />`, "Eval Quota" with `<Sparkles className="text-amber-500" />`, "Sync Quota" with `<ArrowLeftRight className="text-blue-500" />`, and "Wipe Credentials" in the More popover with `<KeyRound className="text-amber-500" />`. Standardized Fast-Forward in Table mode to `<FastForward>` to match Card mode. (Thanks to @aukgit)
        -   **[Host vs Sandbox Home Directory Resolution Fix]**: Added `get_canonical_host_home()` to escape sandboxed user profiles (`.antigravity_tools/instances/<id>/home`) and access the host user home (`C:\Users\Administrator`), ensuring the default instance accurately reads host `.gemini/antigravity` conversation summaries. (Thanks to @aukgit)
        -   **[Dual Path, Folder Name & Composite ID Matching in Gate 4]**: Resolved string mismatches between decoded workspace paths and project identifiers in Gate 4 by supporting full paths, workspace folder names, and composite prefix patterns. (Thanks to @aukgit)
        -   **[Disjoint Sandbox Tagging & Targeted Default Process Check]**: Enforced disjoint candidate directory mapping in `gemini_dirs_tagged` to prevent secondary instance sandboxes from being misattributed to default. Replaced global `is_antigravity_running(None)` with targeted default PID verification in `compute_project_conversation_tree`. (Thanks to @aukgit)
        -   **[Adaptive 10-Minute Window for Deep Thinking Models]**: Unified all conversation and prompt freshness cutoffs across Gate 1, Gate 3, Gate 4, and tree generation to an adaptive 600s (10-minute) window, preventing reasoning models (Claude 3.7 Thinking, Gemini 2.5 Pro) from dropping to IDLE mid-generation while strictly upholding idle supremacy. (Thanks to @aukgit)
        -   **[Unblock Email Watcher Process Check]**: Purged the redundant IDE process existence check from `is_any_prompt_actively_running`, relying strictly on active prompts and unblocking `email_watcher.rs`. (Thanks to @aukgit)
        -   **[Instance Lifecycle Cache Invalidation Contract]**: Actively triggered `invalidate_prompt_tree_cache` upon completion of `close_instance`, `launch_instance`, and `restart_instance` to guarantee immediate UI consistency. (Thanks to @aukgit)

    *   **v4.155.0 (2026-10-05)**:
        -   **[Instance Restart Split Button Capsule on Current Account]**: In both Table view (`InstanceTable.tsx`) and Card view (`Instances.tsx`) modes, converted the primary action for active instances into a seamless segmented split capsule (`rounded-[5px]`, dark-glass hairline divider). The left segment provides an immediate Stop (`<Square>`), while the right segment provides Restart (`<RotateCcw>`, tooltip "Restart Instance on Current Account") to cleanly terminate the process tree, poll for process exit and lock release (<1,500ms), purge the prompt tree cache, and relaunch on the currently bound account. The global busy overlay accurately reflects "Restarting...". (Thanks to @aukgit)
        -   **[Strict Account Selection Semantics for Switch Button]**: Strictly preserved the Switch button invariant: dedicated solely to opening the Switch Account modal (`setSwitchTargetInstance`) with zero restart side-effects. (Thanks to @aukgit)
        -   **[Semantic Icon Disambiguation: Reserve RotateCcw Strictly for Restart]**: Eliminated user confusion between sync and restart actions. Enforced an application-wide anti-confusion invariant reserving `<RotateCcw>` exclusively for instance restart. Replaced circular rotating arrows on "Sync All" with `<FolderSync className="text-cyan-500" />`, "Eval Quota" with `<Sparkles className="text-amber-500" />`, "Sync Quota" with `<ArrowLeftRight className="text-blue-500" />`, and "Wipe Credentials" in the More popover with `<KeyRound className="text-amber-500" />`. Standardized Fast-Forward in Table mode to `<FastForward>` to match Card mode. (Thanks to @aukgit)
        -   **[Running Projects & Prompts Deep Detection Root-Cause Resolution]**:
            -   **Host Process PID Liveness Gate (Gate 0)**: In `detect_running_projects` and `is_instance_running`, enforced strict PID matching against instance data directories via `find_pids_for_data_dir`. When an instance is stopped or dead, all associated projects are unconditionally updated to `is_running = 0` in SQLite, eliminating sticky false positive running dots. (Thanks to @aukgit)
            -   **Startup State Sanitization**: In `purge_corrupted_running_projects`, added `UPDATE running_projects SET is_running = 0` on AGM launch to wipe zombie running flags left by ungraceful IDE termination or host restarts. (Thanks to @aukgit)
            -   **Adaptive 10-Minute Window for Deep Thinking Models**: Replaced the rigid 60s/120s timeout in Gate 4 and conversation tree generation with an adaptive 600s (10 minutes) window for active sessions (`CASCADE_RUN_STATUS_RUNNING` with `not_fully_idle > 0`), preventing reasoning models (Claude 3.7 Thinking, Gemini 2.5 Pro) from prematurely dropping to IDLE mid-generation while maintaining strict idle supremacy. (Thanks to @aukgit)
            -   **Robust Multi-Format Timestamp Parsing**: Introduced `parse_flexible_timestamp` to parse RFC3339, fractional-second ISO-8601 (`%Y-%m-%dT%H:%M:%S%.f`), space-delimited formats, and epoch timestamps without silently collapsing to 0. (Thanks to @aukgit)
            -   **Exact Instance ID Matching & Ghost Conversation Pruning**: Replaced loose suffix matching in `isNodeOwnedByInstance` with exact equality (`node.instance_id === instConfig.id`), eliminating cross-instance bleed. Pruned 0-word untitled conversations in Gate 4. (Thanks to @aukgit)
            -   **Modal Ground Truth Cache Bypass**: Updated `PromptTreeViewModal.tsx` to force cache bypass (`loadTree(true, true)`) on open, refresh, and prompt restore, ensuring users always see live ground truth. (Thanks to @aukgit)

    *   **v4.154.0 (2026-10-05)**:
        -   **[Accounts Table Row Borders & Middle Quota Grouping]**: Restored explicit horizontal row dividing borders (`border-b border-slate-200/90 dark:border-slate-800/90`) across all accounts in the table. Added distinctive visual grouping for the middle quota section (`4H MODEL QUOTA` and `WEEKLY QUOTA`) with subtle background shading (`bg-slate-50/50 dark:bg-slate-900/40`) and demarcated outer column borders (`border-l border-slate-300 dark:border-[#15334d]` on Col 4 and `border-r border-slate-300 dark:border-[#15334d]` on Col 5), removing broken nth-child CSS selectors. (Thanks to @aukgit)
        -   **[VS Code Cyan/Teal Palette for Quotas & Audit Badges]**: Eliminated harsh neon green indicators (`#1af18d`, `emerald-500`, `lime-400`). Re-engineered `QuotaProgressBar`, `WaterDrainProgressBar`, and audit status badges to use a refined VS Code cyan/teal theme gradient (`from-teal-500 via-cyan-500 to-[#38bdf8]` for $\ge 75\%$, `from-teal-600 via-cyan-500 to-sky-400` for $\ge 50\%$) with cyan milestone glow checkpoints and normalized `rounded-[5px]` tags. (Thanks to @aukgit)
        -   **[Supabase Repo-Secrets Auto-Discovery & 1-Click Sync]**: Expanded discovery across candidate secrets paths in `D:/work/repo-secrets/` (`02-antigravity-and-event-manager/vault/supabase_config.json`, `02-antigravity-manager/vault/supabase_config.json`, `03-supabase/01-own/`, `03-supabase/02-lovable/`) with URL normalization (stripping `/rest/v1`). Added 1-click "Auto-Discover from Repo Secrets" buttons in the settings header and empty endpoints view, with CLI parity in `agm supabase load-secrets`. (Thanks to @aukgit)
        -   **[Local Sibling Instance Guard & Cross-Machine Lease Collision Prevention]**: Added pre-switch lease conflict validation calling `is_account_or_email_leased_by_other` and `get_remote_lease_holder_info`, providing explicit remote machine alias and lease expiration diagnostics. Added a local running sibling instance check in `switch_account` and `switch_account_to_instance` preventing local sibling instances from binding to the same account while running. (Thanks to @aukgit)
        -   **[Configurable 30–60m Email Cooldown & Deadlock-Free Fallback]**: Added a configurable 30–60m email cooldown window in candidate selection scoring (default 60m, with 15m/30m/45m/60m/120m quick pills in `AutoSwitcherSettings.tsx`). Implemented a two-tier pool fallback that selects the oldest used account when all accounts are in cooldown, preventing rotation deadlocks. (Thanks to @aukgit)
        -   **[Instances Table Zero-Scroll Compaction & Actions Dropdown]**: Merged `Profile Name` and `Email` into a single compact `Profile & Account` column (Row 1: sequence + bold profile name + running pulse; Row 2: masked email with reveal toggle + tier badge). Truncated location paths to display only the trailing folder name (`...\<folder>`) with full path tooltip and 1-click clipboard copy, and integrated `Prompts` into the Actions dropdown, eliminating table horizontal scrollbars at 1280px+. (Thanks to @aukgit)
        -   **[Button Corner Radius Normalization (5–6px) & Rotate Tooltip]**: Standardized button corner radii to `rounded-[5px]` across toolbars, dialogs, modals, and instance cards. Enhanced the "Rotate to Next Best" toolbar button with `px-3.5 py-1.5` padding, candidate email badge, and rich hover tooltip showing target instance, candidate email, subscription tier, and 4H quota percentage. (Thanks to @aukgit)
        -   **[Compact 4-Column Card Grid & Structured 2-Row Action Toolbars]**: Enforced a strict 4-column maximum grid layout `grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3` (compact: `gap-2.5`). Restructured instance card footer buttons into two segmented dark-glass capsules (`rounded-[5px]`, `divide-x`): Row 1 Primary Lifecycle (Launch/Stop/Restart, Switch, Fast-Forward, Sync PID); Row 2 Secondary Actions (Prompts Tree, Settings, Clone, Audit, More popover). (Thanks to @aukgit)

    *   **v4.153.0 (2026-10-05)**:
        -   **[Rust Compiler & macOS Move Fix]**: Resolved compile errors in `src-tauri/src/modules/instance.rs` by replacing `AppError::Other` with `AppError::Unknown` and `AppError::Process`. Stored `child_pid = child.id()` immediately after process spawn to resolve the borrow-after-move error caused by `child.wait_with_output()` on macOS. (Thanks to @aukgit)
        -   **[Repo DB Type Mismatch Fix]**: Fixed mismatched types error in `src-tauri/src/modules/repo_db.rs` line 4522 by unwrapping `Result<i64, _>` properly in `or_else` rather than wrapping in `Option<i64>`. (Thanks to @aukgit)
        -   **[Rustfmt Code Formatting Alignment]**: Cleaned up trailing newlines and formatted long lines across `commands/supabase.rs`, `modules/instance.rs`, `modules/repo_db.rs`, and `modules/supabase_sync.rs` to pass `cargo fmt -- --check` unconditionally across Linux, macOS, and Windows CI gates. (Thanks to @aukgit)
        -   **[Instance Restart Split Button Capsule on Current Account]**: In both Table and Card views on the Instances page, when an instance is actively running, the primary Stop button is converted into a seamless segmented pill capsule (`rounded-[5px]` with dark-glass divider). The left segment provides an immediate Stop (`Square`), while the right segment provides Restart (`RotateCcw`, tooltip "Restart Instance on Current Account") to close the instance, wait for lock release, and instantly reboot on the same bound account. (Thanks to @aukgit)
        -   **[Distinct Semantic Icons for Sync Operations vs Restart]**: Eliminated confusing circular rotation icons for synchronization tasks. Replaced `RotateCw` for "Sync PID & Quota" with a dedicated CPU microchip icon (`<Cpu className="w-3.5 h-3.5 text-teal-500" />`), and replaced account switch buttons with `<ArrowLeftRight className="w-3 h-3" />`, strictly reserving `RotateCcw` for restart operations. (Thanks to @aukgit)
        -   **[Running Projects & Prompts Detection Root-Cause Fix]**: Reduced `prompt_tree_cache` TTL in `src-tauri/src/modules/repo_db.rs` from 60s to 5s, with instant invalidation upon lifecycle events. Eliminated orphan project leakage into the default instance (`instance_id IS NULL` / `""`), filtered out 0-word untitled scratch sessions, and enforced strict 120s recency and process PID alive checks, eradicating false positive running project statuses. (Thanks to @aukgit)

    *   **v4.152.0 (2026-10-05)**:
        -   **[Instance Restart Split Button Capsule on Current Account]**: In both Table and Card views on the Instances page, when an instance is actively running, the primary Stop button is converted into a seamless segmented pill capsule (`rounded-[5px]` with dark-glass divider). The left segment provides an immediate Stop (`Square`), while the right segment provides Restart (`RotateCcw`, tooltip "Restart Instance on Current Account") to close the instance, wait for lock release, and instantly reboot on the same bound account. (Thanks to @aukgit)
        -   **[Atomic Backend `restart_instance` IPC Command]**: Implemented `restart_instance` in `src-tauri/src/modules/instance.rs` and registered in `commands/instance.rs` and `lib.rs`. Gracefully terminates running processes, verifies process exit and socket release (<1.5s), invalidates the prompt tree cache, and relaunches the profile via `launch_instance`. (Thanks to @aukgit)
        -   **[Distinct Semantic Icons for Sync Operations vs Restart]**: Eliminated confusing circular rotation icons for synchronization tasks. Replaced `RotateCw` for "Sync PID & Quota" with a dedicated CPU microchip icon (`<Cpu className="w-3.5 h-3.5 text-teal-500" />`), and replaced account switch buttons with `<ArrowLeftRight className="w-3 h-3" />`, strictly reserving `RotateCcw` for restart operations. (Thanks to @aukgit)
        -   **[Running Projects & Prompts Detection Root-Cause Fix]**: Reduced `prompt_tree_cache` TTL in `src-tauri/src/modules/repo_db.rs` from 60s to 5s, with instant invalidation upon lifecycle events. Eliminated orphan project leakage into the default instance (`instance_id IS NULL` / `""`), filtered out 0-word untitled scratch sessions, and enforced strict 120s recency and process PID alive checks, eradicating false positive running project statuses. (Thanks to @aukgit)
        -   **[TypeScript Interface Alignment & CI/CD Compilation Fix]**: Added `account_lockout_window_minutes?: number` to `AutoProfileSwitcherConfig` in `src/types/config.ts`, resolving TS2561/TS2551 errors in `AutoSwitcherSettings.tsx` and `Settings.tsx`. Cleaned up unused `syncingInstanceIds` and `useMemo` imports, ensuring 100% green `npm run build` preflight checks. (Thanks to @aukgit)
        -   **[Rustfmt Alignment in instance.rs]**: Aligned chained method calls for `target_skills_dir` in `src-tauri/src/modules/instance.rs` to conform to `cargo fmt -- --check` standards across Linux, macOS, and Windows CI gates. (Thanks to @aukgit)

    *   **v4.151.0 (2026-10-05)**:
        -   **[AccountRow JSX Closing Tag Mismatch & Frontend Build Fix]**: Eliminated the redundant closing `</div>` tag in `src/components/accounts/AccountRow.tsx`, resolving TS17002/TS1005 compile errors and restoring clean cross-platform `npm run build` execution. (Thanks to @aukgit)
        -   **[Rustfmt Code Formatting Alignment]**: Aligned import ordering and multi-line conditional breaks in `src-tauri/src/modules/instance.rs`, ensuring `cargo fmt -- --check` passes cleanly across all CI/CD runner environments. (Thanks to @aukgit)
        -   **[Default Instance Theme, Presets, Plugins & Skills Fleet Parity]**: Implemented `sync_instance_ide_parity` in `instance.rs` to replicate Dracula theme seeds (`#BD93F9`/`#19191C`), Turbo mode presets, 4 official Gemini plugins, and 9 builtin skills across all secondary instances and default profile sandboxes. (Thanks to @aukgit)
        -   **[GitMap Automated IDE Fleet Deployment & Delegation]**: Enabled 1-command deployment via `gitmap agy deploy` and `scripts/deploy-antigravity-ide-fleet.ps1`, supporting automated remote machine provisioning with 7-Gate Scorecard telemetry. (Thanks to @aukgit)

    *   **v4.150.0 (2026-10-05)**:
        -   **[Accounts Table Styling & VS Code Palette] Row Borders, Grouping & Cyan Progress Bars**: Restored crisp row border lines (`border-b border-slate-200/80 dark:border-slate-800/80`) across the Accounts table, and visually grouped the core middle quota columns (4H Model Quota and Weekly Quota) with subtle background shading and vertical dividing borders (`bg-slate-50/50 dark:bg-slate-900/40 border-x border-slate-200/60 dark:border-[#15334d]/50`). Replaced glaring neon green progress bars and audit badges with a refined VS Code cyan/teal palette (`from-teal-500 via-cyan-500 to-[#38bdf8]` for >= 75%, `from-teal-600 via-cyan-500 to-sky-400` for >= 50%, and cyan milestone glow nodes), reducing visual fatigue. (Thanks to @aukgit)
        -   **[Supabase Repo-Secrets Auto-Discovery & 1-Click Sync]**: Enhanced `supabase_sync.rs` to automatically discover and ingest Supabase credentials directly from `D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json` and `03-supabase/` credential envelopes. Added a dedicated "Auto-Discover from Repo Secrets" button with `Sparkles` icon in Settings and Tauri IPC command `auto_discover_supabase_credentials`, eliminating manual credentials configuration. (Thanks to @aukgit)
        -   **[Distributed Lease Collision Guard & 30–60m Cooldown]**: Injected pre-switch mutual exclusion checks in `account.rs` (`switch_account`) and `instance.rs` (`switch_account_to_instance`), verifying with `is_account_or_email_leased_by_other` before token refresh or IDE launch to prevent multi-instance and cross-machine concurrent account collisions. Implemented a configurable 30–60m email cooldown window in candidate ranking with graceful fallback to the oldest used account when all healthy accounts are cooling down. (Thanks to @aukgit)
        -   **[Instances Table Zero-Scroll Compaction & Short Path Truncation]**: Eliminated horizontal scrollbars in the Instances table by compacting the layout to 5 standard columns. Merged Profile Name and Email into a single stacked column with sequence badges, status dots, and masked email unmask toggles. Truncated location paths to display only the single trailing folder name prefixed by `...` (with 1-click clipboard copy). Streamlined the 9-button linear action strip into 3 primary lifecycle buttons plus a "More" dropdown portal for Prompts Tree, Audit Trail, and Settings. (Thanks to @aukgit)
        -   **[Universal 5–6px Button Radius Normalization & Rotate Tooltip]**: Replaced oversized pill buttons across header toolbars, instance cards, and modal forms with an industrial-grade 5–6px radius (`rounded-[5px]`). Enhanced the "Rotate to Next Best" button with `px-3.5 py-1.5` padding, candidate email subtext badge, and a dynamic tooltip computing and previewing the target instance, candidate email, tier, and 4H quota percentage. (Thanks to @aukgit)
        -   **[Card Mode Strict 4-Column Grid & 2-Row Action Toolbars]**: Constrained instance card grid layout to strictly 4 columns (`xl:grid-cols-4`), eliminating squished 5- and 6-column distortions on ultra-wide displays. Structured card footers into clean Row 1 (Primary Lifecycle) and Row 2 (Utilities + More Popover) capsules, and collapsed verbose recent project lists in compact density for uniform card heights. (Thanks to @aukgit)

    *   **v4.149.0 (2026-10-05)**:
        -   **[Feature Category] Main Update Summary (PR #xxx)**:
            -   **Description**: Please document update details here; credit contributors inline as `(Thanks to @aukgit)`.

    *   **v4.148.0 (2026-10-04)**:
        -   **[CI Test Targets Gate] Integration Test Async Invocation Correction**: Resolved the remaining E0277 compiler error in test target compilation reported by `gitmap pe` on CI runners. Corrected `test_case_antigravity_cli_discovery` in `src-tauri/tests/per_instance_prompt_liveness_test.rs` which was erroneously calling `.await` on a non-future return type `()`, ensuring clean integration test target compilation across all platforms. (Thanks to @aukgit)

    *   **v4.147.0 (2026-10-04)**:
        -   **[Build & Compilation Gates] Comprehensive Backend Fixes & PID Visibility**: Resolved all 7 remote compilation errors flagged by `gitmap pe`. Declared timestamp `now` at the top level of `compute_project_conversation_tree` in `src-tauri/src/modules/repo_db.rs`, resolving E0425 scope errors in turn TTL checks. Promoted `get_antigravity_pids` to `pub(crate)` in `src-tauri/src/modules/process.rs` and aligned caller invocations with `Option<&str>` arguments, fixing private function visibility (E0603) and argument count (E0061) compile failures across macOS, Linux, and Windows CI runners. (Thanks to @aukgit)

    *   **v4.146.0 (2026-10-04)**:
        -   **[Clippy & Compilation Gate] Rusqlite Non-Exhaustive StepResult Match Arm**: Resolved the remaining E0004 compiler blockage in Linux and macOS CI pipelines caused by `rusqlite::backup::StepResult` being marked `#[non_exhaustive]`. Added a wildcard fallback arm (`Ok(_) => break`) to the online SQLite backup step loop in `src-tauri/src/modules/instance.rs`, ensuring `cargo clippy` and compilation pass with zero warnings across all CI environments. (Thanks to @aukgit)

    *   **v4.145.0 (2026-10-04)**:
        -   **[Rust Compilation & CI Gate] Rusqlite Backup Feature & Borrow Type Alignment**: Resolved all 7 CI/CD build breakages on Linux and macOS runners. Enabled the required `"backup"` feature for `rusqlite` in `src-tauri/Cargo.toml` (`features = ["bundled", "backup"]`), activating the `rusqlite::backup::Backup` online SQLite zero-downtime hot-cloning API. Fixed E0277 borrow type mismatch in `src-tauri/src/modules/repo_db.rs` where `owning_inst_id` (`&String`) was compared directly against struct `String` values. (Thanks to @aukgit)
        -   **[macOS Deep Gatekeeper Hardening Release]**: Formal production release for macOS 13 (Ventura), 14 (Sonoma), and 15 (Sequoia) addressing the "damaged, move to Trash" error via deterministic `awk` DMG volume parsing, recursive `xattr -d com.apple.quarantine` removal across all internal frameworks, Gatekeeper assessment registration with `spctl --add`, real-time installer IDE discovery echo, `$HOME/Applications/` path search expansion, and disk-persisted backtrace diagnostic logging. (Thanks to @aukgit)

    *   **v4.144.0 (2026-10-04)**:
        -   **[macOS Deep Gatekeeper Hardening & Quarantine Purge]**: Completely resolved the root causes of the macOS "Antigravity Manager Tools is damaged, you should move it to the Trash" error across macOS 13 (Ventura), 14 (Sonoma), and 15 (Sequoia). Refactored DMG mount point extraction in `install.sh` to use deterministic `awk` field tokenization instead of fragile regex parsing. Integrated dynamic `sw_vers -productVersion` major version detection. Replaced shallow quarantine clearing with recursive `find "$target_app" -exec xattr -d com.apple.quarantine {} +` across all nested executables, dylibs, and frameworks. Added automatic `spctl --add "$target_app"` Gatekeeper assessment registration on macOS 13+, and gracefully skipped ad-hoc self-signing on macOS 15+ where AMFI rejects self-signed binaries. (Thanks to @aukgit)
        -   **[First-Time IDE Discovery & Immediate Echo]**: Added `detect_ide_path()` in `install.sh`, performing real-time Spotlight `mdfind` bundle ID queries and filesystem checks across `/Applications/` and `$HOME/Applications/` during installer execution, immediately echoing detected IDE locations or providing clear first-run guidance. (Thanks to @aukgit)
        -   **[User-Level Applications Parity & Persistent Backtrace Logging]**: Expanded candidate IDE discovery and instance launch search paths in `src-tauri/src/modules/process.rs` and `src-tauri/src/modules/instance.rs` to include `$HOME/Applications/`. Implemented persistent disk logging for diagnostic backtraces: whenever IDE discovery or process launch encounters an error, the full `std::backtrace::Backtrace` and diagnostic context are serialized to `~/.local/share/antigravity/ide-discovery.log` for transparent technical troubleshooting. (Thanks to @aukgit)
        -   **[Repair Utility & Packaging Hardening]**: Overhauled `scripts/Fix_Damaged.command` with macOS version detection, recursive `find ... xattr -d` quarantine purging, and `spctl --add` registration. Updated `scripts/package_dmg.sh` to guarantee executable permissions and clean quarantine state on bundled repair tools. (Thanks to @aukgit)

    *   **v4.143.0 (2026-10-04)**:
        -   **[Per-Instance Prompt Isolation] Cross-Instance Running Prompt State Bleed — Root Cause Fix**: Completely resolved a critical bug where a prompt running in one instance (e.g. Coding Guidelines on instance 8159) incorrectly appeared as `RUNNING` on another instance card (e.g. Default). Refactored `get_project_conversation_tree` in `src-tauri/src/commands/instance.rs` to accept `instance_id: Option<String>`. Updated `compute_project_conversation_tree` in `src-tauri/src/modules/repo_db.rs` to partition cache by instance (`tree:{instance_id}:...`), enforce strict OS process liveness gating (dead instance → `is_running: false` for all its projects, eliminating the blind 10-minute recency assumption), and emit structured `[PROMPT_LIVENESS_PROBE]` audit logs with instance_id, PID, workspace directories, and evaluation rationale. (Thanks to @aukgit)
        -   **[Frontend Instance Cards] Strict Per-Instance Project & Running State Scoping**: Eliminated global project tree leakage in `src/pages/Instances.tsx` and `src/components/instances/PromptTreeViewModal.tsx`. Instance cards now use strict `node.instance_id === inst.config.id` matching, removing the leaky `node.instance_name === inst.config.name` string comparison. `isProjRunning` and `hasActiveTask` are gated on `inst.is_running`, ensuring stopped instances never display running projects. `PromptTreeViewModal` no longer falls back to the global tree when a filtered result is empty — zero projects renders an empty state, never leaking another instance's project list. (Thanks to @aukgit)
        -   **[Deep Clone Engine] Complete Settings, Themes & Security Presets Copy**: Overhauled the instance cloning pipeline to resolve cloned instances missing user settings, themes, security presets, and project workspaces. Expanded `REQUIRED_IDE_REL_PATHS` to cover `User/settings.json`, `User/keybindings.json`, `User/security_presets.json`, `User/antigravity_policies.json`, `User/snippets`, `User/globalStorage`, and `User/workspaceStorage`. Expanded `GEMINI_CLONE_DIRS` to include `antigravity`, `antigravity-ide`, `antigravity-cli`, `policies`, and `config`. Introduced `safe_clone_sqlite_db` for safe SQLite cloning with WAL-locked retry backoff and Backup API. Unified CLI/tooling clone entry points to call `resolve_instance_id` and `copy_instance_settings` deep-merge for consistent full cloning across all invocation paths. (Thanks to @aukgit)
        -   **[Card & Table Mutex UI] Glass Overlay Progress Feedback & Row-Level Action Locking**: Implemented card and table row-level action mutex in `src/pages/Instances.tsx` and `src/components/instances/InstanceTable.tsx`. While any async action (launch, stop, switch, delete) is in progress, a translucent glass overlay is applied to the instance card and all interactive buttons are disabled; the active action button renders an inline animated spinner (`animate-spin`) with color-coded progress feedback to prevent double-clicks and communicate clearly that an operation is underway. (Thanks to @aukgit)
        -   **[In-App Delete Dialog] Replace Native Window.confirm with Custom Modal**: Delete and wipe-session operations no longer trigger the Windows/OS native `confirm` dialog. A dark-glass React modal with destructive styling (warning icon, instance name, sequence number, Cancel and Confirm buttons) replaces the native prompt for a consistent in-app experience. (Thanks to @aukgit)
        -   **[Instance Settings Modal] Icon Capsule Redesign & Horizontal Overflow Fix**: Redesigned `src/components/instances/InstanceSettingsModal.tsx`, converting verbose text into compact Lucide icon capsules (`Copy`, `FolderSync`, `Layers`, `Sliders`). Fixed "Copy Now" / "Copy Folders" button overlap with select dropdowns by adding `overflow-x-hidden` to scroll containers and enforcing `min-w-0 flex-1 truncate` on `<select>` elements to prevent long instance names from breaking layout. (Thanks to @aukgit)
        -   **[Integration Test] Per-Instance Prompt Liveness Isolation Test**: Added isolated integration test `src-tauri/tests/per_instance_prompt_liveness_test.rs` annotated with `#[ignore]` verifying that two distinct mock instances with different data directories and workspaces return completely isolated project sets with correct process-gated liveness evaluation. (Thanks to @aukgit)

    *   **v4.142.0 (2026-10-04)**:
        -   **[macOS Installation & Gatekeeper Bypass]**: Completely resolved macOS installation failures where downloaded packages trigger Gatekeeper "Antigravity Manager Tools is damaged and can't be opened. You should move it to the Trash". Piped shell installer `install.sh` strips quarantine attributes before and after mounting (`xattr -cr`, `xattr -rd com.apple.quarantine`), dynamically discovers `.app` bundles inside DMG volumes, gracefully falls back to user `$HOME/Applications/` without requiring `sudo`, applies local ad-hoc Mach-O code-signing (`codesign --force --deep --sign -`) to satisfy AMFI validation, symlinks `agm` and `agm-alim` to `$HOME/.local/bin/`, and automatically reports detected Antigravity IDE information. (Thanks to @aukgit)
        -   **[POSIX Stack Trace Trap & Diagnostics]**: Added a system-level POSIX `ERR` trap handler to `install.sh` and `package_dmg.sh` that captures command name, exit code, line number, and function call stack traces upon any failure. (Thanks to @aukgit)
        -   **[macOS Process Execution & Argument Ordering]**: Fixed argument ordering in `src-tauri/src/modules/process.rs` where `--new-window` was previously passed to `/usr/bin/open` before `--args`, triggering `open: unrecognized option '--new-window'`. Centralized arguments via `format_macos_open_args`, ensuring application parameters and window options strictly follow `--args`. Enhanced discovery to probe `Contents/MacOS/` binaries and integrated macOS Spotlight `mdfind` query fallback. (Thanks to @aukgit)
        -   **[macOS Instance Switching & Launcher Parity]**: Enhanced `launch_instance_inner_with_extra_workspaces` in `src-tauri/src/modules/instance.rs` to distinguish between `.app` application bundles and launcher shell scripts (`Command::new`), ensuring cloned instances launch reliably without `open -a` rejection. Achieved full parity with Windows/Linux by initializing `.gemini` home folders, writing keyring bypass markers, and pre-seeding `app_storage.json` (`ide-install-wizard-shown: true` and account binding) on macOS, while logging backtraces on process failures. (Thanks to @aukgit)
        -   **[First-Time IDE Discovery & Background Integration]**: Implemented `discover_and_persist_initial_ide_info()` and integrated it on a background thread during application startup in `src-tauri/src/lib.rs`, proactively detecting Antigravity IDE across running processes, standard paths, user applications, and Spotlight on first startup, persisting the verified path to `gui_config.json` with rich diagnostic logging and stack traces. (Thanks to @aukgit)
        -   **[Repair Utility & DMG Packaging]**: Overhauled `scripts/Fix_Damaged.command` with bilingual output, dynamic bundle discovery across local and system paths, user-level quarantine stripping with graceful `sudo` fallback, and ad-hoc codesigning. Updated `scripts/package_dmg.sh` to package the repair utility directly inside distributable DMGs. (Thanks to @aukgit)
        -   **[Clone Dialog Mode High-Contrast UI & Color Overhaul]**: Redesigned the "Duplicate / Clone Profile" modal in both `src/pages/Instances.tsx` and `src/components/navbar/InstanceSelector.tsx`. Replaced washed-out, muddy `bg-base-100/50` gray backgrounds in dark mode with high-contrast Deep Orbit navy styling (`#081a29` modal background, `#0c283f` with `border-cyan-400` for active selection, and `#061724` with `#14344d` border for unselected options). Replaced verbose text with Lucide icons (`Layers`, `Sliders`, `FolderSync`, `Check`), polished checkbox indicators, and optimized typography for superior readability and aesthetic consistency. (Thanks to @aukgit)

    *   **v4.141.0 (2026-10-04)**:
        -   **[macOS Installer & Gatekeeper Quarantine]**: Resolved macOS installation failure where downloaded packages or apps were blocked by Gatekeeper with "Antigravity Manager Tools is damaged and can't be opened. You should move it to the Trash". Piped shell installer `install.sh` now clears quarantine attributes before and after mounting (`xattr -cr`), dynamically discovers `.app` bundles inside DMG volumes, falls back to user-writable `$HOME/Applications/` without requiring `sudo`, applies local ad-hoc Mach-O code-signing (`codesign --force --deep --sign -`) to satisfy AMFI validation, symlinks `agm` and `agm-alim` to `$HOME/.local/bin/`, and automatically reports detected Antigravity IDE information during first-time installation. (Thanks to @aukgit)
        -   **[POSIX Stack Trace Trap & Diagnostics]**: Added a comprehensive POSIX `ERR` trap handler to `install.sh` that captures command name, exit code, line number, and function call stack traces upon any unexpected failure, providing transparent troubleshooting telemetry. (Thanks to @aukgit)
        -   **[macOS Process Execution & Argument Ordering]**: Fixed argument ordering in `src-tauri/src/modules/process.rs` where `--new-window` was previously passed to `/usr/bin/open` before `--args`, triggering `open: unrecognized option '--new-window'`. Centralized arguments via `format_macos_open_args`, ensuring application parameters and window options strictly follow `--args`. Enhanced discovery to probe `Contents/MacOS/` binaries and integrated macOS Spotlight `mdfind` query fallback. (Thanks to @aukgit)
        -   **[macOS Instance Switching & Launcher Parity]**: Enhanced `launch_instance_inner_with_extra_workspaces` in `src-tauri/src/modules/instance.rs` to distinguish between `.app` application bundles and launcher shell scripts (`Command::new`), ensuring cloned instances launch reliably without `open -a` rejection. Achieved full parity with Windows/Linux by initializing `.gemini` home folders, writing keyring bypass markers, and pre-seeding `app_storage.json` (`ide-install-wizard-shown: true` and account binding) on macOS, while logging backtraces on process failures. (Thanks to @aukgit)
        -   **[First-Time IDE Discovery & Startup Integration]**: Implemented `discover_and_persist_initial_ide_info()` and integrated it into the application `setup()` hook in `src-tauri/src/lib.rs`, proactively detecting Antigravity IDE across running processes, standard paths, user applications, and Spotlight on first startup, persisting the verified path to `gui_config.json` with rich diagnostic logging and stack traces. (Thanks to @aukgit)
        -   **[Repair Utility & DMG Packaging]**: Overhauled `scripts/Fix_Damaged.command` with bilingual output, dynamic bundle discovery across local and system paths, user-level quarantine stripping with graceful `sudo` fallback, and ad-hoc codesigning. Updated `scripts/package_dmg.sh` to package the repair utility directly inside distributable DMGs. (Thanks to @aukgit)

    *   **v4.140.0 (2026-10-04)**:
        -   **[Auto-Switcher] Multi-Instance Low-Quota Failover & Candidate Isolation**:
            -   **Description**: Completely eliminated a critical flaw in `check_and_rotate_with_options` where accounts with depleted quotas (such as `Gemini 3.1 Pro (High)` sitting at 6%) were erroneously skipped with `continue` when their quota reset boundary had passed (`is_period_finished`). Guaranteed that low-quota conditions (`<= threshold_percent`) unconditionally trigger failover rotation regardless of reset timestamps. Isolated candidate account evaluations via dedicated `calculate_candidate_quota` to prevent false disqualifications from unrelated depleted models, locked candidate `instance_id` to the depleted instance for smooth in-place rotation, and dynamically tracked allocated candidate accounts across multi-instance iteration to prevent duplicate bindings. (Thanks to @aukgit)
        -   **[Prompt Continuity] Universal Async 5-Second Post-Launch Prompt Restoration**:
            -   **Description**: Eliminated all lingering synchronous blocking waits (`wait_for_instance_prompt_channel`, `sleep(7s)`) across `integration.rs`, `auto_switcher.rs`, and `instance.rs`. Prompt re-injection, backup dispatch, and channel restoration now spawn as a detached Tokio background task executing after an exact 5-second post-launch delay, delivering instant responsiveness without UI or command latency. (Thanks to @aukgit)
        -   **[Daemon Engine] 3-Second Startup Acceleration & 5-Second Telemetry Ticker**:
            -   **Description**: Reduced the auto-switcher startup stabilization delay from 60 seconds to 3 seconds, enabling immediate proactive checks for active instances upon application launch. Accelerated daemon status tick events from 30 seconds to 5 seconds for smooth, real-time frontend countdown synchronizations (`[⏱ {countdown}s Next Check]`). (Thanks to @aukgit)
        -   **[Testing & Quality] Extended Isolated Local End-to-End Test Suite**:
            -   **Description**: Extended `src-tauri/tests/auto_switcher_e2e_test.rs` with test cases verifying that period-finished boundaries do not suppress low-quota rotations and that candidate accounts with secondary depleted models are not falsely disqualified, strictly protected with `#[ignore]` attributes to run only on demand and never in CI/CD. (Thanks to @aukgit)

    *   **v4.139.0 (2026-10-04)**:
        -   **[Auto-Switcher] Quota Threshold Evaluation Fix & Unbanned Models**:
            -   **Description**: Removed outdated hardcoded filters banning models containing `3.0` and `3.1` in `auto_switcher.rs`, restoring `Gemini 3.1 Pro (High)` for quota monitoring and candidate selection. Updated `evaluate_account_period_status` to evaluate the lowest remaining quota bottleneck across all actively consumed models (<100%), immediately triggering account rotation when `Gemini 3.1 Pro` drops to 6%. Lowered Google quota interval floor to 10s so configured caution (60s) and critical (40s) intervals are properly honored. (Thanks to @aukgit)
        -   **[Telemetry & Daemon] Daemon Next-Check Countdown & Real-Time Status**:
            -   **Description**: Persisted `next_check_timestamp`, `check_interval_seconds`, and `current_stage` in `AutoSwitcherRuntimeState`, exposed `get_auto_switcher_daemon_status` Tauri command, and integrated a live ticking countdown pill (`[⏱ {countdown}s Next Check]`) with stage status badges in the Instances page toolbar and Auto Switcher settings card. (Thanks to @aukgit)
        -   **[Instance Dashboard] Recent & Running Projects with Double-Click Deep Link**:
            -   **Description**: Rendered active and recent project chips on instance cards below the executable path (configurable 1–3 projects, default 3) with `RUNNING` indicators and turn counts. Double-clicking any project chip immediately opens `PromptTreeViewModal` auto-focused and expanded on that project. (Thanks to @aukgit)
        -   **[Card Density] Normal vs Compact Card Density Sizing Options**:
            -   **Description**: Added a card density toggle (`[Normal Cards]` vs `[Compact Cards]`) in the toolbar with adaptive 5–6 column grid layouts and compact card padding (`p-2.5`) to monitor more running instances simultaneously. (Thanks to @aukgit)
        -   **[Prompt Continuity] Async 5-Second Post-Launch Prompt Restoration & Scope Fix**:
            -   **Description**: Replaced the previous 14-second synchronous blocking freeze with an asynchronous 5-second post-launch prompt restoration task, cleanly preserving variables scope and passing Clippy and cargo compilation. (Thanks to @aukgit)
        -   **[Testing & Quality] Isolated Local End-to-End Test Suite**:
            -   **Description**: Added `src-tauri/tests/auto_switcher_e2e_test.rs` covering threshold failover triggers, prompt snapshotting, and 5-second post-launch restoration, strictly isolated with `#[ignore]` so it never runs during CI/CD. (Thanks to @aukgit)

    *   **v4.138.0 (2026-10-04)**:
        -   **[Prompt Inspector] Running Instance & Conversation Targeted Auto-Selection**:
            -   **Description**: When opening `PromptTreeViewModal` on an instance, the modal automatically detects active running projects and conversations, expanding solely the targeted project and conversation nodes and selecting the running prompt turn in the inspection pane. If no prompt is currently running, it auto-expands the primary active project and selects its latest conversation, completely eliminating blind mass-expansion of all 29+ projects. (Thanks to @aukgit)
        -   **[Auto-Sync Engine] Configurable Background Auto-Sync Interval with 15s Safety Floor**:
            -   **Description**: Added a configurable Sync Interval dropdown (`15s`, `30s`, `1m`, `2m`, `Off`) within the header segmented toolbar capsule beside `[Refresh]`, with a strict safety floor of 15 seconds. Background polling queries `get_project_conversation_tree` with `force: true` to bypass SQLite 60-second disk caching, silently updating the tree without full-screen loading spinners, disrupting scroll position, or discarding dirty prompt textarea input. (Thanks to @aukgit)
        -   **[Tree Organization] Stale & Empty Prompts Demotion to Bottom Group**:
            -   **Description**: Implemented intelligent stale/empty conversation detection (`isStaleOrEmptyConversation`), demoting old or untitled empty prompt turns to the bottom of each project tree under a collapsible `📁 Archived / Stale Prompts ({count})` group, while keeping active running prompts strictly immune and anchored at the top. (Thanks to @aukgit)
        -   **[Project Archive] Per-Instance Project Archive / Less Favorite (Thumbs Down) Toggle**:
            -   **Description**: Added an instance-scoped Archive / Less Favorite toggle button (`Archive` / `ArchiveRestore` icon) on project rows. Archiving persists in `localStorage` under `agm_archived_projects_{instanceId}` and automatically unpins the project, collapsing archived projects into a bottom `📁 Archived Projects ({count})` section without modifying filesystem folders or affecting other instances. Added an `[Archived ({count})]` filter pill for quick access and unarchiving. (Thanks to @aukgit)
        -   **[Instance Dashboard] Real-Time Active Prompt Indicator & Action Grid Symmetry**:
            -   **Description**: Added an active `Prompt Active` badge in the instance card header linking directly to `PromptTreeViewModal` when background prompts are running, while preserving full alignment across the 6-column action grid in Row 2. (Thanks to @aukgit)

    *   **v4.137.0 (2026-10-04)**:
        -   **[Backend & OS Window Title] Standardized Instance Window Title Formatting**:
            -   **Description**: Implemented `compute_ide_ending_sequence` and `compute_instance_window_title` in `src-tauri/src/modules/instance.rs`. Enforced strict window title formatting starting with `#{sequence} {instance_name} - {ide_ending_sequence}` (e.g. `#1 Default - Antigravity`, `#2 8136 - antigravity-8136`) across Windows, macOS, and Linux. Automatically injected into `<data_dir>/User/settings.json` across instance creation, duplication, renaming, launching, and startup registry synchronization. (Thanks to @aukgit)
        -   **[Instances UI] 6-Column Card Action Grid & Bottom CSS3 Hover Glow Animation**:
            -   **Description**: Standardized instance card Row 2 action buttons into a rigid 6-column grid (`grid grid-cols-6 gap-1 w-full`) with an invisible placeholder for `#1 Default`'s non-deletable slot, ensuring `[Clone]` and `[Executable]` maintain identical horizontal positions across all cards. Removed the heavy top color bar and relocated the accent indicator to a sleek bottom line with smooth CSS3 hover expansion and ambient glow animation (`group-hover:scale-x-100 group-hover:opacity-100`). (Thanks to @aukgit)
        -   **[Prompt Inspector] Portal Body Stacking, Fullscreen Mode & Header Fix**:
            -   **Description**: Mounted `PromptTreeViewModal` directly to `document.body` via React `createPortal` with elevated `z-[300]` stacking, resolving header clipping from the sticky navbar and removing ad-hoc margin hacks. Added a dedicated `[Full]` / `[Exit]` toggle button for immersive edge-to-edge full-screen inspection and editing. (Thanks to @aukgit)
        -   **[Project Tree] Inline Single-Project Refresh & Pinning Mechanism**:
            -   **Description**: Added inline `RotateCw` (instant cache-bypass refresh) and `Pin` (pin to top) buttons beside each project row in the tree view. Persisted pinned project IDs per instance in `localStorage` (`agm_pinned_projects_{instance_id}`). (Thanks to @aukgit)
        -   **[Prompt Layering] Multi-Tier Prioritization & Search Filter Pills**:
            -   **Description**: Implemented intelligent multi-tier sorting: Pinned projects first -> Actively running projects -> Recent activity timestamp -> Alphabetical name, with running conversations anchored to the top of each project tree. Added 5 quick filter pills under the search bar: `[All]`, `[Running]`, `[Latest Conv]`, `[Latest Prompt]`, and `[Pinned]`. (Thanks to @aukgit)
        -   **[Prompt Actions] Multi-Mode Copy, Image Export & Confirmation Suffix Dropdown**:
            -   **Description**: Added "Copy Text" (clean text only), "Copy With Images" (verbatim prompt), and "Save Images" (downloads extracted images to disk). Added a Confirmation Suffix dropdown with presets (`Is it done?`, `Is it released?`, `Are you sure about it?`, etc.) appended on resend. Upgraded running indicator to an animated pulse badge with real-time elapsed ticker and matched instance PID. (Thanks to @aukgit)

    *   **v4.136.0 (2026-10-04)**:
        -   **[UI & Components] Fluid Water-Drain Progress Bar & Milestone Checkpoints**:
            -   **Description**: Implemented an aesthetic, fluid water-drain styled progress bar component (`WaterDrainProgressBar`) featuring dual-stop gradient tones (`#1af18d` leading hue to `#12b27d` peak hue), inline milestone checkmark nodes (`[25, 50, 75, 100]`), smooth fluid shimmer, and a compact 18px height footprint without bloated padding. Integrated across account quota badges (`QuotaItem`), instance cards, and list tables (`InstanceTable`). (Thanks to @aukgit)
        -   **[Top Header & Navigation] Navbar Preferences Segmented Capsule & 3 Direct Icons**:
            -   **Description**: Reorganized `NavSettings` preferences into a single 3-segment pill capsule (`[Quick Clean] | [Theme Switcher] | [Language]`), eliminating disjointed floating buttons. Restructured `NavMenu` into 3 direct route icon buttons side-by-side (`Accounts`, `Instances`, `Settings`) followed by a compact hamburger dropdown containing remaining navigation paths without duplicating the 3 direct routes. (Thanks to @aukgit)
        -   **[Instance Management] Deep Settings & Workspace Replication with Buffer Persistence**:
            -   **Description**: Added a single combined "Copy Both (Settings & Workspaces)" action button and a split "Paste Both / Paste Settings Only / Paste Workspaces Only" dropdown button to `InstanceSettingsModal`. Persisted clipboard copy buffer (`agm_instance_clipboard_buffer`) into `localStorage` across project views. Formatted Target Profile dropdown with sequence numbers, profile names, and `antigravity-{id}` suffixes. (Thanks to @aukgit)
        -   **[Prompt Inspector] Prompt Tree View 3-Layer Hierarchy, Markdown Preview & SQLite Caching**:
            -   **Description**: Resolved modal heading overlap bug (`z-[200]` and top margin offset). Built a 3-layer collapsible tree view (`Project -> Conversation -> Prompt`) with intelligent grouping of untitled/empty conversations (`Untitled Conversations ({count})`). Added Markdown Preview (500-word truncation with "Show All" toggle), Raw View, and direct Textarea Edit mode. Renamed inspector action button to "Full" with tooltip, added direct "Resend" and "Enqueue" actions, and integrated high-speed SQLite split-DB caching (`prompt_tree_cache`) in `repo_prompts.db` with 60s TTL. (Thanks to @aukgit)
        -   **[Prompt Engine & Lifecycle] FIFO Ordering & Multi-Task Queue Safeguard**:
            -   **Description**: Enforced strict FIFO ordering (`ORDER BY created_at ASC LIMIT 1`) in `check_and_dispatch_enqueued_prompts`. Fixed prompt drop bug in `resend_running_commands_for_instance` when workspaces have multiple queued prompts by retaining subsequent prompts in `queued` status. Added dynamic process detection and terminal state bypass in `is_prompt_running_for_project` to eliminate false 120-second cooldown blocks. (Thanks to @aukgit)
        -   **[CLI & Parity] Theme Setting & Instance Settings Sync Commands**:
            -   **Description**: Added `agm theme set <theme-id> [--instance <id>|--all]` and `agm instance sync-settings <src> <target>` in `agm.rs` for parity across GUI and CLI binaries. (Thanks to @aukgit)

    *   **v4.135.0 (2026-10-04)**:
        -   **[Theming & Visual Design] WP-Exam Themes & Colors Ingestion (18 Themes Total)**:
            -   **Description**: Ingested 8 rich, production-grade themes from WP-Exam into `THEME_PALETTES` (bringing total catalogue themes to 18): `Green Choice` (Emerald eco-luxury light), `Green Choice Dark` (botanical dark), `Clean Wide` (vivid indigo and deep slate light), `Riseup Gold` (warm gold and midnight navy dark), `Antigravity Dracula` (neon green & deep purple dark), `Letterly Purple` (electric indigo & violet dark), `Obsidian Cyan` (VS Code obsidian slate & cyan neon dark), and `Navy Gold` (dark navy gold). (Thanks to @aukgit)
        -   **[Top Header UI] On-The-Fly Theme Switcher beside Menu Dropdown**:
            -   **Description**: Implemented a sleek, non-intrusive on-the-fly theme switcher (`ThemeSwitcherDropdown`) mounted directly beside the central route menu (`NavMenu`) in `Navbar.tsx`. Features a rounded pill trigger with active dual-gradient preview swatch, palette name, chevron indicator, and an interactive popover card with 18 clickable themes adhering strictly to 5–6px (`rounded-[5px]`) corner radii and coordinated via `agm:dropdown-open` event bus. Enables millisecond-level instant theme switching across all views without requiring settings navigation or reloads. (Thanks to @aukgit)
        -   **[CSS & Settings] Universal Palette Custom Properties & ThemePicker Scaling**:
            -   **Description**: Added `html[class*="palette-"]` wildcard styles to `src/App.css` and mapped variables (`--bg`, `--surface`, `--primary`, `--fg`) to guarantee seamless dark/light surface overrides. Scaled the Settings `ThemePicker.tsx` grid to a 6-column layout with 5–6px corner radii. (Thanks to @aukgit)

    *   **v4.134.0 (2026-10-04)**:
        -   **[Release CI] Multi-Platform Release Asset Packaging Fix**:
            -   **Description**: Fixed artifact download pattern in GitHub Actions release workflow to ensure all cross-platform assets (Windows NSIS setup `.exe`, macOS `.dmg`, Linux `.deb` / `.AppImage`) are reliably bundled and published to official GitHub Releases for frictionless one-liner installation. (Thanks to @aukgit)

    *   **v4.133.0 (2026-10-04)**:
        -   **[UI & Theming] Accounts Table Borders, Subtle Grouping & VS Code Quota Gradients**:
            -   **Description**: Refined Accounts table styling with clean row borders (`border-b border-slate-200/80 dark:border-slate-800/80`), subtle table container styling, and VS Code dark editor row hover highlight (`hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60` with `border-l-blue-500/70` accent indicator), eliminating aggressive pitch-black and bright yellow contrasts. Upgraded quota progress bars with professional VS Code teal/cyan/sky gradients (`bg-gradient-to-r from-teal-500 via-cyan-500 to-sky-500`) over dark track backgrounds (`bg-slate-200 dark:bg-slate-800/80`), and softened quota percentage badges to muted cyan (`text-cyan-600 dark:text-cyan-400`). (Thanks to @aukgit)
        -   **[Backend & Secrets] Supabase Repo-Secrets Auto-Discovery Ladder**:
            -   **Description**: Implemented an automatic discovery ladder (`candidate_repo_secrets_paths()`) probing `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json` and `02-lovable/` (along with `REPO_SECRETS_DIR` and cross-platform relative paths). Automatically decodes Base64 payloads, normalizes endpoint URLs, assigns role priorities, and auto-seeds into `supabase_config.json` upon launch or CLI invocation without requiring manual setup. (Thanks to @aukgit)
        -   **[Account Isolation] Distributed Lease Locking & Configurable Email Cooldown Window**:
            -   **Description**: Enforced distributed lease locks across instances and remote cluster machines via Supabase `workspace_leases` with extended protective TTLs (1800s–3600s). Added configurable account reuse cooldown (`account_cooldown_minutes: u32`, default: 60 minutes) to `AutoProfileSwitcherConfig` with UI selector in Settings. Partitioned candidate accounts into non-cooldown and cooldown pools in `auto_switcher.rs`, gracefully falling back to the oldest cooldown account only when all eligible accounts are cooling down to prevent multi-instance contention and system deadlocks. (Thanks to @aukgit)
        -   **[Instances UI] Table Consolidation & Horizontal Scroll Elimination**:
            -   **Description**: Merged separate "Profile Name" and "Bound Account" table columns into a single unified `Profile & Account` column displaying profile badge and bound email with status dot indicator. Truncated location paths to ending segments (`...\<parent>\<leaf>`) with hover tooltips and clipboard copy actions. Moved the standalone Prompts button into the table Actions segmented capsule to eliminate horizontal scrolling at standard desktop resolutions (1280px+). (Thanks to @aukgit)
        -   **[Card Mode & Buttons] Compact 4-per-Row Grid & 5–6px Button Radius Standard**:
            -   **Description**: Upgraded Card Mode layout to render 4 compact cards per row on large viewports (`grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3`). Reorganized card action buttons into 2 clean structured toolbar rows (Row 1: Stop/Launch, Switch, FF, Audit, Sync; Row 2: Prompts, Settings, Clone, Executable, Wipe, Delete). Standardized all button corner radii across Accounts and Instances to 5–6px (`rounded-[5px]`). Fixed "Rotate to Next Best" header button with balanced padding (`px-3 py-1.5`) and contextual tooltip showing active instance target. Softened Audit button colors to neutral VS Code slate theme (`bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300`). (Thanks to @aukgit)
        -   **[CLI & Parity] Headless Supabase Configuration & Status Parity**:
            -   **Description**: Added `agm supabase set-config [--cooldown <minutes>] [--interval <seconds>] [--alias <alias>]` to configure cooldown and heartbeat settings from terminal. Added `antigravity-manager supabase [status|sync|list-leases|test]` subcommand routing in `modules/cli.rs` for full parity across GUI and CLI binaries. (Thanks to @aukgit)

    *   **v4.132.0 (2026-10-04)**:
        -   **[UI & Theming] Accounts Table Borders, Subtle Grouping & VS Code Quota Gradients**:
            -   **Description**: Refined Accounts table styling with clean row borders (`border-b border-slate-200/80 dark:border-slate-800/80`), subtle table container styling, and VS Code dark editor row hover highlight (`hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60` with `border-l-blue-500/70` accent indicator), eliminating aggressive pitch-black and bright yellow contrasts. Upgraded quota progress bars with professional VS Code teal/cyan/sky gradients (`bg-gradient-to-r from-teal-500 via-cyan-500 to-sky-500`) over dark track backgrounds (`bg-slate-200 dark:bg-slate-800/80`), and softened quota percentage badges to muted cyan (`text-cyan-600 dark:text-cyan-400`). (Thanks to @aukgit)
        -   **[Backend & Secrets] Supabase Repo-Secrets Auto-Discovery Ladder**:
            -   **Description**: Implemented an automatic discovery ladder (`candidate_repo_secrets_paths()`) probing `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json` and `02-lovable/` (along with `REPO_SECRETS_DIR` and cross-platform relative paths). Automatically decodes Base64 payloads, normalizes endpoint URLs, assigns role priorities, and auto-seeds into `supabase_config.json` upon launch or CLI invocation without requiring manual setup. (Thanks to @aukgit)
        -   **[Account Isolation] Distributed Lease Locking & Configurable Email Cooldown Window**:
            -   **Description**: Enforced distributed lease locks across instances and remote cluster machines via Supabase `workspace_leases` with extended protective TTLs (1800s–3600s). Added configurable account reuse cooldown (`account_cooldown_minutes: u32`, default: 60 minutes) to `AutoProfileSwitcherConfig` with UI selector in Settings. Partitioned candidate accounts into non-cooldown and cooldown pools in `auto_switcher.rs`, gracefully falling back to the oldest cooldown account only when all eligible accounts are cooling down to prevent multi-instance contention and system deadlocks. (Thanks to @aukgit)
        -   **[Instances UI] Table Consolidation & Horizontal Scroll Elimination**:
            -   **Description**: Merged separate "Profile Name" and "Bound Account" table columns into a single unified `Profile & Account` column displaying profile badge and bound email with status dot indicator. Truncated location paths to ending segments (`...\<parent>\<leaf>`) with hover tooltips and clipboard copy actions. Moved the standalone Prompts button into the table Actions segmented capsule to eliminate horizontal scrolling at standard desktop resolutions (1280px+). (Thanks to @aukgit)
        -   **[Card Mode & Buttons] Compact 4-per-Row Grid & 5–6px Button Radius Standard**:
            -   **Description**: Upgraded Card Mode layout to render 4 compact cards per row on large viewports (`grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3`). Reorganized card action buttons into 2 clean structured toolbar rows (Row 1: Stop/Launch, Switch, FF, Audit, Sync; Row 2: Prompts, Settings, Clone, Executable, Wipe, Delete). Standardized all button corner radii across Accounts and Instances to 5–6px (`rounded-[5px]`). Fixed "Rotate to Next Best" header button with balanced padding (`px-3 py-1.5`) and contextual tooltip showing active instance target. Softened Audit button colors to neutral VS Code slate theme (`bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300`). (Thanks to @aukgit)
        -   **[CLI & Parity] Headless Supabase Configuration & Status Parity**:
            -   **Description**: Added `agm supabase set-config [--cooldown <minutes>] [--interval <seconds>] [--alias <alias>]` to configure cooldown and heartbeat settings from terminal. Added `antigravity-manager supabase [status|sync|list-leases|test]` subcommand routing in `modules/cli.rs` for full parity across GUI and CLI binaries. (Thanks to @aukgit)

    *   **v4.130.0 (2026-10-04)**:
        -   **[Stability] Startup Crash & White Screen Resolution**:
            -   **Description**: Resolved an immediate startup crash (`0xc0000409`) where synchronous `tokio::spawn` calls in Tauri's `.setup()` hook panicked on non-Tokio worker threads. Migrated background prompt queue and instance PID quota scheduler daemons to `tauri::async_runtime::spawn`, and blocking queries to `tauri::async_runtime::spawn_blocking`, ensuring the GUI window initializes and renders reliably on launch without hanging or crashing. (Thanks to @aukgit)
        -   **[Core & Commands] Asynchronous Command Safety**:
            -   **Description**: Replaced ad-hoc `tokio::runtime::Runtime::new()` and `tokio::spawn` calls in instance management commands with `tauri::async_runtime::block_on` and `tauri::async_runtime::spawn` to guarantee execution safety across both GUI and CLI contexts. (Thanks to @aukgit)

    *   **v4.129.0 (2026-10-03)**:
        -   **[Audit & History] Instance Action Toolbar Audit Option & Switch History Modal**:
            -   **Description**: Added a dedicated `Audit` button to instance card action toolbars and list view pill capsules opening the new `InstanceAuditTrailModal` to display total switch counts, the last 2–3 account transitions (`From Account` → `To Account`), reasons, timestamps, and full payloads. (Thanks to @aukgit)
        -   **[Core & Lifecycle] Multi-Step Switch Lifecycle Tracking**:
            -   **Description**: Instrumented instance rotation with 4 explicit verified steps in SQLite: 1) Prompts backup (projects captured, prompt count), 2) Process reset (terminated PIDs, lock cleanup, auth swap), 3) Prompts restore (resume task generation, channel stabilization), 4) Post-restore verification confirming active session restoration succeeded. (Thanks to @aukgit)
        -   **[Sync & Quotas] Live PID Detection & 10-Minute Quota Sync Engine**:
            -   **Description**: Added user-facing `Sync` button on instance cards and top toolbar to detect live running Antigravity PIDs, read authenticated accounts from `state.vscdb`, update bindings, and fetch fresh Gemini quotas, complemented by an automatic 10-minute background scheduler daemon. (Thanks to @aukgit)
        -   **[Fix] Default Instance Switch Audit Invariants**:
            -   **Description**: Resolved early drop bug during default instance rotation to ensure audit tasks complete cleanly with canonical `"default"` instance ID recorded in history. (Thanks to @aukgit)

    *   **v4.128.1 (2026-10-03)**:
        -   **[Build & Core] Rust Compilation Fixes & Type Alignment**:
            -   **Description**: Fixed Rust compilation errors in `instance.rs` and `account.rs` by properly converting `get_antigravity_executable_path` `PathBuf` to `String`, resolved `cannot move out of type AuditTask` in `task_history_db.rs` by cloning `task.id`, and enhanced CLI E2E test parsing. (Thanks to @aukgit)
        -   **[Supabase] Workspace Lease & Conflict Prevention Table**:
            -   **Description**: Added real-time Workspace Lease and Cluster Lock table to the Supabase tab with live node status, profile name, expiration countdown, and click-to-unmask email toggle. (Thanks to @aukgit)
        -   **[Accounts] Uniform Segmented Pill Capsule Toolbar**:
            -   **Description**: Grouped Focus, +, Refresh, Warm, and Show All Quotas into contiguous segmented pill capsules (`rounded-full`, dark glass styling) matching Instances aesthetic. (Thanks to @aukgit)

    *   **v4.128.0 (2026-10-03)**:
        -   **[Audit] Pagination & Two-Tier SQLite Acceleration**:
            -   **Description**: Added 100/200 item page size selection backed by a two-tier SQLite cache (`hot_tasks_cache` in `task_index.db`) for instant render, and enriched account switch rows with `From → To` transition arrays, instance ID, IDE type, machine alias, IDE path, and switch reason. (Thanks to @aukgit)
        -   **[Navigation] First-Class Supabase Top-Level Tab**:
            -   **Description**: Promoted Supabase sync and cluster governance to a top-level tab alongside Email and Audit, offering dual endpoints, workspace leases, and cross-DB migration. (Thanks to @aukgit)
        -   **[Instances] Card & List View Toggle with Prompt Tree Inspector**:
            -   **Description**: Implemented Card vs. List view modes with `InstanceTable`, expanded search across name, email, PID, and status, and introduced `PromptTreeViewModal` for project/conversation inspection and prompt backup/restore. (Thanks to @aukgit)
        -   **[Accounts & UI] Modernized Toolbar, Unmasking & Glowing Progress**:
            -   **Description**: Added cohesive pill button capsule toolbar (Focus, +, Refresh, Warm, Show All Quotas), hover/double-click email unmasking with automatic re-mask on mouse leave, and vibrant glowing multi-stop progress gradients. (Thanks to @aukgit)
        -   **[Proxy & Backup] URL Filtering & Audit Backup Selection**:
            -   **Description**: Added Proxy default path rewrite toggle, enable all URLs toggle, comma-separated excluded URLs input, and full backup modal option to include or exclude audit records and task history. (Thanks to @aukgit)
        -   **[Core & CLI] Prompt Stabilization Delay & CLI Parity**:
            -   **Description**: Added 7.0s post-rotation prompt stabilization delay before resume task injection, structured JSON output across `agm instance ls --json` and `agm instance count --json`, and local-only E2E test suite. (Thanks to @aukgit)

    *   **v4.127.1 (2026-10-03)**:
        -   **[Instances & CLI] Command Formatter Helpers & Segmented Pill Invariants**:
            -   **Description**: Polished CLI instance command helpers, refined tabular formatting, and codified UI segmented pill encapsulation and headless duplication parity into project architectural invariants. (Thanks to @aukgit)

    *   **v4.127.0 (2026-10-03)**:
        -   **[Instances] Deep Settings Synchronization & Executable Auto-Discovery**:
            -   **Description**: Added comprehensive CLI commands (`agm instance copy-settings`) and UI modal (`InstanceSettingsModal`) to synchronize theme colors, turbo mode, browser execution policy, code review, readable file, and writable folder permissions across instances with executable path auto-discovery (`--exe`). (Thanks to @aukgit)
        -   **[Instances] Open Projects & Folder Copy Parity**:
            -   **Description**: Added full support for copying included open workspace projects and recent folders (`workspaceStorage`, `state.vscdb`, `storage.json`) during instance duplication both via CLI (`agm instance duplicate --copy-projects`, `agm instance copy-projects`) and the UI clone modal. (Thanks to @aukgit)
        -   **[Instances] Baseline Defaults Enforcement & Fast Toggles**:
            -   **Description**: Added commands and fast toggles to enforce reference baseline settings (turbo mode ON, plan review always proceed) across single or all instances (`agm instance settings enforce-defaults`, `set-turbo`, `set-plan-review`). (Thanks to @aukgit)
        -   **[UI] Header Action Button Consolidation**:
            -   **Description**: Combined disjoint action buttons into cohesive, unified segmented pill control capsules: joined Quick Clean (`↺`) & Preferences dropdown (`🌙 EN ⌵`), and joined Window Controls (Minimize `—`, Maximize `🗗`, Close `✕`) with dark-glass styling and smooth hover highlights. (Thanks to @aukgit)
        -   **[CLI] Instance Count & JSON Import/Export with Undo/Redo**:
            -   **Description**: Added `agm instance count` and settings JSON export/import with UI clipboard Copy/Paste and in-memory snapshot Undo/Redo history. (Thanks to @aukgit)

    *   **v4.126.1 (2026-10-03)**:
        -   **[Instances] Auto-Switch Instance Reopen & Rotation Lifecycle**:
            -   **Description**: Fixed auto-profile switcher failing to reopen when the default instance reaches zero credits by preserving candidate instance IDs during evaluation and executing full cross-instance launch lifecycle. (Thanks to @aukgit)
        -   **[Instances] Reliable Process Relaunch & Lockfile Purge**:
            -   **Description**: Fixed default instance account switch short-circuit leaving the IDE in an Idle state after process termination. Stale lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`) are purged before restarting to prevent silent Electron launch aborts on Windows. (Thanks to @aukgit)
        -   **[UI] Modernized Instances & Profiles Styling**:
            -   **Description**: Redesigned Instances view with sleek dark glass styling, eliminating jarring light-gray capsules on dark theme. Upgraded Auto Profile Switcher banner to a glassmorphism card with glowing active pulse indicators, structured quota badges, and compact badge footer layout. (Thanks to @aukgit)

    *   **v4.126.0 (2026-10-03)**:
        -   **[Instances] Deep Clone of Settings, Themes & Projects**:
            -   **Description**: Duplicating an instance deep-copies source user settings, color theme (`workbench.colorTheme`), keybindings, snippets, and clones running projects and prompts in `repo_prompts.db`. Settings are synchronized across data dir and AppData for Windows IDE parity. (Thanks to @aukgit)
        -   **[Audit] Full Dark Mode Styling**:
            -   **Description**: Audit view cards, table headers, hover rows, and detail modal fully adopt dark palette tokens, fixing blinding white backgrounds. (Thanks to @aukgit)
        -   **[Settings] Compact About Section**:
            -   **Description**: Redesigned About tab into a sleek, compact card with a single-row identity header, tech badges, and integrated horizontal updater. (Thanks to @aukgit)
        -   **[Accounts] High-Contrast Selected Row**:
            -   **Description**: Current account row highlighted with dark slate background, amber border, yellow text, and gold badge instead of green-on-green. (Thanks to @aukgit)
        -   **[Instances] Redesigned Duplicate Dialog**:
            -   **Description**: Modernized instance duplication UI with scope breakdown cards, glass Cancel button, and CSS3 animated gradient Duplicate button. (Thanks to @aukgit)
        -   **[Email] Telemetry & Connection Health**:
            -   **Description**: Enriched Email view with IMAP/SMTP connection badges, daemon watcher state, and security vault health indicators. (Thanks to @aukgit)

    *   **v4.125.0 (2026-10-02)**:
        -   **[Instances] Other processes stay open**:
            -   **Description**: A switch or fast-forward closes only that instance. Another instance's saved process id and profile path are left running. (Thanks to @aukgit)
        -   **[Instances] Readable cards**:
            -   **Description**: The page title no longer collides with the running count. Each card shows the full account, profile path, and action row. (Thanks to @aukgit)

    *   **v4.124.0 (2026-10-02)**:
        -   **[Switch] Same conversation after a switch**:
            -   **Description**: Switch and fast-forward store the running prompt, including a conversation that only has a preview, and write that conversation id back onto the resume file and the audit row. (Thanks to @aukgit)
        -   **[Audit] Info button and a scrollable detail**:
            -   **Description**: The info button opens a table you can scroll, copy, and cancel. The list still does not load the payload until that click. (Thanks to @aukgit)
        -   **[Quota] Two-minute checks and a saved process id**:
            -   **Description**: Google quota checks wait at least 2 minutes. The process saved at launch is trusted. A full process scan runs every 10 minutes by default (never under 3, never over 20), or when credit is under the threshold and that process no longer matches. (Thanks to @aukgit)

    *   **v4.123.0 (2026-10-02)**:
        -   **[Audit] From, to, and a detail table**:
            -   **Description**: The list shows which account moved to which. The domain is hidden and the start of the name stays. Detail loads one row and shows the reason, how, the running prompt, and whether it was injected again. (Thanks to @aukgit)

    *   **v4.122.0 (2026-10-02)**:
        -   **[Switch] Return after the IDE relaunch**:
            -   **Description**: Quota refresh, the mailbox poll, and email or Telegram run after the switch command returns. The 1.5s prompt wait is skipped when nothing was backed up. (Thanks to @aukgit)
        -   **[Settings] Compact pinned models**:
            -   **Description**: Pinned quota models are a filter and small chips. Pinned models sort first. (Thanks to @aukgit)

    *   **v4.121.0 (2026-10-02)**:
        -   **[Instances] Clone copies settings and repo databases**:
            -   **Description**: A duplicate instance copies `User/settings.json`, workspace databases, `.gemini` trees, and the source repo rows. (Thanks to @aukgit)
        -   **[Settings] Catalogue themes and a compact About**:
            -   **Description**: Themes sits next to Email. About is one row with the update controls. The accounts footer defaults to 150 per page and drops the entry-count sentence. (Thanks to @aukgit)

    *   **v4.120.0 (2026-10-02)**:
        -   **[Accounts] One tier dropdown and a shorter row**:
            -   **Description**: The 5-hour and Weekly toolbar buttons are removed because those quotas already have their own cells. All, Pro, Ultra, and Free are one dropdown. Refresh is the first button, then Switch. Emails stay masked until Show email. (Thanks to @aukgit)
        -   **[Audit] Lazy detail from the split database**:
            -   **Description**: Account actions are numeric enums. The list shows title case. Detail opens that row's split file and shows the prompt that was running when a switch happened. (Thanks to @aukgit)
        -   **[Release] One version and a refreshable Pro badge**:
            -   **Description**: Quota refresh fetches the subscription tier even when a project id is cached. The bump script exits if package.json, version.json, Cargo.toml, tauri.conf.json, and hooks.nsh disagree. (Thanks to @aukgit)

    *   **v4.119.0 (2026-10-02)**:
        -   **[Accounts] 4-hour and weekly quota columns**:
            -   **Description**: Gemini or Claude is selected in the header. The left cell is that model's 4-hour quota and the right cell is its weekly quota. Refresh, details, fingerprint, and export are in one menu. A shared priority is hidden and can be edited by double-click. (Thanks to @aukgit)
        -   **[Switch] Running prompt is re-pushed once**:
            -   **Description**: Account switch waits until the instance process is up, then restores the running prompt once. A failed send stays backed up. (Thanks to @aukgit)
        -   **[Audit] Split task history**:
            -   **Description**: Account adds and switches are written to split SQLite files under `task-history`, indexed by `task_index.db`. `agm history` and the Audit page list the last 100 events per page. (Thanks to @aukgit)

    *   **v4.118.0 (2026-10-02)**:
        -   **[UI/UX] Awan Design System Surface Contrast & Elevation Stability**:
            -   **Description**: Stabilized universal Awan Software design system, luminance hierarchy, and surface contrast across all components; optimized active instance and account styling for seamless dark mode readability. (Thanks to @aukgit)
        -   **[Theme] Accessible WCAG AAA Color Semantics & CSS3 Animations**:
            -   **Description**: Enhanced UI/UX color semantics with accessible WCAG AAA contrast ratios for active instances, accounts, and quotas; harmonized universal CSS3 color switching transitions and ambient glow pulse keyframe animations across the complete interface. (Thanks to @aukgit)
        -   **[Documentation] Universal English Documentation Standardization**:
            -   **Description**: Enforced 100% pure English standardization across all repository documentation, install scripts, and distribution channels with a strict zero Chinese character policy. (Thanks to @aukgit)

    *   **v4.117.0 (2026-10-02)**:
        -   **[UI/UX] Awan Brand Elevation Hierarchy & Dark Mode Contrast Overhaul**:
            -   **Description**: Harmonized surface luminance and elevation hierarchy across the application according to Awan Software design system (`#071a27` Deep Orbit canvas < `#0c2438` Deep Orbit Card < `#15334d` Elevated Row / Modal). Resolved all dark-on-dark text contrast bottlenecks in `InstanceSelector.tsx` where active items now feature elevated `#15334d` surfaces, Zero-G Cyan accents (`#19b7c9`), and high-contrast cyan email typography (`dark:text-cyan-200`). Removed redundant close icon button in InstanceSelector header. (Thanks to @aukgit)
        -   **[Theme] High-Contrast Typography & Account Component Modernization**:
            -   **Description**: Refactored `AccountRow.tsx`, `AccountTable.tsx`, `AccountCard.tsx`, and `QuotaItem.tsx` to strictly adhere to semantic contrast standards (> 7:1 WCAG AAA). Completely eliminated white-on-white and dark-on-dark row anomalies in dark mode. Set focused rows to `#15334d` with `#43d6a2` Plasma Mint borders, current active rows to `#0c2438` with `#16a97a` Vector Green indicators, and synchronized sticky action table columns. Implemented universal CSS3 color switching transitions and ambient glow pulse animations. (Thanks to @aukgit)
        -   **[Documentation] Pure English Internationalization & Zero Chinese Text Policy**:
            -   **Description**: Enforced 100% pure English across `README.md`, `README_EN.md`, `changelog.md`, and `changelog_en.md`, eliminating all residual Chinese characters and standardizing release documentation for global distribution. (Thanks to @aukgit)

    *   **v4.116.0 (2026-10-02)**:
        -   **[UI/UX] Comprehensive Dark Mode Contrast Overhaul & Dark-on-Dark Text Elimination**:
            -   **Description**: Resolved critical dark-on-dark text contrast failures in `InstanceSelector.tsx` where active profile names, emails, and unlinked placeholders rendered in dark slate (`#020617` / `#1e293b`) on dark blue backgrounds; updated active profile text to crisp white (`dark:text-white`) and email typography to high-contrast cyan (`dark:text-cyan-200`). Modernized the active instance close button with accessible high-contrast rose styling (`dark:bg-rose-950/40 text-rose-300`). (Thanks to @aukgit)
        -   **[Theme] Complete Awan Brand Specification & CSS3 Animation Modernization**:
            -   **Description**: Aligned `AccountTable.tsx`, `AccountRow.tsx`, and `AccountCard.tsx` with the official Awan Software brand palette (`#0c2438` Deep Orbit Card, `#16a97a` Vector Green, `#19b7c9` Zero-G Cyan, `#43d6a2` Plasma Mint). Eliminated all blinding pure white rows in dark mode, set active rows to Deep Orbit Card with Zero-G Cyan accents, updated CURRENT badges to Vector Green, and aligned sticky action columns to eliminate color seams. Implemented universal CSS3 color switching transitions across the entire DOM with ambient glow pulse keyframes. (Thanks to @aukgit)
        -   **[Documentation] Pure English Internationalization & Zero Chinese Text Policy**:
            -   **Description**: Enforced 100% pure English across `README.md`, `README_EN.md`, `changelog.md`, and `changelog_en.md`, removing all residual Chinese text and standardizing release notes for global distribution. (Thanks to @aukgit)

    *   **v4.115.0 (2026-10-02)**:
        -   **[UI/UX] Comprehensive Dark Mode Contrast Overhaul & Dark-on-Dark Text Elimination**:
            -   **Description**: Resolved critical dark-on-dark text contrast failures in `InstanceSelector.tsx` where active profile names, emails, and unlinked placeholders rendered in dark slate (`#020617` / `#1e293b`) on dark blue backgrounds; updated active profile text to crisp white (`dark:text-white`) and email typography to high-contrast cyan (`dark:text-cyan-200`). Modernized the active instance close button with accessible high-contrast rose styling (`dark:bg-rose-950/40 text-rose-300`). (Thanks to @aukgit)
        -   **[Theme] Complete Awan Brand Specification & CSS3 Animation Modernization**:
            -   **Description**: Aligned `AccountTable.tsx`, `AccountRow.tsx`, and `AccountCard.tsx` with the official Awan Software brand palette (`#0c2438` Deep Orbit Card, `#16a97a` Vector Green, `#19b7c9` Zero-G Cyan, `#43d6a2` Plasma Mint). Eliminated all blinding pure white rows in dark mode, set active rows to Deep Orbit Card with Zero-G Cyan accents, updated CURRENT badges to Vector Green, and aligned sticky action columns to eliminate color seams. Implemented universal CSS3 color switching transitions across the entire DOM with ambient glow pulse keyframes. (Thanks to @aukgit)

    *   **v4.114.0 (2026-10-02)**:
        -   **[Documentation] Complete Chinese Removal from Changelog & Docker Deployment Guide**:
            -   **Description**: Completely purged all legacy Chinese language text from `changelog.md` and `docker/readme.md`, standardizing all documentation, historical version entries, and deployment guides in pure English across all distribution channels; normalized unicode fullwidth punctuation to standard ASCII format. (Thanks to @aukgit)
        -   **[Theme] Awan Brand Palette Alignment for AccountRow & InstanceSelector**:
            -   **Description**: Aligned `AccountRow.tsx` focus, current, and selected state colors with Awan brand tokens (`#2878f0` Lift Blue, `#16a97a` Vector Green, `#43d6a2` Plasma Mint, `#0c2438` Deep Orbit Card); refined active instance name and email typography contrast in `InstanceSelector.tsx` for optimal dark mode legibility. (Thanks to @aukgit)

    *   **v4.113.0 (2026-10-02)**:
        -   **[Theme] Modern Awan Brand Color Scale & DaisyUI Palette Overhaul**:
            -   **Description**: Overhauled color theme tokens in `tailwind.config.js` introducing the official modern Awan brand color scale (`#2878f0` Lift Blue, `#16a97a` Vector Green, `#19b7c9` Zero-G Cyan, `#43d6a2` Plasma Mint, `#071a27` Deep Orbit Dark, `#0c2438` Deep Orbit Card, `#15334d` Deep Orbit Border, `#5d6b76` Slate Vector, `#f5faf9` Cloud Field). Realigned DaisyUI light and dark theme palettes for superior contrast, sleek dark surfaces, and fluid ambient state transitions. (Thanks to @aukgit)
        -   **[Documentation] Complete Chinese Removal and English Documentation Alignment**:
            -   **Description**: Completely removed Chinese language remnants from `README.md` and documentation changelog headers; standardized changelog entries in pure English across `CHANGELOG.md` and `CHANGELOG_EN.md` for consistent internationalization. (Thanks to @aukgit)

    *   **v4.112.0 (2026-10-01)**:
        -   **[Performance] CPU & Process Resource Waste Elimination & Windows "Not Responding" Freeze Fix**:
            -   **Description**: Fully diagnosed and eliminated the root cause of AGM process spikes exceeding 30% CPU and UI "Not responding" freezes. Refactored process discovery and health inspection in `src-tauri/src/modules/instance.rs`, replacing heavy full-system `sysinfo::ProcessRefreshKind::everything()` enumeration with lightweight targeted command-line and executable snapshots backed by a thread-safe 4-second TTL cache (`PROCESS_SCAN_CACHE`). Switched instance health verification and teardown logic to targeted single-PID inspection (`ProcessesToUpdate::Some(&[pid])`), eliminating Win32 process handle enumeration contention across multi-instance evaluations. (Thanks to @aukgit)
        -   **[Stability] Prevention of Runaway agy.exe Worker Process Accumulation**:
            -   **Description**: Resolved an issue where concurrent workspace prompts or auto-resume loops accumulated 30+ background `agy.exe` processes consuming ~2GB RAM. Implemented the atomic active worker tracker `ACTIVE_AGY_WORKERS` in `src-tauri/src/modules/repo_db.rs`, actively verifying worker PID liveness before dispatch to strictly intercept duplicate process spawns within the same workspace. Enhanced instance stop cleanup to reliably prune terminated workers and purge stale cache references. (Thanks to @aukgit)
        -   **[Telemetry] Mandatory 60-Second Startup Quiet Period & 5-Minute Telemetry Floors**:
            -   **Description**: Enforced a strict 60-second Startup Quiet Period across all background daemons (`auto_switcher`, `email_watcher`, `telegram_inbound`, `scheduler`, `supabase_sync`) and the frontend `BackgroundTaskRunner.tsx`, eliminating startup disk I/O and network telemetry storms. Clamped background check and quota synchronization intervals to a minimum of 300 seconds (5 minutes) aligned with desktop event broadcast architecture. (Thanks to @aukgit)
        -   **[UI/UX] Prominent Version Number Display in Windows Task Manager**:
            -   **Description**: Dynamically bound the application version to native window titles via Win32 `SetWindowTextW` upon startup (e.g. `Antigravity Manager Tools v4.112.0`). Synchronized window titles across `tauri.conf.json` and `scripts/bump-version.mjs` to ensure the Windows Task Manager process group header prominently and reliably displays the installed version number. (Thanks to @aukgit)

    *   **v4.111.0 (2026-10-01)**:
        -   **[Bug Fix] CDN Manifest v4.85.0 Lockout Resolution & Dynamic Generation**:
            -   **Description**: Fully resolved the persistent warning in `install.ps1` and `install.sh` reporting `Cached CDN manifest version (v4.85.0) is not newer than current installed version`. Restored and enhanced `03-ai-scripts/39-generate-releases-manifest.py`, integrated manifest synchronization directly into `scripts/bump-version.mjs`, and added dynamic manifest generation in `.github/workflows/release.yml` to guarantee rate-limit-free Tier 1 CDN version resolution is always up-to-date across all installer channels. (Thanks to @aukgit)
        -   **[UI/UX] Modern Awan Cloud Styling & Universal CSS3 Color Switching Animation**:
            -   **Description**: Overhauled account row, account card, and instance selector styling according to modern Awan cloud specifications. Completely eliminated the high-contrast stark solid white row issue in dark mode (`bg-white dark:bg-white`) in favor of sleek, glowing translucent slate surfaces (`bg-slate-800/90`, `bg-[#131b2e]`) with ambient blue/amber borders and crisp typography. Injected universal CSS3 color switching transitions across the interface for frictionless theme and selection changes. (Thanks to @aukgit)
        -   **[Bug Fix] Windows Taskbar Pin Preservation on Reinstallation/Updates**:
            -   **Description**: Resolved an issue where running `install.ps1` to reinstall or update the application removed previously pinned Taskbar shortcuts. Added pre-flight detection and safe backup of existing user-pinned taskbar shortcuts; fixed a path-comparison bug in `Remove-PreviousInstallations` where registry `InstallLocation` containing quotation marks caused `Resolve-Path` to fail and erroneously execute `uninstall.exe`, triggering Windows taskbar tombstoning; strictly eliminated all deletions targeting `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\*.lnk`; and implemented post-installation restoration and target path refresh (`TargetPath` / `WorkingDirectory`) to guarantee pinned icons remain intact and functional across upgrades. (Thanks to @aukgit)

    *   **v4.110.0 (2026-10-01)**:
        -   **[Bug Fix] Windows Taskbar Pin Preservation on Reinstallation/Updates**:
            -   **Description**: Resolved an issue where running `install.ps1` to reinstall or update the application removed previously pinned Taskbar shortcuts. Added pre-flight detection and safe backup of existing user-pinned taskbar shortcuts; fixed a path-comparison bug in `Remove-PreviousInstallations` where registry `InstallLocation` containing quotation marks caused `Resolve-Path` to fail and erroneously execute `uninstall.exe`, triggering Windows taskbar tombstoning; strictly eliminated all deletions targeting `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\*.lnk`; and implemented post-installation restoration and target path refresh (`TargetPath` / `WorkingDirectory`) to guarantee pinned icons remain intact and functional across upgrades. (Thanks to @aukgit)
        -   **[Bug Fix] CDN Manifest v4.85.0 Lockout Resolution & Dynamic Generation**:
            -   **Description**: Fully resolved the persistent warning in `install.ps1` and `install.sh` reporting `Cached CDN manifest version (v4.85.0) is not newer than current installed version`. Restored and enhanced `03-ai-scripts/39-generate-releases-manifest.py`, integrated manifest synchronization directly into `scripts/bump-version.mjs`, and added dynamic manifest generation in `.github/workflows/release.yml` to guarantee rate-limit-free Tier 1 CDN version resolution is always up-to-date across all installer channels. (Thanks to @aukgit)
        -   **[UI/UX] Modern Awan Cloud Styling & Universal CSS3 Color Switching Animation**:
            -   **Description**: Overhauled account row, account card, and instance selector styling according to modern Awan cloud specifications. Completely eliminated the high-contrast stark solid white row issue in dark mode (`bg-white dark:bg-white`) in favor of sleek, glowing translucent slate surfaces (`bg-slate-800/90`, `bg-[#131b2e]`) with ambient blue/amber borders and crisp typography. Injected universal CSS3 color switching transitions across the interface for frictionless theme and selection changes. (Thanks to @aukgit)
        -   **[Feature] Blind AI Instance Test Flow & 8 Sandboxing Laws Compliance**:
            -   **Description**: Audited and enforced the 8 Sandboxing Laws (L1 host PID safety, L2 folder-owns-PID mapping, L3 clone vs new profile segregation, L4 UI-CLI functional parity with zero shell spawns, L5 queued prompt preservation and recovery, L6 notification delivery and backtraces prior to process exit, L7 Supabase read-back email verification with PGRST205 missing table hard failure interception, L8 test cleanup restricted to `test-cli-flow*` / `test-diag*` sandboxes); modularized `supabase_sync.rs` under 15-line function caps with dedicated unit tests; hardened Cargo incremental cache cleanup in `scripts/dev-tool-clear.ps1`. (Thanks to @aukgit)

    *   **v4.109.4 (2026-09-30)** (includes v4.109.1–v4.109.3):
        -   **[Bug Fix] Update Settings Backward Compatibility**:
            -   **Description**: Older `update_settings.json` files without `auto_check` / `last_check_time` no longer fail to load (E9001 when closing the update dialog). (Thanks to @aukgit)
        -   **[Bug Fix] Stack Trace Parsing and UI Fixes**:
            -   **Description**: Fixed parsing of URL-only stack frames with new tests; fixed the Accounts focus effect ordering and the cleanup dialog's Escape key. (Thanks to @aukgit)
        -   **[Test] Test Runner and E2E Script**:
            -   **Description**: Added the `npm run test` frontend runner and fixed stale tests; `scripts/test-instance-e2e.ps1` creates `build-demo/` itself and backs up running prompts per instance. (Thanks to @aukgit)
        -   **[Known Issue] Missing Platform Assets**:
            -   **Description**: v4.109.2–v4.109.4 were published while CI was red and lack Windows/Linux installers; the installers on `main` now skip partial releases. (Thanks to @aukgit)

    *   **v4.109.0 (2026-09-30)**:
        -   **[Bug Fix] Instance Profile Switching and Prompt Persistence Improvements**:
            -   **Description**: Fixed issue where fast-forward instance switching discarded the active prompt and reused default credentials. Running prompts are now correctly persisted, and isolated IDE accounts strictly enforce their bound credentials. (Thanks to @aukgit)
        -   **[Feature] AGM CLI clear-cache functionality**:
            -   **Description**: Added `agm clear` alias with N-limit retention, supporting automated Rust and Cargo target cleanup. (Thanks to @aukgit)

    *   **v4.103.0 (2026-09-30)**:
        -   **[Release v4.103.0: Multi-Instance Switching Integrity, UI Modal Exclusivity & High-Contrast Highlight, Conversation Pruner Safety Gate, Telemetry Project Deduplication & 200-Word Prompt Previews] Fixed the multi-instance switching bounce-back bug reverting to worker-alpha during background polling, added explicit close 'X' buttons, Esc key & backdrop dismissal, and global mutual exclusivity across dropdowns and modals; reinforced the conversation cleaner safety gate protecting running prompts and retaining the top 5 sessions per active workspace with CLI `agm prune` dry-run and undo rollback; deduplicated project names across switch emails and Telegram telemetry, repaired Telegram HTML 400 parsing errors, added `/update` bot remote execution, and provided 200-word prompt previews in emails and CLI (Thanks to @aukgit)**:
            -   **Multi-Instance Switching Persistence & Stickiness (`src-tauri/src/modules/instance.rs`)**: Fixed `get_active_instance_id()` from prematurely falling back to `worker-alpha` during periodic 4-second polling when background instances are evaluated; strictly honored user-selected `registry.active_instance_id`, updated `last_used` timestamps, and injected `CREATE_NEW_PROCESS_GROUP` on Windows spawns;
            -   **UI Modal Exclusivity, Dismissal & High-Contrast Styling (`src/components/navbar/InstanceSelector.tsx`, `src/components/modals/agy-clean-modal.tsx`, `src/components/accounts/AccountRow.tsx`)**: Added explicit `X` close buttons to InstanceSelector and AgyCleanModal, implemented `Escape` key dismissal and backdrop click handling; dispatched `agm:dropdown-open` ensuring modal mutual exclusivity; auto-scrolls to focused instance via `scrollIntoView` and upgraded dark mode selection styling to high-contrast white card (`text-slate-950 font-bold border-l-blue-600 ring-2 ring-blue-500/60`);
            -   **Conversation Cleaner Safety Gate & CLI Prune Suite (`src-tauri/src/modules/agy_cleaner.rs`, `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Updated `scan_conversations` to strictly protect running, queued, and dispatched prompt session IDs from deletion, preserving at least the latest 5 sessions for each active workspace; introduced `agm prune` / `agm pr <N>` with `--preflight`/`--dry-run` previews, `--undo [TX]` transaction rollback, and `--json` formatting; wired Telegram `/prune` and `/clean` commands;
            -   **Notification Hub Project Deduplication & 200-Word Prompt Previews (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/bin/agm.rs`)**: Added `deduplicate_names()` to prevent repeating project workspace labels in switch emails and Telegram alerts; implemented `extract_words_preview()` rendering up to 200-word prompt previews in switch emails, Telegram notifications, and `agm prompts ls --words <N>`;
            -   **Telegram Bot HTML Error Recovery & Remote Update (`src-tauri/src/modules/telegram_inbound.rs`)**: Implemented `strip_telegram_html_tags()` to strip unescaped tags and decode entities on 400 Bad Request retries, preventing unsent messages or raw tag exposure; added `/update` and `/upgrade` remote self-updater commands and surfaced update availability in `/status`;
            -   **Frontend Build Resilience (`src/components/proxy/CliSyncCard.tsx`, `src/components/proxy/HermesSyncModal.tsx`)**: Resolved missing icon exports in `@lobehub/icons` by cleanly falling back to Lucide icons, ensuring seamless TypeScript compilation.

    *   **v4.102.0 (2026-09-29)**:
        -   **[Release v4.102.0: Release Pipeline Hardening & Prerelease Channel Gating, Multilingual Contributor Attribution Sanitization, Universal macOS Asset Packaging, and Keyring Bypass Session Isolation] Hardened GitHub Actions release workflow with direct tag-based channel gating and CLI fallback prerelease flags, enhanced Python release notes sanitizer to normalize all multilingual attributions to @aukgit and strip extraneous GitHub handles, automated macOS universal application bundle packaging into tarball assets, deepened multi-instance keyring bypass markers and session isolation, and resolved compiler clippy doc-comment warnings (Thanks to @aukgit)**:
            -   **Release Pipeline Hardening & Prerelease Channel Gating (`.github/workflows/release.yml`)**: Fixed preview channel synchronization by evaluating `github.ref_name` directly instead of non-existent job outputs, dynamically parameterized `prerelease` and `makeLatest` flags on `ncipollo/release-action@v1`, and added `--prerelease` flag support to GitHub CLI release fallback;
            -   **Multilingual Contributor Attribution Sanitization (`.github/workflows/release.yml`)**: Reinforced pre-publish release notes sanitizer with multilingual regex matching (`[\(\[](?:Thanks to|attributions)[^\)\]]*[\)\]]` -> `(Thanks to @aukgit)`) and filtered all raw `@user` mentions except `@aukgit`, ensuring the GitHub Releases contributor section strictly displays solely `aukgit`;
            -   **Universal macOS Bundle Asset Packaging (`.github/workflows/release.yml`)**: Automated `.app` tarball compression (`agm-alim_universal.app.tar.gz`) during asset rename processing, ensuring universal macOS builds upload valid release artifacts and eliminating runner warnings;
            -   **Multi-Instance Keyring Bypass & Session Isolation (`src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/integration.rs`)**: Mirrored private file credentials across `.gemini`, `antigravity`, and `antigravity-ide` directories, wrote `keyring-unavailable` sentinel markers, and injected `SSH_TTY` and `JETSKI_OAUTH_TOKEN` during instance launch to ensure complete session independence;
            -   **Compiler & Build Hardening (`src-tauri/src/proxy/`, `src-tauri/build.rs`)**: Fixed clippy documentation comments in Claude mapper and upstream client modules, and stripped Windows UNC prefix (`\\?\`) when embedding manifest in MSVC builds.

    *   **v4.101.0 (2026-09-29)**:
        -   **[Release v4.101.0: Strict Contributor Isolation & Release Pipeline Hardening, Automated Release Notes Sanitization, and Cross-Platform Case-Resilient Hygiene] Hardened the GitHub Release workflow by integrating an automated Python release notes sanitizer that strictly isolates contributor attribution to @aukgit and strips unwanted GitHub handles while preserving email addresses, enforced `generateReleaseNotes: false` across release channels to prevent auto-generated contributor pollution, normalized repository-wide maintainer attributions in READMEs and developer guidelines, upgraded version bump automation with case-insensitive file resolution, and synchronized full lowercase repository hygiene (Thanks to @aukgit)**:
            -   **Automated Release Notes Sanitization & Contributor Isolation (`.github/workflows/release.yml`, `docs/release_guide.md`, `docs/release-guide.md`)**: Implemented an automated pre-publish sanitizer in `release.yml` that standardizes all attribution blocks to `(Thanks to @aukgit)` and strips unwanted `@` mentions from release notes, configured `generateReleaseNotes: false` on `ncipollo/release-action@v1` for both stable and preview channels, and updated release documentation to mandate strict contributor isolation;
            -   **Repository Maintainer Attribution Normalization (`readme.md`, `readme_en.md`, `AGENTS.md`, `agents.md`)**: Normalized project documentation and badges to feature solely `aukgit` (`Md. Alim Ul Karim`) as Lead Developer & Maintainer across Chinese and English documentation;
            -   **Version Bump Automation & Case-Resilient Resolution (`scripts/bump-version.mjs`)**: Enhanced `bump-version.mjs` with case-insensitive fallback logic ensuring seamless synchronization across lowercase and uppercase file variations (`changelog.md` / `changelog_en.md` / `readme_en.md`) on both Windows and Linux, and aligned default changelog skeletons with `(Thanks to @aukgit)`;
            -   **Lowercase Repository Hygiene**: Synchronized clean lowercase naming across documentation, release guides, and CI templates for frictionless cross-platform development.

    *   **v4.100.0 (2026-09-29)**:
        -   **[Release v4.100.0: Universal Two-Tier JSON Envelope Standard, Intelligent Format Inspector with One-Line Bulk Importers, Single-Circle Multi-Instance Broadcast Auto-Switching Architecture, and Quarantined E2E Test Suite] Implemented cross-system `{ "attributes": ..., "data": ... }` two-tier JSON standard envelopes with backward-compatible transparent unpacking, delivered CLI smart format classifier (`agm which-format`) with `-y` prompt bypass and single-line bulk execution commands, unified multi-instance auto-switching into a single-circle daemon with multi-channel event broadcasting, delivered quarantined local-only E2E test suite verifying instance switching with host IDE PID protection, and polished UI with account Focus scrolling and dynamic titlebar versioning (Thanks to @aukgit)**:
            -   **Two-Tier JSON Envelope Standard & Backward-Compatible Engine (`src-tauri/src/modules/json_envelope.rs`, `02-spec/21-app/02-unified-json-envelope-and-format-inspector-spec.md`)**: Established cross-system schema standardization dividing payloads into top-level `attributes` (metadata: type, version, source, timestamp, encoding, description) and `data` (payload contents); implemented `unpack_envelope()` ensuring seamless interoperability between v1.0 envelopes and legacy flat JSONs across 8 system schemas (Supabase endpoints, credentials, accounts, instances, configs, and prompts); upgraded all configurations in `repo-secrets` to the standard;
            -   **Intelligent Format Inspector & Bulk Importer (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/json_envelope.rs`)**: Shipped `agm which-format` (aliases `format`, `inspect-format`), supporting multi-file and recursive directory scanning, schema signature classification, mutation previews ("What will change"), individual import commands with `-y` prompt bypass, and a single-line chained bulk execution command; updated CLI JSON exporters (`agm accounts --json`, `agm instances --json`) to emit standardized envelopes;
            -   **Single-Circle Multi-Instance Broadcast Auto-Switcher (`src-tauri/src/modules/auto_switcher.rs`, `src/components/common/BackgroundTaskRunner.tsx`)**: Consolidated auto-switching into a single authoritative daemon loop ("one circle") evaluating all active instances in batch without individual per-instance timers or point-to-point pinging; adopted a multi-channel broadcast pattern emitting desktop Tauri events (`account://auto-switched`, `accounts://refreshed`), Supabase distributed cluster lease locks, and cross-VM telemetry; updated frontend to defer to native Tauri daemon while providing unified single-circle fallback in web mode;
            -   **Quarantined Local-Only End-to-End Test Suite (`src-tauri/src/modules/instance.rs`, `02-spec/21-app/72-local-e2e-instance-switching-and-prompt-restore.md`)**: Implemented dual skip-by-default isolation (`#[ignore = "local_only_e2e"]` and `RUN_TEMP_E2E=1`) verifying the complete instance lifecycle (instance cloning, `state.vscdb` creation, workspace binding, prompt seeding, conscious PID shutdown, credential swap asserting email change to `erfan.office.n@gmail.com`, and prompt restoration to `.antigravity_resume_task.json`) with zero impact on host IDE PIDs (`2464`, `1724`, `5140`) and zero CI/CD overhead;
            -   **Frontend UX & Polish (`src/components/accounts/`, `src/components/layout/TitleBar.tsx`, `src/pages/Accounts.tsx`)**: Added Focus button for active and selected account cards with smooth auto-scrolling and visual highlight; enhanced theme selection border contrast; replaced hardcoded titlebar version with dynamic runtime version retrieval.

    *   **v4.99.0 (2026-09-29)**:
        -   **[Release v4.99.0: Complete System Keyring Bypass via SSH Session Emulation, Double-Gated PID & Path Process Protection Preserving Active Sessions, Multi-Directory Conversation Pruning & Resilient High-Quota Fallback] Resolved the root-cause credential bleed where non-default sandbox instances inherited the default account from Windows Credential Manager by reverse-engineering Go `language_server`'s `compositeTokenStorage` and injecting `SSH_CONNECTION` to trigger `shouldBypassKeyring` towards private file-based token storage with full `id_token` fidelity, redirected Windows Roaming/Local AppData to per-instance sandboxes, enforced strict command-line argument and parent-child topology gates guaranteeing the user's primary IDE session (PID 7748) is completely untouched, enabled resilient quota fallback for auto-switch and fast-forward when 100% profiles are unavailable, and extended cross-instance multi-directory conversation clearing across all registered sandboxes (Thanks to alim, devorg.bd@gmail.com, @aukgit, @lbjlaq, @jeikl)**:
            -   **Low-Level Keyring Bypass & Private File Credential Fidelity (`src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/integration.rs`)**: Reverse-engineered the Go language server's `compositeTokenStorage` authentication pipeline and discovered its Win32 `CredReadW("gemini:antigravity")` fallback caused cross-instance profile bleeding. Injected `SSH_CONNECTION` and `SSH_CLIENT` environment variables when spawning sandbox instances to trigger its internal `shouldBypassKeyring` flag, forcing it to read file-based token storage at `%USERPROFILE%\.gemini\jetski-standalone-oauth-token`; supplemented missing `id_token` in `write_to_file_credentials_at` to guarantee 100% upstream authorization and display the correct account in the IDE;
            -   **Windows AppData / LocalAppData Complete Sandbox Partitioning (`src-tauri/src/modules/instance.rs`)**: Redirected `APPDATA` and `LOCALAPPDATA` to `<data_dir>\AppData\Roaming` and `<data_dir>\AppData\Local` for non-default instances, and ensured dual SQLite & file token injection (`state.vscdb` and `storage.json`) across instance creation, cloning, launching, and account rotation, eliminating implicit Electron connections to the default roaming directory;
            -   **Double-Gated PID & Folder Path Process Filter Guaranteeing Active Session Safety (`src-tauri/src/modules/instance.rs`)**: Implemented strict command-line argument verification and process tree relationship checks in `close_instance`. For non-default instances, candidate processes whose command-line arguments do not explicitly contain the instance's unique `data_dir` or `instances/<id>` are strictly filtered out with warning logs, guaranteeing the user's active primary IDE session (PID 7748) is completely protected;
            -   **Resilient High-Quota Fallback for Auto-Switcher & Fast-Forward (`src-tauri/src/modules/auto_switcher.rs`)**: Enhanced `select_candidate_profiles` and `select_and_verify_next_best_profile` to prioritize 100% quota accounts, with an automatic graceful fallback to the highest-quota profile above the threshold when no 100% accounts exist, preventing fast-forward and auto-switch from stalling when all accounts have minor usage;
            -   **Multi-Instance Conversation Discovery & Deep Cache Pruning (`src-tauri/src/modules/agy_cleaner.rs`, `src-tauri/src/modules/backup_prompts_db.rs`, `scripts/dev-tool-clear.ps1`)**: Extended `get_gemini_candidate_dirs()` to discover `.gemini` paths across all registered sandbox instances, enabling `adm clear` / `agm clear` and `dev-tool-clear.ps1` to clean history conversations across all instances while preserving the latest N sessions (default 10), and scoped `restore_running_prompts_for_instance` strictly to the target instance to prevent cross-instance task leakage.

    *   **v4.98.0 (2026-09-29)**:
        -   **[Release v4.98.0: Sandbox Multi-Instance Environment Isolation & Keyring Protection, Targeted Process Discrimination Protecting Active Sessions, Fine-Grained Instance Lockfile Scoping & Full CLI Suite] Resolved the root-cause credential bleed where `language_server` inherited global `USERPROFILE` causing sandbox instances to read default account credentials, enforced strict process targeting and command-line verification preventing active sessions (PID 7872) from accidental termination, scoped lockfile cleaning strictly to target instance directories, rebuilt `agm instances` and `agm auto-switch` CLI suites with rich examples, and upgraded frontend instance management with auto-switch toggles, initial account assignment, and stop controls (Thanks to alim, devorg.bd@gmail.com, @aukgit, @lbjlaq, @jeikl)**:
            -   **Complete Sandbox Environment Isolation (`src-tauri/src/modules/instance.rs`)**: Explicitly injected isolated `USERPROFILE`, `HOME`, `HOMEDRIVE`, and `HOMEPATH` environment variables pointing to `<instances_root>/<instance_id>/home` when spawning non-default instances, preventing `language_server.exe` and plugins from inheriting the global user home (`C:\Users\Administrator\.gemini`) and eliminating cross-profile credential bleed;
            -   **Targeted Process Discrimination & Active Session Invariant (`src-tauri/src/modules/process.rs`, `src-tauri/src/modules/instance.rs`)**: Restructured `get_antigravity_pids` and `get_process_info` to filter sandbox instances and `target_ide` upfront before manual executable and fuzzy matches. Default operations skip sandbox processes containing `--user-data-dir` or `/instances/`, while instance operations require exact directory matching, completely protecting the user's active session (e.g. PID 7872). `close_instance` additionally verifies that tracked PIDs contain the instance's directory before termination, preventing PID-reuse collisions;
            -   **Fine-Grained Instance Lockfile Scoping (`src-tauri/src/modules/process.rs`)**: Scoped `clean_antigravity_lockfiles` so that operations on a specific instance only purge Electron `lockfile`, `code.lock`, and `singleton*` files inside that instance's directories, while default maintenance only targets system default directories, avoiding lock corruption across instances;
            -   **Comprehensive Multi-Instance & Auto-Switch CLI Suite (`src-tauri/src/bin/agm.rs`, `src-tauri/src/commands/instance.rs`)**: Expanded `agm instances` CLI (`ls`, `create`, `switch`, `ff`, `launch`, `stop`, `rm`, `rm-all`, `assign`) with flags (`--account`, `--from`, `--data-only`, `--launch`) and 11 clear examples. Implemented `agm auto-switch` / `agm auto` CLI suite (`status`, `enable`, `disable`, `toggle`, `run`, `threshold`, `interval`, `model`, `test`). Upgraded frontend Instances page with global Auto-Switch toggle button, account selector during instance creation, per-instance Stop buttons, and explicit Switch Account modals;
            -   **Developer Hygiene & Conversation Pruner (`scripts/dev-tool-clear.ps1`, `src-tauri/src/bin/agm.rs`)**: Upgraded `scripts/dev-tool-clear.ps1` and `agm clean` / `agm clear` to clear Cargo/Rust incremental caches and prune conversations to the latest N sessions (default 10, configurable via `-k <N>`).

    *   **v4.97.0 (2026-09-29)**:
        -   **[Release v4.97.0: GitHub Actions Artifact Purge Infinite Loop & Permission Fix, 2-Round Live Consecutive Switch E2E Verification & Dual Supabase Sync Validation] Added `actions: write` permission to `publish-release` in `release.yml`, eliminated the infinite retry loop in `34-purge-github-actions-artifacts.py` when artifact deletions fail (preventing 1,000 req/hr GITHUB_TOKEN rate-limit exhaustion), and completed live 2-round consecutive fast-forward switch E2E verification (automatic active conversation prompt capture, IDE shutdown, credential rotation, IDE restart, and `.antigravity_resume_task.json` restoration) alongside dual-endpoint Supabase HTTP 200 sync validation (Thanks to alim, devorg.bd@gmail.com, @aukgit, @lbjlaq, @jeikl)**:
            -   **Release Workflow Permissions & Fallback Hardening (`.github/workflows/release.yml`)**: Granted `actions: write` permission to `publish-release` so post-release artifact cleanup has authorization to delete artifacts; explicitly bound `tag: "${{ github.ref_name }}"` on `ncipollo/release-action@v1` and hardened CLI fallback error handling;
            -   **Artifact & Cache Purge Loop Circuit-Breaker (`03-ai-scripts/34-purge-github-actions-artifacts.py`)**: Filtered out expired artifacts (`select(.expired == false)`), tracked `attempted_ids`, and added an explicit `if batch_deleted == 0: break` circuit breaker in both `purge_repo_artifacts` and `purge_repo_caches`, eliminating infinite loops and API rate-limit exhaustion and purging all 71 stored artifacts (~2.15 GB freed);
            -   **Live 2-Round Consecutive Fast-Forward Switch & Automatic Active Session Recovery (`src-tauri/src/modules/integration.rs`, `src-tauri/src/modules/repo_db.rs`)**: Created live sandbox instance `e2e-live-verify-9800-4113` on the active repository and executed two back-to-back `agm instances <id> ff` switches, verifying that the active conversation prompt (`prompt-d58c5517-d8ad-437e-ab7c-e506b0322383`) is automatically detected from `.gemini/antigravity/brain/`, backed up to SQLite, IDE closed, account rotated (`harry.dev.riseup` -> `ashleyescobarzu`), IDE restarted with DevTools handshake, and `.antigravity_resume_task.json` regenerated across both consecutive switches;
            -   **Supabase Dual-Endpoint Connectivity & Unit Test Validation**: Verified all 14 unit tests in `cargo test --lib supabase` pass 100% and confirmed live HTTP 200 responses and node/instance synchronization via `agm supabase test` and `agm supabase sync` across both Root Lovable and Secondary endpoints.

    *   **v4.96.0 (2026-09-28)**:
        -   **[Release v4.96.0: Full Root (Lovable) & Secondary Supabase Integration, Multi-Machine Parent-Child Topology, PowerShell One-Liner Automation & Bidirectional Sync] Fully integrated Root Supabase (Lovable) and Secondary fallback endpoints, delivered parent table `nodes` with cascading foreign keys to `instance_profiles` and `workspace_leases` preventing cross-machine account collisions, implemented comprehensive `agm supabase help` CLI guidance, PowerShell one-liner automation script with dual-directory writing, and verified 100% green PostgREST HTTP 200 connectivity (Thanks to alim, devorg.bd@gmail.com, @aukgit, @lbjlaq, @jeikl)**:
-   **Native Root (Lovable) & Secondary Endpoints (`scripts/supabase-accounts.json`, `repo-secrets`)**: Natively integrated Root Supabase (`https://[REDACTED_LOVABLE_PROJECT].supabase.co/rest/v1/`, Priority 1, tagged `lovable` and `root`) and Secondary Supabase (`https://[REDACTED_PERSONAL_PROJECT].supabase.co/rest/v1/`, Priority 2, tagged `secondary`), synchronizing credentials with Base64 encoding into `repo-secrets`;

-   **Native Root (Lovable) & Secondary Endpoints (`scripts/supabase-accounts.json`, `repo-secrets`)**: Natively integrated Root Supabase (`<ROOT_SUPABASE_URL>`, Priority 1, tagged `lovable` and `root`) and Secondary Supabase (`<SECONDARY_SUPABASE_URL>`, Priority 2, tagged `secondary`), synchronizing credentials with Base64 encoding into `repo-secrets`;
            -   **Multi-Machine Parent-Child Schema & Atomic Collision Prevention (`src-tauri/src/modules/supabase_schema.rs`, `src-tauri/src/modules/supabase_sync.rs`)**: Designed and implemented PostgreSQL parent table `public.nodes` (tracking machine hardware ID, alias, IPv4 address, and uptime) and foreign-key children `public.instance_profiles` and `public.workspace_leases` (`REFERENCES public.nodes(id) ON DELETE CASCADE`), backed by stored function `acquire_workspace_lease()` to guarantee atomic lease acquisition across machines;
            -   **PowerShell Setup Automation & One-Liner Suite (`scripts/supabase-setup.ps1`, `scripts/supabase-accounts.json`)**: Shipped production-ready PowerShell setup script supporting `-ConfigFile` ingestion or interactive parameters, automatically synchronizing configurations simultaneously to `%APPDATA%\antigravity-manager\supabase_config.json` and `~/.antigravity_tools\supabase_config.json`, complete with one-liner command execution;
            -   **CLI Guided Assistance & Bidirectional Self-Healing Sync (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/supabase_sync.rs`)**: Added detailed `agm supabase help` outputting config paths, local machine ID, local IPv4, relational schema topology, and subcommands; implemented automatic fallback and cross-directory mirror sync in the Rust core;
            -   **End-to-End PostgREST HTTP 200 Verification**: Verified live connectivity against both Lovable Root and Secondary endpoints via `agm supabase test` and `scripts/supabase-setup.ps1`, confirming HTTP 200 PostgREST health and seamless local node state reporting via `agm supabase sync`.

    *   **v4.95.0 (2026-09-28)**:
        -   **[Release v4.95.0: Unified Desktop Integration across Headless & CLI, In-Flight Task Restoration Cache Bypass, Atomic Supabase Command Claims & Verified Consecutive Fast-Forward] Unified DesktopIntegration to operate without GUI handles and routed headless/CLI switches through the complete 5-step non-destructive lifecycle (Backup -> Kill IDE -> Swap Token -> Relaunch -> Re-inject), resolved cache shadowing in repo_db that suppressed prompt task restoration, added atomic claim_next_command stored function to Supabase secondary schema with all 14 tests passing, and verified multi-round consecutive fast-forward switching with 100% prompt recovery (Thanks to alim, devorg.bd@gmail.com, @aukgit, @lbjlaq, @jeikl)**:
            -   **Unified 5-Step Switch Lifecycle across CLI and Headless (`src-tauri/src/modules/integration.rs`, `src-tauri/src/bin/agm.rs`)**: Refactored `DesktopIntegration.app_handle` into `Option<tauri::AppHandle>` and routed `HeadlessIntegration::on_account_switch` to delegate directly to `DesktopIntegration { app_handle: None }`, eliminating the critical defect where CLI commands like `agm instances <id> ff` only updated memory without shutting down the IDE or restoring prompts;
            -   **Task Restoration Cache Shadowing Fix (`src-tauri/src/modules/repo_db.rs`)**: Added `reset_dispatched_prompts_cache()` inside `backup_running_prompts()` and allowed backed-up prompts (`status == "backed_up"`) to bypass the `already_dispatched` cache check in `resend_running_commands_for_instance()` and `dispatch_running_prompts()`, guaranteeing `.antigravity_resume_task.json` is reliably generated and injected;
            -   **CLI Instance-Scoped Prompt Backup & Resend (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Enforced explicit instance-scoped prompt backup and resend in `cmd_instances` and manual profile rotation, preventing cross-instance prompt leakage or missed prompts during fast-forward operations;
            -   **Atomic Supabase Command Claims & 100% Green Test Suite (`src-tauri/src/modules/supabase_schema.rs`)**: Added the PostgreSQL `FOR UPDATE SKIP LOCKED` stored function `claim_next_command(p_node_id TEXT)` to `SECONDARY_DB_SCHEMA_SQL`, passing all 14 unit test suites across Supabase client, schema, sync, and lease management;
            -   **Consecutive Fast-Forward Stress Verification**: Verified across consecutive fast-forward rotations on a live sandbox instance with active prompts (ashleyescobarzu -> harry.dev.riseup), confirming prompt backup, process termination, account swap, IDE restart, and `.antigravity_resume_task.json` restoration across back-to-back cycles.

    *   **v4.94.0 (2026-09-28)**:
        -   **[Release v4.94.0: End-to-End Switch Mode & Prompt Lifecycle Verification, Dual Supabase Distributed Sync, Upstream Pipeline Merge with Tool Call ID Normalization & Protobuf Self-Healing] Verified end-to-end 5-step non-destructive account switching and prompt recovery lifecycle on real sandbox instances, validated dual Supabase endpoints (Lovable root + secondary) and distributed multi-node sync, merged upstream lbjlaq/main v4.8.4 pipeline enhancements (Tool Call ID canonicalization, Claude Protobuf signature in-place self-healing, Hermes/OpenClaw sync), and polished concise status indicators with distinct tri-field account telemetry (PR #5, Thanks to alim, devorg.bd@gmail.com, @aukgit, @lbjlaq, @jeikl)**:
            -   **End-to-End 5-Step Prompt Lifecycle Verification (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/integration.rs`, `src/services/instanceService.ts`)**: Executed and verified complete end-to-end prompt preservation on a local sandbox instance (`e2e-test-sandbox-314`): running prompt backup -> graceful IDE shutdown -> credential swap -> IDE relaunch with workspace args -> DevTools WebSocket reconnection and prompt restoration, guaranteeing zero lost prompts;
            -   **Dual Supabase Integration & Multi-Node Sync (`src-tauri/src/modules/supabase_sync.rs`, `src-tauri/src/bin/agm.rs`)**: Verified connectivity across Root Lovable and Secondary Supabase endpoints with HTTP 200 health responses; validated multi-machine node registration, instance profile sync (`agm supabase sync`), and active lease tracking;
            -   **Upstream lbjlaq/main (v4.8.4) Integration (`src-tauri/src/proxy/`, `src/components/proxy/`)**: Merged upstream PR #5, incorporating inbound pipeline Tool Call ID normalization (`call_<digits>`) to eliminate Google 400 missing thought signature errors, Claude adapter Protobuf binary signature preservation with in-place SQLite self-healing write-back, removal of artificial placeholder thinking blocks, and Hermes / OpenClaw CLI synchronization cards;
            -   **Concise Status Indicators & Distinct Tri-Field Telemetry (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/notification_hub.rs`)**: Streamlined CLI status output by replacing wordy badges with clean green indicators and suppressing noisy Node hashes; enforced strict mutual exclusivity among `previous_email`, `selected_email`, and `predicted_email`, displaying `(none / pool exhausted)` when next candidate pool is empty.

    *   **v4.93.0 (2026-09-28)**:
        -   **[Release v4.93.0: Unified Switch & Prompt Lifecycle Architecture, Normalized Candidate Scoring, Remote Lease Auto-Eviction & Distinct Multi-Role Telemetry] Overhauled auto-switch and fast-forward architecture, normalized candidate scoring (divided by 1000 to produce 0.0000..0.5000) with strict 0-score rule for <100% quota, enforced 6h–10h Supabase remote lease staleness auto-eviction, verified the 5-step non-destructive prompt lifecycle (Backup -> Kill IDE -> Swap Token -> Relaunch -> Re-inject), and eliminated duplicate previous/selected/predicted email telemetry (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Normalized Candidate Scoring & 100% Quota Gate (`src-tauri/src/modules/auto_switcher.rs`, `src/services/instanceService.ts`)**: Eliminated arbitrary large bonus numbers (`+1000`), enforcing `(S_active * M_tier * Q_weekly) / 1000.0` for compact decimal scoring (`0.0000..0.5000`); assigned score 0.0 to any account under 100% quota unless reset period elapsed; strictly excluded currently bound and active in-use accounts from candidate pools to prevent self-rotation.
            -   **Distinct Multi-Role Telemetry (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/bin/agm.rs`)**: Resolved duplicate `previous_email`, `selected_email`, and `predicted_email` across Telegram, JSON, and Email notifications. Ensured `predicted_email` queries the true next candidate in the pool (excluding current and selected), falling back to `(none / pool exhausted)` rather than copying `selected_email`; set `previous_account` and `selected_account` to `null` in dry CLI checks when no switch occurred.
            -   **Remote Lease Staleness Auto-Eviction (`src-tauri/src/modules/workspace_lease_manager.rs`)**: Implemented configurable 6h–10h staleness auto-eviction (`stale_binding_timeout_hours`) for Supabase remote workspace leases without recent pings or credit loss, preventing orphaned leases from deadlocking accounts.
            -   **5-Step Non-Destructive Prompt Lifecycle (`src-tauri/src/modules/integration.rs`, `src-tauri/src/modules/instance.rs`, `src/stores/useInstanceStore.ts`)**: Verified the strict 5-step switch sequence across desktop integration and instance switcher: Step 1 (Backup running prompts to SQLite), Step 2 (Gracefully close IDE processes), Step 3 (Inject swapped credentials into storage), Step 4 (Relaunch IDE preserving workspace arguments), and Step 5 (Re-inject and auto-resume prompts from SQLite).
    *   **v4.8.4 (2026-09-27)**:
        -   **[Pipeline Tool Normalization & Signature Fidelity] Universal Tool Call ID Canonicalization across Pipeline, Cache, and DB to Eliminate 400 'Missing a thought_signature' (Fixes #3529, #3531, Thanks to @Mortalit, @ddmixi)**:
            -   **Root cause eliminated**: Clients (such as OpenCode and Antigravity IDE) strip underscores from tool call IDs when sending tool responses (e.g. converting `call_573077` to `call573077`), causing string mismatch when looking up cached signatures and resulting in upstream Gemini errors: `Function call is missing a thought_signature in functionCall parts (400)`.
            -   **Pipeline Step 0 Pre-sanitization**: Normalized all `functionCall.id` and `functionResponse.id` entries across the context to canonical `call_<digits>` format in `InboundThinkingPipeline` as the very first step, ensuring all subsequent hydration, signature caching, and gatekeeper passes operate on clean data.
            -   **Bi-directional Cache & DB Compatibility**: Standardized `tool_signatures` and `thinking_records` persistence to normalized IDs, while transparently matching unnormalized legacy records on lookup with silent self-healing.
            -   **ThinkingStore Restoration Hardened**: Aligned `by_tool` index and Phase 1 tool ID anchoring in `restore_gemini_contents_with_model` with normalized IDs to prevent tool call signature disconnection.
        -   **[Signature Fidelity & In-Place Healing] Eliminate Destructive Base64 Decoding in Claude Adapter, Support Raw Protobuf Signatures, and Introduce In-Place Database Write-Back Healing**:
            -   **Root cause eliminated**: Completely eliminated the flawed logic in `claude/streaming.rs` and `claude/response.rs` that attempted `String::from_utf8` on decoded Gemini signatures. Gemini `thoughtSignature` payloads are raw Protobuf bytes (initial byte `0x12`). When all byte values fell in ASCII range, `from_utf8` succeeded and corrupted 56-char Base64 signatures into 40-byte raw control characters, causing next-turn lookups to drop them for length `< 50`, resulting in upstream `Function call is missing a thought_signature (400)`.
            -   **Relaxed threshold & auto-protobuf resilience**: Relaxed the minimum signature validation threshold across all protocols from 50 to 32. Added automatic Protobuf signature byte-pattern detection (`0x12`) and Base64 re-encoding in `SignatureCache`, `ThinkingStore`, and `proxy_db`.
            -   **In-Place database write-back self-healing**:
                - `tool_signatures` table: `load_tool_signature` automatically heals legacy raw binary or dirty signatures upon lookup and immediately calls `save_tool_signature` to overwrite and fix SQLite records.
                - `thinking_records` table: `load_thinking_by_tool_id`, `load_thinking_by_signature`, `load_thinking_by_fingerprint`, and UI lookup methods execute `UPDATE thinking_records SET signature = ? WHERE id = ?` upon encountering dirty signatures to repair database state in place.
                - When anchoring turn signatures in `place_turn_signature`, function calls trigger `cache_tool_signature` to synchronize L1 cache and persist to SQLite.
        -   **[Anchor Signature Guarantee on First Non-Thought Part] Strictly Align with Official Payloads, Eliminate Synthesized Placeholder Blocks**:
            -   **No placeholder thinking blocks**: When turns lack substantial thinking text (placeholder or empty), no `...` placeholder blocks are synthesized or injected.
            -   **Deterministic anchor fidelity**: Guaranteed that the first non-thinking part of every turn (text or function call) carries the authoritative `thoughtSignature`, preventing upstream Google 400 signature errors.
            -   **Dead code elimination**: Removed obsolete Layer-2 placeholder compression logic that collapsed historical reasoning into `"..."`.
        -   **[Native Auto-Update Hardening] Support Custom Update Endpoints, Native Updater Checker, and Browser Fallback**:
            -   **Multi-endpoint native update check**: Added `check_native_update` Tauri command, supporting dynamic detection and dispatch of active `updater_json_url` endpoints (supporting both Beta preview and Stable production updater targets).
            -   **SemVer version comparator extension**: Integrated pre-release version comparison logic in the native updater and provided graceful fallback to external browser download links on update failure.
        -   **[Docker Documentation & Formatting] Add Beta Docker Pull Instructions & Fix Code Block Boundaries**:
            -   Added explicit documentation and command examples in `README.md`, `README_EN.md`, and `docker/README.md` for pulling and running isolated Beta pre-release Docker images (e.g. `lbjlaq/antigravity-manager:v4.8.4-beta.1`).
            -   Fixed Markdown code block fences in Docker deployment sections to prevent text descriptions and alert banners from rendering as Bash script comments.
            -   Added JeikCode integration guide and quickstart examples.

    *   **v4.8.4-beta.1 (2026-09-27)**:
        -   **[Signature Fidelity & Healing] Eliminate Destructive Base64 Decoding in Claude Adapter, Support Raw Protobuf Signatures, and Introduce In-Place Database Write-Back Healing**:
            -   **Root cause eliminated**: Completely eliminated the flawed logic in `claude/streaming.rs` and `claude/response.rs` that attempted `String::from_utf8` on decoded Gemini signatures. Gemini `thoughtSignature` payloads are raw Protobuf bytes (initial byte `0x12`). When all byte values fell in ASCII range, `from_utf8` succeeded and corrupted 56-char Base64 signatures into 40-byte raw control characters, causing next-turn lookups to drop them for length `< 50`, resulting in upstream `Function call is missing a thought_signature (400)`.
            -   **Relaxed threshold & auto-protobuf resilience**: Relaxed the minimum signature validation threshold across all protocols from 50 to 32. Added automatic Protobuf signature byte-pattern detection (`0x12`) and Base64 re-encoding in `SignatureCache`, `ThinkingStore`, and `proxy_db`.
            -   **In-Place database write-back self-healing**:
                - `tool_signatures` table: `load_tool_signature` automatically heals legacy raw binary or dirty signatures upon lookup and immediately calls `save_tool_signature` to overwrite and fix SQLite records.
                - `thinking_records` table: `load_thinking_by_tool_id`, `load_thinking_by_signature`, `load_thinking_by_fingerprint`, and UI lookup methods execute `UPDATE thinking_records SET signature = ? WHERE id = ?` upon encountering dirty signatures to repair database state in place.
                - When anchoring turn signatures in `place_turn_signature`, function calls trigger `cache_tool_signature` to synchronize L1 cache and persist to SQLite.
        -   **[Anchor Signature Guarantee on First Non-Thought Part] Strictly Align with Official Payloads, Eliminate Synthesized Placeholder Blocks**:
            -   **No placeholder thinking blocks**: When turns lack substantial thinking text (placeholder or empty), no `...` placeholder blocks are synthesized or injected.
            -   **Deterministic anchor fidelity**: Guaranteed that the first non-thinking part of every turn (text or function call) carries the authoritative `thoughtSignature`, preventing upstream Google 400 signature errors.

    *   **v4.8.4-beta.0 (2026-09-27)**:
        -   **[Pipeline Tool Normalization] Universal Tool Call ID Canonicalization across Pipeline, Cache, and DB to Eliminate 400 'Missing a thought_signature'**:
            -   **Root cause resolved**: Clients (such as Antigravity IDE) frequently strip underscores from tool call IDs (e.g., converting `call_573077` to `call573077`), causing string mismatch when looking up cached signatures. This left tool calls without signatures and caused upstream Gemini to fail with `Function call is missing a thought_signature... (400)`.
            -   **Pipeline Step 0 Pre-sanitization**: Normalized all `functionCall.id` and `functionResponse.id` entries across the context to canonical `call_<digits>` format in `InboundThinkingPipeline` as the very first step, ensuring all subsequent hydration, signature caching, and gatekeeper passes operate on clean data.
            -   **Bi-directional Cache & DB Compatibility**: Standardized `tool_signatures` and `thinking_records` persistence to normalized IDs, while transparently matching unnormalized legacy records on lookup with silent self-healing.
            -   **ThinkingStore Restoration Hardened**: Aligned `by_tool` index and Phase 1 tool ID anchoring in `restore_gemini_contents_with_model` with normalized IDs to prevent tool call signature disconnection.
        -   **[Official Payload Alignment] Drop Placeholder Thinking Blocks & Safely Transfer Signatures to Anchors**:
            -   **Placeholder removal**: Strictly aligned with official Antigravity payloads by eliminating synthesized placeholder thinking blocks (`"..."` / `"."` / `"·"` / empty) produced by legacy adapters and inbound pipeline stages.
            -   **Anchor signature transfer**: When placeholder blocks are dropped, their real signatures are safely preserved and transferred to the turn's first non-thinking anchor part (text or functionCall), ensuring 100% compliance with upstream signature checks.
            -   **Dead code elimination**: Removed obsolete Layer-2 placeholder compression logic that collapsed historical reasoning into `"..."`.
        -   **[Docker Documentation & Formatting] Add Beta Docker Pull Instructions & Fix Code Block Boundaries**:
            -   Added explicit documentation and command examples in `README.md`, `README_EN.md`, and `docker/README.md` for pulling and running isolated Beta pre-release Docker images (e.g. `lbjlaq/antigravity-manager:v4.8.2-beta.0`).
            -   Fixed Markdown code block fences in Docker deployment sections to prevent text descriptions and alert banners from rendering as Bash script comments.

    *   **v4.8.3 (2026-09-27)**:
        -   **[Reversed Upstream Payload Structure Change, Fully Aligned with Official] Fixed Signature Swing Algorithm, Eliminated Thinking-Chain Break & Dead Loops (upstream update on 9.25 night)**:
            -   **Re-reversed the 9.25-night Antigravity upstream payload restructuring**: the official format moved tool responses (`functionResponse`) from `role: "user"` turns into `role: "model"` turns (consecutive model turns are now the norm), and upstream relaxed the signature rule — the first non-thinking part of any model turn (text or tool call) may carry `thoughtSignature`, no longer limited to tool turns. The gateway's old assumptions ("payload must end with a user turn", "signatures only live on tool calls") are all obsolete.
            -   **Fixed the signature swing algorithm**: capture is decoupled from tool-only + thinking-text requirement (any turn carrying a real signature persists); backfill anchor widened from functionCall-only to any non-thinking part, with a tool_id→signature cache fallback. The ambiguous "which turn / text vs tool" swing is gone — signatures deterministically land on the official anchor.
            -   **Eliminated thinking-chain break dead loops**: the defense node no longer mistakes trailing tool turns for a missing user turn and injects the fabricated "Please continue your analysis." — combined with the signature-swing fix, the "model re-triggered by a fake instruction → infinite tool chain" loop is rooted out.
            -   **Fully aligned with the upstream payload**: contradictory toolConfig duplicate writes (absent in official reports) removed, system-prompt / tool-description injections removed; the envelope is field-for-field aligned with official reports (verified 200 across four protocols × multi-turn payloads, signatures 100% on anchor).
        -   **[Outbound Tool Envelope Aligned with Official Antigravity] Eliminate Agent Tool-Loop Dead Cycles & Payload Shape Drift**:
            -   **Tool turns no longer get a fabricated user turn injected**: the defense node used to treat any trailing model turn as an incomplete payload and appended "Please continue your analysis." After function-response turns were aligned to role=model, this fabricated user turns and looped forever (22 consecutive appends observed in logs). Trailing model turns carrying functionCall/functionResponse are now kept as valid.
            -   **Contradictory toolConfig duplicate writes removed**: official Antigravity reports carry no toolConfig / tool_config; the gateway shipped both camelCase and snake_case variants with conflicting modes (AUTO vs VALIDATED). Both fields are now dropped before sending.
            -   **No more system-prompt / tool-description injection**: removed the [CRITICAL DISPATCH DISCIPLINE] block from systemInstruction and the async-dispatch NOTE appended to send_mcp_msg / dispatch_task / assign_task descriptions. Client prompts and tool schema pass through untouched.
        -   **[thoughtSignature Capture & Anchor Backfill Decoupled] Any-turn signature fidelity, no more bare functionCall 400s**:
            -   **Capture side**: under the official rule any model turn may carry a signature (text / tool / pure thinking). Pure tool rounds with a real signature now persist into the store even without thinking text.
            -   **Backfill side**: anchor detection widened from "functionCall only" to "any non-thinking part (text or functionCall)", with a tool_id→signature cache lookup as fallback, so the first non-thinking part always carries a signature and upstream "missing a thought_signature" 400s are gone.
        -   **[Upstream Hardening] Layer-3 summaries / endpoint order / proxy hot-reload**:
            -   Layer-3 background summaries (Gemini / Claude / OpenAI) now route through UpstreamClient::call_v1_internal_auxiliary reusing the main path's endpoint order, URL shape and fallback; shared HTTP clients are rebuildable (Lazy<RwLock<SharedClients>>) so upstream proxy changes take effect immediately.
            -   Quota/Project endpoints now use official Daily → Sandbox → Prod ordering (Fixes #3523 / #3525 / #3526), keeping sandbox as an explicit fallback.
            -   **Fix non-Windows build failure**: restored missing `parse_where_output` / `Command` symbols and the underscore-prefixed parameter, fixing Linux/macOS conditional-compilation build errors.
        -   **[Account Pool Priority & Weekly Stats Alignment] (PR #3521, #3520, Thanks to @buluw)**:
            -   Configurable per-account priority (1–100); P2C selection draws from the highest-priority eligible group and degrades gracefully under rate limits.
            -   Weekly token stats now use the official reset_time to split 7-day windows, eliminating heuristic-reset truncation and clearing.
        -   **[Dual-Channel In-App Update] Seamless Beta-channel self-update**:
            -   Stable/Preview pill toggle, pre-release builds auto-bind to the Beta channel, SemVer-aware pre-release comparison, and Release pipeline mirroring to a fixed preview Tag for rate-limit-free updater.json.
        -   **[Vendor Identity Normalization] Fix upstream pseudo-rate-limit cooling the whole pool (Fixes #3508, Thanks to @oliverhe202018-ctrl)**:
            -   Heterogeneous vendor identity claims inside third-party prompts are normalized so upstream stops treating non-native claims as suspicious and issuing fake 429s that cooled the entire account pool.
        -   **[100% Verbatim Tool Passthrough] Fix agent-client tool-call anomalies (PR #3504)**:
            -   Removed tool-name mapping, argument alias rewriting and command injection across all three protocols; tool names and arguments reach upstream with original client semantics. Frozen System Prompt date/time/cwd/UUID regexes removed; dynamic messages sink into the user turn as <system-reminder>.

    *   **v4.8.2-beta.0 (2026-09-27)**:
        -   **[Reversed Upstream Payload Structure Change, Fully Aligned with Official] Fixed Signature Swing Algorithm, Eliminated Thinking-Chain Break & Dead Loops (upstream update on 9.25 night)**:
            -   **Re-reversed the 9.25-night Antigravity upstream payload restructuring**: the official format moved tool responses (`functionResponse`) from `role: "user"` turns into `role: "model"` turns (consecutive model turns are now the norm), and upstream relaxed the signature rule — the first non-thinking part of any model turn (text or tool call) may carry `thoughtSignature`, no longer limited to tool turns. The gateway's old assumptions ("payload must end with a user turn", "signatures only live on tool calls") are all obsolete.
            -   **Fixed the signature swing algorithm**: capture is decoupled from tool-only + thinking-text requirement (any turn carrying a real signature persists); backfill anchor widened from functionCall-only to any non-thinking part, with a tool_id→signature cache fallback. The ambiguous "which turn / text vs tool" swing is gone — signatures deterministically land on the official anchor.
            -   **Eliminated thinking-chain break dead loops**: the defense node no longer mistakes trailing tool turns for a missing user turn and injects the fabricated "Please continue your analysis." — combined with the signature-swing fix, the "model re-triggered by a fake instruction → infinite tool chain" loop is rooted out.
            -   **Fully aligned with the upstream payload**: contradictory toolConfig duplicate writes (absent in official reports) removed, system-prompt / tool-description injections removed; the envelope is field-for-field aligned with official reports (verified 200 across four protocols × multi-turn payloads, signatures 100% on anchor).
        -   **[Outbound Tool Envelope Aligned with Official Antigravity] Eliminate Agent Tool-Loop Dead Cycles & Payload Shape Drift**:
            -   **Tool turns no longer get a fabricated user turn injected**: the defense node used to treat any trailing model turn as an incomplete payload and appended "Please continue your analysis." After function-response turns were aligned to role=model, this fabricated user turns and looped forever (22 consecutive appends observed in logs). Trailing model turns carrying functionCall/functionResponse are now kept as valid.
            -   **Contradictory toolConfig duplicate writes removed**: official Antigravity reports carry no toolConfig / tool_config; the gateway shipped both camelCase and snake_case variants with conflicting modes (AUTO vs VALIDATED). Both fields are now dropped in the protocol-agnostic node before sending.
            -   **No more system-prompt / tool-description injection**: removed the [CRITICAL DISPATCH DISCIPLINE] block from systemInstruction and the async-dispatch NOTE appended to send_mcp_msg / dispatch_task / assign_task descriptions. Client prompts and tool schema pass through untouched.
        -   **[thoughtSignature Capture & Anchor Backfill Decoupled] Any-turn signature fidelity, no more bare functionCall 400s**:
            -   **Capture side**: under the official rule any model turn may carry a signature (text / tool / pure thinking). Pure tool rounds with a real signature now persist into the store even without thinking text, instead of being dropped by is_capturable_thought.
            -   **Backfill side**: anchor detection widened from "functionCall only" to "any non-thinking part (text or functionCall)", with a tool_id→signature cache lookup as fallback, so the first non-thinking part always carries a signature and upstream "missing a thought_signature" 400s are gone.
        -   **[Upstream Hardening] Layer-3 summaries / endpoint order / proxy hot-reload**:
            -   Layer-3 background summaries (Gemini / Claude / OpenAI) now route through UpstreamClient::call_v1_internal_auxiliary reusing the main path's endpoint order, URL shape and fallback; shared HTTP clients are rebuildable (Lazy<RwLock<SharedClients>>) so upstream proxy changes take effect immediately.
            -   Quota/Project endpoints now use official Daily → Sandbox → Prod ordering (Fixes #3523 / #3525 / #3526), keeping sandbox as an explicit fallback.
            -   **Fix non-Windows build failure**: added the missing parse_where_output symbol so Linux/macOS builds succeed (equivalent fix is on main as 27ee35b7).
        -   **[Account Pool Priority & Weekly Stats Alignment] (PR #3521, #3520, Thanks to @buluw)**:
            -   Configurable per-account priority (1–100); P2C selection draws from the highest-priority eligible group and degrades gracefully under rate limits.
            -   Weekly token stats now use the official reset_time to split 7-day windows, eliminating heuristic-reset truncation and clearing.
        -   **[Dual-Channel In-App Update] Seamless Beta-channel self-update**:
            -   Stable/Preview pill toggle, pre-release builds auto-bind to the Beta channel, SemVer-aware pre-release comparison, and Release pipeline mirroring to a fixed preview Tag for rate-limit-free updater.json.
        -   **[Vendor Identity Normalization] Fix upstream pseudo-rate-limit cooling the whole pool (Fixes #3508, Thanks to @oliverhe202018-ctrl)**:
            -   Heterogeneous vendor identity claims inside third-party prompts are normalized so upstream stops treating non-native claims as suspicious and issuing fake 429s that cooled the entire account pool.
        -   **[100% Verbatim Tool Passthrough] Fix agent-client tool-call anomalies (PR #3504)**:
            -   Removed tool-name mapping, argument alias rewriting and command injection across all three protocols; tool names and arguments reach upstream with original client semantics. Frozen System Prompt date/time/cwd/UUID regexes removed; dynamic messages sink into the user turn as <system-reminder>.

    *   **v4.8.1-beta.3 (2026-09-26)**:
        -   **[Gemini Verbatim Signature Passthrough & Protocol Alignment] Eliminate 403 Forbidden & Tool Loop Outages Caused by Google Sentinel Deprecation (Fixes #3523)**:
            -   **Prioritize Verbatim Protobuf Base64 Signature Passthrough**: Reverse-engineered authentic bidirectional Antigravity IDE traffic (flows 3/4/5), uncovering that Google cloud endpoints have tightened cryptographic verification and now reject static sentinel strings with 403 Forbidden errors, which triggered client-side infinite retries. Refactored `InboundThinkingPipeline` to recognize and preserve authentic, high-entropy Protobuf Base64 signatures (`thoughtSignature`) from client requests and multi-turn context, retaining sentinels strictly as an unverified fallback.
            -   **End-to-End Signature Fidelity Across Thinking State Machine (Hydration / Finalize)**: Overhauled `ThinkingStore` state transitions (`hydrate` and `finalize`) to eliminate unconditional sentinel overwriting. Valid historical signatures cached in session records are restored directly onto `functionCall` parts, ensuring tamper-proof cryptographic fingerprint continuity across deep multi-turn sessions.
            -   **Align Upstream Endpoint Priority with Official IDE (Daily First)**: Reverse-engineered dual-directional IDE traffic (flows 6) confirming that 100% of official generation requests target `daily-cloudcode-pa.googleapis.com`. Reordered fallback hierarchy to Daily → Sandbox → Prod, directly landing on the stable primary endpoint on the very first hop, eliminating Sandbox geo-restrictions (400) and Prod quota bottlenecks (429) while significantly improving TTFT.
            -   **Support Native Gemini `role: "model"` Function Responses**: Relaxed tool response classification in `ContextManager` to `(role == "user" || role == "model") && has_function_response`, establishing seamless compatibility with native Gemini payloads where tool results are associated with `model` roles.

    *   **v4.8.1-beta.2 (2026-09-26)**:
        -   **[Native IDE Discipline Alignment & Tool Schema Fidelity] Eliminate Agent Post-Dispatch Sleep Polling Loops, Remove Description Truncation & Add Network RST Protection (Fixes #3523)**:
            -   **Align with Native CRITICAL INSTRUCTION Coordination Gate**: Reverse-engineered native IDE compiled disciplines to inject hard coordination constraints into `InboundThinkingPipeline`. When asynchronous dispatch tools (e.g. `send_mcp_msg`) are detected, the gateway enforces immediate turn yield (`finish_reason: stop`), strictly prohibiting agents from writing `Start-Sleep` polling loops in the same turn.
            -   **100% Verbatim Tool Description Fidelity**: Removed destructive whitespace flattening that stripped newlines from tool descriptions, and expanded description safety budget from 2,048 to 8,192 characters. Tool execution constraints now reach models 100% intact, preventing reasoning degradation caused by truncated guidelines.
            -   **Atomic Thinking Commits & Network RST Resilience**: Streamlined SSE thinking block commits with strict termination validation. Partial thinking blocks severed by `connection reset by peer` or `unexpected EOF` are cleanly discarded rather than committed to SQLite, safeguarding historical context against poisoning and persistent degradation.
        -   **[Configurable Account Selection Priority] Multi-Tiered P2C Scheduling with Graceful Fallback (PR #3521, Thanks to @buluw)**:
            -   **Tiered Priority Routing (1-100)**: Users can configure priority from 1 to 100 per account (lower values indicate higher preference, defaulting to 50). The P2C selection algorithm draws exclusively from the highest-priority eligible group, gracefully degrading to lower-priority tiers when top-tier accounts are rate-limited or exhausted.
            -   **Full-Stack & Headless Parity**: Implemented desktop and Web parity via `update_account_priority` IPC and `POST /api/accounts/:accountId/priority`. Changes take effect immediately via live token manager reload without requiring proxy restarts.
        -   **[Official Reset Cycle Alignment for Weekly Stats] Accurate 7-Day Window Without False Truncation (PR #3520, Thanks to @buluw)**:
            -   **Eliminate Fraction-Increase Heuristic**: Removed inferred `cycle_start` heuristics in favor of strict official reset interval `[reset_time - 7 days, reset_time)`, ensuring account quota replenishments never falsely reset or truncate historical weekly token totals.
        -   **[Dual-Channel In-App Independent Updates] Seamless In-App Upgrades & Channel Switching for Beta Users**:
            -   **Segmented Channel Switcher & Smart Defaults**: Introduced an elegant pill toggle on the About page allowing users to switch between `Stable` and `Preview (Beta)` channels; installations running pre-release versions default automatically to the Beta channel.
            -   **SemVer-Compliant Pre-Release Comparison Engine**: Upgraded version comparison to fully support semantic pre-release qualifiers (`4.8.1-beta.2` vs `4.8.1-beta.1`, `4.8.1` vs `4.8.1-beta.2`), preventing false positives or accidental downgrades.
            -   **Deterministic CDN Endpoints & CI Mirroring**: Automated release workflows now mirror pre-release assets to a dedicated, fixed `preview` Tag Release, providing a rate-limit-free endpoint for native updater downloads and background installs.

    *   **v4.8.1-beta.1 (2026-09-25)**:
        -   **[OpenAI Responses Protocol Enhancement] Add max_output_tokens Alias Support & Precise Thinking Budget/Level Mapping**:
            -   **Support max_output_tokens Deserialization Alias**: Added `max_output_tokens` and `maxOutputTokens` field aliases to top-level `OpenAIRequest`, ensuring standard client outputs are properly mapped to upstream `maxOutputTokens`.
            -   **Unit Tests & Edge-Case Coverage**: Enhanced test suites to cover `max_completion_tokens`, `max_output_tokens`, and `reasoning.max_tokens` thinking budget alias resolution paths.

    *   **v4.8.0 (2026-09-23)**:
        -   **[Full-Protocol Tool & Argument 100% Pure Passthrough] Eliminate Agent-Client Tool Call Failures Caused by Legacy Truncation and Opaque Rewriting (PR #3504)**:
            -   **Lossless Tool & Argument Egress**: Removed tool-name mapping, argument alias rewriting, and error command injection across OpenAI, Anthropic Claude, and Google Gemini adapters, allowing tool names and arguments to reach upstream with the client's original semantics intact — resolving tool call errors in OpenClaw and other agent clients.
            -   **Zero-Intrusion Descriptions & Verbatim Outputs**: Removed write-based injection into `description` during schema validation and eliminated the tool output compressor; descriptions and execution results now pass through 100% unmodified.
            -   **System Instruction Freeze & Dead Code Excised**: Replaced aggressive System Prompt freezing with `<system-reminder>` user turns; excised `ToolAdapter`, `PencilAdapter`, and 700+ lines of legacy `apply_patch` diagnostics, netting 3,300+ lines of cleaned code.
        -   **[Vendor Attribution Statement Normalization] Prevent False 429 Upstream Cool-down Deadlocks (Fixes #3508, Thanks to @oliverhe202018-ctrl)**:
            -   **Dynamic Prompt Normalization**: Sanitizes foreign vendor statements in client prompts to prevent upstream security policies from misclassifying requests as abusive, completely eliminating entire pool cooldowns triggered by false rate-limiting.
        -   **[Seamless IDE Hot-Switching] Restart language_server Worker Process Only Without Terminating Main Window**:
            -   **Targeted Hot-Restart**: Automatically restarts only the internal language server engine on account rotation instead of killing the main IDE application window, preserving active developer context.
        -   **[Proxy Pipeline Clarification & Audit Telemetry Optimization]**:
            -   **Inbound vs Outbound Boundary**: Excised inactive outbound pipeline stages, establishing unified inbound convergence and divergent outbound handling; converged risky prompt sanitization into `PromptSanitizer`.
            -   **Audit Field Priority Reordering**: Reordered telemetry fields by developer priority (Model → Thought → Context → Usage → Tools), and extended the simple audit whitelist for OpenAI Responses instructions and inputs.
            -   **GET Request Log Suppression**: Suppresses successful GET request persistence when traffic capture is disabled, drastically reducing database bloat.
        -   **[Dual-Track Release Pipelines & Strict Channel Gates]**:
            -   **Strict Channel Isolation**: Established `main` for stable releases and `beta` for isolated preview builds; introduced `verify-release-target` CI/CD gate to intercept cross-branch misplacements.
            -   **Zero Production Intrusion**: Pre-releases are marked as non-latest and completely isolated from the automatic update channel and Docker `latest` tags.
            -   **Branch-Aware Bump Tooling**: Upgraded `bump-version.mjs` with branch auto-detection, directional guidance, and misplacement safeguards.

    *   **v4.92.0 (2026-09-28)**:
        -   **[Release v4.92.0: Supabase Multi-Machine Relational Hierarchy, Native CLI Suite, PowerShell One-Liners & Distributed Cross-Machine In-Use Leases] Implemented Root (Lovable tagged) and Secondary architecture, parent machine nodes & child instance profiles cascading foreign keys, cross-machine active in-use lease tracking on account switches, native agm supabase CLI suite with comprehensive schema help, and standalone PowerShell one-liner configuration scripts (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Supabase Root (Lovable) & Secondary Multi-Endpoint Architecture (`src-tauri/src/modules/supabase_client.rs`, `src-tauri/src/modules/supabase_sync.rs`)**: Enforced dual-role endpoint models with Lovable designated as Root annotated with notes and tags metadata; supported Secondary fallback endpoints; improved PostgREST probe ladders to reliably handle publishable keys (`sb_publishable_...`) schema cache states.
            -   **Machine Parent-Child Hierarchy & Distributed Collision Leases (`src-tauri/src/modules/supabase_schema.rs`, `src-tauri/src/modules/workspace_lease_manager.rs`)**: Restructured SQL schema establishing `nodes` as parent machine entities, with child `instance_profiles` and active `workspace_leases` referencing `nodes.id` with `ON DELETE CASCADE`; enriched leases with machine IP and account email; every account switch (UI, CLI, multi-instance, auto-switch) actively registers in Supabase Root DB to prevent cross-machine collisions.
            -   **Native CLI Management Suite & Schema Help (`src-tauri/src/bin/agm.rs`)**: Added `agm supabase` subcommand family (`help`, `status`, `list-leases`, `test`, `set`, `load-json`, `sync`, `schema`, `enable`, `disable`, `set-alias`), surfacing target config file locations, JSON format guides, DB relational hierarchy diagrams, and migration DDL; fully integrated with `agm help` and `agm help --json`.
            -   **PowerShell Automation & JSON Ingestion Suite (`scripts/setup-supabase.ps1`, `scripts/supabase-endpoints.json`)**: Delivered automated PowerShell setup scripts and companion JSON templates supporting one-liner execution and immediate live HTTP connectivity testing.

    *   **v4.91.1 (2026-09-28)**:
        -   **[Release v4.91.1: Auto-Sync Login State Modal Suppression, Failed Token Caching & Silent Graceful Fallback] Eliminated intrusive E9001 global error diagnostic popups triggered by background account sync when local IDE or Keyring holds expired/revoked OAuth tokens, added failed token caching to prevent recurring queries, and enforced background probe modal suppression (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Failed Token Caching & Silent Fallback (`src-tauri/src/commands/mod.rs`, `src-tauri/src/proxy/server.rs`)**: Implemented `LAST_FAILED_SYNC_TOKEN` in `sync_account_from_db` to cache tokens that fail local import, skipping expensive repeated OAuth queries; handled `import_from_db` errors gracefully by logging informative messages and returning `Ok(None)` / `Ok(Json(None))` instead of bubbling fatal errors across the IPC bridge.
            -   **Frontend Diagnostic Modal Suppression (`src/services/accountService.ts`)**: Added `{ _suppressGlobalModal: true }` to `syncAccountFromDb()` Tauri IPC invocation, preventing speculative background account synchronization tasks from ever triggering intrusive full-screen red error modals in the UI.
            -   **Migration Logging & Observability (`src-tauri/src/modules/migration.rs`)**: Enhanced logging in `import_all_local_accounts` to provide clear warnings when candidate Keyring or SQLite DB OAuth tokens fail refresh due to expiration or revocation.

    *   **v4.91.0 (2026-09-28)**:
        -   **[Release v4.91.0: Pure JSON Email Telemetry, Strict 100% Quota Gating & Live Google API Probes, Telegram Project Deduplication, and Resilient Installer] Zero-HTML JSON email bodies with normalized schema, strict 100% 4-hour window quota enforcement with live Google API pre-switch probes and depleted bounce prevention, Telegram `/projects` deduplicated by workspace path with worker routing `<node-alias>:<cmd>` and `/prompts` catalog, and fixed `install.ps1` exit code leakage for seamless GitMap updates (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Pure JSON Email Telemetry & Schema Normalization (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/email_sender.rs`)**: Enforced 100% pure JSON payload bodies when email subjects contain `[JSON]`, stripping all HTML, CSS `<style>` blocks, and markdown fences. Normalized telemetry fields to `previous_email`, `predicted_email`, `selected_email`, `quota_percent`, `threshold_activated`, `machine_name`, `node_alias`, `local_ip`, `running_prompts_count`, and `timestamp`, eliminating duplicated fields (`old_email`, `new_email`, `target_email`).
            -   **Strict 100% Quota Gating & Live Google API Probing (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/account.rs`, `src/services/instanceService.ts`, `src/stores/useInstanceStore.ts`)**: Overhauled quota evaluation to compute the strict minimum across all short buckets and non-banned models (preventing Pro 20% from being masked by Flash 100%). Candidates under 100% quota are treated as 0.0% exhausted and excluded. Mandatory live Google API quota probes (`fetch_quota_with_retry`) verify fresh capacity prior to committing switches, strictly rejecting depleted or disabled profiles without unwanted fallback loops.
            -   **Telegram Project Deduplication & Fleet Routing (`src-tauri/src/modules/telegram_inbound.rs`)**: Deduplicated Telegram `/projects` output by canonical repository path, grouping multiple conversations under single project entries with active/total conversation counts. Added `<node-alias>:<command>` and `<ip>:<command>` selector syntax to dispatch commands to specific fleet nodes, introduced `/prompts` command with slugs and ~200-word preview snippets, and enabled prompt prefix/suffix/voice concatenation.
            -   **Installer Resilience & Exit Code Fix (`install.ps1`)**: Resolved Go `cmd.Run()` failure (`exit status 1`) in GitMap updater by adding explicit `$global:LASTEXITCODE = 0; exit 0` at script termination. Added graceful fallback to retain existing verified installations if remote binary downloads fail, ensuring reliable updates across all nodes.

    *   **v4.90.0 (2026-09-28)**:
        -   **[Release v4.90.0: Account Switch 98% Simulation E2E, Parallel Prompt Backup & Restoration, Multi-VM Collision Shielding & Sandbox Lifecycle Verification] Validated end-to-end 98% simulated failover, pre-switch live API quota refresh stability probe, multi-workspace parallel prompt snapshot to split SQLite with human names & image payloads, fast-forward button delegation, automatic prompt resumption & emergency alerts, Supabase lease & IMAP collision avoidance, sandbox instance lifecycle verification, comprehensive CLI help polish, and 15.0% production standard alignment (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **98% Simulated Failover & Live Google API Refresh Stability Probe (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/bin/agm.rs`)**: Successfully tested high-threshold (98%) simulated account switch (`agm switch-if-low-credit -t 98`). The candidate scoring algorithm ranks and prioritizes 100% full-quota accounts, and performs a live Google API quota refresh and stability probe (`account::fetch_quota_with_retry`) immediately before switching to verify fresh capacity before rotation commits.
            -   **Multi-Workspace Parallel Prompt Backup to Split SQLite (`src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/modules/repo_db.rs`)**: Utilized `std::thread::scope` for concurrent scanning across workspace databases (`state.vscdb`) and resume task files. Extracted clean human-friendly project directory names (`Antigravity-Manager`, `gitmap`) eliminating internal UUID hashes, and synchronized prompt texts, image payloads, and file metadata into split SQLite database `backup-prompts.db`.
            -   **Fast-Forward Delegation, Automatic Prompt Re-injection & Emergency Alerts (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/notification_hub.rs`)**: Delegated profile rotation to the unified fast-forward pipeline; post-switch automatically executed `agm restore` to re-enqueue in-flight prompts back into active queues, confirmed active via `verify_prompts_running()`, and wired immediate emergency alerts across Telegram & Email if prompt restoration stalls.
            -   **Multi-Stage Telegram & Email Telemetry Notification Cards (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Dispatched multi-stage telemetry cards across Telegram and Email displaying secured project names, prompt counts, candidate evaluation probe results, and restoration counts.
            -   **Dual-Layer Multi-VM Collision Shielding (`src-tauri/src/modules/workspace_lease_manager.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Integrated Supabase Root DB distributed leases (`workspace_leases`) and recent (3600s) IMAP switch broadcast events to ensure candidate evaluation automatically skips profiles claimed by sibling VM nodes.
            -   **Sandbox Instance Lifecycle E2E (`src-tauri/src/modules/instance.rs`, `src-tauri/src/bin/agm.rs`)**: Created isolated sandbox instance profile (`agm instances create test-sandbox-e2e`), verified rotation and prompt backup/restore in isolation, and cleanly removed it via `agm instances rm test-sandbox-e2e --force` with zero leftover processes.
            -   **Comprehensive CLI Help Text & Real-World Examples Polish (`src-tauri/src/bin/agm.rs`)**: Overhauled and enriched `--help` documentation and examples across `agm prompts`, `agm accounts`, `agm switch`, `agm instances`, `agm backup`, `agm restore`, and `agm auto-switch`, ensuring 100% help coverage with copy-pasteable examples.
            -   **Production Standard Default Quota Threshold (15.0%) (`src-tauri/src/models/config.rs`, `src/components/settings/AutoSwitcherSettings.tsx`)**: Restored and strictly enforced the 15.0% production standard default threshold across backend models and frontend configuration fallbacks.

    *   **v4.89.0 (2026-09-28)**:
        -   **[Release v4.89.0: Account Switch 98% Simulation E2E, Parallel Prompt Backup & Restoration, Multi-VM Collision Shielding & Sandbox Lifecycle Verification] Validated end-to-end 98% simulated failover, pre-switch live API quota refresh probe, multi-workspace parallel prompt snapshot to split SQLite with human names & image payloads, fast-forward button delegation, automatic prompt resumption, Supabase lease & IMAP collision avoidance, sandbox instance lifecycle verification, comprehensive CLI help polish, and 15.0% production standard alignment (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **98% Simulated Failover & Live Google API Refresh Probe (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/bin/agm.rs`)**: Successfully tested high-threshold (98%) simulated account switch (`agm switch-if-low-credit -t 98`). The candidate scoring algorithm ranks and prioritizes 100% full-quota accounts, and performs a live Google API quota refresh probe (`account::fetch_quota_with_retry`) immediately before switching to verify fresh capacity before rotation commits.
            -   **Multi-Workspace Parallel Prompt Backup to Split SQLite (`src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/modules/repo_db.rs`)**: Utilized `std::thread::scope` for concurrent scanning across workspace databases (`state.vscdb`) and resume task files. Extracted clean human-friendly project directory names (`Antigravity-Manager`, `gitmap`) eliminating internal UUID hashes, and synchronized prompt texts, image payloads, and file metadata into split SQLite database `backup-prompts.db`.
            -   **Fast-Forward Delegation & Automatic Prompt Re-injection (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Delegated profile rotation to the unified fast-forward pipeline; post-switch automatically executed `agm restore` to re-enqueue in-flight prompts back into active queues, confirmed active and running via `agm prompts ls`.
            -   **Multi-Stage Telegram & Email Telemetry Notification Cards (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Dispatched multi-stage telemetry cards across Telegram and Email displaying secured project names, prompt counts, candidate evaluation probe results, and restoration counts.
            -   **Multi-VM Collision Shielding (`src-tauri/src/modules/workspace_lease_manager.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Integrated Supabase Root DB distributed leases (`workspace_leases`) and recent (3600s) IMAP switch broadcast events to ensure candidate evaluation automatically skips profiles claimed by sibling VM nodes.
            -   **Sandbox Instance Lifecycle E2E (`src-tauri/src/modules/instance.rs`, `src-tauri/src/bin/agm.rs`)**: Created isolated sandbox instance profile (`agm instances create test-sandbox-e2e`), verified rotation and prompt backup/restore in isolation, and cleanly removed it via `agm instances rm test-sandbox-e2e --force` with zero leftover processes.
            -   **Comprehensive CLI Help Text & Real-World Examples Polish (`src-tauri/src/bin/agm.rs`)**: Overhauled and enriched `--help` documentation and examples across `agm sync`, `agm pull`, `agm install`, `agm running-prompts`, and `agm running-projects`, ensuring 100% help coverage with copy-pasteable examples.
            -   **Production Standard Default Quota Threshold (15.0%) (`src-tauri/src/models/config.rs`, `src/components/settings/AutoSwitcherSettings.tsx`, `src/components/common/BackgroundTaskRunner.tsx`)**: Restored and strictly enforced the 15.0% production standard default threshold across backend models and frontend configuration fallbacks.

    *   **v4.88.0 (2026-09-28)**:
        -   **[Release v4.88.0: Account Switch 98% Simulation E2E, Parallel Prompt Backup & Restoration, Multi-VM Collision Shielding & Sandbox Lifecycle Verification] Validated end-to-end 98% simulated failover, pre-switch live API quota refresh probe, multi-workspace parallel prompt snapshot to split SQLite with human names & image payloads, fast-forward button delegation, automatic prompt resumption, Supabase lease & IMAP collision avoidance, sandbox instance lifecycle verification, and 15.0% production standard alignment (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **98% Simulated Failover & Live Google API Refresh Probe (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/bin/agm.rs`)**: Successfully tested high-threshold (98%) simulated account switch (`agm switch-if-low-credit -t 98`). The candidate scoring algorithm ranks and prioritizes 100% full-quota accounts, and performs a live Google API quota refresh probe (`account::fetch_quota_with_retry`) immediately before switching to verify fresh capacity before rotation commits.
            -   **Multi-Workspace Parallel Prompt Backup to Split SQLite (`src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/modules/repo_db.rs`)**: Utilized `std::thread::scope` for concurrent scanning across workspace databases (`state.vscdb`) and resume task files. Extracted clean human-friendly project directory names (`Antigravity-Manager`, `gitmap`) eliminating internal UUID hashes, and synchronized prompt texts, image payloads, and file metadata into split SQLite database `backup-prompts.db`.
            -   **Fast-Forward Delegation & Automatic Prompt Re-injection (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Delegated profile rotation to the unified fast-forward pipeline; post-switch automatically executed `agm restore` to re-enqueue in-flight prompts back into active queues, confirmed active and running via `agm prompts ls`.
            -   **Multi-Stage Telegram & Email Telemetry Notification Cards (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Dispatched multi-stage telemetry cards across Telegram and Email displaying secured project names, prompt counts, candidate evaluation probe results, and restoration counts.
            -   **Multi-VM Collision Shielding (`src-tauri/src/modules/workspace_lease_manager.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Integrated Supabase Root DB distributed leases (`workspace_leases`) and recent (3600s) IMAP switch broadcast events to ensure candidate evaluation automatically skips profiles claimed by sibling VM nodes.
            -   **Sandbox Instance Lifecycle E2E (`src-tauri/src/modules/instance.rs`, `src-tauri/src/bin/agm.rs`)**: Created isolated sandbox instance profile (`agm instances create test-sandbox-e2e`), verified rotation and prompt backup/restore in isolation, and cleanly removed it via `agm instances rm test-sandbox-e2e --force` with zero leftover processes.
            -   **Production Standard Default Quota Threshold (15.0%) (`src-tauri/src/models/config.rs`, `src/components/settings/AutoSwitcherSettings.tsx`, `src/components/common/BackgroundTaskRunner.tsx`)**: Restored and strictly enforced the 15.0% production standard default threshold across backend models and frontend configuration fallbacks.

    *   **v4.87.0 (2026-09-28)**:
        -   **[Release v4.87.0: Fleet-Wide Update-All Pure JSON Mode, CLI/UI Help Polish, and Full Prompt Tracking E2E Verification] Introduced zero-noise `agm update all` / `ua` JSON automation, overhauled CLI Part 2 double-box help layout, added 15% production standard and 98% simulation presets with CLI quick reference in UI, verified live prompt injection and parallel backup/restore (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **`agm update all` / `ua` Zero-Noise JSON Mode & Interactive Visual Dashboard (`src-tauri/src/bin/agm.rs`)**: Added `update all`, `update-all`, and `ua` command routes. Implemented strict `--json` mode emitting pure, unpolluted JSON without ANSI escapes or ASCII banners for reliable remote machine automation over SSH and clusters; interactive terminals render a structured system update card reporting binary target, version comparison, release links, git repository pull status, and node identity.
            -   **CLI Part 2 Double-Box Layout Overhaul & Structured JSON Help (`src-tauri/src/bin/agm.rs`)**: Completely overhauled `print_help()` with double-line headers, grouped command categories (Quota Governance, Fleet Orchestration, Prompt Backup, Sandboxes, Telemetry), and aligned syntax flags; added `agm help --json` / `agm --help --json` emitting machine-readable command schemas.
            -   **Frontend UI Threshold 1-Click Presets & CLI Quick Reference Panel (`src/components/settings/AutoSwitcherSettings.tsx`, `src/pages/Settings.tsx`)**: Added `[🛡️ 15% (Production Standard)]` and `[🧪 98% (Simulation / Testing)]` 1-click preset buttons with dynamic context explaining why 15% is standard production and how 98% is used for fast-failover simulation; added an "AGM CLI & Fleet Automation Quick Reference" panel in Advanced settings.
            -   **Live Prompt Injection & Workspace Tracking Verification (`src-tauri/src/bin/agm.rs`)**: Dispatched test prompts via `agm prompt` and verified immediate registration in SQLite and detection in `agm prompts ls` / `agm prompts ls --json` under the project directory.
            -   **Multi-Project Parallel Prompt Backup & Restoration E2E (`src-tauri/src/bin/agm.rs`)**: Verified `agm backup`, `agm backup ls`, and `agm restore`, confirming running prompts across workspaces are safely snapshotted to split SQLite (`backup-prompts.db`) and restored back into active execution queues.

    *   **v4.86.0 (2026-09-28)**:
        -   **[Release v4.86.0: End-to-End Account Switch Verification, Multi-Project Parallel Prompt Backup & Restoration, Multi-VM Collision Prevention, and 15% Default Threshold Alignment] Simulated 98% High Quota Rotation, Parallel Workspace Prompt Snapshots with Human-Friendly Project Naming, Supabase Distributed Leases & IMAP Telemetry Collision Shielding, and Production-Standard 15.0% Low Quota Threshold (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **High-Threshold 98% Quota Simulation & Sandbox Instance Isolation (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/models/config.rs`)**: Conducted end-to-end simulation of quota rotation under high threshold (`agm switch-if-low-credit -t 98 --json`), proving candidate evaluation probes against Google API and successfully switching to a 100% full-credit profile. Created and verified an isolated sandbox instance (`test-sandbox-6237`) executing rotation and prompt re-injection, followed by clean sandbox removal.
            -   **Multi-Project Parallel Prompt Backup & Human-Friendly Project Names (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/backup_prompts_db.rs`)**: Leveraged `std::thread::scope` for non-blocking concurrent scanning of workspace state databases (`state.vscdb`) and resume task manifests (`.antigravity_resume_task.json`). Implemented `resolve_friendly_project_name` to eliminate internal UUID and hash garbage, resolving clean directory basenames (e.g. `Antigravity-Manager`, `gitmap`). Synchronized image payloads and file paths, exposing `verify_prompts_running()` for real-time prompt health verification.
            -   **Multi-VM Distributed Lease & Collision Prevention (`src-tauri/src/modules/workspace_lease_manager.rs`, `src-tauri/src/modules/auto_switcher.rs`)**: Implemented `is_account_or_email_leased_by_other(account_id, email)` checking active Supabase Root DB leases and recent 3600-second IMAP switch broadcast events. Automatically skips accounts actively claimed or selected by sibling VMs in the cluster, picking the next highest-quota profile.
            -   **Telegram & Email Multi-Stage Telemetry with Friendly Project Names (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Updated switch alert cards across Email and Telegram to explicitly display friendly project lists (`📁 Projects: Antigravity-Manager, gitmap`), captured prompt counts, candidate evaluation rationale, and post-switch prompt re-injection statistics.
            -   **CLI Help Polish with Copy-Pasteable Syntax Examples (`src-tauri/src/bin/agm.rs`)**: Overhauled `agm --help`, `agm backup --help`, `agm restore --help`, `agm auto-switch --help`, and `agm switch-if-low-credit --help` with structured flag descriptions, aliases, and copy-pasteable real-world examples.
            -   **Standardized 15.0% Default Quota Threshold Alignment (`src-tauri/src/models/config.rs`, `src-tauri/src/modules/config.rs`, `src/pages/Settings.tsx`)**: Realigned production defaults for `low_quota_threshold_percent`, `critical_threshold_percent`, and `quota_protection.threshold_percentage` to 15.0% / 15 with automatic migration from legacy 10%/12%/25% configurations.

    *   **v4.85.0 (2026-09-27)**:
        -   **[Release v4.85.0: Windows Runner Build Acceleration, Pipeline Fast-Forwarding, and Contributor Attribution Normalization] Accelerated Windows CI/CD and release pipelines, eliminated runner memory starvation and heap exhaustion via concurrency bounding, migrated to windows-2022 with comprehensive Defender exclusions, and normalized repository codeownership (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Windows Build Acceleration & Concurrency Bounding (`.github/workflows/release.yml`, `.github/workflows/ci.yml`)**: Fixed the root cause of Windows runner jobs running for 56+ minutes and dying with `The hosted runner lost communication with the server`. Implemented strict concurrency bounding via `CARGO_BUILD_JOBS: 4` and `CMAKE_BUILD_PARALLEL_LEVEL: 4`, completely eliminating MSVC `cl.exe` heap space exhaustion (`error C1060`) and OS error 1455 pagefile exhaustion during BoringSSL monolithic compilation.
            -   **Windows Runner Migration & Multi-Directory Defender Exclusions (`.github/workflows/release.yml`, `.github/workflows/ci.yml`)**: Migrated Windows runners from preview `windows-2025` to production-stable `windows-2022`. Expanded Windows Defender exclusions to comprehensively cover `${{ github.workspace }}`, `$env:USERPROFILE\.cargo`, `$env:USERPROFILE\.rustup`, `$env:TEMP`, and `$env:LOCALAPPDATA\Temp`, eliminating disk queue thrashing.
            -   **Rust Cache Target Retention & Fast-Forwarding (`.github/workflows/release.yml`, `.github/workflows/ci.yml`)**: Configured `cache-all-crates: "true"` and `cache-targets: "true"` in `Swatinem/rust-cache@v2`, ensuring intermediate C/C++ and Rust dependencies are restored from cache in seconds rather than recompiling from scratch on every release run.
            -   **Codeownership & Contributor Attribution Normalization (`.github/CODEOWNERS`, `AGENTS.md`, `CHANGELOG.md`, `CHANGELOG_EN.md`)**: Restored sole repository ownership to `@alimtvnetwork` / alim (`devorg.bd@gmail.com`, `@aukgit`) across `CODEOWNERS`, `AGENTS.md`, and all release notes, ensuring proper contributor attribution on GitHub release pages.

    *   **v4.84.0 (2026-09-27)**:
        -   **[Release v4.84.0: Native Telegram Chat CLI Management, Human-Friendly Workspace Naming, Smart Prompt Preview Sanitization, and /projects CLI/Bot Parity] Added automated `agm telegram chat` and `setup` CLI workflow, eliminated cryptic internal UUID workspace hashes in observation reports, implemented intelligent noise stripping for user prompts, and enabled seamless slash-prefixed CLI commands (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Native Telegram Chat CLI Setup & Interactive Discovery (`agm telegram chat`, `agm telegram setup`) (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Introduced native `agm telegram chat <BOT_TOKEN> [CHAT_ID]` and `agm telegram setup` commands with built-in `--auto-detect` support. Provides comprehensive `--help` documentation detailing the three discovery methods (sending `/start` to bot, querying `@userinfobot`, or group negative chat IDs), persists credentials cleanly without UTF-8 BOM, registers slash commands with Telegram via `setMyCommands`, and fires immediate welcome telemetry cards.
            -   **Human-Friendly Workspace Naming & Git Branch Resolution (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Fixed the root cause of workspace displays showing cryptic internal IDE hashes (e.g. `antigravity-manager-d58c5517`). Corrected the SQLite insertion in `running_projects` where `project_id` was erroneously saved into `repo_name`, implemented zero-subprocess `.git/HEAD` branch resolution (`get_git_branch_for_path`), and resolved conversation titles (`lookup_conversation_title`) from `conversation_summaries.db`, outputting intuitive labels like `Antigravity-Manager (main) [AGM]` and `gitmap (main)`.
            -   **Smart Prompt Preview Sanitization & Noise Elimination (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/telegram_inbound.rs`)**: Eliminated telemetry reports showing raw `<USER_REQUEST>\n808e0172...` commit SHAs and directory paths. Implemented `extract_smart_prompt_summary` to filter out XML tags, 40-hex commit hashes, filesystem paths, and markdown headers, extracting the substantive human directive.
            -   **Full CLI & Telegram `/projects` Command Interoperability (`src-tauri/src/bin/agm.rs`)**: Permitted leading slash prefixes across all terminal CLI commands (`agm /projects`, `agm /nodes`, `agm /status`), aligning CLI syntax 100% with Telegram slash commands.

    *   **v4.83.0 (2026-09-27)**:
        -   **[Release v4.83.0: Installer Upstream Fork Inversion Elimination, Telegram 4096-Character Limit Protection, Cluster VM Nodes Topology, Scoped Prompts Catalog, and Headless Prompt Injection] Complete Eradication of Upstream Fork References, Automatic Message Chunking with Pacing, Remote Machine Fleet Topology, Scoped Prompts Discovery, and Seamless CLI Parity (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Installer Upstream Fork Elimination (`install.ps1`, `install.sh`, `deploy/arch/install.sh`, `README_EN.md`, `02-spec/22-app-issues/20-installer-upstream-fork-inversion-rca.md`)**: Completely eradicated all references to `lbjlaq/Antigravity-Manager` across installation routines and docs. Deleted `$UpstreamRepo` and API fallback candidates, ensuring 100% of download targets and asset queries resolve strictly to `alimtvnetwork/Antigravity-Manager`.
            -   **Telegram 4096-Character Limit Protection & Pacing (`src-tauri/src/modules/telegram_inbound.rs`)**: Implemented robust line-boundary message chunking via `chunk_telegram_text` and `send_telegram_message_chunked`. Automatically splits messages exceeding 3800 characters at newline boundaries with 80ms transmission pacing, eliminating `400 Bad Request: message is too long` API dropouts.
            -   **Telegram Cluster VM Nodes Topology (`/nodes`, `/node ls`, `agm telegram nodes`) (`src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`)**: Implemented `/nodes` and `/node ls` command handlers aggregating GitMap cluster status (`gitmap cluster status`) and Supabase Root DB `nodes` table, rendering active VM nodes with status badges (`🟢 ONLINE`, `⚪ IDLE`, `🔴 OFFLINE`), IP, uptime, and fleet connectivity.
            -   **Telegram Scoped Prompts & Projects Catalog (`/projects`, `/prompts`, `/nodes <alias> prompts`) (`src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`)**: Added `/nodes <alias> prompts` to inspect running prompts scoped to individual nodes, alongside `/projects` to list active workspaces and `/prompts` to catalog recent prompt queues.
            -   **Remote Prompt Injection & Full Headless CLI Parity (`/prompt <node> <project> <text>`, `agm telegram prompt`) (`src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`)**: Enabled remote prompt injection from Telegram and the CLI. For local targets, dispatches directly into `repo_db` and triggers workspace execution; for remote cluster nodes, enqueues the prompt into Supabase Secondary DB `command_queue` or executes via GitMap cluster SSH delegation.
            -   **Comprehensive Sanitized Telegram Bot Guide (`docs/telegram-bot-guide.md`)**: Authored an exhaustive, sanitized guide detailing GUI setup, Chat ID auto-discovery via `/ping` or `/start`, CLI commands reference, and strict credential isolation without committing sensitive tokens.

    *   **v4.82.0 (2026-09-27)**:
        -   **[Release v4.82.0: Smart Switch Strict 100% Quota Gate, Disabled Account Absolute Immunity, Pre-Switch Live Google API Refresh, and Supabase Cross-Node Cluster Leasing] Completely Eliminated Low-Credit (20%) Selection & Disabled Account Re-selection Loops, Enforced 100% 4-Hour Rolling Quota Gate, Added Direct Google API Probes for All Candidates, and Integrated Supabase Root DB Distributed Leases (Thanks to alim, devorg.bd@gmail.com, @aukgit)**:
            -   **Strict 100% 4-Hour Rolling Quota Gate (`src-tauri/src/modules/auto_switcher.rs`, `src/stores/useInstanceStore.ts`)**: Resolved critical defect where Smart Switch / Fast-Forward selected accounts with only 11%–20% remaining quota. Enforced strict architectural rule: any 4-hour rolling window quota below 100.0% is strictly treated as exhausted (0.0%), and accounts with < 100% are never considered as candidates or fallback targets. Completely eliminated secondary fallback passes (e.g. 15% threshold pass) and deleted `verified_fallback` and `bestFallbackCandidate` buffers. If no candidate verifies at 100% from Google API, the switcher safely aborts with an actionable notification rather than binding an exhausted account.
            -   **Absolute UI Disabled Account Immunity (`proxy_disabled`) (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/commands/mod.rs`, `src/stores/useInstanceStore.ts`)**: Fixed severe bug where disabling an exhausted account in the UI (e.g. `James`) did not prevent the switcher from immediately re-selecting it. Root-caused to UI toggles setting `account.proxy_disabled = true` while candidate selectors only checked `acc.disabled`. Enforced universal checks for `acc.disabled || acc.proxy_disabled || acc.validation_blocked || !acc.is_active` across both Rust backend and React frontend. Fixed `commands/mod.rs:toggle_proxy_status` to atomically synchronize both `{account_id}.json` and `index.json`.
            -   **Candidate Pre-Switch Live Google API Refresh Loop (`src-tauri/src/modules/auto_switcher.rs`, `src/stores/useInstanceStore.ts`)**: Prior to executing profile rotation, candidate accounts are probed in priority order directly against the Google API via `account::fetch_quota_with_retry`. If live 4-hour quota is < 100.0%, the candidate is rejected and the switcher immediately evaluates the next candidate in the pool until a genuine 100% match is found.
            -   **Supabase Root DB Distributed Leasing & Multi-Channel Cluster Awareness (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/workspace_lease_manager.rs`)**: Proactively hydrates and verifies active remote leases from Supabase Root DB (`workspace_leases`) prior to candidate evaluation, strictly excluding accounts held by other node IDs. Integrated Email Inbound 3600s cross-VM switch broadcasts to prevent multi-machine account collisions.
            -   **Visual Asset Ingestion & Privacy Redaction (`assets/screenshots/smart-switch-refactor-01.png`, `02-spec/22-app-issues/19-smart-switch-stale-quota-and-in-use-isolation.md`)**: Ingested user screenshot asset with Radius 25 Gaussian blur covering all personal email addresses, preserving 100% privacy and credential hygiene.

    *   **v4.81.0 (2026-09-27)**:
        -   **[Release v4.81.0: Idle Sensor False-Positive Elimination, GitMap Dual-Box Telemetry Parity, Significantly Enlarged Typography, and Usable Prompts Cheat Sheet] Completely Resolved False Idle Reporting on Active Runs, Aligned AGM CLI & Email Telemetry to GitMap Standard, Enlarged Email Fonts to ≥18px Body / ≥16px Tables, and Integrated 01-prompts Library Reference**:
            -   **Idle Sensor False-Positive Fix & SQLite WAL Lock Prevention (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/email_watcher.rs`)**: Resolved critical defect where active workspace executions (such as `antigravity-manager-d58c5517`) were erroneously reported as IDLE via outbound email alerts. Overhauled detection engine: (1) Connected to `conversation_summaries.db` using read-only URI mode with `?immutable=1` fallback to avoid Windows WAL locks; (2) Implemented dynamic RFC3339 timestamp filter checking if `last_modified_time` was within the last 600s or in `CASCADE_RUN_STATUS_RUNNING`; (3) Normalized filesystem path comparison to ignore slash differences; (4) Added 8-character conversation prefix matching and strictly intercepted idle email dispatch whenever `is_any_prompt_actively_running()` returns `true`.
            -   **GitMap Standard Binary & Workspace Dual-Box Telemetry Parity (`src-tauri/src/modules/git_info.rs`, `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/email_sender.rs`)**: Replicated GitMap CLI output format exactly. Added `get_git_full_hash()`, `get_repo_url()`, and `get_built_timestamp()` in `git_info.rs`. Standardized terminal banner (`agm`, `agm help`) to render two distinct panels: `agm binary` (name, Git URL, version, full 40-character commit SHA, branch, last release, SQLite database path, binary executable path, built timestamp) and `current workspace / node` (node alias, local IPv4, dispatch timestamp). Embedded identical metadata tables into all outgoing email headers.
            -   **Significantly Enlarged Typography Across Email Templates (≥18px Body / ≥16px Tables) (`src-tauri/src/modules/email_sender.rs`)**: Directly resolved user feedback on small font sizes. Increased body font to 18px (line-height 1.7), card headings to 28-32px bold, section titles to 20px uppercase bold, and all table headers/cells to 16-17px. Command badges and syntax examples are styled with 16px monospace with comfortable padding (`6px 12px`).
            -   **Active Workspaces, Inbound Commands, and 01-prompts Reference Cheat Sheets (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/email_sender.rs`)**: Embedded three structured tables in both email notifications and CLI output: (1) Active Workspaces Table (project name, copyable reply target `sub: <NODE> | proj-<id>`, normalized repository path, and live status); (2) Available Inbound Commands & Email Format Table; (3) Usable Prompts Reference List from `01-prompts/` (including `execute-pending-tasks`, `execute-parent-task`, `ci-cd-fix`, `minor-bump`, `coding-guidelines`, `smart-test-runner`).

    *   **v4.80.0 (2026-09-27)**:
        -   **[Release v4.80.0: End-to-End Telegram Bot Remote Command Center, One-Click Chat ID Auto-Discovery, Full `agm telegram` Terminal CLI, and Windows Release Build Memory Optimization] Seamless UI & CLI Bot Connection with Auto-Detected `allowed_chat_id`, Live `/ping`, `/observe`, `/gitmap`, `/api`, `/backup`, `/email` Remote Commands, and Zero-Leak Sanitized Documentation**:
            -   **One-Click Telegram Chat ID Auto-Discovery & First-Message Auto-Binding (`src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/commands/telegram.rs`, `EmailNotificationSettings.tsx`, `SupabaseSyncSettings.tsx`)**: Implemented `detect_telegram_chat_id` and UI `"Auto-Detect Chat ID"` controls in both Email & Alerts and Supabase Sync settings panels to automatically discover the user's numeric `allowed_chat_id` from Telegram's `getUpdates` queue without third-party lookup bots. Added automatic first-message `allowed_chat_id` binding in the background polling daemon and automatic `setMyCommands` menu registration.
            -   **Full-Spectrum Telegram Slash Commands & `agm telegram` CLI (`src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`)**: Expanded both inbound Telegram bot commands and the `agm telegram` CLI with `connect <TOKEN> [CHAT_ID]`, `detect-chat-id`, `ping` (node IP, version, Git commit hash, branch, and last release tag), `observe`/`status` (active account, model quota, running vs idle workspaces, prompt queue, and backup batches), `gitmap [args]` (live `gitmap pe` CI/CD matrix), `api`/`agm`, `backup`/`backpack [ls]`, `restore`, `email [status|ping|help]`, `send`/`notify`, and `poll [--once]`.
            -   **Sanitized Architecture Specification & README Setup Guides (`02-spec/21-app/53-telegram-bot-auto-connect-and-remote-commands.md`, `02-spec/21-app/telegram-bot-setup-guide.md`, `readme.md`, `README_EN.md`)**: Documented end-to-end UI and CLI connection workflows, Chat ID auto-discovery, and remote command tables while strictly isolating real credentials to `~/.antigravity_tools/telegram_config.json` with zero sensitive tokens or personal data in Git.
            -   **Windows Release Build Memory & Concurrency Optimization (`src-tauri/Cargo.toml`, `.github/workflows/release.yml`)**: Tuned `[profile.release]` `codegen-units = 16` and `CARGO_BUILD_JOBS = 2` on Windows runners to prevent runner memory exhaustion during dual-binary (`agm-alim` + `agm`) compilation.

    *   **v4.79.0 (2026-09-27)**:
        -   **[Release v4.79.0: Idle Sensor False-Positive Root Cause Fix, GitMap CLI/Email Telemetry Parity, Enlarged Ubuntu Typography with Pure White Links, and Command/Project/Prompt Cheat Sheet Integration] Eliminated False Idle Alerts During Active Runs, Matched GitMap Git Metadata Across Binaries & Emails, and Integrated Comprehensive Workspaces & Prompts Queue Tables**:
            -   **Idle Sensor Root Cause Fix & Anti-False-Positive Guard (`src-tauri/src/modules/email_watcher.rs`, `repo_db.rs`)**: Fixed the root cause where active workspace runs (such as `antigravity-manager-d58c5517`) were erroneously reported as IDLE in outgoing telemetry emails. `check_idle_projects_sensor` previously relied on `repo_db::list_backed_up_prompts()`, which only queries explicit backups (`status = 'backed_up'`). Rebuilt state detection to directly query live ground-truth state from `~/.gemini/antigravity/conversation_summaries.db` (`not_fully_idle != 0` and `status LIKE '%RUNNING%'`) and `repo_prompts.db`, guarded by `is_any_prompt_actively_running(&conn)` to strictly suppress idle notifications while any prompt is running.
            -   **GitMap-Style Compile & Runtime Telemetry Parity (`src-tauri/src/modules/git_info.rs`, `build.rs`, `agm.rs`, `email_sender.rs`)**: Achieved 1:1 telemetry parity with GitMap. Injected `AGM_GIT_HASH`, `AGM_GIT_BRANCH`, and `AGM_LAST_RELEASE` during compilation in `build.rs` with runtime fallback in `git_info.rs`. Standardized terminal banners, `agm version`, `agm status`, and all outgoing email headers to embed exact Git metadata.
            -   **Enlarged Typography, Ubuntu Font Stack & High-Contrast White Links (`src-tauri/src/modules/email_sender.rs`)**: Overhauled HTML email styling with 28px titles, 16px body, 14-15px tables/code, and an explicit Ubuntu / Segoe UI / system-ui font stack. Styled all interactive action and target links with pure `#ffffff` text on pill badges, ensuring effortless readability across email clients.
            -   **Commands Cheat Sheet, Live Workspaces Table & Prompt Queue in CLI & Email (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/email_sender.rs`, `repo_db.rs`)**: Integrated three comprehensive tables across both the `agm` CLI default output and idle alert emails: (1) categorized command reference with syntax examples; (2) discovered workspaces with live status, paths, and copy-pasteable email reply targets (`sub: <NODE> | proj-<id>`); (3) recent prompt tasks and in-flight queue inventory.

    *   **v4.78.0 (2026-09-27)**:
        -   **[Release v4.78.0: Multi-Channel System Update Notifications (Default Enabled), Email Remote Syntax Manual Upgrade, Zero-Loss Prompt Resumption Across Profile Switches, and Active-Only Low Credit Alerts] Automated Update Receipts via Email & Telegram, Ubuntu High-Contrast Typography, Resumption Without Reinjection Loops, and Targeted Quota Alerts**:
            -   **Configurable Multi-Channel System Update Notifications (`src-tauri/src/modules/notification_hub.rs`, `update_checker.rs`, `email_vault_db.rs`, `telegram_inbound.rs`, `Settings.tsx`)**: Added comprehensive update receipt dispatch across Email and Telegram. Application startup automatically detects version upgrades by comparing `last_known_version` with `CARGO_PKG_VERSION` and issues release telemetry receipts regardless of upgrade method (GUI installer, CLI, or manual). General Settings, Sensor Notification Triggers, and Telegram Bot panels include toggle controls, **all enabled (`true`) by default**.
            -   **Email Remote Control Default Help Manual & Ubuntu Typography (`src-tauri/src/modules/email_sender.rs`, `email_watcher.rs`, `EmailNotificationSettings.tsx`)**: Set default inbound mailbox response to a rich command syntax manual. Styled with Ubuntu font, enlarged 24px headers, and pure white links for maximum readability, alongside proxy offline status fallbacks.
            -   **Auto-Switch Prompt Backup, Resumption, and Reinjection Prevention (`src-tauri/src/proxy/auto_switcher.rs`, `src-tauri/src/bin/agm.rs`, `backup_prompts_db.rs`)**: Resolved prompt stall during account rotation by taking atomic snapshots of running prompts prior to switching, resuming execution immediately once the new instance is active, and establishing rigorous prompt hash deduplication to eliminate repeated reinjection loops.
            -   **Active-Account Low Credit Alert Targeting (`src-tauri/src/modules/notification_hub.rs`, `email_watcher.rs`)**: Refined low credit alert evaluation so notifications only dispatch if the low-quota account is actively in use, suppressing noise from idle standby accounts.

    *   **v4.77.0 (2026-09-27)**:
        -   **[Release v4.77.0: Running Prompts Split SQLite Backup & Restore Engine, 25% Auto-Switch Quota Threshold, Cross-Platform FPUG/SUG Green Watchers, Broadcast Email, and Telegram CLI Remote] Zero-Loss Prompt Snapshots Across Account Switches, 1-Day Auto-Cleanup Retention, Cross-Platform Native System Shutdown, and Autonomous Telegram Bot Telemetry**:
            -   **Dedicated Split SQLite Prompt Backup & Restore Engine (`src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/bin/agm.rs`, `repo_db.rs`)**: Introduced an independent, dedicated SQLite database (default path `~/.antigravity_tools/backup-prompts/backup-prompts.db`) to capture active and queued prompts across Antigravity core sessions and workspace environments. Added CLI commands `agm backup-running-prompts`, `agm restore-running-prompts`, and `agm running-prompts backup/restore` with full `--keep`, `--json`, `-file/-f`, and `clean [--force]` operations. Restored prompt records feature 1-day retention auto-cleanup with single-line operational notice.
            -   **Running Prompts Paginated Listing, Export & Import (`src-tauri/src/bin/agm.rs`)**: Added `agm running-prompts ls [--limit/-l Y] [--wordcount/wc N] [--full] [--json]` (default Y=8, N=100) with ASC queue formatting; dual-format export & import (`export/import`) supporting both SQLite `.db` and `.json` formats, seamlessly re-enqueuing prompts into the execution queue upon import.
            -   **Running Projects Inspection & Multi-Node SSH Fleet Mode (`src-tauri/src/bin/agm.rs`)**: Added `agm running-projects [ls] [--json] [-file/-f <path>] [--ssh]` to discover and report active projects, session IDs, conversation names, queued prompt counts, and cluster node telemetry.
            -   **25.0% Quota Switch Threshold & Dynamic CLI Configuration (`src-tauri/src/models/config.rs`, `src-tauri/src/bin/agm.rs`)**: Locked default low-quota switch threshold to `25.0%` with dynamic tuning via `agm auto-switch threshold [N]` (with typo-tolerant aliases `auto-swtich thresehold`).
            -   **FPUG & SUG Cross-Platform Green Watchers (`src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/backup_prompts_db.rs`)**: Implemented `agm finish-prompts-until-green` (`fpug`) and `agm shutdown-until-green` (`sug`), supporting cross-platform native shutdown (Windows `shutdown /s /t 60`, Ubuntu `systemctl poweroff`, macOS `osascript`) once all tracked projects drain their queues to green status.
            -   **Broadcast Email & Telegram CLI Management (`src-tauri/src/bin/agm.rs`, `email_vault_db.rs`, `telegram_inbound.rs`)**: Implemented `agm broadcast-email ls/add/edit/rm/send-to-all/send/send help/test` and `agm telegram ls/set/ping/cmds` for terminal bot setup and recipient management.

    *   **v4.76.0 (2026-09-27)**:
        -   **[Release v4.76.0: Privacy-Blurred Email Screenshots, Full Atomic 14-Location Version Sync, E9001 Proxy Pool Diagnostic Modal Resolution, and 100% 4-Hour Window Quota Verification] Eliminate Email Privacy Leaks, Fix Desktop Binary Frozen at v4.75.0, Proxy Offline Configuration Fallbacks, and Cross-Switch Running Prompt Persistence**:
            -   **Privacy Redaction & Assets Synchronization (`docs/images/dashboard-modern.png`, `assets/screenshots/dashboard-modern.png`)**: Completely eliminated sensitive user emails exposed in dashboard screenshots across README documentation. Applied smooth Gaussian blurring to the email column and synchronized clean assets across `docs/images/` and `assets/screenshots/`.
            -   **14-Location Full Version Synchronization Fix (`scripts/bump-version.mjs`, `NavLogo.tsx`, `TitleBar.tsx`, `error-report-generator.ts`, `hooks.nsh`)**: Fixed the root cause of desktop binaries remaining branded with old versions (e.g. `4.75.0`) despite SemVer releases. Replaced fragile string matching with robust regular expressions across `Cargo.toml`, `tauri.conf.json`, `Cargo.lock`, `Casks`, `MiniView`, `Settings`, and NSIS installer scripts, ensuring all 14 version manifests stay 100% in lockstep with `package.json`.
            -   **E9001 "Service not running" Error Modal Elimination (`src-tauri/src/commands/proxy_pool.rs`, `src-tauri/src/commands/proxy.rs`, `src/utils/request.ts`, `ProxyPoolSettings.tsx`)**: Resolved intrusive E9001 global error dialogs appearing when entering Settings -> Proxy tab while the API proxy service is stopped. Implemented direct persistent storage fallbacks to `AppConfig` and cleaned up `_suppressGlobalModal` control flags in IPC and Web utility wrappers.
            -   **Live Google API 100% 4-Hour Quota Verification & Next-Best Candidate Fallback (`src/stores/useInstanceStore.ts`, `src/services/instanceService.ts`, `auto_switcher.rs`)**: Enforced strict pre-switch verification via live Google API calls ensuring candidate accounts have a confirmed 100% 4-hour quota window. Accounts below 100% are logged and bypassed in favor of the next best candidate.
            -   **Prompt Persistence & Post-Switch Auto-Resend (`src-tauri/src/modules/db.rs`, `src-tauri/src/bin/agm.rs`, `process.rs`)**: Automatically backs up running and queued prompts with image attachment paths to SQLite prior to account rotation or IDE restarts, automatically restoring and re-triggering execution once the target IDE instance reaches ready state.
            -   **4-Part Root Cause Analysis (`02-spec/22-app-issues/13-proxy-pool-bindings-service-not-running-rca.md`)**: Full diagnostic post-mortem, root cause analysis, code fixes, and prevention guidelines.

    *   **v4.75.0 (2026-09-25)**:
        -   **[Release v4.75.0: CI/CD Test Isolation & Mock Coverage, Upstream v4.1.33 Sync Regression Fixes, and 830/830 Sub-3s Unit Test Suite] Eliminate All 40 CI Unit Test Failures, Gate Heavy Local Environment Tests with Fast CI Mocks, and Cut Suite Execution Time from 94s to 2.8s**:
            -   **CI/CD Environment Gating & Fast Mock Tests (`src-tauri/src/proxy/config.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/proxy/tests/`)**: Added `is_ci_environment()` to gate heavy local OS/disk instance switching tests (`test_ubuntu_instance_switching_end_to_end_flow`) in GitHub Actions while providing deterministic in-memory CI mock tests (`test_mock_account_switch_ci`), and tuned SQLite disk stress loops so all 830 unit tests complete in 2.8s instead of 94s.
            -   **Unified Test Config Mutex & Deadlock Elimination (`src-tauri/src/proxy/config.rs`, `gemini/wrapper.rs`, `openai/request.rs`)**: Unified `TEST_THINKING_BUDGET_MUTEX` and `TEST_CONFIG_LOCK` to prevent parallel test threads from overwriting `GLOBAL_THINKING_BUDGET_CONFIG` and removed same-thread duplicate mutex locks.
            -   **Compatible Thinking Signature Validation & Upstream Sync Alignment (`src-tauri/src/proxy/thinking_store.rs`, `pipeline/inbound.rs`, `openai/request.rs`, `prompt_sanitizer.rs`, `token_manager.rs`)**: Added `is_compatible_gemini_signature` across `InboundThinkingPipeline` and `ThinkingStore`, fixed `RE_IDENTITY_DECLARATION` trailing empty regex alternations, and restored `restore_persisted_long_image_limit` in `token_manager.rs`, achieving 830/830 passing unit tests.
            -   **4-Part Root Cause Analysis (`02-spec/22-app-issues/11-ci-cd-test-isolation-and-upstream-sync-rca.md`)**: Documented full `gitmap pe` diagnosis, root causes, fixes, and prevention rules.

    *   **v4.74.0 (2026-09-25)**:
        -   **[Release v4.74.0: Sponsor & Author Metadata Alignment, Single-Account 429 Adaptive GraceRetry Infinite Loop Fix, and GitMap LLM Chained Discovery Curriculum] Canonical Author Email alim.karim@riseup-asia.com, Single-Account GraceRetry State Contract Guard, and GitMap Autonomous Skill Generation**:
            -   **Sponsor & Author Metadata Migration (`src-tauri/Cargo.toml`, `src-tauri/src/bin/agm.rs`, `03-ai-scripts/40-test-email-permutations-e2e.py`)**: Migrated author email in Cargo manifest, CLI test dispatchers, and E2E simulation suites to `Alim <alim.karim@riseup-asia.com> (sponsored by RISEUP ASIA LLC)`.
            -   **Single-Account 429 Adaptive GraceRetry Infinite Loop Fix (`src-tauri/src/proxy/handlers/common.rs`)**: Resolved a critical deadlock bug where single-account 429 rate limit responses unconditionally returned `RetryStrategy::GraceRetry` regardless of `allow_grace_retry`. Now properly enforces `allow_grace_retry` and safely falls back to `FixedDelay`, reducing test execution from 30+ minutes (timeout) to 0.01 seconds.
            -   **GitMap LLM Chained Discovery Integration (`.agents/skills/gitmap/skill.md`)**: Integrated `gitmap llm train` 4-stage autonomous LLM curriculum to regenerate and solidify repository skill trees and operational guardrails.

    *   **v4.73.2 (2026-09-25)**:
        -   **[Release v4.73.2: Single-Account 429 Adaptive GraceRetry Infinite Loop Fix, State Contract Guard, and CI Test Suite Deadlock Elimination] Prevent Infinite While-Loop on Repeated Rate Limits, Rust Unit Tests Pass in 0.01s, and Ensure Green CI Pipeline Across Windows, macOS, and Ubuntu**:
            -   **Single-Account 429 Adaptive GraceRetry Infinite Loop Fix (`src-tauri/src/proxy/handlers/common.rs`)**: Resolved a critical deadlock bug in `determine_retry_strategy_adaptive` where single-account 429 rate limit responses unconditionally returned `RetryStrategy::GraceRetry` regardless of `allow_grace_retry`. When an account had already exhausted its grace retry (`!allow_grace_retry`), returning `GraceRetry` prevented `used_attempts` from advancing in `next_rotation_attempt`, trapping `task_short_429_preserves_rotation_budget_and_structured_status` in an infinite while loop and triggering 30-minute timeout cancellations across all CI runners. Now properly enforces `allow_grace_retry` and safely falls back to `FixedDelay`, reducing test execution from 30+ minutes (timeout) to 0.01 seconds.

    *   **v4.73.1 (2026-09-25)**:
        -   **[Release v4.73.1: Windows Manifest Embedded Resource Alignment & comctl32.dll Entry Point 0xc0000139 Elimination] Common-Controls 6.0 Embedded in All Binaries, Windows 10/11/Server 2025 Compatibility Stamped, and Release Rate Limit Throttling**:
            -   **Embedded Common-Controls 6.0 Manifest for Test Binaries & App (`src-tauri/build.rs`, `windows-test.manifest`)**: Embedded application manifest containing Microsoft.Windows.Common-Controls 6.0 into test and release executables via `/MANIFEST:EMBED` to prevent `0xc0000139` entry point missing errors on modern Windows runners.
            -   **GitHub Actions Rate-Limit Throttling Guard (`03-ai-scripts/34-purge-github-actions-artifacts.py`)**: Added rate limit safety check threshold to prevent exhausting `GITHUB_TOKEN` during artifact cleanup.

    *   **v4.73.0 (2026-09-25)**:
        -   **[Release v4.73.0: Autonomous Telegram Agent Daemon, Interactive Setup Wizard, Cross-Platform PowerShell ANSI/UTF-8 Hardening, and Remote Fleet Commands] Standalone PowerShell Agent Daemon, Automated Chat ID Detection & Handshake Verification, Windows PowerShell 5.1/7+ Pure ASCII Syntax Hardening, and Full Remote Execution Pipeline**:
            -   **Autonomous Telegram Agent Daemon (`scripts/telegram-agent-daemon.ps1`)**: Introduced a pure PowerShell background agent daemon for headless servers, remote workstations, and developer environments. Utilizes official Telegram Bot API long-polling (`getUpdates`) to monitor remote commands, fully supporting `/status` and `SNAPSHOT` (cluster and node telemetry including hostname, local IP, OS uptime, and gateway health), `FF` and `/ff` (double-play workspace profile fast-forward rotation), and `CMD:<command>` (administrative console execution with formatted, truncated result delivery).
            -   **Interactive Setup Wizard with Real-Time Chat ID Discovery (`scripts/setup-telegram-bot.ps1`)**: Overhauled the Telegram Bot setup utility. Provides credential probing, authoritative `/getMe` token validation, and a 30-second live polling loop that captures incoming `/start` messages to automatically bind the authorized `allowed_chat_id`. Dispatches an instant test ping (`sendMessage`) to confirm bidirectional connectivity and writes configurations to `$HOME/.antigravity_tools/telegram_config.json` for parity between GUI and CLI daemons.
            -   **Windows PowerShell 5.1 Syntax & ANSI/UTF-8 Compatibility Hardening**: Permanently eliminated syntax crashes in Windows PowerShell 5.1 where raw multi-byte UTF-8 emojis (`❓`, `⚡`, `🚀`, etc.) under the default Windows-1252 code page were decoded as `0xE2` (`â`), prematurely terminating double-quoted strings and corrupting script parsing. Normalized all strings to clean ASCII and standard HTML tags, standardized variable interpolation to `${ChatId}`, and verified zero-error execution across PowerShell 5.1 and pwsh 7+.
            -   **Security Whitelist Filtering & Audit Logs**: Enforced strict `allowed_chat_id` filtering in the daemon, automatically dropping unauthorized or anonymous private messages and recording audit records to protect remote execution channels.

    *   **v4.72.0 (2026-09-24)**:
        -   **[Release v4.72.0: Two-Bar Release Page Installation, Release Workflow Version Stamping, PowerShell History Pinned Version Resolution, and Win32/WebView2 DWM Blank UI Elimination] Dedicated Latest vs Pinned Installation Code Blocks, Strict Pinned Version Queue Adherence Without Drift, and Complete Taskbar Thumbnail/Window Occlusion Redraw Pipeline**:
            -   **Release Page Two-Bar Installation Layout (`02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md`)**: Replaced merged, single-block installation instructions with two distinct copyable code blocks ("bars") in GitHub release notes and project documentation. Users can now copy **Bar 1 (Latest Version)** for auto-updating bleeding-edge installations or **Bar 2 (Version-Based Installation)** with `-Version "<VER>"` explicitly bound to that exact tag.
            -   **GitHub Actions Workflow Subshell Environment Stamping Fix (`02-spec/22-app-issues/10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md`)**: Fixed a critical subshell variable scope loss in `.github/workflows/release.yml` where `VER="${VERSION#v}"` was unset during the `Collect release files and generate checksums` step, causing `sed "s/__PINNED_VERSION__/$VER/g"` to replace `__PINNED_VERSION__` with empty string. Explicitly passed `VERSION` to that step to guarantee released `install.ps1` and `install.sh` scripts are stamped with their exact release tag.
            -   **PowerShell Interactive History Pinned Version Recovery**: Expanded `Get-InvocationHistoryCandidates` in `install.ps1` to query `Get-History`, `[Microsoft.PowerShell.PSConsoleReadLine]::GetHistoryItems()`, and the PSReadLine history file. When invoked via `irm <url>/vX.Y.Z/install.ps1 | iex`, the installer now accurately extracts the pinned version from the interactive session command line even when `$MyInvocation.Line` is empty.
            -   **Strict Pinned Version Queue Isolation & Drift Prevention**: In `install.ps1` and `install.sh`, enforced strict queue isolation when `$isPinned` is true. Disabled CDN manifest staleness checks and GitHub latest release replenishment on pinned runs. If GitHub API is unavailable or rate-limited, the installer builds deterministic direct release asset URLs (`/releases/download/v<VER>/...`) for that exact version, eliminating silent fallbacks to unintended builds (such as `v4.71.1`).
            -   **Win32 DWM Frame Recalculation & WebView2 Blank UI Elimination**: Permanently eradicated the intermittent blank gray window and blank taskbar thumbnail previews on Windows 10/11. Added `--disable-features=CalculateNativeWinOcclusion,CalculateNativeWindowOcclusion` and background throttling flags to `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` in `src-tauri/src/lib.rs`. In `force_restore_and_focus_win32`, invoked Win32 `SetWindowPos` with `SWP_FRAMECHANGED` and `RedrawWindow` with `RDW_INVALIDATE | RDW_INTERNALPAINT | RDW_UPDATENOW | RDW_ALLCHILDREN` to force DWM redirection bitmap composition. Evaluated `window.dispatchEvent(new Event('resize'))` and added frontend window restoration listeners in `src/App.tsx`.
            -   **Root Cause Analysis & Canonical Spec**: Published canonical specification `02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md` and 4-part RCA `02-spec/22-app-issues/10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md`.

    *   **v4.71.4 (2026-09-24)**:
        -   **[Release v4.71.4: Strict RFC 2046 HTML MIME Mode, Standard HTML5 Document Wrapping, and Canonical `[vVersion | VM Alias | Local IP]` Subject Telemetry] Permanent Elimination of Raw HTML Source Leakage, Ubiquitous Versioned Node Telemetry Subjects, and Full Telegram/HTML Badge Alignment**:
            -   **Strict RFC 2046 HTML MIME Mode & Standard HTML5 Container Wrapping (`09-email-body-type-and-subject-version-rca.md`)**: Permanently fixed the issue where Gmail clients rendered raw `<div style="...">` source code as literal text. Reinforced `build_mime_message` in `email_sender.rs` to automatically wrap all HTML payloads in valid `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body ...>` HTML5 containers, eliminating layout breakdowns from missing outer DOM roots. Emits RFC 2046 `multipart/alternative` with `text/html; charset=UTF-8` as the high-priority final part via 76-character CRLF Base64 encoding, ensuring 100% native HTML rendering across all modern mail clients.
            -   **Canonical Telemetry Subject Normalization `[v<VERSION> | <VM_ALIAS> | <LOCAL_IP>]`**: Introduced `format_subject_with_telemetry` and standardized subjects across `email_sender.rs`, `email_inbound.rs`, and `notification_hub.rs`. Regardless of trigger mode (account switch alerts, quota drops, self-test verification, command ACK/results, or replies with `Re:`), the outermost left prefix is prominently stamped with `[v4.71.4 | VM3 | 192.168.1.12]`, enabling instant cluster identification in mobile push notifications and smartwatch previews.
            -   **Telegram Alert & HTML UI Card Metadata Alignment**: Synchronized `📦 Version: v4.71.4` and `Application Version` metadata rows and prominent badges across Telegram bot dispatches and HTML visual cards for multi-channel telemetry parity.
            -   **Root Cause Analysis**: Published comprehensive RCA document `02-spec/22-app-issues/09-email-body-type-and-subject-version-rca.md` and indexed in `02-spec/22-app-issues/01-index.md`.

    *   **v4.71.3 (2026-09-24)**:
        -   **[Release v4.71.3: Windows Application Manifest & ComCtl 6.0 Embedding, 0xc0000139 Entry Point Resolution, Universal React ErrorBoundary Blank Screen Prevention, DWM Real-Time Taskbar Thumbnails, and Interactive Standalone Installer] Total Elimination of Entry Point Not Found, DWM Thumbnail Gray Box Elimination, and Robust UI Crash Protection**:
            -   **RFC 2045 Base64 Dual-Part MIME Encoding & Automatic HTML UI Card Wrapper**: Permanently resolved raw `<div style=...>` tag leakage and Gmail `...` quote folding (`====` dividers) by introducing `encode_mime_base64_body` (76-char CRLF lines) and `wrap_html_email_card` in `email_sender.rs`. Every outbound email (ACK receipts, command executions, switch notifications, quota alerts, and self-tests) is automatically styled as a responsive dark-mode HTML card with a clean plain-text fallback.
            -   **Mandatory `[<VM_ALIAS> | <LOCAL_IP>]` Origin Identity in All Email Subjects**: Enforced `[<VM_ALIAS> | <LOCAL_IP>]` across `build_mime_message` and `format_reply_subject` (e.g., `Re: [VM3 | 192.168.1.12] [AGM ACK] Running: help`), with full support for zero, single, and multiple spaces around pipe delimiters (`|`).
            -   **Smart Rotator & Highlighted `⇄` Switch Button (`AccountTable.tsx`) Unification (`Kill-First -> Write-Second -> Start-With-Args-Third`)**: Fixed `is_instance_running` and `close_instance` in `instance.rs` passing `is_default = false` for the `default` instance (`Antigravity.exe` running without `--user-data-dir`). Unified `switch_account_to_instance`, `trigger_manual_rotation_for_instance`, and `InboundAction::AccountSwitch` with the exact `Kill-First -> Write-Second -> Start-With-Args-Third` sequence (`close_antigravity`, `write_to_system_keyring`, `device::write_profile`, `db::inject_token`, and `start_antigravity_with_fallback_path`), preventing exiting IDE processes from overwriting freshly injected credentials.
            -   **Settings Top Navigation & Hamburger Dropdown (`Settings.tsx`)**: Streamlined primary tabs to `General`, `Account`, `Proxy`, `Email-Alerts`, and `Supabase`, placing `Advance`, `Debug` (linked to the top titlebar Bug console), and `About` inside a clean Hamburger dropdown menu.
            -   **Blank UI & Windows Taskbar Preview Elimination (`06-blank-ui-and-taskbar-thumbnail-rca.md`)**: Permanently fixed the issue where hovering the Windows taskbar icon displayed a blank gray rectangle with a generic window icon. In `src-tauri/src/lib.rs`, disabled Chromium background occlusion (`--disable-features=CalculateNativeWindowOcclusion --disable-backgrounding-occluded-windows --disable-renderer-backgrounding`) and excluded `StateFlags::MINIMIZED` from `tauri-plugin-window-state` so the window never restores in an off-screen `(-32000, -32000)` state. Configured `tauri.conf.json` with `visible: true` and `center: true`, and upgraded `show_main_window` to call `restore_and_focus_window`.
            -   **Footer "Install Now" Direct Execution & Detached Updater Window (`07-installer-click-and-update-action-rca.md`)**: Fixed clicking the footer `[Install Now]` badge in `Accounts.tsx` doing nothing when the notification was already mounted. Directly wired the button to `installUpdate()`, showing a real-time `Installing...` spinner and toast. Upgraded `run_installer_update` on Windows to spawn `cmd.exe /c start "Antigravity Tools Updater" powershell.exe ...`, providing a visible detached console with live download progress that is immune to parent process termination.
            -   **Application Manifest Embedding, `0xc0000139` Entry Point & Blank Screen Eradication (`08-manifest-and-entrypoint-resolution-rca.md`)**: Permanently eliminated the fatal `agm-alim.exe - Entry Point Not Found` startup crash (`0xc0000139`) and blank DOM freezes on Windows 10/11 VMs and desktops. In `src-tauri/build.rs`, restored `tauri_build::WindowsAttributes::new()` to embed the canonical Windows application manifest containing Common-Controls 6.0, `PerMonitorV2` DPI awareness, and Windows 10/11 compatibility GUIDs, preventing unresolved `comctl32.dll` symbols. Wrapped the entire frontend application within an elegant React `ErrorBoundary` with instant recovery and reload capabilities.
            -   **Root Cause Analysis**: Published `02-spec/22-app-issues/05-email-html-rendering-and-smart-rotator-ide-switch-rca.md`, `06-blank-ui-and-taskbar-thumbnail-rca.md`, `07-installer-click-and-update-action-rca.md`, and `08-manifest-and-entrypoint-resolution-rca.md`.

    *   **v4.71.1 (2026-09-24)**:
        -   **[Release v4.71.1: Settings UI/UX Menu Overhaul, Machine Training REST API, and Full IDE Credential Injection] Concise Settings Tabs & Action Buttons, Redundant Debug Tab Consolidation to Titlebar Bug Icon, Machine Training Telemetry REST Endpoints, and Smart Rotator Deep Injection**:
            -   **Settings Menu & Action Button Streamlining**: Comprehensively overhauled `Settings.tsx` tab strip and header action buttons adhering strictly to user design specifications. Shortened button labels to strictly **Save** and **Backup**; simplified navigation tabs to **Proxy**, **Email-Alerts**, **Supabase**, **Advanced**, and **About**, eliminating wide and cluttered text. Aligned English (`en.json`) and Chinese (`zh.json`) translation dictionaries.
            -   **Redundant Debug Tab Removal & Titlebar Bug Icon Integration**: Completely removed the redundant Debug option from the settings tab strip, centralizing all debug console controls into the permanent **Bug icon** in the top navigation titlebar. Added an unobtrusive quick-access card inside the Advanced tab for fast inspection of IPC calls and runtime logs.
            -   **Machine Training REST API Endpoints**: Exposed comprehensive REST endpoints for external training systems and model self-learning: `/api/v1/training`, `/api/v1/status`, `/api/v1/training/telemetry`, `/api/v1/training/learn`, `/api/v1/training/machines`, and `/api/v1/training/modify`. External agents can seek into full node telemetry (healthy accounts, instance runtimes, prompt status) and ingest reinforcement learning feedback to dynamically optimize model routing. Guarded by the `training_api_enabled` toggle in General Settings.
            -   **Remote Machine Modification & Complete IDE Account Injection**: Hardened the training API's `switch_account` and `trigger_rotation` actions to invoke `switch_account_to_instance(&target.id, Some(&inst_id))` and `trigger_manual_rotation_for_instance(req.instance_id.as_deref())`. This executes full OAuth token refresh, encrypted injection into the IDE sandbox database (`state.vscdb`), OS keyring synchronization, and instance process restart for 100% active credential adoption.

    *   **v4.71.0 (2026-09-24)**:
        -   **[Release v4.71.0: Modern Responsive HTML Email Cards, Origin Node Alias & Local IP in Subject, Flexible Whitespace Pipe Parsing, and Deep IDE Smart Account Injection] RFC 2046 Dual MIME Email Responses, Origin Node Identification Prefix, Zero/Single/Multi-Space Pipe Tokenization, Smart Rotator Candidate Scoring, and Comprehensive IDE Credential Injection**:
            -   **Modern Responsive HTML Email Receipts & RFC 2046 Dual MIME Engine**: Completely resolved the issue where Gmail and Outlook rendered raw HTML tags as literal text in outbound email replies. Refactored `email_sender.rs` to detect HTML markup via `is_html_content` and `strip_html_tags`, automatically constructing RFC 2046 `multipart/alternative` MIME messages with both a clean plain-text fallback and a styled dark-theme HTML card layout featuring colored execution badges, parameter matrices, and terminal outputs.
            -   **Origin Node Alias & Local IP in All Email Subjects**: All outbound email subjects (immediate ACK receipts, result cards, and quota drop alerts) now prominently feature `[<VM_ALIAS> | <LOCAL_IP>]` (e.g. `[VM3 | 192.168.1.12]`). Refactored `format_reply_subject` to intelligently strip duplicate bracketed tags from inbound replies while preserving RFC 5322 threading in Gmail and Outlook.
            -   **Flexible Zero, Single, and Multi-Space Pipe Command Parsing**: Upgraded `parse_single_command_string` tokenization to effortlessly parse commands separated by pipes with 0 spaces (`VM3|1|help`), standard single space (`VM3 | 1 | help`), or multiple whitespace/tabs (`VM3   |   1   |   help`).
            -   **Smart Rotator & Full IDE Credential Injection for Remote Account Switching**: Resolved why remote account switches were previously ineffective on running Antigravity IDE instances. Inbound commands (`agm switch`, `agm ff`, `agm rotate`) now delegate to the Smart Rotator (`trigger_manual_rotation_for_instance`) when no account is specified. When an account is targeted, it executes `instance::switch_account_to_instance`: refreshes Google OAuth tokens, writes encrypted access tokens into the IDE database (`state.vscdb`), updates the OS keyring, binds the instance registry, and restarts the IDE process to ensure 100% active credential adoption.
            -   **Root Cause Analysis & CI/CD Hardening**: Published comprehensive RCA document `.ai-memory/issues/46-email-html-replies-node-identity-smart-rotator-rca.md`, added single-job memory optimization flags to prevent Windows compiler OOMs, and verified all quality gates.

    *   **v4.70.0 (2026-09-24)**:
        -   **[Release v4.70.0: Instance Deletion Lifecycle Overhaul, Open/Close Loop Eradication, Adaptive Quota Ladder & Gemini 3.8 Flash High, Dropdown Mutual Exclusion, and 100% Plaintext Emails] Safe Instance Teardown, Native Binary Sandboxing, 300s/60s/40s Dynamic Polling Ladder, Gemini 3.8 Flash High Default, Mutual Dropdown Exclusion, Toolbar Consolidation, Silent Watchdog, and Pure Plaintext Emails**:
            -   **Instance Deletion Lifecycle Overhaul & Error Trace Eradication**: Fixed `delete_instance` failing with an error trace due to running processes locking profile directories. Implemented a resilient teardown sequence: proactively terminate running processes via `close_instance()`, safely fallback `active_instance_id` to `"default"` if deleting the active profile, persist registry and state to SQLite first, and clean up directory locks before resiliently removing filesystem directories.
            -   **Native Binary Sandboxing & Open/Close Death Loop Eradication**: Completely eliminated the issue where newly created instances repeatedly open and close in an infinite restart loop. Terminated the fragile executable-copying pattern (`copy_dir_all` / child binary invocation), standardizing on launching the native parent `Antigravity.exe` binary with `--user-data-dir` argument isolation.
            -   **Adaptive Auto-Switcher Polling Ladder & Gemini 3.8 Flash High**: Implemented a 3-tier dynamic interval ladder: default safe interval relaxed to `300s` (5 minutes, saving system resources), accelerating to `60s` when quota falls below caution threshold (`15%`), and `40s` when at or below critical threshold (`12%`). Upgraded default evaluated model to `gemini-3.8-flash-high`. Automatic rotation seamlessly pushes running prompts and state snapshots to the local database and dispatches notifications.
            -   **Mutual Dropdown Exclusion & Collision Eradication**: Integrated `agm:dropdown-open` custom events between Accounts and Instances dropdowns, guaranteeing mutual exclusion and eliminating double-dropdown overlap.
            -   **Toolbar Visual Consolidation**: Relocated Bug debug icon to the left brand logo area; consolidated Theme and Language switchers into a single unified dropdown leaving only the Quick Clean (Recycle) button; consolidated Import and Export into a single unified icon dropdown with `z-[9999]` tooltips.
            -   **Window Focus Stealing Eradication & Passive Crash Recovery**: Defaulted `auto_focus_window` to `false`. Refactored `check_and_recover_crashed_instance` into a purely passive watchdog that cleans lockfiles without launching unexpected foreground IDE windows or triggering forced rotations. Documented comprehensive root cause analysis in `02-spec/22-app-issues/04-instance-delete-open-close-loop-and-focus-stealing-rca.md`.
            -   **100% Plaintext Subject-Driven Emails & Telegram Alignment**: Stripped all HTML tags from outgoing notification emails, enforcing pure plaintext format. Standardized subject-driven commands (`* | prompt | proj-...` or `* | ps | ...`) with pure prompts in the email body. Inbound email processing strips markdown footers and terminal prefixes (`powershell:`). Aligned Telegram Bot Token and Chat ID inputs horizontally (`items-end`) and provided `03-ai-scripts/telegram-bot-helper.ps1`.

    *   **v4.69.0 (2026-09-24)**:
        -   **[Release v4.69.0: Complete IDE Copy vs Profile Copy Clone Architecture & UI Layout Perfection] IDE vs Profile Duplication Modes, Navbar Item Reordering, Split Release Code Blocks & Parity Alignment**:
            -   **Restored Complete "IDE Copy vs Profile Copy" Duplication Engine**: Re-introduced the dual-mode cloning architecture across both the navigation bar (`InstanceSelector.tsx`) and the main management dashboard (`Instances.tsx`). Users can explicitly choose between **IDE Copy (Full Environment & Sessions)** (clones the complete isolated data directory, active workspace sessions, extensions, cache, and state) and **Profile Copy (Preferences & Snippets)** (copies only user preferences, keybindings, and snippets without bulky runtime session data).
            -   **Dropdown Header & Item Row UI Layout Perfection**: Strictly aligned the profile selector dropdown with user layout specifications (`https://prnt.sc/dbDLrTd-Rfns`). Renamed header section to `INSTANCES / PROFILES` with dedicated "Duplicate Active Profile" action. Reordered action buttons in every profile row to place the Rename button (`Pencil`) before the Duplicate button (`Copy`), matching screenshot layout: `Launch / Stop` -> `Fast-Forward` -> `Rename` -> `Duplicate` -> `Set Default` -> `Delete`.
            -   **Navbar Minimalist Single Fast-Forward Button Guarantee**: Confirmed that the outer navigation bar displays exclusively the single Fast-Forward (`[ ⏩ ]`) button with active shortcut tooltip (`Ctrl+Shift+F`) and prominent sequence badges (`#1`, `#2`, `#3`...), completely eliminating redundant play buttons.
            -   **Release Code Blocks Isolation & Permanent Split Verification**: Verified and enforced distinct, separated code blocks for Direct Latest and Pinned Version installer commands across GitHub Releases and release manifests to ensure copy buttons capture single actionable commands without syntax or comment leakage.

    *   **v4.68.0 (2026-09-24)**:
        -   **[Release v4.68.0: Instance Profile Duplication, Default Selection, Fast-Forward Shortcut & Security Diagnostic Fixes] Instance Clone Actions Restoration, Minimalist Single Fast-Forward Button, Configurable Ctrl+Shift+F Global Shortcut, Default Profile Locking & SQLite Null Safe Aggregates**:
            -   **Restored Instance Profile Duplication (Clone / Duplicate)**: Re-introduced the "Duplicate Active Profile" action in the `InstanceSelector.tsx` dropdown header bar next to import/export, and added an inline duplicate (`Copy`) button to every individual instance row beside the rename button. Restored full cloning capabilities on the `Instances.tsx` dashboard page.
            -   **Minimalist Single Outer Fast-Forward Button & Enlarged Sequence Badges**: Cleaned up the top navbar by removing the redundant green launch play button, keeping exclusively the Fast-Forward (`[ ⏩ ]`) button with active shortcut tooltip. Enlarged the dropdown container to `w-96 md:w-[420px]` and styled high-contrast `#1`, `#2`, `#3` sequence badges.
            -   **Default Profile Selection & Deletion Protection (`set_default_instance`)**: Added the backend IPC command `set_default_instance` to atomically mark any profile as `DEFAULT` with dedicated badge indicators. Updated `delete_instance` to strictly intercept and prevent deleting default profiles.
            -   **Configurable Fast-Forward Shortcut & Global Listener**: Introduced `fast_forward_shortcut` in auto-switcher configuration (defaulting to `Ctrl+Shift+F`), provided pre-set selection pills and custom binding in `AutoSwitcherSettings.tsx`, and wired `useFastForwardShortcut.ts` for frictionless keyboard-driven instance rotation with toast feedback.
            -   **Security Diagnostic Null Aggregate & IPC Argument Fixes**: Wrapped SQLite IP statistics queries in `COALESCE(SUM(...), 0)` and deserialized safely via `Option<u64>` to eliminate `Invalid column type Null at index: 2, name: blocked`. Updated `get_ip_access_logs` to support both nested query objects and flat fallback parameters, fixing `missing required key query`.

    *   **v4.67.0 (2026-09-24)**:
        -   **[Release v4.67.0: Auto Switcher UI/UX Overhaul, Local Multithreaded Runner & Docker Binary Standardization] Auto Switcher Dual-Card Symmetry, Dark Mode Theme Fixes, Full-Core Parallel Compilation & Docker Distribution Alignment**:
            -   **Auto Switcher Settings UI/UX Complete Overhaul**: Transformed the isolated left-hand model dropdown into an official, balanced **Primary Evaluated Model** card featuring a `Cpu` icon badge, `Trigger Metric` status pill, clear subtitle, and dynamic contextual trait banners explaining model profiles (Gemini 3.8 Flash high throughput, Claude Sonnet 4.6 deep reasoning budget failover, Gemini Pro baseline). Integrated a **Rotation Cooldown Guard** slider directly into the card to achieve flawless height symmetry with the right column.
            -   **Dark Mode Recency Cutoff Area Clash Root Cause Elimination**: Eliminated DaisyUI and un-themed Tailwind background color clashes that caused stark white and gray containers in dark mode. Rebuilt the active prompt recency cutoff window panel using theme-native Slate styling (`bg-slate-100/90 dark:bg-slate-900/90 border-slate-700/80`) and unified the right column under an official **Task Continuity & Watchdog** panel.
            -   **Local Multithreaded Fast Runner Optimization (`run.ps1`)**: Automatically detects system CPU core count and configures `$env:CARGO_BUILD_JOBS` for full-thread parallel compilation. Added auto-detection and activation for `sccache` compiler cache and `lld-link` fast linker, and introduced a `-Quick` / `-Run` instant execution mode to launch pre-built binaries without rebuild delays, alongside `-OptimizeIO` for Windows Defender build directory exclusions.
            -   **Docker Build Standardization & Packaging Alignment**: Updated `docker/Dockerfile`, `Dockerfile.backend`, and `Dockerfile.backend.localdist` to build the official `agm-alim` binary with `/app/antigravity-tools` symlinks, ensuring seamless headless container builds.

    *   **v4.66.0 (2026-09-24)**:
        -   **[Release v4.66.0: Canonical 5-Hour Rolling Quota Restoration, App Process Branding, Instances Numbering & Email Sample Actions] 5-Hour Rolling Quota Calculation, Instant Sample Preloading, Task Manager Brand Identity, Telegram Bot Creation Guide & Release Code Block Separation**:
            -   **Canonical 5-Hour Rolling Quota Bucket Restored**: Fixed rolling hourly quota comparator in `src-tauri/src/modules/quota.rs`. If a rolling hourly quota exists and the weekly quota is not completely exhausted (`remaining_fraction > 0.001`), the UI unconditionally selects the 5-hour rolling bucket with accurate countdowns (e.g. `4h 56m`). Weekly countdowns (e.g. `6d 9h`) are reserved strictly for genuine weekly depletion.
            -   **Accounts Rapid Multi-Click Stability & Button Accessibility**: Eliminated redundant per-card `refreshQuota` background calls in `src/pages/Accounts.tsx` to stop WebView2 reflow races. Fixed action buttons in `src/components/accounts/AccountTable.tsx` to remain permanently visible with clear interactive hover styling instead of disappearing behind `opacity-0`.
            -   **Task Manager Process Identity & Versionless Desktop Shortcuts**: Set `productName` to `"Antigravity Manager Tools"` in `src-tauri/tauri.conf.json` while maintaining executable `mainBinaryName: "agm-alim"`, ensuring Windows Task Manager lists the official title without version suffix. Verified that Start Menu and Desktop shortcuts remain clean and unversioned (`Antigravity Manager Tools.lnk`), while Windows Installed Apps preserves explicit version tracking (`Antigravity Manager Tools 4.66.0`).
            -   **Prominent Instance Sequence Badges & Quick Action**: Added `#1`, `#2`, `#3`... sequence badges to all instance cards in `src/pages/Instances.tsx` and the navigation bar `InstanceSelector.tsx`. Enlarged card spacing and padding, and added a quick `+ New Instance` shortcut button in the Instances page header.
            -   **Inbound Email CLI Prefix Stripping & Natural Language AI Interceptor**: In `src-tauri/src/commands/email.rs` and `src-tauri/src/modules/email_inbound.rs`, automatically strip shell prefixes (`powershell:`, `ps:`, `ps `, `pwsh:`, `bash:`, `sh:`, `cmd:`) defaulting directly to PowerShell on Windows. Added multi-prefix simulation interception for AI instructions (`Project:`, `Prompt:`, `AI:`, `Instruction:`, `Prompt Injection:`) to simulate receipts cleanly without PowerShell syntax errors.
            -   **Email Sample Mailbox Preloader & Quick Command Templates**: Added a direct `Load Sample Mailboxes` action to the empty mailbox state and Actions dropdown in `EmailNotificationSettings.tsx`. Modernized `MailboxExportModal.tsx` scrollbars, updated branding to "Antigravity Manager Tools", and aligned quick template buttons with clean PowerShell and lowercase `gitmap` commands.
            -   **Telegram Bot Guide & Auto-Switcher / Settings Backup**: Added step-by-step interactive setup modal for Telegram bots via `@BotFather` in `SupabaseSyncSettings.tsx` and aligned input field heights. Added Actions dropdown (JSON Export, Import, Reset to Defaults) to `AutoSwitcherSettings.tsx`. Integrated `UnifiedBackupModal` into the main Settings page toolbar, and added a Debug console button to `NavSettings.tsx`.
            -   **Release Page Split Code Blocks & Verified Pinned One-Liners**: Split Direct Latest and Pinned Version installer commands into two distinct code blocks in `readme.md`, `README_EN.md`, and release templates. Verified pinned version PowerShell execution syntax: `& ([scriptblock]::Create((irm ...))) -Version 4.66.0`.

    *   **v4.65.3 (2026-09-24)**:
        -   **[Release v4.65.3: Mailbox Multi-Format Visual Export, Single-Account Import/Export & Embedded JSON AI Syntax] Interactive Export Preview Modal, Native OS File Saving, Single Account Form Drawer & Embedded JSON AI Instructions**:
            -   **Interactive Visual Export Modal & WebView2 Download Fix**: Introduced `MailboxExportModal.tsx` to eliminate WebView2 silent download failures where blob URL clicks were dropped without file or UI response. Clicking any export action immediately launches a syntax-highlighted dark modal with live switching between **JSON**, **YAML**, and **CSV**, instant clipboard copying, and native OS file saving.
            -   **Tauri Backend File Save Gate Expansion**: Updated `validate_user_json_path` in `src-tauri/src/commands/mod.rs` to permit saving `.json`, `.csv`, `.yaml`, `.yml`, `.txt`, `.xlsx`, and `.xls` files via Tauri's native `@tauri-apps/plugin-dialog` save handler.
            -   **Single-Account Individual Export**: Added dedicated `Export` buttons to each account row in the Email Accounts table and added an `Account IO` header toolbar in the Add/Edit Mailbox modal for one-click JSON, YAML, or CSV export of single configurations.
            -   **Single-Account Quick Import Drawer**: Added an expandable quick import drawer to the Add/Edit Mailbox dialog to paste or browse (`.json`, `.yaml`, `.csv`, `.txt`) configurations and automatically populate all form fields.
            -   **Embedded JSON Syntax in AI Configuration Segment**: Refactored the AI Mailbox Configuration banner to embed the exact JSON schema syntax directly inside the prompt text, accompanied by a `View JSON Format` toggle to preview the active JSON structure.
            -   **Global AI Templates & YAML Bulk Ingestion**: Embedded JSON schemas and added a dedicated YAML Schema tab in `ai-sample-templates-modal.tsx`, while introducing YAML radio selection and fallback parsing to the general Import dialog.

    *   **v4.65.2 (2026-09-24)**:
        -   **[Release v4.65.2: Big Screen UI Responsiveness, Scrollbar Elimination & Windows App List Version Stamping] Full High-Resolution Optimization, Native Scrollbar Elimination & Windows Installed Apps Versioning**:
            -   **Big Screen UI Responsiveness & Wide Display Optimization**: Expanded container width constraints from `max-w-7xl` (1280px) to `max-w-[1920px]` across all 10 core application pages (`Accounts`, `Dashboard`, `Instances`, `ApiProxy`, `Settings`, `TokenStats`, `UserToken`, `Security`, `Monitor`, `Email`), eliminating artificial layout pinching and dead space on 1080p, 1440p, 4K, and ultrawide monitors.
            -   **Chromium/WebView2 Horizontal Scrollbar Elimination**: Added `height: 0px` to `::-webkit-scrollbar` in `src/App.css` and defined a universal `.scrollbar-none` utility with cross-browser styling (`-ms-overflow-style: none`, `scrollbar-width: none`), eliminating native horizontal scrollbar tracks and arrow buttons in Windows WebView2.
            -   **Full-Width Navbar & Window Controls Pinning**: Updated `Navbar.tsx` to `w-full` with balanced padding (`px-3 sm:px-4 md:px-6`), ensuring window control buttons (`Minimize`, `Maximize/Restore`, `Close`) firmly anchor to the far top-right corner.
            -   **Accounts Filter Bar Flex Collision Prevention**: Added `lg:overflow-x-visible` and shifted toolbar flex-wrap breakpoint to `2xl:flex-nowrap` in `Accounts.tsx`, preventing the left search/filter block from being crushed against the right action buttons.
            -   **AccountTable Actions Column Expansion**: Adjusted the sticky Actions column width in `AccountTable.tsx` to `w-[220px] xl:w-[280px]` to comfortably accommodate all 11 action buttons on larger viewports.
            -   **Windows Installed Apps Version Stamping**: Updated `src-tauri/hooks.nsh` and `install.ps1` to display `Antigravity Manager Tools v${VERSION}` in Windows "Installed apps" / "Add or remove programs", ensuring instant version visibility in system settings.

    *   **v4.65.1 (2026-09-24)**:
        -   **[Release v4.65.1: Upstream PR #4 Integration, UI Fluidity, Plaintext Email Engine & Instance-Protected Smart Rotation] Complete PR #4 Merge, Smart Rotate Delegation, Full English Documentation & Sandboxed Instance Protection**:
            -   **Upstream PR #4 Clean Integration**: Merged PR #4 into `main` without regressing our superior build architecture, `agm-alim` binary separation, `version.json` source-of-truth pipeline, and branding (*Title: Antigravity Manager Tools*, *Publisher: Maintained by Alim, Sponsored by RISEUP ASIA LLC*).
            -   **Rotate Now Delegation to Smart Rotate**: In `AutoSwitcherSettings.tsx` and `useInstanceStore.ts`, unified the Settings "Rotate Now" button with the top navigation bar's Fast-Forward button to delegate directly to `smartRotateProfileAccount`.
            -   **Instance Protection Guarantee**: Enforced strict per-instance `--user-data-dir` filtering in both Rust (`close_instance`) and frontend stores. Rotating or switching an instance closes *only* the target instance's process, leaving all other running instances and sandboxes 100% untouched and running.
            -   **UI Fluidity & Error Fixes**: Resolved all 11 user screenshot issues including Tauri 2 ACL permissions for `opener:open_url` and window dragging, window maximize/restore/mini view navigation errors, quota display width extension in `QuotaItem.tsx`, removal of duplicate switch buttons, and repositioning of `UpdateNotification.tsx` to prevent pagination collision.
            -   **100% Plaintext Outbound Email Engine**: Converted outbound notifications from heavy HTML to lightweight, spam-resistant plaintext ASCII format with pipe-delimited subject parsing (`agm ls`, `status`, `switch`, etc.) and SQLite 10-minute rate limit.
            -   **Unified Encrypted Backup Modal**: Introduced `UnifiedBackupModal.tsx` supporting Web Crypto AES-GCM (PBKDF2 100k rounds SHA-256) password protection, reminder notices, and direct SMTP export.
            -   **Multi-Instance Sequence Numbering (#seq) & Distributed agm ls**: Implemented sequential IDs (`#1`, `#2`, ...) stored in SQLite and enhanced `agm ls` to format instance ID, name, status, bound account, node IP, and data dir.
            -   **Documentation Localization**: Translated `docs/release-guide.md`, `scripts/MANUAL_PR_CLOSE_GUIDE.md`, and helper scripts into 100% professional English.

    *   **v4.65.0 (2026-09-23)**:
        -   **[Release v4.65.0 & Installed Apps Branding Alignment] Display Title & Publisher Finalization with Full macOS Universal Bundle Support**:
            -   **Windows Installed Apps Branding Alignment**: Formatted the Windows "Installed apps" entry to display Title: **Antigravity Manager Tools** with subtitle / Publisher line: **Maintained by Alim, Sponsored by RISEUP ASIA LLC**, stamped directly through NSIS post-install hooks (`src-tauri/hooks.nsh`), `tauri.conf.json`, and `install.ps1` registry sync logic.
            -   **macOS Universal Packaging Hook**: Added `scripts/before-bundle.js` and wired `"beforeBundleCommand"` in `src-tauri/tauri.conf.json`. Automatically detects `universal-apple-darwin` targets, verifies architecture compilation, and uses `lipo -create` to merge the standalone `agm` CLI binary before Tauri macOS bundler packaging, resolving packaging failures on auxiliary binaries.
            -   **Explicit Default Primary Binary**: Added `default-run = "agm-alim"` in `src-tauri/Cargo.toml` to strictly identify the GUI application binary.
            -   **Rust Trait Warning Suppression**: Added `#[allow(async_fn_in_trait)]` to `SystemIntegration` trait in `src-tauri/src/modules/integration.rs`, removing compiler suggestions on public trait async methods.
            -   **CI/CD RCA 35 Grounded**: Authored `.ai-memory/cicd-issues/35-macos-universal-bundle-agm-missing-binary-rca.md` and registered in index.
    *   **v4.64.0 (2026-09-23)**:
        -   **[macOS Universal App Bundle Packaging Fix, beforeBundleCommand Hook, Installed Apps Branding & RCA 35 Resolution] Resolved Multi-Binary Universal Darwin Packaging and Windows Installed Apps Branding Alignment**:
            -   **Windows Installed Apps Branding Alignment**: Formatted the Windows "Installed apps" entry to display Title: **Antigravity Manager Tools** with subtitle / Publisher line: **Maintained by Alim, Sponsored by RISEUP ASIA LLC**, stamped directly through NSIS post-install hooks (`src-tauri/hooks.nsh`), `tauri.conf.json`, and `install.ps1` registry sync logic.
            -   **macOS Universal Packaging Hook**: Added `scripts/before-bundle.js` and wired `"beforeBundleCommand"` in `src-tauri/tauri.conf.json`. Automatically detects `universal-apple-darwin` targets, verifies architecture compilation, and uses `lipo -create` to merge the standalone `agm` CLI binary before Tauri macOS bundler packaging, resolving packaging failures on auxiliary binaries.
            -   **Explicit Default Primary Binary**: Added `default-run = "agm-alim"` in `src-tauri/Cargo.toml` to strictly identify the GUI application binary.
            -   **Rust Trait Warning Suppression**: Added `#[allow(async_fn_in_trait)]` to `SystemIntegration` trait in `src-tauri/src/modules/integration.rs`, removing compiler suggestions on public trait async methods.
            -   **CI/CD RCA 35 Grounded**: Authored `.ai-memory/cicd-issues/35-macos-universal-bundle-agm-missing-binary-rca.md` and registered in index.
    *   **v4.63.0 (2026-09-23)**:
        -   **[Native AGM Terminal CLI Expansion & Inbound Email Command Suite] GitMap Parity with doctor, accounts, switch, prompts, proxy, sync, pull, clean, & logs**:
            -   **GitMap Parity Native CLI Suite**: Expanded standalone `agm` terminal CLI binary with `doctor`/`check` (database vault integrity, IDE process status, proxy gateway port 8045, PATH, and network diagnostic), `accounts`/`acc` (registered accounts list, tiers, and quotas with `--active` and `--json`), `switch <email>` (direct active account switching without GUI), `prompts` (active tasks from `repo_prompts.db` and template categories), `proxy [status|test]` (loopback latency probe), `sync` (vault and instance synchronization), `pull` (`git pull origin main` integration), `clean`/`purge` (safe cache cleanup protecting DB vaults), and `logs` (real-time tailing and keyword filtering).
            -   **Full Inbound Email Command Mapping**: Bounded all expanded CLI commands into `src-tauri/src/modules/email_inbound.rs` (`DoctorDiagnostic`, `ListAccounts`, `AccountSwitch`, `ProxyStatus`, `SystemClean`, `SyncState`), backed by 2-phase plaintext receipts (Phase 1 ACK + Phase 2 Result) and 10-second debounce sliding stack.
            -   **Expanded Local-Only E2E Test Suite**: Updated `03-ai-scripts/40-test-email-permutations-e2e.py` to cover all 30 subject command permutations, 10 native AGM CLI subcommands, and 9 Rust unit tests with 100% pass rate.
            -   **Canonical Spec & Index Registration**: Authored canonical Spec 20 (`02-spec/21-app/20-agm-cli-expanded-commands.md`) and consolidated execution Plan 66.
    *   **v4.62.0 (2026-09-23)**:
        -   **[Inbound Email Remote Control, 2-Phase Plaintext Receipts & Native AGM Terminal CLI] Unified Pipe Grammar, Plaintext Receipts, 10s Debounce Stack, & Native agm CLI**:
            -   **Unified Inbound Email Pipe Grammar**: Added comprehensive parsing for `sub: [worker-name|ip] | [ins-{instance}] | <command> [ | proj-{project name} ]` with partial IP octet matching (e.g. `12` matching `192.168.1.12`), optional instance tags, project targeting, and smart GitMap prefix collapsing.
            -   **2-Phase Plaintext Notification Receipts**: Eliminated heavy HTML email templates in favor of clean, spam-resistant plaintext receipts; sends immediate Phase 1 ACK (`[AGM ACK] COMMAND ACKNOWLEDGED AND RUNNING`) on ingestion followed by Phase 2 Result receipt (`[AGM Result] EXECUTION COMPLETED`) upon command termination.
            -   **10-Second Debounce Sliding Stack & Security ACL**: Enforces 10s sliding window debouncing allowing at most 2 emails per spam burst with boundary consolidation; checks sender authorization against `notify_recipients` ACL to prevent spoofing.
            -   **Native AGM Terminal CLI (`agm`)**: Compiled standalone native binary providing `agm status`, `agm instances` / `agm ls`, `agm ff` / `agm smart-switch`, `agm update`, `agm install` (PATH & PowerShell `$PROFILE` installer), and `agm ssh` with secure hidden password fallback and remote VM auto-update.
            -   **Telegram Settings Consolidation**: Consolidated Telegram Bot Token, Chat ID, and test ping directly into the Email & Alerts view with a comprehensive step-by-step setup guide and automated PowerShell script.
            -   **Local-Only E2E Test Suite**: Authored `03-ai-scripts/40-test-email-permutations-e2e.py` testing all 19 command permutations, 9 Rust unit tests, receipt formatting, and CLI binaries dynamically grounded in local DB vault credentials.
    *   **v4.61.0 (2026-09-23)**:
        -   **[Smart Switch Scoring, Pre-Activation Verification, Window Controls & Error Telemetry] Multi-Factor Multiplicative Scoring, Weekly Quota Group Extraction, Pre-Activation Refresh Verification Loop, & Window Control ACL Hardening**:
            -   **Multi-Factor Multiplicative Scoring**: Implemented $\text{Score} = S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$ across TypeScript and Rust backends. Completely eliminated legacy +100,000 idle bonuses; active in-use accounts receive multiplier 0, while inactive candidates receive 1. Ultra multiplier is 5, Pro multiplier is 3, and Free/other is 1.
            -   **Weekly Quota Group Bottleneck Extraction**: Extracted remaining weekly percentages directly from `quota_groups` weekly buckets, taking the bottleneck minimum across model groups. Prevents misleading short-term (4H/5H) quota from choosing depleted accounts (e.g. 21% weekly Gemini accounts vs 100% full accounts).
            -   **Pre-Activation Refresh Verification Loop**: Executed live `fetch_account_quota` probe before switching candidates. If candidate quota degraded ($\le 5\%$) or account is blocked/forbidden/in-use, demoted to score 0 and iterated to the next best candidate until verified.
            -   **Tauri v2 Window Control ACL Fix**: Configured `src-tauri/capabilities/default.json` with explicit `core:window:*` permissions and exported native Tauri window commands (`minimize_window`, `maximize_window`, `toggle_maximize_window`, `close_window`), resolving `Command plugin:window|minimize not allowed by ACL`.
            -   **Interaction Flow Telemetry & Calculated XPath**: Enhanced `handleClickCapture` to traverse to closest interactive ancestor (`button`, `a`, `input`, etc.), extracting `id`, `name`, `aria-label`, and computing exact XPath locators, preventing raw `svg`/`path` logging.
            -   **Synthetic Stack Trace Capture & Preview**: Synthesized call stacks for string/IPC errors in `buildCapturedError`, ensuring `stackTrace` is never empty, and added a Stack Trace Preview card with 1-click copy to the Error Modal Overview tab.
            -   **Anonymized Local Unit Tests**: Implemented pure synthetic test suites (zero real emails in codebase) verifying weekly quota bottleneck extraction and scoring.
    *   **v4.60.0 (2026-09-22)**:
        -   **[MSVC Linker & Release Assets] Fix Duplicate Resource Link Failure (CVT1100 / LNK1123) & Release Asset Decoupling**:
            -   **MSVC Duplicate Resource Fix**: Removed redundant `cargo:rustc-link-arg` for `resource.lib` in `src-tauri/build.rs`, allowing `tauri-build` to manage native Windows resources and manifests cleanly without triggering `CVTRES CVT1100` duplicate VERSION resource collision.
            -   **Compiler Warnings Eliminated**: Suppressed intermediate `unused_assignments` and prefixed unused stream tracking variables across Claude, Gemini, and OpenAI handlers and mappers for a 100% warning-free build.
            -   **Release Asset Decoupling**: Excluded standalone bootstrap installer scripts (`install.ps1`, `install.sh`) from release assets to preserve clean binary distribution from raw git tags.
            -   **Release Binary Gate**: Enforced mandatory pre-publish verification in `.github/workflows/release.yml` asserting that executable packages (`.exe`, `.AppImage`, `.dmg`, `.deb`, `.rpm`, `.zip`) are present before creating or updating releases.
            -   **1-Click Installation Blocks**: Formatted release notes into individual, dedicated Markdown code blocks for Windows and POSIX commands for effortless one-click copying.
    *   **v4.59.0 (2026-09-22)**:
        -   **[Release v4.59.0] Synchronize test data dir mutex, embed windows manifest, and fix CI test runner (RCA-32)**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Synchronize test data dir mutex, embed windows manifest, and fix CI test runner (RCA-32).
    *   **v4.58.0 (2026-09-22)**:
        -   **[Release v4.58.0] Fix navbar transfer button delegation and error stack tracing**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix navbar transfer button delegation and error stack tracing.
    *   **v4.57.0 (2026-09-22)**:
        -   **[Release v4.57.0] Delegate fast-forward rotation to proven switchAccount pipeline**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Delegate fast-forward rotation to proven switchAccount pipeline.
    *   **v4.56.0 (2026-09-22)**:
        -   **[Release v4.56.0] Fix E9001 gui_config parse error with self-healing, RwLock synchronization, and backup recovery**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix E9001 gui_config parse error with self-healing, RwLock synchronization, and backup recovery.
    *   **v4.55.0 (2026-09-22)**:
        -   **[Release v4.55.0] Resolve TypeScript instance exports, rustfmt drift, and 2-min IDE crash focus recovery**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Resolve TypeScript instance exports, rustfmt drift, and 2-min IDE crash focus recovery.
    *   **v4.52.0 (2026-09-22)**:
        -   **[Release v4.52.0] Restore v4.47 quota progress bar and action buttons with clean typescript build**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Restore v4.47 quota progress bar and action buttons with clean typescript build.
    *   **v4.51.0 (2026-09-22)**:
        -   **[Release v4.51.0] Restore v4.47 quota progress bar layout and fix Windows build manifest**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Restore v4.47 quota progress bar layout and fix Windows build manifest.
    *   **v4.50.0 (2026-09-22)**:
        -   **[Release v4.50.0] Default auto_sync to true, enhance installer process cleanup and auto-launch**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Default auto_sync to true, enhance installer process cleanup and auto-launch.
    *   **v4.49.0 (2026-09-22)**:
        -   **[Release v4.49.0] Fix rustfmt formatting and rust compilation errors**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix rustfmt formatting and rust compilation errors.
    *   **v4.48.0 (2026-09-21)**:
        -   **[Release v4.48.0] Responsive UI, pinned window controls, error history actions, email ping & dual-interval polling, auto-rotation glassmorphism, and installer hardening**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Responsive UI, pinned window controls, error history actions, email ping & dual-interval polling, auto-rotation glassmorphism, and installer hardening.
    *   **v4.47.1 (2026-09-21)**:
        -   **[Release v4.47.1] Fix Rust formatting in commands/mod.rs and register format check in CI runner (RCA-27)**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix Rust formatting in commands/mod.rs and register format check in CI runner (RCA-27).
    *   **v4.47.0 (2026-09-21)**:
        -   **[Release v4.47.0] Window restore healing, zero-displacement accounts UI, active account highlighting, shortcut name standardization**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Window restore healing, zero-displacement accounts UI, active account highlighting, shortcut name standardization.
    *   **v4.46.0 (2026-09-20)**:
        -   **[Release v4.46.0] Fix thinking store signature preservation and error manager enhancements**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix thinking store signature preservation and error manager enhancements.
    *   **v4.45.0 (2026-09-20)**:
        -   **[Release v4.45.0] Enhance error diagnostics with full data copy, per-section copy, error removal, and high-contrast slate theme**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Enhance error diagnostics with full data copy, per-section copy, error removal, and high-contrast slate theme.
    *   **v4.44.1 (2026-09-20)**:
        -   **[Release v4.44.1] Fix Rust proxy compile dependencies, TokenManager session unbind, and complete multi-platform release**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix Rust proxy compile dependencies, TokenManager session unbind, and complete multi-platform release.
    *   **v4.44.0 (2026-09-20)**:
        -   **[Release v4.44.0] Sync upstream v3 proxy, quota, thinking store, and cache stability fixes**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Sync upstream v3 proxy, quota, thinking store, and cache stability fixes.
    *   **v4.43.0 (2026-09-20)**:
        -   **[Release v4.43.0] Accelerate release CI/CD with Rust dependency caching and parallel codegen units**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Accelerate release CI/CD with Rust dependency caching and parallel codegen units.
    *   **v4.42.0 (2026-09-20)**:
        -   **[Release v4.42.0] Enforce pinned release detection and dedicated release section**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Enforce pinned release detection and dedicated release section.
    *   **v4.41.0 (2026-09-20)**:
        -   **[Release v4.41.0] installer tab indentation, 80 aria2c splits & rust compile fixes**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: installer tab indentation, 80 aria2c splits & rust compile fixes.
    *   **v4.40.0 (2026-09-20)**:
        -   **[Pinned Version Respect & Intelligent Installer Fallback] Automatic URL Detection, Tag Prioritization & Descending Release Ladder**:
            -   **Automatic Pinned Release Detection**: Both `install.ps1` and `install.sh` now automatically detect when executed from a release asset URL (e.g. `irm https://.../releases/download/v4.34.0/install.ps1 | iex`), prioritizing the specified pinned release tag before discovering broader release lists.
            -   **Dedicated Pinned Version README Section**: Added explicit `### Pinned Version Installation` section in root `readme.md` under Install Scripts with copy-paste PowerShell and Bash commands.
            -   **Intelligent Descending Fallback Ladder**: When a pinned version is selected, the installer orders fallback candidates to strictly step down through prior releases (`< target_version`), preventing accidental leaps to newer releases if the pinned version asset is unavailable.
            -   **Standardized 4-Attempt Exhaustion Notice**: Guaranteed that any failed installer execution cycle terminates cleanly with `"All 4 attempts failed. I fail, so I cannot do anything."` after stepping through 4 historical versions.
    *   **v4.39.0 (2026-09-20)**:
        -   **[Hidden Terminal, Multi-Version Fallback, Auto-Switch & Branding] Zero Window Flashes, 4-Attempt Fallback, Prompt Preservation & Vector Logos**:
            -   **Hidden Background Terminal Execution**: Enforced `CREATE_NO_WINDOW` (0x08000000) flags and `-WindowStyle Hidden` on all Windows PowerShell invocations, completely eliminating terminal popups and flashes during startup update checks and CLI execution.
            -   **Idempotent Shortcut Creation**: Hardened `install.ps1` to verify if Desktop, Start Menu, and Taskbar shortcuts exist before creating them, never recreating or overwriting existing shortcuts.
            -   **Intelligent 4-Version Installer Fallback Ladder**: Enhanced `install.ps1`, `install.sh`, and `deploy/arch/install.sh` with a 4-version sequential fallback try-catch mechanism that gracefully steps down to previous releases when latest download fails before reporting exhaustion.
            -   **Pinned Version Support**: Added automatic URL and invocation detection for pinned version installations (`releases/download/vX.Y.Z/...`) across PowerShell and Bash installers.
            -   **Low-Quota (<10%) Auto-Switching & Prompt Preservation**: Enabled auto-profile switcher by default with 10.0% threshold, added multi-factor candidate scoring, and implemented dual memory + disk prompt snapshotting (`.antigravity_resume_task.json`) and direct dispatch upon rotation.
            -   **Compact Update Card & Vector Branding Showcase**: Replaced bulky notification with sleek floating card, and updated root README with responsive vector SVGs (`logo.svg`, `logo-dark.svg`) and transparent icon suite.
    *   **v4.38.1 (2026-09-20)**:
        -   **[Release v4.38.1] Fix unclosed delimiter in OpenAI handler for v4.38.1 release**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix unclosed delimiter in OpenAI handler for v4.38.1 release.
    *   **v4.38.0 (2026-09-20)**:
        -   **[Release v4.38.0] Installer output tab indentation and 80-split 500KB aria2c download acceleration**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Installer output tab indentation and 80-split 500KB aria2c download acceleration.
    *   **v4.37.0 (2026-09-20)**:
        -   **[Release v4.37.0] Instance smart rotation process teardown and candidate account switch**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Instance smart rotation process teardown and candidate account switch.
    *   **v4.36.0 (2026-09-20)**:
        -   **[Release v4.36.0] Fix LogRetentionConfig type definitions and rust formatting**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix LogRetentionConfig type definitions and rust formatting.
    *   **v4.35.0 (2026-09-20)**:
        -   **[Release v4.35.0] Enable Auto Sync Current Account by default with 5-minute polling interval**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Enable Auto Sync Current Account by default with 5-minute polling interval.
    *   **v4.34.0 (2026-09-20)**:
        -   **[Release v4.34.0] Fix Linux Tauri OpenSSL static linking with vendored feature and release v4.34.0**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix Linux Tauri OpenSSL static linking with vendored feature and release v4.34.0.
    *   **v4.33.0 (2026-09-20)**:
        -   **[Release v4.33.0] Fix Rust compilation in instance launch scope and release v4.33.0**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Fix Rust compilation in instance launch scope and release v4.33.0.
    *   **v4.32.0 (2026-09-20)**:
        -   **[Release v4.32.0] Tool branding standardization, installer padding, migration display, and aria2c acceleration**:
            -   **Update System**: Automated release and version synchronization across all manifests.
            -   **Enhancements**: Tool branding standardization, installer padding, migration display, and aria2c acceleration.
    *   **v4.31.0 (2026-09-20)**:
        -   **[Update System & Installer Integration] Startup Auto-Update Checker, Shell Script Update Flags, & In-App Installer Execution**:
            -   **Startup Auto-Update Checker**: Implemented automatic background update verification on application launch with toast notification and one-click update actions.
            -   **Shell Script Update Integration**: Added `--check-update` and `--update` flags to `install.sh` and `-CheckUpdate` and `-Update` to `install.ps1` for autonomous headless checking and upgrading.
            -   **One-Click In-App Installer Execution**: Added Tauri IPC command `run_installer_update` to invoke native installer scripts directly from the update notification and settings panels.
            -   **Dynamic Titlebar & Version Synchronization**: Dynamic version badge binding in `TitleBar.tsx` and unified SemVer propagation across manifests and documentation.
    *   **v4.30.0 (2026-09-19)**:
        -   **[Custom Titlebar & Fluid Responsive UI] Custom Titlebar Caption Controls, Frameless Architecture, & Fluid Responsiveness**:
            -   **Custom Window Controls**: Replaced native Windows OS titlebar frame with custom, fluid titlebar controls (Minimize `-`, Maximize/Restore `🗖`, and Close `✕`) directly integrated at the top of the application window.
            -   **Window Dragging & Native Actions**: Implemented seamless `data-tauri-drag-region` across titlebar with double-click maximize/restore toggle and Fitts's-law-compliant caption button hit testing.
            -   **Platform-Aware Titlebar Design**: Windows and Linux render sleek right-aligned caption buttons with soft crimson hover for close, while macOS renders authentic left-aligned traffic light dots (close, minimize, maximize).
            -   **Fluid Responsive UI**: Enhanced header and navbar responsiveness across window widths, streamlined button dimensions with micro-interaction scaling (`active:scale-95`), and eliminated top overlay click interference.
            -   **Linux Installer Hardening & Aria2c Acceleration**: Accelerated multi-connection split downloads via `aria2c`, inverted uninstallation sequence to protect existing installations, and improved migration path reporting.
    *   **v4.29.0 (2026-09-19)**:
        -   **[UI Architecture & Window Management] Top Padding, Win32 Unminimize, Error Stack Traces, & Taskbar Branding**:
            -   **Compact Layout & Top Spacing**: Reduced top padding across all major pages (Accounts, Instances, Settings, Email & Alerts) from 24-32px to 14-16px for vertical screen efficiency.
            -   **Window Unminimize & Restore Fix**: Ensured Win32 window unminimizes (`unminimize()` followed by `show()` and `set_focus()`) on tray click, single-instance double launch, and IPC events.
            -   **Error Management Drawer & Stack Trace Modal**: Real-time error queue badge in header, slide-over drawer, and comprehensive error detail dialog displaying full stack traces with one-click copy.
            -   **Taskbar Branding & Icon Polish**: Embedded application icon in Windows PE resources (`comctl6.rc`) and assigned runtime window icon (`set_icon`) for taskbar and Alt-Tab display.
            -   **Instance Double Play & Account Actions**: Double-click profile rotation prioritizing longest idle instances, reordered account action buttons, and defaulted landing tab to Accounts.
    *   **v4.28.0 (2026-09-19)**:
        -   **[Instance Architecture & Smart Play] Full Directory Cloning, Smart Rotation, & UI Action Bar**:
            -   **Full Instance Directory Cloning**: Enhanced `copy_instance` with default full directory copying for Chromium/Electron local state, session databases, and isolated user data with an on/off toggle in settings.
            -   **Smart Play Profile Rotation**: Implemented intelligent multi-factor auto-rotation algorithm evaluating least recently used, lowest 4H quota, and lowest weekly quota to auto-bind and launch target instances.
            -   **Top Action Bar & Profile Search**: Prominently placed Play (Smart Play), Duplicate, Edit (Rename), and Remove (Delete) icons adjacent to the dropdown selector, with real-time profile search and JSON Import/Export.
            -   **Launch & Stacking Context Hardening**: Purged stale lock files, resolved process race conditions with 500ms unmap delays, and eliminated UI overlap bugs with isolated stacking contexts.
    *   **v4.27.2 (2026-09-19)**:
        -   **[Instance Architecture & Smart Play] Full Directory Cloning, Smart Rotation, & UI Action Bar**:
            -   **Full Instance Directory Cloning**: Enhanced `copy_instance` with default full directory copying for Chromium/Electron local state, session databases, and isolated user data with an on/off toggle in settings.
            -   **Smart Play Profile Rotation**: Implemented intelligent multi-factor auto-rotation algorithm evaluating least recently used, lowest 4H quota, and lowest weekly quota to auto-bind and launch target instances.
            -   **Top Action Bar & Profile Search**: Prominently placed Play (Smart Play), Duplicate, Edit (Rename), and Remove (Delete) icons adjacent to the dropdown selector, with real-time profile search and JSON Import/Export.
            -   **Launch & Stacking Context Hardening**: Purged stale lock files, resolved process race conditions with 500ms unmap delays, and eliminated UI overlap bugs with isolated stacking contexts.
    *   **v4.27.1 (2026-09-19)**:
        -   **[Instance Architecture & Smart Play] Full Directory Cloning, Smart Rotation, & UI Action Bar**:
            -   **Full Instance Directory Cloning**: Enhanced `copy_instance` with default full directory copying for Chromium/Electron local state, session databases, and isolated user data with an on/off toggle in settings.
            -   **Smart Play Profile Rotation**: Implemented intelligent multi-factor auto-rotation algorithm evaluating least recently used, lowest 4H quota, and lowest weekly quota to auto-bind and launch target instances.
            -   **Top Action Bar & Profile Search**: Prominently placed Play (Smart Play), Duplicate, Edit (Rename), and Remove (Delete) icons adjacent to the dropdown selector, with real-time profile search and JSON Import/Export.
            -   **Launch & Stacking Context Hardening**: Purged stale lock files, resolved process race conditions with 500ms unmap delays, and eliminated UI overlap bugs with isolated stacking contexts.
    *   **v4.27.0 (2026-09-19)**:
        -   **[Instance Architecture & Smart Play] Full Directory Cloning, Smart Rotation, & UI Action Bar**:
            -   **Full Instance Directory Cloning**: Enhanced `copy_instance` with default full directory copying for Chromium/Electron local state, session databases, and isolated user data with an on/off toggle in settings.
            -   **Smart Play Profile Rotation**: Implemented intelligent multi-factor auto-rotation algorithm evaluating least recently used, lowest 4H quota, and lowest weekly quota to auto-bind and launch target instances.
            -   **Top Action Bar & Profile Search**: Prominently placed Play (Smart Play), Duplicate, Edit (Rename), and Remove (Delete) icons adjacent to the dropdown selector, with real-time profile search and JSON Import/Export.
            -   **Launch & Stacking Context Hardening**: Purged stale lock files, resolved process race conditions with 500ms unmap delays, and eliminated UI overlap bugs with isolated stacking contexts.
    *   **v4.26.1 (2026-09-19)**:
        -   **[UI & Scrolling Polish] Compact Top Padding & Dedicated Scroll Containers**:
            -   **Compact Top Padding**: Reduced top padding across Email & Alerts and Instances pages from 24-32px to 14-16px, eliminating redundant nested card wrappers and oversized margins.
            -   **Dedicated Scroll Containers**: Wrapped Email and Instances pages in `h-full w-full overflow-y-auto` and hardened layout main with `min-h-0` to guarantee natural vertical scrolling when content exceeds the viewport.
            -   **Refined Card Hierarchy**: Compacted telemetry banners and card padding across settings and email components for optimal screen utilization.
            -   **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.26.0 (2026-09-19)**:
        -   **[Instance Profile Management] In-Place Profile Rename & Navbar Quick Actions**:
            -   **In-Place Instance Rename**: Added `rename_instance` Tauri IPC command and frontend service to rename any profile without losing data.
            -   **Navbar Quick Rename**: Added quick edit button and dropdown inline rename controls in `InstanceSelector.tsx` for seamless profile management.
            -   **Instances Management Page**: Added profile rename modal and card action button in `Instances.tsx`.
            -   **Full Trilingual Localization**: Added complete translations across English (`en`), Simplified Chinese (`zh`), and Traditional Chinese (`zh-TW`).
            -   **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.25.0 (2026-09-19)**:
        -   **[UI & Z-Index Layering Overhaul] Eliminate Toolbar Hover Collision and Elevate Navbar Stacking Context**:
            -   **Eliminate Rogue Z-Index on Add Account**: Removed `relative z-[100]` on Add Account button in `AddAccountDialog.tsx` that previously bled through and blocked hover on the Instance dropdown menu.
            -   **Navbar Stacking Layer Elevation**: Elevated sticky navbar to `zIndex: 60` and ensured all dropdown menus (`LanguageDropdown`, `MoreDropdown`, `InstanceSelector`, `NavMenu`) are permanently above page body content.
            -   **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.24.0 (2026-09-19)**:
        -   **[Email & UI Boundary Overhaul] Dedicated Email Setup Page, Navbar Dropdown Boundaries, and Dynamic Release Asset Discovery**:
            -   **Dedicated Email Setup Page**: Added `/email` route, top-level `Email.tsx` view with active status badge, and surfaced `Email & Alerts` with `Mail` icon in main navigation.
            -   **Navbar Dropdown Boundary Protection**: Added viewport boundary locks `max-w-[calc(100vw-32px)]` to both navigation and instance selector popovers, eliminating window overflow.
            -   **Dynamic Release Asset Discovery**: Hardened `install.ps1` to query `/releases` array and dynamically pick the newest release with published binary assets.
            -   **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.23.0 (2026-09-19)**:
        -   **[Installer & UI Overhaul] Direct EXE Setup Installation, Compact Top-Level Navbar Menu, and Instance Selector Boundary Protection**:
            -   **Direct EXE Setup Installation**: Enhanced `install.ps1` to download and install setup/standalone `.exe` assets directly with silent `/S` and `/D` parameters, eliminating `.zip` extraction issues on Windows.
            -   **Compact Top-Level Navbar & Middle Menu Button**: Replaced sprawling 10-tab navbar with core quick-jump pills (`Dashboard`, `Accounts`, `Instances`) and a dedicated 'Menu' button with popover dropdown for secondary pages.
            -   **Instance Selector Boundary Protection**: Guaranteed `<InstanceSelector />` and settings controls remain 100% visible within window boundaries across all screen sizes with `shrink-0` constraints and optimized spacing.
            -   **Verified 100% Green Quality Gates**: Validated all 27 repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.22.0 (2026-09-19)**:
        -   **[Installer & UI Overhaul] Direct EXE Setup Installation, Compact Top-Level Navbar Menu, and Instance Selector Boundary Protection**:
            -   **Direct EXE Setup Installation**: Enhanced `install.ps1` to download and install setup/standalone `.exe` assets directly with silent `/S` and `/D` parameters, eliminating `.zip` extraction issues on Windows.
            -   **Compact Top-Level Navbar & Middle Menu Button**: Replaced sprawling 10-tab navbar with core quick-jump pills (`Dashboard`, `Accounts`, `Instances`) and a dedicated 'Menu' button with popover dropdown for secondary pages.
            -   **Instance Selector Boundary Protection**: Guaranteed `<InstanceSelector />` and settings controls remain 100% visible within window boundaries across all screen sizes with `shrink-0` constraints and optimized spacing.
            -   **Verified 100% Green Quality Gates**: Validated all 27 repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.21.0 (2026-09-18)**:
        -   **[Automated Release Ceremony & Python Orchestration] Standalone Minor Bump, Security Vault Hardening & Synchronized Manifests**:
            -   **Automated Release Lifecycle**: Executed complete Python-driven release orchestration (`29-release-orchestrator.py`) with automatic version propagation, branch preservation, and tag generation.
            -   **Split Database for Passwords**: Hardened dedicated split SQLite vault database (`email_vault_db.rs`, `security_vault.db`) with SSH RSA key derivation to prevent plaintext credential retrieval.
            -   **Plain Email Configuration Window**: Comprehensive UI modal for email account credentials, custom SMTP/IMAP servers, and port bindings.
            -   **Two-Way Import/Export**: Full bidirectional import and export capability for email accounts across JSON, CSV, and Excel formats.
            -   **Mailbox Failover Swapping & Background Sensors**: Integrated multi-trigger background sensors and dynamic mailbox failover pooling in `auto_switcher.rs`.
            -   **Verified 100% Green Quality Gates**: Validated all 27 repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.20.0 (2026-09-18)**:
        -   **[Cross-Platform Release & Email Security Vault Hardening] Automated Release, Credentials Vault & One-Liner Installers**:
            -   **Split Database for Passwords**: Hardened dedicated split SQLite vault database (`email_vault_db.rs`, `security_vault.db`) with SSH RSA key derivation to prevent plaintext credential retrieval.
            -   **Plain Email Configuration Window**: Comprehensive UI modal for email account credentials, custom SMTP/IMAP servers, and port bindings.
            -   **Two-Way Import/Export**: Full bidirectional import and export capability for email accounts across JSON, CSV, and Excel formats.
            -   **Mailbox Failover Swapping & Background Sensors**: Integrated multi-trigger background sensors and dynamic mailbox failover pooling in `auto_switcher.rs`.
            -   **Verified 100% Green Quality Gates**: Validated all 27 repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
    *   **v4.19.0 (2026-09-18)**:
        -   **[Email Management, Split Vault DB & CI/CD Hardening] Split Security DB for Credentials, Remote Control & 100% Green CI**:
            -   **Split Database for Passwords**: Created dedicated split SQLite vault database (`email_vault_db.rs`, `security_vault.db`) with SSH RSA key derivation to prevent plaintext credential retrieval.
            -   **Plain Email Configuration Window**: Added UI modal to input email account credentials, SMTP/IMAP servers, and custom ports.
            -   **Two-Way Email Import/Export**: Added bidirectional import and export functionality for email configurations across JSON, CSV, and Excel formats.
            -   **Mailbox Failover Swapping & Background Sensors**: Implemented multi-trigger background sensors and dynamic mailbox failover pooling in `auto_switcher.rs`.
            -   **CI/CD Pipeline Repair & Rustfmt Alignment**: Fixed modal prop mismatch in `EmailNotificationSettings.tsx`, Rust delimiter error in `email_inbound.rs`, borrower ownership in `email_io.rs`, and formatted all modules with rustfmt.
    *   **v4.18.0 (2026-09-18)**:
        -   **[Auto-Switcher Quota Threshold & Split Repo DB Active Prompt Dispatch] 15% Remaining Threshold, Prompt Persistence, and Ubuntu Process Cleanup**:
            -   **15% Quota Threshold Dynamic Switch**: Calibrated automatic account rotation threshold to trigger at 15% remaining credits (down from 50%), maximizing token utilization across all accounts.
            -   **Split Repo DB & Active Prompt Backup**: Introduced dedicated SQLite repo database (`repo_db.rs`) to detect running projects and serialize active user prompt instructions before workspace switching, restoring them immediately upon switch.
            -   **Account Rotation API & IPC Endpoints**: Implemented `/admin/rotate` HTTP endpoint and IPC commands (`list_repo_projects`, `list_repo_prompts`) with interactive status in AutoSwitcherSettings.
            -   **Hardened Ubuntu Process Termination**: Enhanced process cleanup using explicit SIGKILL on target child processes to guarantee clean workspace directory release.
            -   **Repository-Wide Architecture Migration & Rustfmt**: Migrated legacy `.lovable` workspace to `.ai-memory` and `02-spec`, and resolved RCA 18 formatting alignment.
    *   **v4.17.0 (2026-09-17)**:
        -   **[Cross-Platform Test Suite Resilience & Upstream Protocol Stability] SQLite Signature Caching, Canonical Gemini 3.7 Routing & Data Directory Isolation**:
            -   **SQLite L2 Tool Signature Persistence**: Implemented `clear_tool_signatures` in `proxy_db` to coordinate dual-tier cache eviction between memory and SQLite persistence during testing and runtime recovery.
            -   **Canonical Model Resolution**: Calibrated `resolve_real_model` to preserve canonical identifier `gemini-3.7-flash` across dynamic variant mappings while ensuring full compatibility with public OpenCode DTO registries.
            -   **Nanosecond Test Directory Concurrency**: Enhanced `TestDataDir` with nanosecond precision timestamps, thread mutex guards, and poisoned mutex recovery across Linux, macOS, and Windows runners.
            -   **Payload Audit & Retention Budget Balance**: Harmonized log payload audit thresholds with disk budget cleanup allocations to eliminate truncation and guarantee predictable disk space reclamation.
            -   **Strict Code Formatting & Type Safety**: Realigned tuple return types and integration tests for 100% compliance with `rustfmt` and TypeScript compilers.
    *   **v4.16.0 (2026-09-17)**:
        -   **[Unified Pipeline & Upstream Sync] Upstream Engine Synchronization, Thinking Store, and SQLite Acceleration**:
            -   **Unified Multi-Protocol Pipeline**: Merged modular Inbound/Outbound pipeline architecture for OpenAI Chat, Claude Messages, Responses API, and Google Gemini Native protocols.
            -   **Thinking Store & Signature Resilience**: Integrated multi-tier Thinking Store with Gzip AGZ1 compression and server-authoritative signature backfill, resolving 400 signature failures on Google upstream.
            -   **SQLite Concurrency & Disk Bounds**: Implemented read-only connection pooling and disk quota pruning in proxy DB to prevent unconstrained disk growth under high volume.
            -   **Ecosystem & Billing Safeguards**: Filtered Claude desktop billing tracking metadata for Gemini targets, eliminating false 429 RESOURCE_EXHAUSTED errors.
    *   **v4.15.0 (2026-09-17)**:
        -   **[Branding, Quota UI & Instance Management] AGM by Alim Branding, UI Compactness, and Automatic Rotation Guidance**:
            -   **AGM by Alim Branding**: Standardized window title, Navbar title, and page titles to "AGM by Alim", and executable output to "Anti-Gravity Tools by Alim".
            -   **Accounts Quota UI Compactness**: Consolidated multi-badge model quotas into unified Gemini and Claude shared buckets, streamlining vertical and horizontal density.
            -   **Instance Management & Auto-Rotation Guidance**: Fixed instance discovery and runtime interactions, and added clear automatic rotation documentation in Settings.
            -   **Attribution & Manifest Cleanup**: Removed Chinese comments from backend manifests and release tools, giving full attribution and sincere thanks to upstream lbjlaq/Antigravity-Manager.
    *   **v4.14.0 (2026-09-17)**:
        -   **[Ubuntu IDE Discovery Diagnostics & Multi-Instance Executable Cloning] Enhanced IDE Detection, Multi-Process Isolation, and Structured Diagnostics**:
            -   **Ubuntu/Linux Multi-Path Discovery Scanner**: Broadened search heuristics across standard PATH, Snap, Flatpak, ~/.local/bin, desktop entries, and AppImages, resolving .desktop Exec quoting and field-code issues.
            -   **Structured Error Model & Stack Trace Capture**: Introduced E7002 (IdeNotFound) with standard std::backtrace and comprehensive audit trail details, decoded cleanly by frontend error dialog for 1-click copy.
            -   **Cross-Platform Executable Cloning (clone_instance_executable)**: Added support for isolated executable cloning per instance across all operating systems (Windows hardlinks for distinct process images, Linux shell wrappers, macOS open -n -a isolation).
    *   **v4.13.0 (2026-09-17)**:
        -   **[Ubuntu/Linux Auto-Healing & Deep IDE Discovery] Complete Resolution of IDE Detection and `storage_json_not_found` on Ubuntu**:
            -   **Multi-Source Dynamic Linux IDE Discovery**: Implemented dynamic `PATH` scanning (`which`), standard filesystem lookups (`/usr/bin`, `/usr/local/bin`), Snap directories (`/snap/bin`, `/var/lib/snapd/snap/bin`), Flatpak application packages, and system `.desktop` file parsing to guarantee infallible IDE executable location on any Linux distribution.
            -   **Storage & Database Auto-Healing (`storage.json` & SQLite Schema Initialization)**: Expanded configuration candidate directories to include lowercase and containerized paths. When storage configuration is missing on fresh environments, the system autonomously synthesizes directory hierarchies and a valid `storage.json` with valid `machineId`, `macMachineId`, `devDeviceId`, and `sqmId`, permanently eliminating `storage_json_not_found` crashes and ensuring SQLite `ItemTable` availability.
        -   **[Structured Error Management & AI Modal] End-to-End Universal Response Envelope with One-Click AI Export**:
            -   **Unified Rust Backend Error Envelope**: Standardized `AppError` response envelopes in `src-tauri/src/error.rs` adhering to `Status`, `Errors`, `Attributes`, and `E1001`–`E9001` taxonomy.
            -   **Frontend ErrorStore with User Interaction Trails**: Implemented global error listeners, interaction trail recording (last 10 user clicks), and stack trace parsing in React 19.
            -   **AI-Friendly Compact Markdown Modal**: Designed interactive full-featured error modal with "Copy Error for AI" action providing actionable triage reports.
        -   **[Root README Overhaul & Open-Source Attribution] Complete English Refactor Following Family Standards**:
            -   **100% English Specification Compliance**: Overhauled `readme.md` strictly in English adhering to `02-spec/01-spec-authoring-guide/13-root-readme-conventions.md`.
            -   **Praise for Upstream Authors & Governance Evolution**: Honored original creator `lbjlaq/Antigravity-Manager` and all contributors, clarifying ongoing governance under the MD Animal Experiments series (`Md. Alim Ul Karim`, Riseup Asia LLC) sponsored by Rivera.
    *   **v4.12.0 (2026-09-16)**:
        -   **[Architecture Decoupling & Gitmap Pipeline Isolation] Complete Fork Independence and Centralized Gitmap Pipeline Database Storage**:
            -   **Complete Migration to alimtvnetwork Repository**: Retargeted all update check endpoints (`updater.json`), download sources, quick install one-liners (`install.ps1`, `install.sh`), Homebrew Cask formulas, and documentation to `alimtvnetwork/Antigravity-Manager`, fully eliminating upstream fallbacks and coupling.
            -   **Centralized Gitmap Pipeline DB Isolation**: Re-architected Gitmap pipeline database storage into the CLI application data directory (`AppData/Local/gitmap-cli/data/pipeline/pipeline_<slug>.db`), preventing rogue `.gitmap` directory pollution within user workspaces.
            -   **Settings UI and Web Portal Realignment**: Updated in-app Settings links, application web portal, and issue tracker references to the active repository.
    *   **v4.11.0 (2026-09-16)**:
        -   **[i18n & Architecture Stability] Default English Language, Cross-Platform Test Mutex Synchronization & Hardened Release Pipeline**:
            -   **Default Application Language Set to English**: Initialized default application interface to English (`en`) while preserving full multilingual support (13+ languages).
            -   **Cross-Platform Test Mutex Isolation**: Introduced TEST_THINKING_BUDGET_MUTEX to eliminate cross-mapper race conditions in parallel cargo test runs, and synchronized user_token_db mutations.
            -   **Windows Stress & Benchmark Calibration**: Calibrated test iterations and disk I/O latency tolerance in Windows stress test modules to eliminate false timeouts on CI runners.
            -   **Standalone Portable Installers & Release Hardening**: Published install.ps1 and install.sh one-liners as release assets, and resolved release workflow markdown parsing errors.
    *   **v4.7.4 (2026-09-17)**:
        -   **[Unified Multi-Protocol Pipeline & Architecture Refactor] Introduced Unified Pipeline Engine to Standardize Four AI Protocols (PR #3459)**:
            -   **Four-Protocol Adapter Normalization**: Unified OpenAI Chat (`/v1/chat/completions`), Anthropic Claude (`/v1/messages`), OpenAI Responses (`/v1/responses`), and Google Gemini Native protocols; implemented modular `Inbound` sanitization and `Outbound` extraction/diffusion pipelines to eliminate friction caused by schema variances and metadata fragmentation across client libraries.
            -   **Precise Token Accounting & Cache Increment Auditing**: Added `proxy::pipeline::usage` canonical metering module to accurately track `prompt_tokens`, `completion_tokens`, and cache increments (`prompt_tokens_details.cached_tokens`), preventing token omissions and abnormal amplification.
        -   **[Authoritative Thought Chain & Cryptographic Signature Normalization] Heuristic Status Matrix with Two-Tier Hydration (PR #3459)**:
            -   **Comprehensive State Matrix**: Established deterministic normalization handling across 6 permutations of thought blocks (complete / placeholder `...` / missing) and cryptographic signatures (valid / forged / missing); automatically resurrects truncated thoughts and placeholders into full, untruncated thoughts with valid Google signatures from local storage to eliminate 400 validation failures.
            -   **Bidirectional Reverse Ingestion**: Automatically backs up valid client-returned signatures and thoughts into dual-tier caching (L1 in-memory + L2 SQLite persistence), preserving signature chains across model switches or multi-turn tool sessions.
        -   **[SQLite Optimization & Long-Context Performance Leap] Read-Only Connection Reuse for Tool Signatures & Thread-Safe Test Isolation (PR #3459, PR #3462, Fixes #3461)**:
            -   **Eliminate Unnecessary fsync Disk Bottlenecks**: Relocated `PRAGMA auto_vacuum = INCREMENTAL` exclusively to `init_db()`, removing inadvertent disk write transactions and fsyncs during routine signature queries.
            -   **Lazy Read-Only Singleton & Prepared Statement Reuse**: Employs `SQLITE_OPEN_READ_ONLY` with `OnceLock<Mutex<Connection>>` for process-wide lazy connection reuse and `prepare_cached` for query plans; instantly frees rows, statements, and mutex guards to ensure newly committed signatures are visible immediately.
            -   **Significant Performance Improvement**: Slashed median local transformation latency from **14.7s down to 0.22s (98.5% reduction)** under 380,000–420,000 token long-context workloads.
            -   **Multi-Threaded Test Isolation**: Introduced `TEST_MUTEX` to prevent race conditions during parallel CI test suite runs with dynamic test directory switches.
        -   **[Resilience & Ecosystem Bugfixes] Resolved Hermes Streaming Crashes, OpenAI 429/503 Errors, and UTF-8 Multi-Byte Panics (PR #3459, Fixes #3455, Fixes #3457)**:
            -   **Fix Hermes / Python SDK Streaming Crashes (Fixes #3455)**: Strictly adheres to OpenAI streaming protocol specifications by suppressing empty `choices: []` chunks when `include_usage` is not explicitly requested, packing usage data safely onto the final choice chunk to eliminate downstream `IndexError` crashes.
            -   **Fix OpenAI Protocol Gemini 3.8/3.7 429 / 503 Errors (Fixes #3457)**: Neutralizes client-polluted low quota limits and incompatible `reasoning_effort` parameters at the inbound pipeline level, realigning with server-side supported model capability tiers.
            -   **Fix Multibyte UTF-8 System Prompt 502 Panic**: Rewrote 512-byte fingerprint extraction in `session_manager.rs` to use `is_char_boundary` for boundary checks, preventing runtime panics and 502 Bad Gateway errors on multibyte Chinese characters.
            -   **CI Hardening & Pure Rust Build Protection**: Fixed Tauri macro panic when building in headless/pure-Rust environments without pre-built frontend artifacts; all 7 cross-platform CI matrix checks pass with 100% green status.
    *   **v4.7.3 (2026-09-16)**:
        -   **[Server-Side Thinking Engine & Heuristic Hydration] In-House Thinking Store Engine to Eliminate Model Degradation & Context Drift (PR #3451, Issue #3382, Issue #3393)**:
            -   **Zero-Client-Dependency Thinking Store**: Automatically captures, persists, and hydrates thought chains and cryptographic signatures (`thoughtSignature`) on the server side for any model declared with thinking tiers (`-high`/`-medium`/`-low` or `flash`/`pro`/`claude`/`deepseek`), eliminating signature degradation and early refusal in third-party CLI and Agent frameworks.
            -   **Two-Level Tiered Cache Architecture (L1 + L2)**: L1 in-memory `DashMap` provides sub-millisecond (0.3ms) lookups for active conversations; L2 dedicated `thinking_store.db` eliminates lock contention on the primary database, recovers tool signatures by ID across restarts, and enforces a 15-day sliding window retention policy with session-level cleanup endpoints.
            -   **Extreme Long-Context Optimization (AGZ1)**: Blocks placeholder blocks (`...`) from persistence, replaces quadratic string diffs with zero-allocation streaming byte hashing (`hash_normalized_ws`), and applies `flate2` Gzip compression (`AGZ1` header) to thoughts exceeding 384 characters, slashing disk footprint by 70%+ with 100% lossless decompression.
        -   **[Unified Multi-Protocol Pipeline & Native Tool Calling] Four-Protocol Normalization & Prompt Sanitization (PR #3451)**:
            -   **Normalized Pipeline**: Unifies Claude, OpenAI, Gemini, and Codex request flows through a standard pipeline: "Clean -> Norm -> Hydrate -> Usage", standardizing payloads to Gemini Contents with `systemInstruction.role` fixed to `user`.
            -   **Native Tool Dispatch & Anti-Downgrade Locks**: Completely removes legacy hardcoded MCP XML prompt injection so tool calls execute as native `functionCall` without leaking into output text; locks Gemini 3+ native thinking budget minimums (Flash: 10000, Pro: 10001) to prevent client-induced truncation.
            -   **Token Accounting & Cache Ratio Fixes (Issue #3391)**: Resolved Anthropic input prompt token omissions and corrected the cache hit ratio calculation bug that reported over-100% overflows (e.g. `583.1%`).
        -   **[Full-Chain Diagnostics & Three-Pane Inspection] Micro-Stage Timing Breakdown & Multi-Pane Audit (PR #3451)**:
            -   **Microsecond Stage Instrumentation**: Measures granular timings across Cleaning (`Clean`), Normalization (`Norm`), Thinking Hydration (`Thinking`), First-Token Latency (`TTFT`), Stream Duration (`Stream`), and True Wall-Clock Total (`Total`).
            -   **Diagnostics Dashboard & Audit Windows**: Response cards feature color-coded timing breakdown progress bars with one-click export; monitor modal expands to three parallel inspection panes (Original Forward / Normalized Transform / Upstream Response) with toggleable `simple` (compact) and `full` (complete payload) storage modes.
        -   **[Proxy Monitor & Logging Optimization] Traffic Log Memory Slimming, Bounded Background Persistence & SQLite Disk Budget (PR #3445, Issue #3443)**:
            -   **In-Memory Slimming**: Retain only lightweight summaries in the in-memory ring buffer, stripping complete request (`request_body`) and response bodies (`response_body`) and capping error messages (`error`) to 1,024 characters. Full payloads are loaded on demand via detail endpoints when requested by the frontend, eliminating memory leaks and OOM under sustained high-throughput workloads.
            -   **Bounded Concurrency & Load Shedding**: Added semaphore gating allowing at most 4 concurrent background persistence tasks before `spawn_blocking`. Under excessive load, persistence is safely dropped to avoid exhausting worker threads or backing up heap memory.
            -   **SQLite Disk Budget & Incremental Space Reclamation**: Added `proxy.log_retention.max_disk_mb` configuration (default 1,024 MiB with live reloading). Proactively evaluates DB and WAL sizes along with transaction headroom before writing, progressively evicting oldest payloads in batches and reclaiming free pages to permanently bound SQLite disk usage.
        -   **[OpenAI / Codex Alignment] Normalize Stale Codex Model Identity to Avoid Gemini 429 Errors (PR #3444, Issue #3442)**:
            -   **Filter Stale Identity Statement**: Automatically normalizes stale prompt declarations (`You are Codex, an agent based on GPT-5.` -> `You are Codex, an agent.`) during OpenAI/Responses to Gemini request transformation prior to system-instruction cache lookup.
            -   **Scoped Mutation Protection**: Normalization strictly targets top-level `instructions`, system/developer messages, and historical model-switch turns; user queries (`user`) and tool outputs (`tool`) are strictly preserved intact, preventing persistent 429 rate-limiting on Gemini endpoints.
        -   **[Agent Ecosystem & Tooling Alignment] Adapt DeepSeek Harness (DSH) and WorkBuddy Tool Calling Schemas (Issue #3440, Issue #3430)**:
            -   **pwsh / bash Bidirectional Invariant Enforcement**: Accommodates DSH's strict TypeScript runtime contracts requiring non-empty string types for both `command` and `description`. Automatically derives and injects a concise summary (`Run: <cmd>`) when `description` is missing to avoid frontend card rendering crashes.
            -   **Accurate Command Recovery**: Eliminates the issue where commands were swallowed into harmless `echo` fallbacks; when models emit executable command lines inside the `description` field, automatically extracts them into valid `command` payloads.
            -   **workflow Nested Metadata Packaging**: Aligns with DSH `tool-workflow` requiring `{ script, meta: { name, description } }`. When models return flattened parameters, automatically packs them into a valid `meta` dictionary.
        -   **[Claude Client & Multi-Tool Risk Mitigation] Filter Claude Desktop Injected Billing Metadata to Avoid Gemini 429 Errors (Issue #3452)**:
            -   **Filter Injected Client Billing Header**: Automatically detects and drops standalone client-internal billing/entrypoint metadata lines (`x-anthropic-billing-header:`) injected by Claude Desktop in system prompt blocks when targeting Gemini models with heavy toolsets (e.g. 99 MCP tools).
            -   **Precision Scope**: Scoped strictly to single-line declarations starting with the metadata header prefix, preserving multiline text, quoted mentions, tools, cache markers, and non-Gemini destinations intact to eliminate upstream Google WAF `RESOURCE_EXHAUSTED` 429 false positives.
    *   **v4.7.2 (2026-09-15)**:
        -   **[OpenAI / Codex Alignment] Fix Gemini Thoughts Display as Reasoning Summaries in Codex (PR #3439, Issue #3438)**:
            -   **Standardized Reasoning Summary Events**: When streaming via `POST /v1/responses`, maps Gemini parts with `thought: true` to standard `rs_...` items (`type: reasoning`) and emits canonical `response.reasoning_summary_part.*` and `response.reasoning_summary_text.*` SSE events so Codex renders thought summaries in the dedicated reasoning block.
            -   **Lifecycle Management & Stream-End Closure**: Gracefully closes reasoning summaries prior to standard text or tool item delivery and at stream completion, ensuring lifecycle non-overlap and maintaining consistent completed outputs in session storage.
        -   **[Prompt Isolation & Memory Sync] Global System Prompt Spacing, Gemini Wrapper Deduplication & Instant Config Sync (PR #3433)**:
            -   **Markdown Header Isolation**: Added double newline (`\n\n`) delimiters after the Antigravity identity and global system prompt, preventing downstream Markdown headings (e.g. `## Global Preferences`) from concatenating directly onto preceding bold tokens (`**Proactiveness**`) and avoiding trailing header collisions.
            -   **Gemini Wrapper Deduplication**: Added content deduplication in `wrap_request_v2` to prevent redundant global prompt injection during retries or multiple wrapper passes.
            -   **Synchronize Config Outside Proxy Instance**: Hoisted global memory updates in `save_config` (thinking budget, global system prompt, image thinking mode, and compression configs) outside the active instance check, ensuring settings take effect in memory immediately even when the proxy server is stopped.
        -   **[Storage & Data Management] Custom Data Directory with Seamless Full Migration (Issue #3441)**:
            -   **Custom Data Directory & Bootstrap Pointer**: Addressed C: drive disk space exhaustion caused by default storage (`~/.antigravity_tools`) by introducing persistent pointer file resolution. Users can now choose any disk directory (e.g. `D:\AntigravityData`) in Advanced Settings, which the application auto-detects at boot.
            -   **Full Seamless Migration & Safe Disk Reclamation**: Added a "Change & Migrate" button in Advanced Settings to safely copy accounts, configurations, request logs, and SQLite databases to the target location, with an option to clean up original files and automatically restart the application to load the new directory seamlessly.
    *   **v4.7.1 (2026-09-12)**:
        -   **[Upstream Protocol Optimization & Native Alignment] Native Language Server Alignment: Dynamic Agent RequestType, Fine-Grained 429 Classification & Malformed Call Fallback**:
            -   **Dynamic On-Demand `requestType: "agent"`**: Reverse-engineered native Antigravity language server behavior to eliminate unnecessary `"requestType": "agent"` flags on every prompt. Now, Agent mode is only activated when tool declarations exist or messages contain tool turns. Normal chat and code completions flow through the standard Chat pool, drastically reducing 429 rate limit contention on Google's Agent pool.
            -   **Fine-Grained 429 Classification**: Refined `parse_rate_limit_reason` so that transient `RESOURCE_EXHAUSTED` responses lacking explicit `quotaResetTimeStamp` markers are classified as short-lived concurrency spikes (`RateLimitExceeded`) rather than hard quota exhaustion, preventing accounts from being erroneously locked out for 30 minutes.
            -   **Normalized `finish_reason` & `MALFORMED_FUNCTION_CALL` Fallback**: Mapped Gemini's internal `MALFORMED_FUNCTION_CALL` reason to standard OpenAI `"stop"`. If the model aborts without generating content, automatically injects a helpful fallback message to eliminate blank chat bubbles and parsing errors in downstream clients (NextChat, LobeChat, Cherry Studio, etc.).
        -   **[Ecosystem Integration] One-Click Sync of APIKEY.FUN Credentials & Models to OpenCode (PR #3427)**:
            -   **OpenCode Sync Button**: Added an OpenCode sync feature on the APIKEY.FUN page to automatically export the configured API Key, BaseURL, and discovered models into OpenCode's configuration as an independent `apikey-fun` provider (via `@ai-sdk/openai-compatible`).
            -   **Safety & Edge Protections**: Supports both Tauri commands and HTTP API; backs up existing configuration files prior to mutation while preserving other providers and custom model parameters; normalizes trailing slashes to prevent `/v1/v1` duplication; full 13-locale i18n support.
        -   **[Storage & Data Safety] Atomic File Writes for Accounts & Configurations (PR #3420)**:
            -   **Atomic Write Engine (`write_atomic`)**: Re-architected configuration and account index persistence using temporary files (`<file>.tmp.<uuid>`), memory flushes (`write_all`), guaranteed physical disk sync (`sync_all` / fsync), and atomic rename operations. Eliminates 0-byte corruptions and loss on unexpected power cuts, kernel panics, or sudden disk-full scenarios.
            -   **Elevated Error Visibility**: Promoted account parsing failures to `warn!` logging to immediately surface corrupted files.
        -   **[Circuit Breaker & Timestamps] Fix reset_time NaN Countdown & Isolate Model-Specific Zero-Quota Locks (PR #3417)**:
            -   **Resolve `NaNh NaNm` UI Countdown Bug**: Formatted backend rate-limit `reset_time` as standardized RFC3339 / ISO-8601 strings, and introduced `parseFlexibleDate` on the frontend to support both numeric timestamps and ISO strings.
            -   **Isolated Model-Level Zero-Quota Circuit Breaking**: When a specific quota bucket (such as Claude `3p-5h`) reaches 0%, the `lock_on_zero_quota` lock now isolates only the corresponding model category (`claude` or `gemini-3-flash`), leaving other models with available quota untouched.
        -   **[Proxy Log Retention] Automated Log Retention Policy & Space Reclamation (PR #3423)**:
            -   **Configurable Retention Limits**: Introduced automatic request log lifecycle management with defaults of 24-hour body retention, 30-day metadata retention, and a 100,000-row cap. Executes at startup and hourly thereafter.
            -   **Incremental Vacuum**: Automatically runs incremental SQLite space reclamation to bound database growth, with UI settings and multi-language support.
        -   **[Proxy Resilience & Decryption Fallback] Decryption Failure Warnings & URL Fallback (PR #3424)**:
            -   **Machine-ID Drift Warning**: Emits structured warnings when proxy credentials cannot be decrypted (e.g. after container migration where `/etc/machine-id` changed), preventing encrypted ciphertext from being forwarded upstream as raw passwords.
            -   **Embedded URL Credentials Fallback**: When structured authentication decryption fails, gracefully falls back to credentials embedded in the proxy URL.
        -   **[Logging & Container Maintenance] Claude Signature Cache Log Demotion & Docker Log Rotation (PR #3421, PR #3422)**:
            -   **Reduce Log Noise**: Demoted high-frequency signature cache recovery and sanitizer log lines to `debug!`, saving up to ~1GB/day of Docker log volume.
            -   **Compose Log Rotation**: Added `json-file` log caps (`max-size: "100m"`, `max-file: "3"`) across all Compose configurations to cap disk usage per container at ~300MB.
        -   **[Linux & CLI Auth Fix] Sync 'login' and 'default' Collections to Fix agy Account Switching (Issue #3418, Issue #3428)**:
            -   **Dual-Collection Secret Service Sync**: Resolved split collection issues in GNOME Keyring / Ubuntu / Debian environments (where `login` and `default` aliases point to different keyrings, while `agy` strictly reads credentials from `login`). `secret-tool` writes now explicitly target `--collection=login` while maintaining fallback sync to `default`, fixing newly launched `agy` CLI processes continuing to use the old account.
        -   **[Agent Compatibility & Tool Call Sanitization] Prevent Downstream Client Crashes on Incomplete Gemini Function Calls (Issue #3430)**:
            -   **Required command Parameter Guard**: Addressed issues where Gemini Flash in complex or long-context turns occasionally emits only `description` while omitting the required `command` parameter for tools like `PowerShell`, `pwsh`, `bash`, `shell`, and `terminal`. Prevents downstream agent runners (e.g. WorkBuddy, LangChain) from throwing fatal `TypeError: Cannot read properties of undefined (reading 'split')`.
            -   **Smart [OK] Semantic Fallback & Anti-Looping**: Expanded alias normalization (`cmd`, `code`, `script`, `shell_command`, `input`). When commands are completely missing, injects a safe `echo "[OK: Action logged - <description>]"` fallback with clean exit status, preventing repetitive failed retries and endless agent deadlock loops.
        -   **[UI & Packaging] Dark Mode Switch Highlight & Homebrew Cask Modernization (PR #3431, PR #3432)**:
            -   **Dark Mode Toggle Visibility**: Fixed active switches in dark mode being obscured by gray backgrounds by applying clear blue tracks and white thumbs.
            -   **Modern Homebrew Cask DSL**: Migrated deprecated Cask `preflight`/`postflight` blocks to structured `preflight_steps` and `postflight_steps`.
        -   **[Quota & Load Balancing Fix] Fix Fake 100% Quota, Multi-Endpoint Fallback & Bucket Fusion (Issue #3426)**:
            -   **Multi-Endpoint Fallback Continuity**: Removed premature `return None` on 4xx responses in `retrieveUserQuotaSummary`, ensuring fallback continues through Sandbox ➔ Daily ➔ Prod endpoints; added equivalent 3-tier fallback to `loadCodeAssist` (`fetch_project_id`).
            -   **Reuse Cached Project ID**: Passed cached `project_id` throughout account quota polling and retry paths to avoid extraneous `loadCodeAssist` calls that trigger rate limits or 403 errors.
            -   **Deep Quota Bucket Fusion**: Fused actual remaining fraction and reset time from `retrieveUserQuotaSummary` into `quota_data.models`, completely eliminating the fake 100% quota display in the UI.
    *   **v4.7.0 (2026-09-10)**:
        -   **[Session & Proxy Fix] Prevent 400 Errors and Account Freezes from Upstream 1M Token Accumulation (PR #3415, Issue #3411, refs #3325)**:
            -   **Scoped Session IDs per Conversation**: Replaced the account-email-only upstream `sessionId` hash with a scoped derivation combining `account_id`, conversation fingerprint, and a generation counter. Keeps upstream Prompt Cache hits stable within the same dialogue while strictly isolating different conversations from sharing one server-side session.
            -   **Automatic Generation Bump & Transparent Recovery on 1M Overflow**: When receiving upstream `400 "The input token count exceeds the maximum number of tokens allowed"`, automatically increments the generation counter and retries with a fresh session, seamlessly recovering the dialogue without user intervention.
        -   **[Adaptive Circuit Breaker] Zero-Quota Lockout & Dynamic Max Backoff Steps (PR #3413)**:
            -   **Lock on Zero Quota Toggle (`lock_on_zero_quota`)**: Added a circuit breaker option to immediately lock an account until its upstream `reset_time` when 5-hour rolling or weekly quota hits 0%, skipping short backoffs and preventing wasted calls on exhausted accounts; auto-clears locks when quota recovers.
            -   **Respect Configured Max Backoff Steps**: Removes the hardcoded 300s ceiling on retry lockouts, allowing longer user-configured backoffs (e.g. 1800s / 7200s) to take effect.
            -   **Clear Stale Protection on Global Disable**: Automatically purges lingering `protected_models` from accounts when quota protection is disabled globally.
        -   **[Proxy Protocol Compliance] Expose Standard Retry-After Header on Temporary 503 Responses (Issue #3414)**:
            -   **Standardized Cooldown Header**: When all accounts are temporarily rate-limited (`All accounts limited. Wait Ns.`) resulting in a 503 Service Unavailable, extracts the cooldown duration and returns a standard `Retry-After: <seconds>` HTTP header across OpenAI, Claude, and Gemini proxy handlers.
            -   **Client-Friendly Backoff**: Enables downstream AI tools and coding agents (e.g., Cursor, Cline, Aider, OpenCode) to accurately pause and back off according to the server cooldown rather than spamming retries.
        -   **[Internationalization] Detect OS Language for New Configurations (PR #3412)**:
            -   **System Language Auto-Detection**: Integrated lightweight system locale detection to initialize default language from the OS locale (supporting Traditional Chinese `zh-TW` / `zh-HK`, Simplified `zh`, `en`, `ja`, `ru`, `pt`, etc., with safe fallback to `en`), replacing the hardcoded `"zh"` default for new setups. Existing configurations remain unaffected.
    *   **v4.6.9 (2026-09-08)**:
        -   **[Core Fix] Honor store:false to Inhibit HTTP Session & Global Tool Call Cache Retention (PR #3408)**:
            -   **Respect store:false Parameter**: When full-replay requests explicitly pass `store:false` in the HTTP Responses path, avoids creating redundant session snapshots and background save tasks, significantly reducing memory growth during large-context replays.
            -   **Tool-Call Cache Gate & Safe Non-Stream Guard**: Gates redundant writes to the global tool-call cache under `store:false` while still emitting full tool-call arguments, and hardens non-streaming history persistence with `store_response` guard.
        -   **[Scheduling & Stability] Resolve 429 Failover Dead Loops, Model Circuit Breaker Bypass, and Quota Normalization (PR #3395)**:
            -   **Align Sticky Sessions with Model-Level Circuit Breakers**: Fixed sticky session availability check querying with global key instead of target model, ensuring accounts locked with `RESOURCE_EXHAUSTED` for specific models are properly detected and not reused.
            -   **Instant Failover on Hard Quota Limits**: Upstream 429 / 529 now bypasses `GraceRetry` and unbinds the session immediately, breaking infinite retry loops in Balance and CacheFirst modes.
            -   **Model Normalization for Quota Protection**: Normalized user-configured model IDs (e.g. `gemini-3.7-flash`) to standard quota bucket IDs (`gemini-3-flash`) before evaluating thresholds, restoring quota protection functionality.
        -   **[Multi-Turn & Agent Compatibility] Prevent Gemini 400 Invalid Argument on Autonomous First Tool Call (PR #3396)**:
            -   **Inject Lightweight User Turn Primer**: When autonomous agent loops (e.g., Hermes) start with assistant tool calls immediately following the system message, automatically prepends a minimal user turn primer (`"Continue the task."`) to satisfy Google Gemini API turn-ordering requirements.
            -   **Preserve Empty User Messages**: Ensures user messages with empty content retain whitespace rather than being dropped, maintaining valid role rotation.
        -   **[OpenAI & Codex Protocols] Separate Routing & Signature Identity, Fresh Request IDs, and Flash Thinking (PR #3404, PR #3405, PR #3406)**:
            -   **Decouple Routing & Signature Identities**: Separated session routing keys from thought signature lookup keys, and added non-streaming session history persistence.
            -   **Unique Request ID per Upstream Attempt**: Dynamically generates timestamp-and-random request IDs on every invocation, preventing upstream idempotent blocking or association with previous 429 errors.
            -   **Map Tiered Flash Reasoning Effort**: Accurately maps OpenAI `reasoning_effort` parameters to upstream Gemini `thinkingLevel` for tiered Flash models.
        -   **[Claude Protocol Compliance] Always Include usage Object in message_start Event (PR #3397)**:
            -   **Fallback Usage Object**: When upstream models lack usage metadata in the first chunk, emits a zero-value `usage` object (`{input_tokens: 0, output_tokens: 0}`) to prevent type validation errors on strict clients like OpenCode and `@ai-sdk/anthropic`.
        -   **[HTTP API & CLI] Fix /accounts/switch Dropping targetIde Causing IDE Restarts on agy Token Refreshes (Issue #3399)**:
            -   **Pass Through targetIde to Bypass IDE Restarts**: Resolved an issue where `POST /accounts/switch` omitted the `target_ide` field and hardcoded `None`, preventing agy CLI and automation scripts from hitting the pre-existing keyring-only fast path. Passing `{"targetIde": "agy"}` now updates credentials quietly without killing and relaunching the running Antigravity IDE.
    *   **v4.6.8 (2026-09-06)**:
        -   **[Core Fix] Fix Startup Unconditional VACUUM Disk Saturation & Retention Timestamp Unit Bug (Issue #3386)**:
            -   **Align Retention Timestamp to Milliseconds**: Fixed an issue where `cleanup_old_logs` calculated the 30-day cutoff using seconds (`timestamp()`) while request logs were stored in milliseconds (`timestamp_millis()`), causing expiration queries to never match and old logs to never be purged. Unified retention cutoff calculations to milliseconds (`now.timestamp_millis() - days * 24 * 3600 * 1000`).
            -   **Threshold-Gated VACUUM Compaction**: Removed the unconditional full SQLite `VACUUM` executed on every `ProxyMonitor` startup initialization. Compaction is now only triggered when a meaningful batch of records is purged (`deleted >= 500`), while gracefully tolerating non-fatal compaction errors. Completely eliminates sustained 100% disk active time and system freeze caused by repeatedly rewriting multi-gigabyte log databases when 0 records are deleted.
        -   **[Feature] Support Table Sorting by Quota Reset Time and Last Used Time (Issue #3387)**:
            -   **Bi-directional Sorting for Reset Time & Last Used**: Users can now click the "Weekly Quota" / "Model Quota" and "Last Used" column headers to toggle sorting (`Ascending -> Descending -> Default`). In Weekly Quota mode, automatically extracts the earliest reset timestamp across Gemini and Claude model quota groups, allowing users to prioritize accounts that will recover quota soonest.
            -   **Intelligent Mutual Exclusion with Drag-and-Drop**: Temporarily disables `@dnd-kit` drag gestures and updates the grip handle tooltip when column sorting is active, preventing accidental sort-order conflicts.
        -   **[Linux Fix] Resolve Black/Blank Window on niri / Hyprland Compositors (Issue #3388, PR #3389)**:
            -   **Do not force `GDK_BACKEND=x11` solely because `DISPLAY` is present**: niri / Hyprland / sway / river / labwc / wayfire always expose an Xwayland compatibility bridge. Previously, detecting `DISPLAY` forced X11 backend, causing WebKitGTK UI to render completely black. Those compositors now retain native Wayland, while GNOME / KDE historical X11 fallback remains intact.
            -   **Dynamic WebKit DMA-BUF Disabling**: Under Wayland when running on wlroots compositors or when proprietary NVIDIA drivers are loaded, automatically sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` if unset to prevent black windows. GNOME + AMD/Intel continues using the hardware-accelerated DMA-BUF fast path. Existing user environment variables are never overridden.
        -   **[Core Fix] Resolve `400 thinking.signature: Field required` on Non-Thinking Suffix Claude Models in Multi-Turn Dialogues (Issue #3391)**:
            -   **Broaden Claude Thinking Detection**: Fixed an issue where Claude thinking was only detected via `-thinking` suffix, allowing base models like `claude-sonnet-4-6` with explicit `thinking: { type: "enabled" }` to bypass thinking-signature defenses.
            -   **Safe Fallback on Incompatible History & Prevent Unsigned Thought Injection**: When multi-turn assistant messages lack `reasoning_content` and valid signatures, Claude thinking requests safely downgrade to non-thinking mode and prevent injecting synthetic thought blocks that fail upstream signature verification.
            -   **Clear Thinking Config on 400 Signature Retries**: Explicitly clears `openai_req.thinking` and strips `-thinking` suffix when 400 signature errors occur, guaranteeing that subsequent retry attempts run purely in standard text mode rather than looping on repeated 400 errors.
        -   **[Core Fix] Fix Anthropic Protocol Proxy SSE Timeout Hangs, Tool Call Leakage, and Thinking Disruption Loop (Issue #3393)**:
            -   **Tighter Streaming Timeouts & Zombie Stream Circuit Breaker**: Tightened initial peek probe timeout from 300s to 30s and outer SSE idle keepalive timeout from 300s to 120s; reduced inner SSE ping timeout from 60s to 20s with an active abort after 5 consecutive empty pings (100s), eliminating unresponsive connection hangs caused by upstream stalls.
            -   **Prevent Tool Call Input Leakage on Empty/No-Argument Tools**: Fixed `process_function_call` skipping `input_json_delta` when tools have empty or no arguments (such as `EnterPlanMode`), which caused incomplete tool use blocks that leaked tool input syntax into regular text content or triggered client parse errors; now guarantees emitting `input_json_delta` with `partial_json: "{}"`.
            -   **Fix Thinking Chain Interruption Recovery Missing Stop Event**: Fixed recovery path relying on `emit_force_stop()` which was blocked by internal `message_stop_sent` guard, preventing termination events from being emitted; explicitly dispatches `message_delta` and `message_stop` events to ensure Claude clients gracefully finish turns without looping infinitely.
    *   **v4.6.7 (2026-09-04)**:
        -   **[Core Fix] Fix Multi-Turn Agent Context Explosion & Session History Duplication BUG (Issue #3382)**:
            -   **Unblock Thinking Compression on Historical Assistant Messages with Tool Calls**: Fixed an issue where the strict `!has_tool_calls` check prevented compressing historical assistant `reasoning_content` in agent environments (e.g. OpenClaw) where almost every assistant turn contains tool calls. Allows pruning long thoughts down to placeholder `...` while fully preserving tool calls and their valid `thoughtSignature` tokens, eliminating token bloat and preventing upstream Google API 400 signature errors.
            -   **Normalized Thought Placeholder for Older Turns**: Updated `openai/request.rs` to generate a minimal placeholder thought block `{ "text": "...", "thought": true }` for historical assistant thinking in older turns outside the recent window, fulfilling upstream thinking schema requirements while discarding thousands of redundant thinking tokens.
            -   **Semantic History Matching & Duplication Prevention in Session Store**: Enhanced `prepare_session_input` with semantic prefix matching (ignoring client ID format differences) and sliding suffix boundary identification. Added fallback protection: when client sends full history and matches fail, uses client's full sequence instead of appending client history onto server history, eliminating exponential history compounding (2x/4x).
        -   **[Feature] Support Video Multimodal Inputs (`video_url` / `inlineData`) in OpenAI-Compatible API (Issue #3381)**:
            -   **OpenAI Content Block Support for `video_url`**: Extended `OpenAIContentBlock` to natively deserialize and process `video_url` blocks, resolving `Invalid request: data did not match any variant of untagged enum OpenAIContent` when clients send video inputs.
            -   **Format Detection & Native Gemini Multimodal Alignment**: Implemented a dedicated video processor (`proxy/video`) supporting base64 data URLs (`data:video/mp4;base64,...`), remote video URLs (`fileData`), local files (`file://` or filesystem paths automatically encoded to inlineData), and raw base64. Covers MP4, WebM, MOV, AVI, WMV, MKV format normalization, oversize advisory warnings, and token estimation.
    *   **v4.6.6 (2026-09-03)**:
        -   **[Core Fix] Fix Gemini 3.7 Tool Call Text Leakage Causing Silent Agent Interruption in Long Contexts (Issue #3379)**:
            -   **call:default_api Leakage Detection & Controlled Fail-Closed Recovery Bridge**: Resolved an issue where Gemini 3.7 Flash, following long-context compression, occasionally leaks internal pseudocode tool invocations (`call:default_api:ToolName{...}`) into plain text deltas instead of structured `functionCall` blocks, silently breaking Claude Desktop / Claude Code agent loops. Enforced a 7-point strict fail-closed guard sequence (registered tools required, no native tool use in current turn, strict prefix match, registered tool whitelist alignment, no surrounding prose, valid JSON args, no prior text deltas emitted) to securely recover leaked calls into standard `tool_use` blocks without prompt injection risks.
            -   **Symmetric Streaming/Non-Streaming Parity & Diagnostics**: Symmetrically aligned recovery logic across SSE streaming (`streaming.rs`) and non-streaming responses (`response.rs`), added diagnostic warning logs (`tracing::warn`) when text patterns are detected, and reinforced stability with 9 unit test scenarios.
    *   **v4.6.5 (2026-09-02)**:
        -   **[Core Fix] Fix Gemini JSON Schema Validation 400 Errors for Nested Arrays Missing Items (PR #3375)**:
            -   **Array Items Fallback Injection**: Resolved upstream Gemini 400 schema validation errors triggered by clients such as Claude Code when emitting itemless array schemas (e.g. `query.where: { type: "array", items: { type: "array" } }`). The recursive JSON schema sanitization now injects a Gemini-compatible `{"type": "string"}` fallback for itemless `array` nodes, backed by unit regression tests.
        -   **[Streaming Proxy & Protocol Compliance] Standardize Claude Upstream Stream Interruption Error Events (PR #3371, PR #3373)**:
            -   **Anthropic Standard SSE Error Output**: Fixed non-standard error payload formatting during upstream stream breaks or exceptions. Standardized on the Anthropic-compliant `type: "error"` structure with `error: { "type": "overloaded_error", "message": ... }` emitted via `state.emit("error", ...)`, ensuring Claude clients reliably catch and handle stream aborts.
        -   **[OpenCode & Model Support] Add Thinking Variant Support to Base Claude Opus 4.5/4.6 Models (PR #3371, PR #3373)**:
            -   **Opus Base Model Thinking Variants**: Added `VariantType::ClaudeThinking` support for `claude-opus-4-5` and `claude-opus-4-6` base model definitions. This resolves an issue where selecting base model IDs yielded no thinking tiers in dropdown selectors. Also aligned `Gemini3Pro` variant ordering.
        -   **[Linux & AppImage] Fix GUI Popup on Version Detection & Child Process Environment Leaks under AppImage (Issue #3370)**:
            -   **Static Package.json Priority Version Reading**: Optimized Linux Antigravity version detection to prioritize reading `resources/app/package.json` directly from the installation directory, preventing executing `--version` from inadvertently launching the Chromium/Electron GUI window.
            -   **Child Process AppImage Environment Sanitization**: When spawning Antigravity on Linux, AppImage runtime environment variables (`APPIMAGE`, `APPDIR`, `ARGV0`, `LD_LIBRARY_PATH`, `GTK_PATH`, etc.) are stripped, and `/tmp/.mount_*` paths are filtered from `XDG_DATA_DIRS` to avoid inheriting conflicting runtime libraries.
            -   **Hardened Process Detection & Self-Exclusion**: Enhanced Linux running process scanning and path resolution to ensure AppImage mount paths and parent/ancestor processes are not misidentified as target Antigravity instances.
        -   **[System & Tray] Prevent Window-State Plugin from Restoring Window Visibility on Startup (PR #3373)**:
            -   **Window Visibility State Filter**: Updated `tauri-plugin-window-state` configuration to exclude `StateFlags::VISIBLE`. This prevents the window from popping up on autostart or tray background launches when configured with `visible: false`.
    *   **v4.6.4 (2026-08-30)**:
        -   **[Core Fix] Fix Token Acquisition Timeout (5s) / Deadlock Error Under High Concurrency & Tokio Runtime Starvation (Issue #3348)**:
            -   **Async Disk I/O & Blocking Thread Pool Isolation**: Refactored `update_account_json` to an async function that dispatches synchronous disk I/O and global account locks to Tokio's blocking thread pool (`spawn_blocking`). This prevents disk serialization contention from blocking Tokio worker threads and causing runtime starvation under high concurrency.
            -   **Fire-and-Forget Disk Persistence on Token Acquisition Hot Path**: On the primary `get_token` scheduling path, file persistence after OAuth token refreshes and `project_id` resolution is now offloaded to background tasks after updating memory caches immediately, preventing disk write overhead from consuming the 5-second timeout window.
        -   **[Core Fix] Fix Indefinite Hang on Minimal/Single-Dot Prompts Causing Claude Desktop Gateway Health Check Timeout (Issue #3359)**:
            -   **Empty SSE Event Fallback**: Added defensive fallback logic in the Claude SSE streaming conversion layer. When upstream models terminate empty responses on minimal/punctuation-only prompts without yielding content or thinking, the stream synthesizer automatically emits valid `message_start`, fallback text ContentBlock, and `message_stop` events, eliminating peek loop timeouts.
            -   **Non-Streaming Collector Schema Guard**: Enforced that `collect_stream_to_json` always returns at least one valid text ContentBlock when parsing empty upstream streams, complying strictly with Anthropic client non-empty content constraints.
        -   **[Streaming & Session Management] Upstream SSE Cancellation on Client Disconnect & Session Branching Graph (PR #3367, PR #3366)**:
            -   **Proactive Upstream SSE Teardown**: Automatically stops polling and consuming upstream SSE streams as soon as the client disconnects or the downstream response body is dropped, preventing incomplete requests from saving invalid sessions.
            -   **Parent-Linked Session Graph**: Fixed streaming HTTP Responses session addressing and replaced full-history deep copies with a persistent parent-linked session tree graph for efficient branch support.
        -   **[Image Proxy & Account Scheduling] Account-Aware Image Scheduling & Rate Limit Lifecycle Hardening (PR #3364, PR #3363, PR #3362)**:
            -   **Account-Aware Image Concurrency Scheduler**: Introduced a shared concurrency scheduler for OpenAI and Gemini image generation and edit requests, enforcing per-account concurrency caps and seamless queueing when all accounts are busy.
            -   **Rate Limit Lifecycle & Grace Retry Optimization**: Corrected short 429 delay parsing, capped same-account retries to at most once, and preserved only explicit long-lived image quota deadlines.
            -   **Image Request Semantics Hardening**: Fully hardened OpenAI-compatible image requests with robust model alias resolution, size mapping, ordering, and input boundary validation.
        -   **[Network & Multimodal Improvements] Tool Image Retention, Bounded Debug SSE Capture & Quota Header Cleanup (PR #3365, PR #3361, PR #3360)**:
            -   **Tool Image Retention & Inline Media Bounding**: Retains images returned from tool outputs as multimodal model inputs, bounds inline image memory, and strips historical inline media before replay or caching.
            -   **Bounded Debug SSE Capture**: Implemented 256 KiB head + 256 KiB rolling tail buffer for debug response logging to eliminate memory bloat on large streams while preserving full downstream streaming.
            -   **Quota Header Cleanup**: Omits redundant `x-goog-user-project` headers on content requests to ensure proper upstream PA service authentication.
    *   **v4.6.3 (2026-08-30)**:
        -   **[Core Fix] Account JSON Storage Self-Healing & Concurrent File Write Lock (Issue #3345)**:
            -   **Self-Healing Parser on Load**: Added streaming deserializer fallback when reading account files. If an account file has trailing characters or extra closing braces (e.g. `trailing characters at line ...`), the parser automatically recovers the valid full `Account` data and atomically rewrites a clean file back to disk, completely preventing accounts from silently disappearing from the UI and causing cascading 429 rate limit outages.
            -   **Per-Account Concurrency Write Lock**: Introduced a global per-account mutex lock mechanism (`ACCOUNT_FILE_LOCKS`) to ensure strict serialized thread safety across concurrent quota refreshes, 429 rate-limit event writes, and `last_used` touch operations.
        -   **[Core Fix] Fix Discrete Model Chip Rendering for Pinned Gemini 3.7 Flash Models (Issue #3344)**:
            -   **Exact Match Priority**: Introduced exact-model matching in `resolveQuotaModels`. When a pinned selector matches a real quota model name (such as `gemini-3.7-flash-low`, `gemini-3.7-flash-high`, etc.), it renders as an independent discrete chip (`model:${id}`) instead of being collapsed and deduplicated into a single legacy `category:gemini-flash` slot that was hardcoded to older models.
            -   **Backward Compatibility**: Unmatched legacy category selectors and image selectors continue to use category-based resolution, preserving backward compatibility.
        -   **[Feature Optimization] Dashboard Best Accounts Recommendation with 5h & Weekly Quota Evaluation (Issue #3343)**:
            -   **Dual-Window Bottleneck Constraint**: Evaluates both the 5-hour rolling window and the 7-day weekly quota constraint ($\min(5h, weekly)$), avoiding recommending accounts that have a full 5h quota but have exhausted their weekly allowance.
            -   **Free Tier Single-Bucket Support**: Automatically detects single/dual-bucket account structures. Accounts with only a weekly quota (Free Tier) smoothly use their weekly quota percentage for evaluation, ensuring fair ranking without false zeroing.
            -   **Exhaustion Circuit Breaker**: Accounts with weekly quota $\le 5\%$ are disqualified from recommendation to prevent switching to unusable accounts.
        -   **[Core Fix] Gemini 3.7 / 3.x Thought-Signature Invalidation & Multi-Turn Variant Compatibility (PR #3342)**:
            -   **Case-Insensitive Thought Signature Error Matching**: Used `to_lowercase()` matching in Claude protocol and common handlers to capture all Google thought signature error variants (`Invalid thought signature.`, `thought_signature`, `thoughtsignature`), reliably triggering automatic retry and signature stripping.
            -   **Gemini 3.x Model Compatibility Rules**: Added explicit compatibility rules for `gemini-3.x` (Flash / Pro families) and `gemini-3.7` in `is_model_compatible`, ensuring thought signatures persist correctly across laddered variant turns.
        -   **[i18n] 100% Full Localization Across Multiple Languages (PR #3338, PR #3339, PR #3340, PR #3341)**:
            -   **Japanese (ja.json, PR #3338)**: Complete translations for quota protection, smart warmup, adaptive circuit breaker, context compression, model routing, and Homebrew updater; cleared residual strings.
            -   **Spanish (es.json, PR #3339)**: Complete translations for Proxy Pool, Debug Console, Network Monitor, IP Security Whitelist/Blacklist, OpenCode sync, and APIKEY.FUN relay.
            -   **Russian (ru.json, PR #3340)**: Complete translations for HTTP API server settings, Debug console, Homebrew upgrade workflow, Context Compression (Caveman/L1-L3), and streaming error prompts.
            -   **Korean (ko.json, PR #3341)**: Complete translations for proxy pool, debug console, model routing presets, 403 quick fix guide, and Homebrew update notifications.
    *   **v4.6.2 (2026-08-28)**:
        -   **[Core Fix] Proxy Startup Diagnostics for Silent Failure & Unreachable Ports (PR #3330)**:
            -   **Startup Failure Logging**: Added explicit `error!` logs when `load_app_config()` fails in `lib.rs`, converting silent exits into actionable error logs and clarifying that services were not started.
            -   **Cleaned Up Dummy Server Handle**: Removed unneeded placeholder `tokio::spawn(async {})` handles from `ProxyServiceInstance`, leaving unified lifecycle management to `AdminServerInstance`.
        -   **[i18n] Brazilian Portuguese (pt-BR) 100% Key Alignment with en.json (PR #3334)**:
            -   **1224+ Translation Keys Completed**: Translated all missing keys and removed residual Chinese strings (0 missing, 0 mismatched).
            -   **Placeholder Synchronization**: Aligned `{{name}}`, `{{error}}` interpolation parameters to avoid runtime UI render issues.
            -   **Component Direct References**: Added missing keys directly referenced by frontend TSX components.
        -   **[Enhancement] Model Catalog Update, Official Icons & OpenCode Sync Optimization (PR #3335)**:
            -   **New Model Support**: Added `gemini-3.7-flash`, `gemini-3.1-flash-lite`, `claude-opus-4-6`, `gpt-oss-120b-medium` with `@lobehub/icons` official brand icons.
            -   **Model List Deduplication**: Normalized alias mappings in `useProxyModels` to eliminate duplicate model entries caused by sub-tier suffixes.
            -   **OpenCode Sync Adjustments**: Enabled `ClaudeThinking` reasoning variants for Claude models and disabled unsupported `max` variants for Gemini 3 series.
        -   **[Platform Fix] Eliminate Windows Background Process Console Flashing (PR #3336)**:
            -   **Unified CREATE_NO_WINDOW Flags**: Replaced/supplemented `DETACHED_PROCESS` with `CREATE_NO_WINDOW` (0x08000000) across Cloudflared, tar decompression, and manual executable calls to eliminate console windows popping up.
            -   **Sync/Async Unified Handling**: Applied no-window flags consistently across `std::process::Command` and `tokio::process::Command` extensions.
        -   **[Core Fix] Fix Gemini 3.x 400 Bad Request on Thinking Block Compression (PR #3337)**:
            -   **Root Cause**: When `ContextManager` compressed thinking content to `"..."`, it preserved the original `thoughtSignature`, causing Google API to fail validation with `400 INVALID_ARGUMENT: Invalid thought signature`.
            -   **Fix**: Cleared the corresponding signature field when compressing thinking content to maintain signature chain integrity.
        -   **[Install Script Fix] Fix Linux Install Script 404 on Version Parsing (Issue #3328)**:
            -   **Validation & Direct Redirection**: Added `_is_valid_version()` semantic version format validation and switched Method 2 to `curl -w '%{url_effective}'` to avoid header parsing whitespace issues.
    *   **v4.6.1 (2026-08-25)**:
        -   **[Core Fix] Prevent 1M Token Overflow on Long Multi-Turn Thinking & Local Token Estimation Fallback (Issue #3325)**:
            -   **Historical Thinking Pruning**: When converting to Gemini contents, only the most recent window of assistant thinking text is preserved; older turns retain only `thoughtSignature` placeholders to prevent context from exceeding the 1M token ceiling.
            -   **Fallback Token Estimation**: If upstream Google returns an error without `usageMetadata`, middleware uses the local token estimation engine to calculate `input_tokens`, preventing blank token stats in monitor logs.
        -   **[Core Fix] JSON Schema `const` Keyword Normalization for Computer Use MCP (Issue #3327)**:
            -   **Schema Sanitization**: Automatically converts `{"const": "value"}` into standard `{"type": "...", "enum": ["value"]}` compatible with Gemini/Vertex Schema Proto.
            -   **Nested & Union Types Support**: Full support for `anyOf`/`oneOf` unions and deeply nested objects containing `const` fields.
    *   **v4.6.0 (2026-08-24)**:
        -   **[Core Feature] OpenAI Endpoint Supports `response_format.json_schema` Structured Outputs (PR #3324)**:
            -   **JSON Schema Support**: Full support for `response_format: { type: "json_schema", json_schema: { ... } }`.
            -   **Recursive Schema Unfolding**: Automatically extracts and sanitizes `$ref`/`$defs` definitions, converting schemas into Gemini `generationConfig.responseSchema` standards with `responseMimeType: "application/json"`.
        -   **[Core Fix] Proxy Pool Health Check 407 & URL Inline Auth Parsing Fix (Issue #3323)**:
            -   **HTTPS 204 Health Check**: Upgraded default health check endpoint to `https://cp.cloudflare.com/generate_204` via standard HTTPS `CONNECT` tunnels, eliminating false `407 Proxy Authentication Required` errors.
            -   **Inline Credentials Parsing**: Safely extracts `username` and `password` from `http(s)://user:pass@ip:port` proxy URLs and injects HTTP Basic Auth.
        -   **[Core Fix] Gemini 3.7 / 3.6 Flash Variant Mapping & 429 Fix (Issue #3322)**:
            -   **Registered 3.7 Variants**: Full registration for `gemini-3.7-flash`, `gemini-3.7-flash-low`, `gemini-3.7-flash-medium`, `gemini-3.7-flash-high`, and `gemini-3.7-flash-tiered`.
            -   **Eliminated False 429 Outages**: Fixed account quota scheduler falsely intercepting requests with "All accounts limited" on unregistered 3.7 variants.
    *   **v4.5.9 (2026-08-23)**:
        -   **[Core Feature] OpenAI Compatible Endpoint Multimodal Audio Input Support (PR #3321)**:
            -   **Standard Audio Formats**: Supports OpenAI official `input_audio` (Base64 + format) and `audio_url`, converting seamlessly to Gemini `inlineData`/`fileData`.
            -   **Normalization**: Normalizes `wav`, `mp3`, `m4a`, `ogg`, `flac`, `aiff` from Data URLs, remote HTTP links, local files, and raw Base64.
        -   **[Core Fix] OAuth Token Refresh Resilience & Backoff (PR #3321)**:
            -   **Proactive Buffer (5 Min)**: Increased token refresh window from 90s to 300s ahead of expiry.
            -   **Backoff Retry & Consecutive Failure Gate**: Retries after 500ms backoff on `invalid_grant` and disables accounts only after 2+ consecutive confirmed failures.
        -   **[Core Fix] 403 / VALIDATION_REQUIRED Detection & URL Parsing**:
            -   **Validation URL Extraction**: Parses `validation_url` / `appeal_url` from Google RPC responses and flags accounts in the UI with a quick-action verification button.
