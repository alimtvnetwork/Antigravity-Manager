# Canonical Folder Structure Specification

> **Target:** `.lovable/folder-structure.md`
> **Authority:** Single Source of Truth for `.lovable/` Ecosystem & Repository Organization
> **Version:** 4.0.0

This document defines the authoritative canonical structure of the repository, including the `.lovable/` AI metadata layer and the `spec/` specification tree. All AI agents MUST strictly adhere to this architecture when reading, indexing, or writing files.

---

## 1. The `XX-<slug>` Deterministic Prefixing System

To maintain deterministic sorting and avoid ambiguity across operating systems, every file and directory within structured modules (`.lovable/plans/`, `01-prompts/`, `03-ai-scripts/`, `spec/`) MUST follow the two-digit zero-padded numeric prefix:

- **Format:** `XX-<slug-name>.ext` or `XX-<slug-name>/`
- **Prefix Range:** Starts at `01-` (or `00-` for root configuration indexes) and increments sequentially (`01-`, `02-`, ...).
- **Lowercase Mandate:** Strictly lowercase ASCII (`[0-9a-z_-]`). No uppercase letters, spaces, or camelCase.
- **Rule:** When creating a new file or subtask, identify the highest existing `XX-` prefix in that directory and increment by 1.

---

## 2. Complete `.lovable/` Directory Architecture

```
.lovable/
├── 01-overview.md                       # Comprehensive repository onboarding for AI agents
├── 02-user-preferences                  # Explicit user preferences & communication style
├── 03-strictly-avoid.md                 # Universal hard prohibitions & CODE RED constraints
├── 04-suggestions.md                    # Pending architectural suggestions & enhancements
├── 05-plan.md                           # Active high-level roadmap and milestone tracker
├── 06-what-to-read.md                   # AI reading order and entrypoint router
│
├── ai-fix-scripts/                      # Persistent high-speed Python automation toolchain
│   ├── 01-index.md                      # Master catalog, command syntax, and tag registry
│   ├── 02-shared-engine.py              # Central engine: constants, regex registry, locks, cache
│   ├── 03-file-manipulator.py           # Case conversion, re-sequencing, and UTF-8 LF fixer
│   ├── 04-newline-fixer.py              # Newline normalizer & trailing whitespace stripper
│   ├── 05-guideline-autofixer.py        # Composite guideline runner (newlines + booleans)
│   ├── 06-cicd-local-runner.py          # Parallel local CI runner for 18 quality gates
│   ├── 07-relative-path-fixer.py        # Absolute filesystem path & file:/// URI sanitizer
│   ├── 08-naming-autofixer.py           # Lowercase & boolean convention linter
│   ├── 09-cli-help-auditor.py           # CLI help parser and command validator
│   ├── 10-encoding-normalizer.py        # Multi-file UTF-8 LF and BOM cleaner
│   ├── 11-fast-file-scanner.py          # High-speed file indexer & cache query engine
│   ├── 12-fast-cached-grep.py           # Parallel regex grep leveraging pre-warmed cache
│   ├── 13-file-size-guard.py            # Large binary blob (>2MB) threshold auditor
│   ├── 14-version-sync-checker.py       # Version parity guard across manifest files
│   ├── 15-sequence-and-title-auditor.py # Sequence gap and H1 header alignment auditor
│   ├── 16-installer-smoke-tester.py     # Shell/PowerShell installer smoke tester
│   ├── 17-fast-file-reader.py           # AI fast file reader and folder explorer (<1ms)
│   ├── 18-codebase-topology-discoverer.py # Polyglot topology routing and discovery
│   ├── 19-artifact-remover.py           # Safe interactive artifact cleaner with git untracking
│   └── 20-plan-consolidator.py          # Lovable plan consolidator and backup generator
│
├── plans/                               # Micro-task execution center
│   ├── 01-index.md                      # Master registry of active and completed plans
│   ├── pending/                         # High-level active parent plans (e.g., 01-auth.md)
│   ├── subtasks/                        # Bounded micro-tasks grouped by parent slug
│   │   └── XX-<parent-slug>/            # Sequential subtasks (01-step.md, 02-step.md)
│   └── completed/                       # Archived completed parent plans & subtasks
│
├── memory/                              # Institutional knowledge base (SINGULAR: memory/)
│   ├── 01-index.md                      # Master memory table of contents & CODE RED rules
│   ├── architecture/                    # System architecture decisions and split-DB patterns
│   ├── constraints/                     # Non-negotiable technical constraints
│   ├── done/                            # Historical milestone logs
│   ├── features/                        # Feature-specific knowledge and requirements
│   ├── issues/                          # Root cause analyses and bug postmortems
│   ├── patterns/                        # Reusable design and coding patterns
│   ├── processes/                       # Operational and maintenance procedures
│   ├── project/                         # Project metadata and author attributions
│   ├── standards/                       # Coding standards and enum specifications
│   ├── style/                           # Code style and naming conventions
│   ├── suggestions/                     # Granular suggestion trackers
│   └── workflow/                        # Sprint and migration state trackers
│
├── prompts/                             # AI Prompt Repository & Category Hierarchy
│   ├── 01-prompts-category/             # Canonical source for categorized prompt modules
│   │   ├── 01-general/                  # Base prompts and conversational behaviors
│   │   ├── 02-core-workflow/            # Unified workflow prompts (v1 through v4)
│   │   ├── 03-read-write/               # Memory read/write and Antigravity sync
│   │   ├── 04-coding-standards/         # Coding guideline enforcement
│   │   ├── 05-coding-guidelines/        # Guideline execution & autofix prompts
│   │   ├── 06-testing-and-qa/           # Autonomous QA, unit tests, and smoke tests
│   │   ├── 07-bug-fix/                  # Root cause analysis & structured bug fixing
│   │   ├── 08-dry-code/                 # DRY architecture, caching, & lazy regex
│   │   ├── 09-commit-and-multi-agent/   # Commit standards & multi-agent delegation
│   │   ├── 13-plan-audit/               # Plan auditing and spec verification
│   │   ├── 14-execute/                  # Task execution loops (batched, n-steps, writer)
│   │   ├── 15-cg-execute/               # Coding guideline category execution rules
│   │   ├── 17-ci-cd/                    # CI/CD pipeline diagnosis and runner scripts
│   │   ├── 18-release-management/       # Semver version bumping and release blueprints
│   │   ├── 20-memory-consolidate/       # Memory and completed plan consolidation
│   │   └── 22-ai-fix-script-prompts/    # Specifications for individual AI scripts
│   └── *.md                             # Synced flat prompts generated from categories
│
├── release/                             # Release automation & version bumping hub
│   ├── release-method.md                # Authoritative version bump registry
│   ├── bump_versions.py                 # Automated regex-based version mutator
│   └── issues/                          # Release failure logs and diagnostics
│
├── question-and-ambiguity/              # Ambiguity resolution & task counters
│   ├── readme.md                        # Open and resolved ambiguity log
│   └── task-counter.md                  # Task iteration metrics
│
├── suggestions/                         # Detailed suggestion proposals
│   ├── 01-index.md                      # Index of active suggestions
│   └── completed/                       # Archived implemented proposals
│
├── cicd-issues/                         # CI pipeline failure diagnostics and RCAs
└── assets/                              # User-provided mockups, diagrams, and media
```

