# Why CI failed, and the one-version rule

Read this before a release bump or a tag. The longer history is [learned/22](../memory/learned/22-cicd-gates-auto-commit-workflow-and-e2e-lessons.md) and the class table in [cicd-index.md](../cicd-index.md).

## Repeated causes

- Rustfmt drift. `cargo fmt -- --check` from `src-tauri` was documentation only until a hook and a release gate existed. A long iterator on one line fails the job. Format before commit. Do not use `--no-verify`.
- A bump commit that also edits Rust. The fmt gate then fails on the release SHA. Feature edits and the version bump are separate commits.
- Tagging a red or cancelled run. `concurrency.cancel-in-progress: true` on `main` can cancel the run you meant to trust. A cancelled job is not green. Tag only after that SHA's Actions run is green.
- Guessed installer asset URLs. Read the release asset list. Do not invent the filename.
- Shared test directories. Tests that write the same data dir collide when CI runs them together.

## The version bug

The header calls Tauri `getVersion()`, which is the `version` in `tauri.conf.json` compiled into the binary. `scripts/bump-version.mjs` reads `package.json` and rewrites the other manifests with regexes. A later edit can move `package.json` or `version.json` and leave `tauri.conf.json` behind. The running window then shows the old baked version. Fallbacks in the title bar and mini view were literals the bump regex never saw.

Before a tag, `package.json`, `version.json` (`version` and `Version`), `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` version and window title, and `src-tauri/hooks.nsh` must be the same number. The bump script exits 1 if they differ after the rewrite.
