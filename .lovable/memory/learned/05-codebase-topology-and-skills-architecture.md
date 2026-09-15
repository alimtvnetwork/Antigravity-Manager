# Codebase Topology & Skills Architecture Memory

> Canonical learned memory from full repository discovery, topology inspection, and Antigravity skills inventory.
> Ingested: 2026-09-04T02:15:00Z
> Status: Active

## 1. Codebase Overview & Polyglot Metrics

The repository is the canonical home of Prompt Architect / Coding Guidelines (`alimtvnetwork/coding-guidelines-v24`).
A high-performance polyglot architecture covering:

- **Markdown (`995` files):** Specifications (`02-spec/`), consolidated guidelines (`02-spec/17-consolidated-guidelines/`), AI prompts (`01-prompts/`), task plans (`.lovable/plans/`), and memory records (`.lovable/memory/`).
- **TypeScript (`246` files):** Documentation viewer, interactive slides deck application (`slides-app`), and tooling manifests (`tsconfig.json`).
- **Python (`234` files):** 28 ultra-fast automation scripts (`03-ai-scripts/`, mirrored in `.agents/scripts/`), local CI/CD runners, AST linters, and checkers (`linter-scripts/`, `linters-cicd/`).
- **Go (`64` files):** Core architecture packages under `04-code/golang/pkg` (`appfault`, `applogger`, `result`, `errtype`) enforcing the `*appfault.AppError` standard.
- **JavaScript (`62` files):** Build, synchronization, and bundle generation scripts (`scripts/`).
- **SQL (`6` files):** SQLite schemas and migrations supporting hierarchical Split-DB architecture (`02-spec/05-split-db-architecture/`).
- **PHP (`5` files):** WordPress plugin architecture reference implementations (`02-spec/18-wp-plugin-how-to/`).
- **CSS / HTML (`7` files):** Design system styles and viewer roots.

## 2. AI Fix Scripts Tooling Catalog (`03-ai-scripts/`)

All scripts leverage `02-shared-engine.py` for thread-safe caching (<15ms full scan, <1ms cache query), lazy regex registries, and POSIX/Windows cross-process locking:

1. `02-shared-engine.py`: Centralized constants, enums, locks, regex registry, and I/O.
2. `03-file-manipulator.py`: Mass lowercasing, sequence fixing, UTF-8 normalization.
3. `04-newline-fixer.py`: Trailing whitespace and newline cleanup.
4. `05-guideline-autofixer.py`: Composite autofixer for guidelines and booleans.
5. `06-cicd-local-runner.py`: High-speed parallel runner executing all 21 CI quality gates.
6. `07-relative-path-fixer.py`: Enforces strict relative Git paths and scrubs `file:///` URIs.
7. `08-naming-autofixer.py`: Enforces boolean prefixes (`is`/`has`) and naming rules.
8. `09-cli-help-auditor.py`: Validates CLI `--help` flags and examples.
9. `10-encoding-normalizer.py`: Normalizes UTF-8 LF line endings.
10. `11-fast-file-scanner.py`: High-speed repository file scanner and cache builder.
11. `12-fast-cached-grep.py`: Multi-threaded pattern matcher over cache.
12. `13-file-size-guard.py`: Audits against oversized binary blobs (>2MB).
13. `14-version-sync-checker.py`: Ensures `version.json` matches manifests.
14. `15-sequence-and-title-auditor.py`: Audits sequence prefixes and `# H1` titles.
15. `16-installer-smoke-tester.py`: Smoke-tests install scripts across platforms.
16. `17-fast-file-reader.py`: Instantaneous cached file reader and explorer.
17. `18-codebase-topology-discoverer.py`: Polyglot language and subsystem topology mapper.
18. `19-artifact-remover.py`: Safe interactive removal of test dumps and binaries.
19. `20-plan-consolidator.py`: Consolidates and re-sequences `.lovable/` plans.
20. `21-sequence-integrity-linter.py`: Verifies numeric ordering across subtasks.
21. `22-doc-path-linter.py`: Lints documentation relative links.
22. `23-coding-guideline-path-consolidator.py`: Consolidates coding guideline references.
23. `24-spec-path-migrator.py`: Migrates legacy spec paths.
24. `25-repo-migrator.py`: Structural asset migrator.
25. `26-go-code-formatter.py`: Cross-platform `gofmt` execution wrapper.
26. `27-misspell-auditor.py`: Fixes British English to US English.
27. `28-go-preflight-ci.py`: Runs Go preflight checks and `golangci-lint`.

## 3. Antigravity Skills Inventory (`.agents/skills/`)

All prompt workflows are mapped to first-class Antigravity skills:

