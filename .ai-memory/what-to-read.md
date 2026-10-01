# What to Read

> Canonical map of what the AI must read before working on this project.
> Last updated: 2026-10-01T14:40:00Z

## Changelog

- 2026-10-01T14:40:00Z, Plan 92 started: CLI instance switch must use a new copy and must not close Cursor or `agm-alim.exe`. Restore now keeps `queued` prompts queued and marks the previously running prompt `backed_up` so dispatch sends it again (`backup_prompts_db.rs`, `repo_db.rs`). `close_instance` refuses Cursor and AGM PIDs. Spec `02-spec/21-app/92-cli-instance-switch-e2e.md`. CI was green on `78f4a589` before this change.
- 2026-10-01T09:00:00Z, Memory write: session lessons (CI gates, `gh` fork default, `workflow` scope, E2E buffering) in `memory/learned/22-cicd-gates-auto-commit-workflow-and-e2e-lessons.md`; maintainer preference "always commit and push, never ask" in `user-preferences/01-auto-commit-and-push.md`; suggestions 04 (CI/CD and release hardening) and 05 (codebase assessment) in `suggestions/`; pending plan 91 (Ubuntu pin before 2026-10-19, Node 24 actions, release gated on full CI success). README version badges and changelog summary synced to `v4.109.4`.
- 2026-10-01T08:25:00Z, Installer asset verification & partial release resilience (RCA 42): resolved 404 download errors in `install.ps1` and `install.sh` caused by guessed asset URLs on partial releases (`v4.109.2`–`v4.109.4` lacking Windows/Linux assets). Implemented `Test-UrlReachable` / `test_url_reachable` pre-flight checks, verified asset parsing from API metadata (`Get-WindowsReleaseAsset` / `parse_github_releases_py`), filtered candidate lists across all discovery tiers to exclude releases missing target platform binaries, added `--dry-run` to `install.sh`, and documented in RCA 42 (`.ai-memory/cicd-issues/42-installer-unverified-asset-urls-and-partial-release-rca.md`) and plan 90.
- 2026-10-01T07:55:00Z, Issue 56 resolved: maintainer granted the `workflow` token scope; `release.yml` rustfmt gate is live on `origin/main` (`6b18ddc4`). Workflow edits (Node 24 action majors, Ubuntu 26 pin before 2026-10-19) are no longer credential-blocked.
- 2026-10-01T07:40:00Z, CI/CD memory accuracy audit (plan 89 re-run): CI green on `bd60e569`; added missing RCAs 35–39 and a recurring-failure-class table to `.ai-memory/cicd-index.md`; appended "Bypassing CI Gates or Tagging on Unverified CI" ban to `strictly-avoid.md`; opened issue 56 (release rustfmt gate unpushed: token lacks `workflow` scope); fixed `33-test-inventory-generator.py --record` crash on the synced manifest schema (RCA 41). Read `.ai-memory/cicd-index.md` "Recurring Failure Classes" before committing or tagging.
- 2026-10-01T01:15:00Z, CI/CD Rustfmt Drift Recovery & Git Hook Gate: resolved recurring cargo fmt check failure in `src-tauri/src/bin/agm.rs:8612` (`cmd_instances`), documented RCA 40 (`.ai-memory/cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md`), wired tracked `.githooks/pre-commit` to `npm install` (`scripts/install-git-hooks.mjs`), committed a `release.yml` rustfmt gate locally (not pushed; see issue 56), and added total ban on single-line long iterator chains to `strictly-avoid.md`.
- 2026-09-30T12:00:00Z, Plan write: spec 85 and pending plans 85-88 (UI/tier/focus/stack trace, update asset verification, per-instance prompt continuity + auto-switch CLI scheduler, email dedupe + Supabase push-settings + vault scripts); issues 47-54, cicd-issue 39, ambiguities 02-07, learned memory 21. Read `02-spec/21-app/85-*.md` before touching these areas.
- 2026-09-28T18:39:00Z, 3-Stage Self-Delegating CLI Update Pipeline (`agm-update-cli` -> `agm update` -> `agm open-ui`): resolved UI update restarting into the old un-updated GUI binary by adding `delegate-update`, `update`, and `open-ui` CLI handlers to `src-tauri/src/modules/cli.rs`, implementing `prepare_isolated_update_cli` (`%TEMP%\agm-updater\agm-update-cli-<pid>.exe` + `%LOCALAPPDATA%\agm-cli\agm-update-cli.exe`), `spawn_delegated_update_cli` (direct Win32 `CREATE_NEW_CONSOLE` without `cmd.exe /c start` quote mangling), `run_cli_update`, and `open_ui` in `src-tauri/src/modules/delegate_updater.rs`, and scheduling an unconditional 650ms post-IPC `std::process::exit(0)` in `run_installer_update` (`src-tauri/src/modules/update_checker.rs`) so `tray_enabled` never blocks UI shutdown.
- 2026-09-28T18:25:00Z, GitMap CLI Parity, Dual Tree View (`[AGM:P001 | GM:#1]` / `[AGM:C001 | GM:<cid>]`), Instance/Node Prompting & Multi-Instance Swap Isolation: synchronized latest GitMap AGY (`active`, `running-prompts ls|backup|restore`, `prompt -n`, `prompt-project`, `fpug`, `sug`, `rerun`) and SSH (`nodes`, `exec`, `update agm`) commands across Telegram `/help`, AGM CLI (`agm tree`, `agm agy`, `agm instances assign`), Settings UI, and Email Cheat Sheet; implemented bracketed Project -> Conversation -> 200-Word Prompt Tree View with dual AGM & GitMap Sequence IDs and `--instance` / `--node` prompt targeting; hardened multi-instance multi-project workspace binding and OS Keyring / `current_account_id` account swap isolation (`02-spec/21-app/02-gitmap-agm-tree-instance-swap-spec.md`, `.ai-memory/plans/completed/02-completed-gitmap-agm-tree-instance-swap.md`).
- 2026-09-27T18:05:00Z, Supabase Connection Probe & URL Normalization: resolved E1002 404 connection test error by implementing canonical URL normalization (stripping trailing slashes and /rest/v1 suffixes) and 3-stage resilient probe ladder (REST root -> Table probe with PGRST detection -> /auth/v1/health fallback); decoupled test probe rejections from global error store (`02-spec/21-app/60-supabase-connection-probe-and-url-normalization.md`, `02-spec/22-app-issues/21-supabase-connection-test-404-rca.md`, `.ai-memory/plans/completed/80-supabase-connection-test-404-fix.md`).
- 2026-09-26T22:30:00Z, CLI & Auto-Switcher Deep Verification Audit: verified and hardened all native AGM CLI commands (`ff`, `status`, `credits`, `swlc`, `sfc`, `ilc`, `wpr`, `rerun prompts`, `prompts ls/pe/pi`, `email ls/add/rm/mv/export/import/status/help`, `clear-cache`, `instances ls/ff/instances-all/rm/create --data-only/rm-all`, `prompt`, `recreate-project`, `recreate`); audited Auto-Switcher 98% threshold evaluation, fallback profile selection, reactive 5-second interval loop, and notification hub invariants across Email HTML table, JSON block, and Telegram (`02-spec/21-app/49-cli-and-autoswitch-deep-verification-and-execution-audit.md`, `.ai-memory/plans/completed/51-cli-and-autoswitch-deep-verification-and-execution-audit.md`).
- 2026-09-26T22:15:00Z, CLI Verification & Telemetry Invariants: hardened Previous vs Predicted Next vs Selected account resolution across all switch alerts, stdout, and JSON; added credit before switch and threshold activated fields; confirmed running prompts count and automatic prompt resend status (`prompts_resent`) via `.antigravity_resume_task.json` with Base64 image payload preservation (`has_images`); updated `agm status`, `agm switch-if-low-credit`, `agm is-low-credit-for-switch`, `agm ff`, and `agm email status` (`02-spec/21-app/48-comprehensive-cli-and-autoswitch-verification.md`, `.ai-memory/plans/completed/50-comprehensive-cli-and-autoswitch-verification.md`).

