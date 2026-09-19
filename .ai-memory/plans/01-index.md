# Plans Index

Master directory of architectural and execution plans.

## Pending Plans

- [04-guideline-prompt-and-installer-upgrade.md](pending/04-guideline-prompt-and-installer-upgrade.md): Guideline prompt and installer enhancements.
- [09-update-prompts-and-release.md](pending/09-update-prompts-and-release.md): Update prompts and release lifecycle (deferred under WOR policy).
- [11-code-red-refactor-remediation.md](pending/11-code-red-refactor-remediation.md): Remediate Code Red enum, boolean, and query wrapper violations across the codebase.

## Archived Plans

- [02-slides-system-overhaul.md](_archive/02-slides-system-overhaul.md): Full slides deck UI and system overhaul (external meta-repo archive).

## Completed Plans

- [01-repository-hygiene-scripts-and-versioning.md](completed/01-repository-hygiene-scripts-and-versioning.md): Repository hygiene, encoding normalization, lowercase conventions, AI scripts `<details>` documentation, and prompt tracking.
- [02-cicd-pipeline-and-quality-automation.md](completed/02-cicd-pipeline-and-quality-automation.md): CI/CD workflows consolidation, 12 reusable quality guards, and release skew RCA.
- [03-appfault-result-monad-and-error-architecture.md](completed/03-appfault-result-monad-and-error-architecture.md): Go AppError namespace constructors, human/logger display methods, Result[T] rich dynamic conversions, number parsing, reflection casting, deterministic map sorting, and monadic Result unwrapping.
- [04-typecast-results-and-verification-systems.md](completed/04-typecast-results-and-verification-systems.md): High-performance typecast, `ReflectSetTo` fast path, `Checker` interface family, `SimpleVerifier` parity, and coredata combinators.
- [05-fileutil-pathinfo-constants-and-io-architecture.md](completed/05-fileutil-pathinfo-constants-and-io-architecture.md): Modular file operations, path context injection, cross-platform temp hierarchy, `FilePathOps` bound struct, lock concurrency, constants centralization, and .NET-style FolderInfo/FileInfo/PathInfo architecture.
- [06-enum-architecture-generator-and-baseenumer.md](completed/06-enum-architecture-generator-and-baseenumer.md): Modular 1:1 enum isolation, dedicated packages, DRY JSON marshaling, Min/Max boundaries, leaf enum parse helpers, cycle elimination, and Python smart enum scaffolder CLI (`30-enum-generator.py`).
- [07-applogger-taxonomy-streaming-and-task-db.md](completed/07-applogger-taxonomy-streaming-and-task-db.md): Structured AppLogger, split SQLite DB logging (`logs.db` + isolated `tasks/<task-id>.db`), configurable rotating file sink, generic LazyOnce, errcmd streaming, task retention pruning, named writers, and typed streamers.
- [08-completed-plans-consolidation.md](completed/08-completed-plans-consolidation.md): Completed plans consolidation, pre-consolidation safety backups, milestone compaction, subtask collapse, and continuous resequencing.
- [12-spec-remediation-completed.md](completed/12-spec-remediation-completed.md): Remediated 100% of Blind-AI audit findings across `02-spec/21-app/`, `02-spec/23-app-db/`, and `02-spec/24-app-ui-design-system/`, resolving all gaps and archiving audit report.
- [13-spec-remediation-completed.md](completed/13-spec-remediation-completed.md): Remediated 100% of Blind-AI audit v2 findings (`F-001` through `F-010`), expanded IPC registry to 153 commands, bound 27 coding guidelines, grounded tests and acceptance criteria, and closed audit gap.
- [14-multi-instance-orchestration-and-ubuntu-parallelism.md](completed/14-multi-instance-orchestration-and-ubuntu-parallelism.md): Multi-Instance isolated profile supervisor, terminal CLI options, Linux process selective filtering via `/proc/<pid>/cmdline`, AppImage sanitization, GNOME Keyring bypass (`--password-store=basic`), and frontend Navbar / Instances view.
- [15-auto-quota-switcher.md](completed/15-auto-quota-switcher.md): Configurable polling timer, low-quota profile auto-switching (<10%), next-best profile ranking, pending QE task state snapshot/recovery, and UI settings / instances status controls.
- [16-repo-structure-installers-and-release.md](completed/16-repo-structure-installers-and-release.md): Folder structure realignment, 01-prompts sync, 02-spec/21-app migration, standalone zip installer scripts, and minor release v4.9.0.
- [17-cicd-pipeline-and-cross-platform-automation.md](completed/17-cicd-pipeline-and-cross-platform-automation.md): Ground local CI runner quality gates, fix documentation link integrity, and verify green quality gates.
- [18-release-page-one-liner-installers.md](completed/18-release-page-one-liner-installers.md): Formatted Rust codebase, enabled GitHub Pages via API, hardened install.ps1 and install.sh for one-liner execution, and automated Quick Install one-liners in GitHub Release notes and asset publishing.
- [19-error-management-and-modal-integration.md](completed/19-error-management-and-modal-integration.md): Full error management and global error modal integration across Rust backend and React frontend with one-click AI-sharable Markdown diagnostic reports.
- [20-ubuntu-ide-detection-and-readme-overhaul.md](completed/20-ubuntu-ide-detection-and-readme-overhaul.md): Ubuntu IDE auto-healing, storage JSON repair, and root README overhaul.
- [21-ubuntu-ide-diagnostics-and-multi-instance-cloning.md](completed/21-ubuntu-ide-diagnostics-and-multi-instance-cloning.md): Ubuntu IDE discovery diagnostics, std::backtrace capture, multi-instance isolation, cross-platform executable cloning, and v4.14.0 release ceremony.
- [22-agm-branding-ui-overhaul-and-v4-15-0-release.md](completed/22-agm-branding-ui-overhaul-and-v4-15-0-release.md): AGM branding, UI compactness, instances fix, and v4.15.0 release.
- [23-upstream-pipeline-sync-and-v4-16-0-release.md](completed/23-upstream-pipeline-sync-and-v4-16-0-release.md): Upstream PR #2 logical sync, streaming pipeline engine, thinking store, and v4.16.0 release.
- [24-auto-switch-quota-running-prompts-backup-and-ubuntu-fix.md](completed/24-auto-switch-quota-running-prompts-backup-and-ubuntu-fix.md): Auto-switch quota threshold update (15%), split repo DB running prompts backup & direct dispatch, Ubuntu switching fix, and root readme refactoring.
- [25-email-management-split-security-db-and-remote-control.md](completed/25-email-management-split-security-db-and-remote-control.md): Email dispatch, mailbox remote management, split security vault DB, and bidirectional remote control.
- [26-installer-exe-and-compact-navbar-menu-ui.md](completed/26-installer-exe-and-compact-navbar-menu-ui.md): Direct EXE installation, compact navbar menu button, and instance selector boundary protection.
- [27-single-menu-button-navbar-and-pure-exe-installer.md](completed/27-single-menu-button-navbar-and-pure-exe-installer.md): Single menu button navbar, space utilization, and pure EXE installer.
- [28-email-setup-page-and-installer-hardening.md](completed/28-email-setup-page-and-installer-hardening.md): Dedicated Email setup page, navbar dropdown boundary locks, and installer asset resolution.
- [29-eliminate-toolbar-hover-collision-and-z-index-hierarchy.md](completed/29-eliminate-toolbar-hover-collision-and-z-index-hierarchy.md): Eliminate toolbar hover collision and elevate navbar stacking context.
- [30-instance-edit-rename-feature.md](completed/30-instance-edit-rename-feature.md): In-place instance profile edit and rename feature across navbar dropdown and Instances page.
- [31-compact-top-padding-and-scrollable-views.md](completed/31-compact-top-padding-and-scrollable-views.md): Compact top padding and scrollable view containers for Email and Instances pages.
- [32-email-management-ui-streamlining-and-templates.md](completed/32-email-management-ui-streamlining-and-templates.md): Email management UI streamlining, telemetry integration, compact actions dropdown, and AI sample templates.
