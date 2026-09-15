# Suggestions

**Version:** 3.15.0
**Updated:** 2026-04-27

---

## Active Suggestions

### Pluggable & Context-Aware Logger Architecture
- **Status:** Pending Review **Priority:** High **Added:** 2026-09-03
- Full architecture proposals for decoupling Formatters (JSON, Console, Logfmt) and Writers (Stdout, File, Remote), alongside native `context.Context` extraction (`FromContext(ctx)`). See [03-pluggable-context-logger-architectures.md](.lovable/suggestions/03-pluggable-context-logger-architectures.md).

### Tree diagnostics: structured-event export
- **Status:** Pending **Priority:** Low **Added:** 2026-04-27
- Add a "Download JSON" button to `TreeDiagnosticsPanel` so the user can attach a full structured event log (not just copied console text) when reporting tree-rendering issues. Source: `src/lib/treeDiagnostics.ts`.

### Tree diagnostics: auto-enable on `?debug=tree` query param

- **Status:** Pending **Priority:** Low **Added:** 2026-04-27
- Convenience: read `URLSearchParams.get("debug")==="tree"` on first load and flip the localStorage toggle on. Easier to share a debug URL than to ask users to run a console command.

### Unit tests for `treeDiagnostics` ring buffer

- **Status:** Pending **Priority:** Low **Added:** 2026-04-27
- Cover: bound enforcement, category filter, snapshot immutability, disabled-state no-op behavior.

### Playwright spec for Cmd/Ctrl+J → Spec Overview

- **Status:** Pending **Priority:** Low **Added:** 2026-04-27
- Add an E2E test that opens the Docs Viewer, presses `Cmd+J`, selects the pinned "Open Spec Overview" item, and asserts `02-spec/00-overview.md` becomes the active file. Guards the resilience-to-stale-tree guarantee.

### Unit test for `findSpecOverviewFile` fallback

- **Status:** Pending **Priority:** Low **Added:** 2026-04-27
- Cover the exact-path hit, the name-based fallback, and the null path. Located at `src/components/docs/specOverviewJump.ts`.

### GitHub Sync Banner: surface drift between live HEAD and built SHA

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-27
- Optional: hit GitHub's `/repos/{owner}/{repo}/commits/{branch}` to compare upstream SHA with `version.json.git.sha`. If different, switch banner tone to amber and surface "X commits behind". Requires `Cloud → secrets` for an unauthenticated rate-limit-friendly PAT or a public-repo unauthenticated call.

### Mobile responsive testing

- **Status:** Pending **Priority:** Low **Added:** 2026-04-16
- Verify sidebar collapse behavior on mobile.

### Search/filter in spec tree

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-16
- Improves navigation for 568+ files.

### Breadcrumb navigation

- **Status:** Pending **Priority:** Low **Added:** 2026-04-16

### Smoke-test BOOL-NEG-001 in full pipeline

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-19
- Run `linters-cicd/run-all.sh` end-to-end with the SQL fixture.

### Add Go-aware BOOL-NEG-001 variant

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-19
- Scan `embed.FS` SQL strings in Go source.

### Unit tests for BOOL-NEG-001

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-19

### Round-trip tests for codegen inversion table

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-19
- Assert `invert(invert(x)) == x` for every canonical pair.

### Wire codegen into CI

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-19
- Add `run-all.sh` step + `git diff --exit-code`.

### Strengthen BOOL-NEG-001 with replacement hints

- **Status:** Pending **Priority:** Low **Added:** 2026-04-19

### Linter for missing descriptive columns

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-19
- Flag entity tables missing `Description`, transactional tables missing `Notes`/`Comments`.

### Cross-link Rule 9 / Rules 10–12 from related specs

- **Status:** Pending **Priority:** Low **Added:** 2026-04-19

### Author 8 real Mermaid diagrams (currently placeholders)

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-24
- `02-spec/13-generic-cli/images/` and `02-spec/14-update/images/` — placeholder SVGs only.

### Real-repo run of orchestrator with new flags

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-24
- Validate `--strict --total-timeout 60 --split-by severity` end-to-end on a non-fixture repo.

### Wire Playwright landing spec into CI