- 2026-09-26T21:45:00Z, Email Telemetry & Low-Credit Switch CLI: enriched email and Telegram switch alerts with From-To account integrity (`from_email -> to_email`), active running prompt snippet, reinjection status (`is_reinjecting`), and image payload attachment status in HTML table and JSON state machine; added `-f` file export in `agm switch-if-low-credit` (`swlc`) defaulting to `agm-<node_alias>-switch.json`; implemented `agm is-low-credit-for-switch` (`ilc`) returning boolean in plain mode and machine telemetry in `--json [-f]` mode (`02-spec/21-app/47-email-prompt-telemetry-and-low-credit-switch-query.md`, `.ai-memory/plans/completed/49-email-prompt-telemetry-and-low-credit-switch-query.md`).
- 2026-09-26T21:30:00Z, UI Smart Fast-Forward & Instance Hardening: implemented reactive BackgroundTaskRunner Auto-Switcher effect triggering smartRotateProfileAccount on 98% threshold slider adjustments, wired switch notification telemetry into default instance switches, persisted live quota to disk immediately upon fetch, wrote device_profile to storage.json for isolated instances, and added 1-based positional resolution in resolve_instance_id (.ai-memory/plans/completed/48-ui-smart-fast-forward-autoswitch-and-instance-hardening.md).
- 2026-09-26T20:25:00Z, CLI & Auto-Switcher Deep Verification: completed minimum-bottleneck quota evaluation, 5s reactive threshold loop, `[JSON]` switch telemetry state machine (`old_email`, `instance_mode`, `condition`), `agm email status/help` dispatch, `agm instances create --data-only(do)`, and `agm recreate-project` conversation cleanup & bootstrap prompt seeding (`02-spec/22-app-issues/12-auto-switcher-quota-and-instance-rotation-rca.md`).
- 2026-09-23T13:00:00Z, Memory write: installer multi-version fallback ladder (10 releases), release asset decoupling, quiet aria2c delegation, and bottom-bar reactive update trigger recorded in learned/20-installer-multi-version-fallback-ladder-and-release-blocks.md.
- 2026-09-22T07:25:00Z, Skills Suite & Memory write: created 6 specialized domain skills for Antigravity-Manager (agm-proxy-engine, agm-thinking-store, agm-multi-instance-sandboxing, agm-split-sqlite-architecture, agm-frontend-react-tauri, agm-email-remote-control) and recorded learned/17-agm-dedicated-domain-skills-suite.md.
- 2026-09-19T02:00:00Z, Memory write: v4.18.0 email management, split security vault DB, bidirectional remote control, 20 CI/CD RCAs, and quality resilience recorded in learned/16-v4-18-0-email-management-split-security-db-and-pipeline-resilience.md.
- 2026-09-18T09:00:00Z, Memory write: v4.17.0 architecture, SQLite L2 tool signatures, test suite resilience, 17 CI/CD RCAs, and .ai-memory migration recorded in learned/15-v4-17-0-architecture-sqlite-tool-signatures-and-test-resilience.md.
- 2026-09-17T14:00:00Z, Memory write: comprehensive project context, v4.14.0 release state, 10 recent git commits, and CODE RED rules recorded in learned/14-comprehensive-project-context-and-v4-14-0-state.md.
- 2026-09-15T15:30:00Z, Architecture Realignment & Release: consolidated canonical prompts to 01-prompts/, migrated technical specs to 02-spec/21-app/ (08 through 14), implemented standalone portable installers (install.ps1, install.sh), and released v4.9.0.
- 2026-09-15T11:15:00Z, Documentation & Plan write: authored multi-instance and UI specifications; created pending plans.
- 2026-09-15T11:00:00Z, Documentation & Memory write: authored 01-instructions/ architecture guides and Mermaid diagrams for token capture, multi-instance isolation, and window customization; recorded learned memory in learned/13-refresh-token-capture-multi-instance-and-window-specs.md.
- 2026-09-15T10:50:00Z, Memory write: Go CLI AppError return type enforcement and centralized DRY help/argument checking architecture recorded in learned/12-go-cli-apperror-and-dry-help-handling.md, strictly-avoid.md updated, specs updated, and minor version bump to 4.8.0.
- 2026-09-10T23:00:00Z, Spec update: remediated 100% of Blind-AI audit v2 findings (F-001 through F-010), expanded IPC registry to 153 commands, bound 27 coding guidelines, grounded tests, and closed audit gap.
- 2026-09-10T19:22:00Z, Spec update: remediated 100% of Blind-AI audit findings across 02-spec/21-app/, 02-spec/23-app-db/, and 02-spec/24-app-ui-design-system/, closing audit gaps and archiving audit report.
- 2026-09-09T05:30:00Z, Prompt update: added mandatory inspection of last 10 git commits and what-to-read prioritization to read-memory-enhanced prompt and skill.
- 2026-09-09T05:00:00Z, Memory write: conversation log & context wrapper protocol, prompt staging, split SQLite logging, task retention, errcmd streaming, atomic file writes, and ApiManager spec.
- 2026-09-04T17:39:00Z, Memory write: parallel multi-worker CI/CD local runner, selective log filtering, streamwriter contracts, and naming standards.
- 2026-09-04T02:15:00Z, Ingested whole codebase, added write-memory and write-antigravity skills, and recorded 05-codebase-topology-and-skills-architecture.md.
- 2026-08-09T18:21:37Z, Memory write: code red refactor and strict absolute path avoidance.

