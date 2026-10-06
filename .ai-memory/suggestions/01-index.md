# Suggestions Index

- `02-fix-absolute-file-paths.md`: Fix absolute file paths to use relative repo paths.
- `03-pluggable-context-logger-architectures.md`: Pluggable & context-aware logger architecture proposals (formatters, writers, context.Context integration).
- `04-cicd-release-and-workflow-hardening.md`: CI/CD and release hardening: Ubuntu pin before 2026-10-19, Node 24 actions, release gated on full CI success, no cancellation on `main`, run fast tests, deny-level Clippy, asset completeness, untrack the per-run cache, `gh` default repo. Plan 91.
- `05-codebase-assessment-2026-10-01.md`: Codebase assessment: size snapshot, strengths, risks (13.5k-line `agm.rs`, prose rules without checks, mixed changelog formats, memory folder sprawl) and what to do about each.
- `06-rust-pre-push-gate-and-instance-key-regression-test.md`: Pre-push `cargo fmt -- --check` gate (12 of the last 30 commits were fmt or compile fixes) and a regression test that keeps instance id and IDE flavor apart (issue 62). Added 2026-10-06.