- **Status:** Pending **Priority:** Low **Added:** 2026-04-24
- `tests/e2e/landing.spec.ts` exists; needs CI workflow.

### Run `npm run sync` after v3.81.0 bump

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-23
- Refresh `version.json` stats and any sync'd doc headers.

### Add `linters-cicd/run-all.sh` pre-commit hook

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-23
- Prevent STYLE-001/004 violations from ever reaching CI.

### Add CI/CD templates page in docs viewer

- **Status:** Pending **Priority:** Low **Added:** 2026-04-23
- Surface the GitLab/Azure/Jenkins recipes alongside the GitHub Actions guide.

### Add a pre-commit hook to run `npm run sync` on `package.json` change

- **Status:** Pending **Priority:** High **Added:** 2026-04-23
- Prevents the recurring "version drift" CI failure (`/tmp/version.head.json` vs `/tmp/version.work.json`).

### Add a markdown heading-level linter for `readme.md`

- **Status:** Pending **Priority:** Medium **Added:** 2026-04-23
- After the CODE-RED walkthrough insertion, several `h2 → h4` skips slipped through. A check would catch this before commit.

### Surface CODE-RED walkthrough in the in-app docs viewer

- **Status:** Pending **Priority:** Low **Added:** 2026-04-23
- The walkthrough lives in `readme.md` only; consider rendering it as a dedicated docs page so the spec viewer can link to it.

---

## Implemented Suggestions

### Batch close v4.24.0 — 11 plan items in one pass — 2026-04-24

- B10 `--strict` config, B11 `--split-by severity`, B8 `--total-timeout` + 2s parse timeouts, B7 PHP plugins (CODE-RED-001..004), B2 Playwright landing spec, 09 + 10 offline E2E shell tests, B6 consistency-report dates, B5 effective-score waiver section, 12 schema-design §6 alignment, plus Release & Migration UI rewrite. linters-cicd v3.22.0 / package v4.24.0.

### Release & Migration UI lock-in — 2026-04-24

- Replaced "Install in One Line" + skip-probe variants with two cards (Windows PowerShell, macOS/Linux Bash). User-locked: never restore the removed variants.

### Repo slug rebrand to `coding-guidelines-v24` — 2026-04-24

- Full sweep of `release-artifacts/release-install.{ps1,sh}` and downstream docs. Canonical slug now `alimtvnetwork/coding-guidelines-v24`.

### Published the app — 2026-04-24

- Suggestion B1 closed by user via remix flow (live preview deployable; publish click pending).

### exclude-paths glob support across linter pipeline (linters-cicd v3.20.0) — 2026-04-24

- TOML [run].exclude-paths + --exclude-paths CLI flag. Threaded through load-config → run-all.sh → 18 check scripts → walker.py (fnmatch with directory pruning). 11 new tests, 102/102 pass.

### SPEC-LINK-001 wired into run-all.sh + regression-locked (linters-cicd v3.19.0) — 2026-04-24

- Confirmed orchestrator already dispatches SPEC-LINK-001 via registry-driven loop. Added 2 integration tests (test_runall_spec_link_wiring.py) so the wiring cannot silently regress. 91/91 tests pass.

### SPEC-LINK-001 zero baseline + promoted to error level (linters-cicd v3.18.0) — 2026-04-24

- 17 → 0 findings. Fixed 17 anchor mismatches across 9 files (all em-dash/ampersand collapse cases). Linter promoted from warning → error; new broken cross-links now fail CI.

### SPEC-LINK-001 baseline cleanup + slugify bugfix (linters-cicd v3.17.0) — 2026-04-24

- 54 → 17 warnings. Slugify no longer collapses consecutive hyphens (em-dash/ampersand fix). `mem://` added to external skip. 14-update renumbered (9 files), 13-generic-cli overview renamed (3 files), 8 Mermaid placeholders created.

### SPEC-LINK-001 cross-link checker (linters-cicd v3.16.0) — 2026-04-24

- New shared lib `_lib/markdown_links.py` (200 LOC). Fence-aware, GH-flavored slugs, inline-identifier filter. Surfaces 54 real broken cross-links in spec/. Warning-level. 18 new tests, 87/87 green.

### Syntax highlighting for Source view — 2026-04-05

