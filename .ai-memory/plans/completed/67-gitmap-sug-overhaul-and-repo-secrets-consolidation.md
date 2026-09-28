# Plan 67: GitMap SUG Overhaul, Real-Time Web UI, Multi-Target PE & Repo Secrets Consolidation

Spec Reference: [02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md](../../../02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md)

## Summary
Overhaul the GitMap `sug` (Shutdown-Until-Green) subsystem to robustly handle direct paths, space-separated commands (`agy-running projects`), help interception on subcommands (`add-projects help`), target existence verification (path, alias, URL) with recovery guidance, watch monitoring with auto-clear on shutdown, a dedicated local real-time Web dashboard UI, path/alias support in `gitmap pe`, help categorization and search documentation, rename `01-git-map` to `01-gitmap` in `repo-secrets` with PascalCase SIO template variables and company/URL normalizations, commit-pull configuration boolean standardization, and a PowerShell test script for `D:\test-gitmap`.

## Deliverables Mapping
- **Task-01**: Ingest visual screenshot asset and author canonical specification 67.
- **Task-02**: GitMap SUG CLI tokenization, target existence validation, aliases, and subcommand help interception.
- **Task-03**: GitMap SUG watch mode loop, auto-clear on shutdown configuration, and real-time browser Web UI.
- **Task-04**: GitMap PE multi-target support for direct folder paths, aliases, and repo URLs.
- **Task-05**: Help menu group filtering, command suggestions, and search vs AUM search documentation.
- **Task-06**: Repo secrets `01-git-map` -> `01-gitmap` rename, VM JSON export, and SIO template PascalCase / array variables.
- **Task-07**: Commit-pull configuration boolean prefix standardization, workDir path deduplication, and `D:\test-gitmap` PowerShell test script.