## Before any task (always)

- `git log -n 10 --stat`, why: inspect the last 10 commits to understand recent file changes, what code/docs were touched, and the latest repository state before starting any task
- `.ai-memory/what-to-read.md`, why: authoritative prioritized reading sequence that must be read and followed before touching any files
- `version.json`, why: single source of truth for the repository version, backend/frontend sections, and sub-package version tracks. All codebases must import this file for version information.
- `.ai-memory/memory/01-index.md`, why: core memory index
- `.ai-memory/memory/learned/01-project-context-and-guidelines.md`, why: canonical learned memory of repo identity, CODE RED rules, coding guidelines, error philosophy, and active plans
- `.ai-memory/memory/learned/03-parallel-cicd-runner-and-log-filtering.md`, why: parallel local runner concurrency, duration tracking, and log suppression standard
- `.ai-memory/memory/learned/04-streamwriter-contracts-and-naming-standards.md`, why: streamwriter contracts, reentrant locker, monadic Bytes[T], JsonResult multi-source ingestion, boolean prefixes, and Id naming standard
- `.ai-memory/memory/learned/05-codebase-topology-and-skills-architecture.md`, why: comprehensive topology ingestion, 28 AI Python scripts catalog, Antigravity skills inventory, and CI/CD quality gate enforcement
- `.ai-memory/memory/learned/07-split-sqlite-logging-and-task-db-migration.md`, why: split SQLite logging architecture and task DB migration contracts
- `.ai-memory/memory/learned/08-task-retention-streaming-atomic-apimanager.md`, why: task retention, live line streaming, atomic file writes, and ApiManager spec
- `.ai-memory/memory/learned/09-conversation-log-and-context-wrapper-protocol.md`, why: conversation log persistence protocol and prompt staging boundary
- .ai-memory/memory/learned/11-antigravity-manager-workspace-onboarding.md, why: Antigravity-Manager workspace onboarding, proxy architecture, and recent commit history
- .ai-memory/memory/learned/12-go-cli-apperror-and-dry-help-handling.md, why: Go CLI AppError enforcement (*appfault.AppError) and centralized DRY help checking
- .ai-memory/memory/learned/13-refresh-token-capture-multi-instance-and-window-specs.md, why: authoritative token capture pathways, multi-instance profile isolation, and window customization specs
- .ai-memory/memory/learned/14-comprehensive-project-context-and-v4-14-0-state.md, why: comprehensive project context, v4.14.0 release state, 10 recent git commits, and CODE RED rules
- .ai-memory/memory/learned/15-v4-17-0-architecture-sqlite-tool-signatures-and-test-resilience.md, why: v4.17.0 architecture, SQLite L2 tool signatures, test suite resilience, 17 CI/CD RCAs, and .ai-memory migration
- .ai-memory/memory/learned/16-v4-18-0-email-management-split-security-db-and-pipeline-resilience.md, why: v4.18.0 email management, split security passwords DB, 20 CI/CD RCAs, and quality resilience
- .ai-memory/memory/learned/17-gitmap-search-and-llm-commands-protocol.md, why: elimination of broad recursive OS filesystem scans in favor of GitMap indexed search
- .ai-memory/memory/learned/18-ruby-homebrew-cask-architecture.md, why: Homebrew Cask package definition architecture and release distribution
- .ai-memory/memory/learned/19-agm-dedicated-domain-skills-suite.md, why: dedicated AGM domain skills suite covering reverse proxy, thinking store, multi-instance sandboxing, split SQLite databases, React UI, and email remote control
- .ai-memory/memory/learned/20-installer-multi-version-fallback-ladder-and-release-blocks.md, why: installer multi-version fallback ladder, quiet aria2c delegation, release asset decoupling, and bottom-bar update trigger
- `02-spec/21-app/01-index.md`, why: master index of application specifications, architecture guides, token capture sequences, and multi-instance blueprints
- `.ai-memory/memory/standards/version-source-of-truth.md`, why: mandatory standard for version.json single source of truth, 'inherit' keyword for sub-packages, and release sync workflow
- `.ai-memory/memory/01-index.md`, why: architectural map of version propagation, sync pipeline, and release ceremony
- `.ai-memory/coding-guidelines.md`, why: baseline rules and coding standards
- `.ai-memory/plans/01-index.md`, why: active roadmap and pending tasks
- `.ai-memory/strictly-avoid.md`, why: hard constraints and anti-patterns
- `.ai-memory/question-and-ambiguity/01-new-ambiguity/`, why: open questions
- `03-ai-scripts/01-index.md`, why: inventory and usage guidelines for automation tools and local CI runners