### Light/dark theme toggle — 2026-04-05

### Persistent theme preference (localStorage) — 2026-04-05

### Discriminated union + enum pattern for state — 2026-04-05

### Validate-guidelines as final health check — 2026-04-05

### Download folder as ZIP — 2026-04-05

### Regenerate `specTree.json` after restructuring — 2026-04-05

### Version bump all docs and UI to 3.1.0 — 2026-04-16

### Update consistency report after error-manage restructuring — 2026-04-16

### Placeholder guidelines 11/12/13 filled with real content — 2026-04-16

### Expanded `04-spec-authoring.md` to 95% — 2026-04-16

### Expanded `19-app-design-system-and-ui.md` to 93% — 2026-04-16

### Created `25-app-database.md` consolidated guideline — 2026-04-16

### Created write-memory prompt — 2026-04-16

- Re-saved as canonical v1.0.0 in `prompts/03-write-prompt.md` on 2026-04-19.

### CICD performance spec implementation — 2026-04-19

- Middle-out walker, `--jobs`, `--check-timeout`, `TOOL-TIMEOUT` synthetic SARIF. linters-cicd v3.12.0.

### `--version` flag on every check script — 2026-04-19

- Shared via `_lib/cli.py` using `argparse` `action="version"`. linters-cicd v3.13.0.

### Database naming Rule 9 (auto-inverted computed fields) — 2026-04-19

- `02-naming-conventions.md` v3.3.0 → v3.4.0 → v3.5.0. Three-bucket Rule 8 + Rules 10/11/12.

### BOOL-NEG-001 linter check — 2026-04-19

- `checks/boolean-column-negative/sql.py` with 10-name allow-list. linters-cicd v3.14.0.

### Inverted-field codegen tool — 2026-04-19

- `linters-cicd/codegen/` for Go methods / PHP traits / TS getter mixins.

### Cross-link Rule 9 from boolean specs — 2026-04-19

- `02-boolean-principles/00-overview.md` + `12-no-negatives.md` (v2.2.0).

### Schema design §6 Mandatory Descriptive Columns — 2026-04-19

- `03-schema-design.md` v3.3.0.

### Restructure `.lovable/` to single-file convention — 2026-04-19

- Removed `completed-tasks/`, `pending-tasks/`, `suggestions/`, `strictly-avoid/` directories.

### Created `quickstart.md` at repo root — 2026-04-23

- Local + GitHub Actions copy-paste recipes for `linters-cicd/run-all.sh`.

### Other-repo CI/CD templates (GitLab / Azure / Jenkins) — 2026-04-23

- SARIF upload + required files documented for non-GitHub platforms.

### Achieved zero CI/CD violations across 612 files — 2026-04-23

- v3.81.0 — InstallSection split, fuzzyMatch refactor, readme/spec example fixed.

### Created `.lovable/cicd-issues/` + `cicd-index.md` — 2026-04-23

- Tracks every validator finding with solution + "what not to repeat".

### README CODE-RED Validation Walkthrough — 2026-04-23

- Real-world `riseup-asia-uploader` example. One rule per snippet (Reject vs Require). Anchored from the CODE-RED Rules summary section.

### README CODE-RED Rules summary table + ToC entry — 2026-04-23

- 8-row CODE-RED-001..023 table mapping each rule to its canonical spec path.

### README Spec References quick-navigation index — 2026-04-23

- 9-row table at the end of the walkthrough; every path `ls`-verified, including the `04-condensed-master-guidelines.md` fix.

### README markdown heading-hierarchy fix — 2026-04-23

- Promoted four `####` headings to `###` to keep level progression contiguous after the new `h2`. Verified 31 balanced code fences and intact TOC anchors.

### Merged stray `.lovable/plans/01-index.md` into `29-plan.md` — 2026-04-23

- Restores single-file convention; full rollout phases now live as a section inside `29-plan.md`.

---

*Suggestions — v3.13.0 — 2026-04-24*

## Active Suggestions

### Fix absolute file paths in previous artifacts

- Status: Pending
- Priority: High
- Description: Update `walkthrough.md` and other artifacts to use standalone repo-relative paths instead of `file:///`.
- Added: 2026-08-09