| Skill Slug | Primary Function | Source Prompt Reference |
|---|---|---|
| `autonomous-qa-and-testing` | Autonomous QA test execution and quality gate verification | `01-prompts/06-testing-and-qa/01-autonomous-qa-and-testing-v4.md` |
| `cg-boolean-and-naming` | Audits boolean naming (`is`/`has`), implicit checks, and enums | `01-prompts/15-cg-execute/04-booleans-and-complex-conditions.md` |
| `cg-error-management` | Universal error management and `*appfault.AppError` compliance | `01-prompts/15-cg-execute/02-error-management.md` |
| `ci-cd-create` | Cross-platform CI/CD pipeline and linter authoring | `01-prompts/16-ci-cd/05-cicd-pipeline-create.md` |
| `ci-cd-fix` | 4-Part RCA and pipeline failure diagnosis | `01-prompts/16-ci-cd/01-ci-cd-fix.md` |
| `clean-artifacts-and-git-history` | Cleans generated test reports, pycache, and preserves git hygiene | `01-prompts/09-commit-and-multi-agent-code-fix/07-clean-artifacts-and-git-history.md` |
| `coding-guidelines` | Core coding guideline review and zero-nesting enforcement | `01-prompts/04-coding-standards/01-coding-guidelines.md` |
| `execute-ai-instruction-writer` | Generates structured AI instructions and specifications | `01-prompts/14-execute/04-execute-ai-instruction-writer.md` |
| `execute-batched-loop` | Multi-agent batched loop execution with 3 sub-agents | `01-prompts/14-execute/03-execute-batched-loop.md` |
| `execute-batched-loop-wor` | Batched loop execution without release ceremony (WOR) | `01-prompts/14-execute/05-execute-batched-loop-wor.md` |
| `execute-coding-guideline-fix` | Micro-batch refactoring for boolean, nesting, and sizing rules | `01-prompts/05-coding-guidelines/02-execute-coding-guideline-fix.md` |
| `execute-parent-task-with-n-steps` | Decomposes and executes parent tasks across N sequential steps | `01-prompts/14-execute/02-execute-parent-task-with-n-steps.md` |
| `execute-pending-tasks` | Executes tasks from `.lovable/plans/pending/` | `01-prompts/14-execute/01-execute-pending-tasks.md` |
| `fix-with-rca` | Grounded 4-Part Root Cause Analysis bug fixing | `01-prompts/07-bug-fix/01-fix-with-rca.md` |
| `inventory-pending-tasks` | Discovers, catalogs, and sequences all pending plans and subtasks | `01-prompts/13-plan-audit/01-inventory-pending-tasks.md` |
| `plan-coding-guideline-audit` | Plans structured coding guideline audits against `02-spec/` | `01-prompts/05-coding-guidelines/01-plan-coding-guideline-audit.md` |
| `read-memory-enhanced` | Loads project identity, CODE RED rules, specs, and plans | `01-prompts/03-read-write/02-read-memory-enhanced.md` |
| `release-management` | SemVer version bumps, package sync, and release ceremony | `01-prompts/17-release-management/04-release.md` |
| `write-antigravity` | Authors and updates Antigravity agent configurations and rules | `01-prompts/03-read-write/01-write-antigravity.md` |
| `write-memory` | Persists session decisions, learned conventions, and index updates | `01-prompts/03-read-write/03-write-memory.md` |

## 4. Quality Gate Verification (21 Checks)

The local CI/CD pipeline (`python 03-ai-scripts/06-cicd-local-runner.py`) validates 21 parallel quality gates:
- Relative Path Check, Prompts Loaded Check, Readme Install Section Check, Forbidden Strings Check, Newline Styling Check (Python + MJS), Fast File Scanner Cache, File Size Guard, Version Sync Check, Bundle Installer Generation, Spec Tree Sync, Codegen Determinism Check, Spec Verification Coverage, Validate Version JSON, Doc Links Check, Check File Sizes Baseline, Spec Folder References Check, Sequence Integrity Check, Prompt & Spec Path Integrity Check, Linters CI/CD Test Suite, Go Base Test Suite.
- Current status: **All 21 checks passing (100% clean).**

## 5. Architectural Standards & Standards Alignment

- **Single Source of Truth:** `version.json` at root governs all version numbers, packages, and releases (`npm run sync`).
- **Strict Relative Git Paths:** TOTAL BAN on absolute filesystem paths or `file:///` URIs.
- **Strict Lowercase File Naming:** All repository files must be strictly lowercase.
- **Go Structured Errors:** Return `*appfault.AppError` from package `04-code/golang/pkg/appfault`.
- **Active Plans in Flight:** 13 pending plans in `.lovable/plans/pending/`.
- **Active Ambiguities:** 1 open ambiguity in `.lovable/ambiguous-questions/01-new-ambiguity/01-pluggable-logger-backend-and-uber-zap-migration.md` awaiting architectural choice on pluggable logger backends.
