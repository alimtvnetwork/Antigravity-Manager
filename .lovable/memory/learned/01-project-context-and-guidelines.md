# Project Context and Guidelines Memory

> Canonical learned memory from comprehensive project ingestion.
> Ingested: 2026-09-02T15:21:00Z
> Status: Active

## 1. Project Identity & Architecture

- **Project:** Prompt Architect / Coding Guidelines (`alimtvnetwork/coding-guidelines-v24`).
- **Core Stacks:** Polyglot repository covering TypeScript/React (Vite + Tailwind + Radix UI), Go (`04-code/golang`), Python (`03-ai-scripts`, `linter-scripts`, `linters-cicd`), PHP, SQL (SQLite Split-DB Architecture).
- **Single Source of Truth:** `version.json` at root is the canonical version standard; all tools and scripts propagate from it via `npm run sync`.
- **Author & Ownership:** Md. Alim Ul Karim, Riseup Asia LLC.

## 2. Core CODE RED & Strict Avoidances

1. **Explicit True Checks (TOTAL BAN):** Never evaluate booleans explicitly against `true` (`if isReady == true` or `if (isValid === true)`). Evaluate implicitly: `if isReady { ... }`.
2. **Mixed Polarity (TOTAL BAN):** Never combine positive and negative checks in the same condition (`if isA && !isB`). Split into separate guard clauses.
3. **CI/CD Disabling (TOTAL BAN):** Never bypass, comment out, or disable CI/CD checks or linters. Fix the underlying code.
4. **Strict Lowercase File Naming:** All repository files, documentation, scripts, and plans must strictly use lowercase filenames (e.g. `readme.md`, `changelog.md`, `skill.md`).
5. **Strict Relative Git Paths:** Total ban on absolute filesystem paths (`C:\...`, `/home/...`) and `file:///` URIs inside markdown files, plans, and code comments.
6. **Go Error Handling Standard:** All Go packages returning structured error metadata must return `*appfault.AppError`. Package `appfault` (`04-code/golang/pkg/appfault`) is the standard; never swallow errors.
7. **readme.txt Timestamp Generator (TOTAL BAN):** Strictly prohibited from suggesting, discussing, building, or modifying any date/time/timestamp generator targeting `readme.txt`.
8. **No Commit of Test Artifacts / Binaries:** Keep `.gitignore` updated; never commit `.test-report.*`, test outputs, binaries, or temp artifacts.

## 3. Coding Guidelines & Language Standards

- **Boolean Prefixes:** Identifiers must strictly use `is` or `has` prefixes (e.g. `isValid`, `hasPermission`). Words like `can`, `should`, `was`, `did` are banned.
- **Nested If Flattening:** Maximum conditional nesting depth is 1. Flatten all logic using early returns and guard clauses.
- **Enum Standards:** All enum declarations across TypeScript, Go, and PHP must end with `Type` suffix (e.g., `UserRoleType`, `SeverityType`).
- **Vertical Formatting:** Blank lines mandatory before `if` statements, after closing braces `}`, before `return`, and around multiline struct definitions.
- **Size Constraints:** Functions preferred <= 8 lines, max <= 15 lines; files max <= 100 lines coding (recommend <= 80 lines).

## 4. Active Plans & Roadmap

- `01-apperror-new-constructors`: Go `appfault.AppError` constructor parity.
- `02-slides-system-overhaul`: Slides system redesign and section dividers.
- `03-apperror-human-logger-methods`: Human-readable logging on error types.
- `04-guideline-prompt-and-installer-upgrade`: Installer script enhancements.
- `05-rename-overviews-and-installer-json`: Renaming spec overviews.
- `06-fix-encoding`: UTF-8 LF normalization.
- `07-trailing-newlines-and-ai-scripts`: Trailing newline fixes across Python tools.
- `08-lowercase-changelog`: Ensure changelog casing and structure.
- `09-update-prompts-and-release`: Prompts alignment from meta-repo.
- `10-rca-and-boolean-fix`: Comprehensive boolean audit and root cause remediation.
- `11-code-red-refactor-remediation`: Remediation of code red violations.
- `12-prompt-architect-version-tracking`: Injection of prompt tracking metadata.
- `13-cicd-pipeline-consolidation-and-owner-review`: CI/CD pipeline consolidation.

## 5. Ingestion Metrics

- Total Markdown Files: 970
- Total TypeScript Files: 246
- Total Python Files: 224
- Total Go Files: 64
- Total JavaScript Files: 62
- Total SQL Files: 6
- Total PHP Files: 5
- Open Ambiguities: 0
- CI/CD Issues Ingested: 0