---

## 3. Canonical `spec/` Specification Hierarchy

The `spec/` directory is organized into numbered tiers:

- **Tiers 01–20 (Core Fundamentals):** Reusable standards, architecture blueprints, database conventions, and design systems.
- **Tiers 21+ (App-Specific Modules):** Application business logic, features, and domain-specific schemas.

| Tier | Module Directory | Focus Area |
|:---:|---|---|
| **01** | `02-spec/01-spec-authoring-guide/` | Meta-guide on writing and maintaining specifications |
| **02** | `02-spec/02-coding-guidelines/` | Cross-language standards (TypeScript, Go, PHP, Rust, C#) |
| **03** | `02-spec/03-error-manage/` | Error envelopes, application errors, and error codes |
| **04** | `02-spec/04-database-conventions/` | Database schema conventions, ORM mappings, views |
| **05** | `02-spec/05-split-db-architecture/` | Split database pattern (operational vs. config DBs) |
| **06** | `02-spec/06-seedable-config-architecture/` | Seedable configuration with versioned changelogs |
| **07** | `02-spec/07-design-system/` | Design tokens, component hierarchies, dark mode |
| **08** | `02-spec/08-docs-viewer-ui/` | Documentation viewer UI and slide presentation system |
| **09** | `02-spec/09-code-block-system/` | Syntax highlighting pipeline and code block copy/run |
| **10** | `02-spec/10-research/` | Technology evaluations and benchmark studies |
| **11** | `02-spec/11-powershell-integration/` | Cross-platform PowerShell automation standards |
| **12** | `02-spec/12-cicd-pipeline-workflows/` | CI/CD workflows, GitHub Actions, and quality gates |
| **13** | `02-spec/13-generic-cli/` | Generic CLI UX, exit codes, and flag semantics |
| **14** | `02-spec/14-update/` | CLI self-update architecture and release delivery |
| **15** | `02-spec/15-distribution-and-runner/` | Cross-platform installers, runners, and forwarding |
| **16** | `02-spec/16-generic-release/` | Universal release pipeline blueprint and asset matrices |
| **17** | `02-spec/17-consolidated-guidelines/` | Single-file AI summaries of all major spec modules |
| **18** | `02-spec/18-wp-plugin-how-to/` | WordPress plugin architecture and REST APIs |
| **19** | `02-spec/19-main-worker-service/` | Split-tier worker service and proxy nodes |
| **21+**| `02-spec/21-app/` ... `02-spec/24-app-ui-design-system/` | App-specific workflows, UI components, and domain DBs |

---

## 4. Governance & Synchronization Invariants

1. **Strict Relative Paths:** All internal markdown links, citations, and subtask paths MUST use relative paths starting from the repository root (e.g. `.lovable/plans/pending/01-task.md`, `02-spec/02-coding-guidelines/01-index.md`). Absolute filesystem paths (`C:\...`, `/home/...`, `file:///...`) are strictly prohibited.
2. **Strict Lowercase:** All generated files and directories MUST be lowercase.
3. **Plurality Invariants:** Always `.lovable/memory/` (singular), `.lovable/plans/` (plural), `01-prompts/` (plural), `.lovable/suggestions/` (plural).
4. **Mirror Parity:** Every script in `03-ai-scripts/` MUST have an exact mirror in `.agents/scripts/`.
5. **Quality Gates:** Before concluding any engineering task, execute `python 03-ai-scripts/06-cicd-local-runner.py` to confirm all 18 quality checks pass.
