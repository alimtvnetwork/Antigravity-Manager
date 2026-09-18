# What to Read

> Canonical map of what the AI must read before working on this project.
> Last updated: 2026-09-09T05:00:00Z

## Changelog

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
- `02-spec/21-app/01-index.md`, why: master index of application specifications, architecture guides, token capture sequences, and multi-instance blueprints
- `.ai-memory/memory/standards/version-source-of-truth.md`, why: mandatory standard for version.json single source of truth, 'inherit' keyword for sub-packages, and release sync workflow
- `.ai-memory/memory/01-index.md`, why: architectural map of version propagation, sync pipeline, and release ceremony
- `.ai-memory/coding-guidelines.md`, why: baseline rules and coding standards
- `.ai-memory/plans/01-index.md`, why: active roadmap and pending tasks
- `.ai-memory/strictly-avoid.md`, why: hard constraints and anti-patterns
- `.ai-memory/question-and-ambiguity/01-new-ambiguity/`, why: open questions
- `03-ai-scripts/01-index.md`, why: inventory and usage guidelines for automation tools and local CI runners

## Before writing code

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
