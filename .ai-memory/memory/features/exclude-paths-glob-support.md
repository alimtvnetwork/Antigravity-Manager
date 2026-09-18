---
name: exclude-paths-glob-support
description: linters-cicd `--exclude-paths` flag + `[run].exclude-paths` TOML key — fnmatch globs against repo-relative posix paths, directory pruning at os.walk time, threaded through 18 check scripts.
type: feature
---
**Where:** `.codeguidelines.toml` `[run].exclude-paths = [...]` OR `--exclude-paths "glob1,glob2"` CLI flag on `run-all.sh` and every individual check script.

**Syntax:** Python `fnmatch` globs (NOT `pathspec`/gitignore semantics). Patterns are matched against **repo-relative posix paths**.

**Examples:**
- `vendor/**` — exclude every file under vendor/
- `**/*.gen.go` — exclude generated Go files anywhere
- `third_party/**` — single subtree
- `legacy/**, **/*.min.js` — multi-pattern via CSV (CLI) or list (TOML)

**Precedence:** CLI flag > TOML > empty (built-in `SKIP_DIRS` always applies regardless: `.git`, `node_modules`, `dist`, `build`, `vendor`, `__pycache__`, `.next`, `.nuxt`, `.cache`, `target`, `bin`, `obj`, `.venv`, `venv`, `release-artifacts`, `coverage` — these are skipped even with no globs configured).

**Performance:** Directory subtrees matching a glob are pruned at `os.walk` time — excluded trees pay zero recursion cost. Per-file matching short-circuits on first hit.

**Architecture (added v4.21.0 / linters-cicd v3.20.0):**
1. `walker.py::walk_files(root, exts, exclude_globs=None)` — optional param, backward-compat.
2. `cli.py::build_parser()` exposes `--exclude-paths` on every check; `parse_exclude_paths(raw)` splits CSV.
3. `load-config.py` reads `[run].exclude-paths` from TOML, emits `EXCLUDE_PATHS=...`.
4. `run-all.sh` parses the flag, threads to load-config and every check via `--exclude-paths "$EXCLUDE_PATHS"`.
5. All 18 check scripts inject `_globs = parse_exclude_paths(args.exclude_paths)` after `args = ...parse_args()` and pass `exclude_globs=_globs` to every `walk_files(...)` call.

**Tests:** `tests/test_walker_exclude_globs.py` (8 tests) + `tests/test_load_config_exclude_paths.py` (3 tests).

**Common author trap:** patterns are NOT gitignore-style — `vendor/` alone won't match files under it. Use `vendor/**`. Don't forget the leading `**/` for filename-only patterns: `*.gen.go` won't match `internal/gen/types.gen.go`, but `**/*.gen.go` will.
