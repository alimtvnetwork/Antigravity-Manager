# Plans Index

Master directory of architectural and execution plans.

## Pending Plans

- [02-slides-system-overhaul.md](pending/02-slides-system-overhaul.md): Full slides deck UI and system overhaul.
- [04-guideline-prompt-and-installer-upgrade.md](pending/04-guideline-prompt-and-installer-upgrade.md): Guideline prompt and installer enhancements.
- [09-update-prompts-and-release.md](pending/09-update-prompts-and-release.md): Update prompts and release lifecycle (deferred under WOR policy).
- [11-code-red-refactor-remediation.md](pending/11-code-red-refactor-remediation.md): Remediate Code Red enum, boolean, and query wrapper violations across the codebase.

## Completed Plans

- [01-repository-hygiene-scripts-and-versioning.md](completed/01-repository-hygiene-scripts-and-versioning.md): Repository hygiene, encoding normalization, lowercase conventions, AI scripts `<details>` documentation, and prompt tracking.
- [02-cicd-pipeline-and-quality-automation.md](completed/02-cicd-pipeline-and-quality-automation.md): CI/CD workflows consolidation, 12 reusable quality guards, and release skew RCA.
- [03-appfault-result-monad-and-error-architecture.md](completed/03-appfault-result-monad-and-error-architecture.md): Go AppError namespace constructors, human/logger display methods, Result[T] rich dynamic conversions, number parsing, reflection casting, deterministic map sorting, and monadic Result unwrapping.
- [04-typecast-results-and-verification-systems.md](completed/04-typecast-results-and-verification-systems.md): High-performance typecast, `ReflectSetTo` fast path, `Checker` interface family, `SimpleVerifier` parity, and coredata combinators.
- [05-fileutil-pathinfo-constants-and-io-architecture.md](completed/05-fileutil-pathinfo-constants-and-io-architecture.md): Modular file operations, path context injection, cross-platform temp hierarchy, `FilePathOps` bound struct, lock concurrency, constants centralization, and .NET-style FolderInfo/FileInfo/PathInfo architecture.
- [06-enum-architecture-generator-and-baseenumer.md](completed/06-enum-architecture-generator-and-baseenumer.md): Modular 1:1 enum isolation, dedicated packages, DRY JSON marshaling, Min/Max boundaries, leaf enum parse helpers, cycle elimination, and Python smart enum scaffolder CLI (`30-enum-generator.py`).
- [07-applogger-taxonomy-streaming-and-task-db.md](completed/07-applogger-taxonomy-streaming-and-task-db.md): Structured AppLogger, split SQLite DB logging (`logs.db` + isolated `tasks/<task-id>.db`), configurable rotating file sink, generic LazyOnce, errcmd streaming, task retention pruning, named writers, and typed streamers.
- [08-completed-plans-consolidation.md](completed/08-completed-plans-consolidation.md): Completed plans consolidation, pre-consolidation safety backups, milestone compaction, subtask collapse, and continuous resequencing.