## Before writing code

- `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`, why: authoritative specification for email dispatch, split security passwords vault, and remote execution bridge
- `02-spec/21-app/17-email-intelligence-acknowledgment-and-universal-import-export.md`, why: authoritative specification for inbound email command intelligence, fuzzy typo matching, immediate acknowledgment receipts, and universal settings import/export with reversible multi-pass Base64 obfuscation
- `02-spec/21-app/`, why: complete reverse-engineered and remediated application architecture, proxy protocols, SQLite schemas, and frontend UI specs
- `spec/`, why: understand feature specifications

## Before adding a feature

- `spec/`, why: ensure it fits within existing specs

## Before writing a spec

- `02-spec/01-spec-authoring-guide/`, why: follow authoring format

## Before adding a unit test

- `02-spec/02-coding-guidelines/`, why: testing conventions

## See also

- Root `readme.md` (must stay in sync with this file)
- .ai-memory/plans/01-index.md
- .ai-memory/plans/pending/02-slides-system-overhaul.md
- .ai-memory/plans/pending/04-guideline-prompt-and-installer-upgrade.md
- .ai-memory/plans/pending/09-update-prompts-and-release.md
- .ai-memory/plans/pending/11-code-red-refactor-remediation.md
- .ai-memory/plans/completed/01-repository-hygiene-scripts-and-versioning.md
- .ai-memory/plans/completed/02-cicd-pipeline-and-quality-automation.md
- .ai-memory/plans/completed/03-appfault-result-monad-and-error-architecture.md
- .ai-memory/plans/completed/04-typecast-results-and-verification-systems.md
- .ai-memory/plans/completed/05-fileutil-pathinfo-constants-and-io-architecture.md
- .ai-memory/plans/completed/06-enum-architecture-generator-and-baseenumer.md
- .ai-memory/plans/completed/07-applogger-taxonomy-streaming-and-task-db.md
- .ai-memory/plans/completed/08-completed-plans-consolidation.md
- .ai-memory/plans/completed/12-spec-remediation-completed.md
- .ai-memory/plans/completed/16-repo-structure-installers-and-release.md
- .ai-memory/plans/completed/25-email-management-split-security-db-and-remote-control.md
