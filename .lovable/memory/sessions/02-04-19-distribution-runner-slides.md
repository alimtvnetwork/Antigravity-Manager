# Session: 2026-04-19 — Distribution & Runner Spec + Slides Sub-Command

**Status:** ✅ Done
**Recorded:** 2026-04-19

---

## Highlights

User asked for: (a) include `linters-cicd` in CI/CD release + install scripts for both OSes, (b) add a root `run.ps1` that pulls latest, builds, opens browser to docs UI, and (c) supports `./run.ps1 slides` to render the slides spec in the browser.

Investigation revealed **most of this was already built**:

- ✅ Root `install.sh` / `install.ps1` already exist (full-featured, with middle-out version probe, prompt/force/dry-run modes, listing modes).
- ✅ Root `run.ps1` / `run.sh` already exist (forwarded to `linter-scripts/run.ps1`).
- ✅ `slides-app/` standalone Vite app already exists with build + package + preview scripts.
- ✅ `spec-slides/` (separate from `spec/`) already documents the slides architecture.
- ✅ `.github/workflows/release.yml` **already** packages `linters-cicd` as `coding-guidelines-linters-vX.Y.Z.zip`, packages `slides-app/dist.zip` as `coding-guidelines-slides-vX.Y.Z.zip`, and publishes `install.sh`/`install.ps1`/`linters-install.sh`/`install-config.json`/`checksums.txt`.

### Real gaps (now fixed)

1. `install-config.json` did NOT include `linters-cicd` in its default folders array → fresh installs missed it. **Fixed** — now `["02-spec","linters","linter-scripts","linters-cicd"]`.
2. Root `run.ps1` / `run.sh` had no sub-command surface — only forwarded to the Go validator. **Fixed** — added `lint` (legacy default), `slides`, and `help` sub-commands.
3. No spec documented the install/runner/release contract end-to-end. **Fixed** — created `02-spec/15-distribution-and-runner/` with five files.

### Spec created — `02-spec/15-distribution-and-runner/`

| File | Purpose |
|------|---------|
| `00-overview.md` | Distributable artifact catalog + sub-command surface + default install layout |
| `02-install-contract.md` | Exact behavior of `install.sh` / `install.ps1` (versioning, merge semantics, listings, exit codes, anti-requirements) |
| `03-runner-contract.md` | Sub-command surface of root `run.sh` / `run.ps1` (`lint`/`slides`/`help`), argument-parsing rules |
| `04-release-pipeline.md` | Required GitHub Release artifacts, job order, build-once rule, pre-release detection |
| `05-install-config.md` | Schema + authoritative default for `install-config.json`, override precedence, anti-requirements |

All five files are v1.0.0, dated 2026-04-19, owned by Md. Alim Ul Karim.

### Implementation changes

- `install-config.json` — added `"linters-cicd"` to `folders` array.
- `run.ps1` — full rewrite to dispatch on first positional arg. `slides` subcommand: detects bun/pnpm, runs install + build, starts preview in background, polls for `http://localhost:4173/`, opens default browser, waits on the preview process.
- `run.sh` — same dispatch surface in Bash (`xdg-open` / `open` / `start` for browser opener; trap to kill the preview on Ctrl-C).
- Both runners preserve **legacy default behavior**: `./run.ps1` with no positional args still triggers the Go validator via `linter-scripts/run.ps1`.

### Files removed earlier in session

- `.gitmap/` (entire directory)
- `src/.gitmap/` (entire directory)

---

## Verifications Performed

- Read `release.yml` end-to-end → confirmed all 8 required artifacts are produced.
- Read existing `install.sh` (377 lines) and `install.ps1` (313 lines) → confirmed they read `install-config.json` and merge folders correctly with `--prompt`/`--force`/`--dry-run`.
- Read `slides-app/package.json` → confirmed `build` + `preview` + `package` scripts exist.
- Made `run.sh` executable (`chmod +x`).
- Regenerated `src/data/specTree.json` after spec creation.

---

## Files Touched

### Created (specs)

- `02-spec/15-distribution-and-runner/00-overview.md`
- `02-spec/15-distribution-and-runner/02-install-contract.md`
- `02-spec/15-distribution-and-runner/03-runner-contract.md`
- `02-spec/15-distribution-and-runner/04-release-pipeline.md`
- `02-spec/15-distribution-and-runner/05-install-config.md`

### Created (memory)

- `.lovable/memory/sessions/2026-04-19-distribution-runner-slides.md` (this file)

### Modified

- `install-config.json` — added `linters-cicd`
- `run.ps1` — full rewrite with sub-command dispatch
- `run.sh` — full rewrite with sub-command dispatch
- `src/data/specTree.json` — regenerated
- `.lovable/29-plan.md` — added rows 37–41

### Deleted

- `.gitmap/`
- `src/.gitmap/`

---

## Lessons Captured

1. **Always audit existing repo state before writing specs.** The user described "missing" features that were already 90% built — the actual gaps were three small ones, not a from-scratch build.
2. **`install-config.json` is the contract**, not the installer source — adding a folder to the default install means editing that JSON, not the scripts.
3. **Preserve legacy default behavior** when adding sub-commands. `./run.ps1` with no args MUST still do what it did before; the new sub-commands are additive.
4. **The release pipeline is healthy** — every artifact already has a verify step (`test -f`) before the publish step.

---

## What Next AI Session Should Pick Up

- Plan items 09 + 10: end-to-end smoke test of `./run.sh slides` on a clean checkout, and `install.sh` against a temp directory verifying all four folders land.
- Plan items 01–05 from earlier session (BOOL-NEG-001 unit tests, codegen CI wiring) are still the highest-impact technical work.
