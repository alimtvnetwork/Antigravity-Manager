# 02 — Scope of `ignored-deleted` reason enrichment

## Original request

> Add a `reason` value for each audit-trail entry so I can see why a
> deleted path was marked `ignored-deleted`.

## Context

- File: `linter-scripts/check-placeholder-comments.py`
- Today every `ignored-deleted` row carries the same static string:
  `"git reported D (deleted): no post-state file to lint"`.
- Only true `D`-status rows enter the `deleted_paths` list. Rename
  and copy **OLD-side** paths are silently dropped (the linter keeps
  only the NEW side). The module-level docstring at
  `_AUDIT_STATUSES` (lines 200–203) *says* rename old-sides should be
  marked `ignored-deleted`, but no code path emits them today.
- Existing tests (`test_list_changed_files_flag.py`,
  `test_only_changed_status_flag.py`,
  `test_dedupe_changed_files_flag.py`) hard-assert the current row
  count (e.g. only 1 `ignored-deleted` row from one `D\t…` input).

## Ambiguity

"Why a deleted path was marked `ignored-deleted`" can mean either:

1. **Diversify the `reason` text per *source***: the row set stays
   identical, but each row's `reason` explains its provenance
   (diff `D` row vs. authored `--changed-files` `D\t…` payload).
2. **Also start emitting rename/copy OLD sides as `ignored-deleted`
   rows** with a reason like `"OLD side of R092 rename — superseded
   by NEW side `<path>`"`. Closer to what the
   `_AUDIT_STATUSES` docstring already promises, but adds rows.
3. **Both** — diversify *and* add OLD-side rows.

## Options considered

### Option A — Diversify reason text only (no new rows)

- **Pros:**
  - Zero behavior change for callers / dashboards counting rows.
  - All current tests stay green; only `reason`-string assertions
    (none today) would care.
  - Smallest, easiest-to-review patch.
- **Cons:**
  - Doesn't fulfil the `_AUDIT_STATUSES` docstring's claim about
    rename OLD sides.
  - User may have been hinting they want OLD sides too.

### Option B — Also emit rename/copy OLD-side rows

- **Pros:**
  - Matches the `_AUDIT_STATUSES` docstring's stated contract.
  - Gives reviewers a complete picture: every path the diff
    mentioned shows up somewhere in the audit.
- **Cons:**
  - Breaks existing tests that assert exact row counts.
  - Doubles the audit volume on rename-heavy PRs.
  - Affects `--dedupe-changed-files` and `--similarity-csv` outputs
    in ways downstream consumers may not expect.
  - Schema change, not just a copy change.

### Option C — Both (diversify + add OLD-side rows)

- **Pros:** Most informative.
- **Cons:** All of Option B's risks plus more surface area.

## Chosen — Option A

**Recommendation rationale:** The user's wording focuses on the
*reason* field ("Add a `reason` value … so I can see why"). The
narrowest faithful reading is "make the existing reason more
specific", which Option A delivers without breaking the audit
row-count contract that several test suites depend on. Option B is
a different feature (capturing OLD sides) that should be requested
explicitly because it changes the row schema for downstream
consumers.

## Reversibility

Trivial. Switching to Option B later is purely additive — just
thread the OLD-side paths into the same `(path, source)` tuple
pipeline this task introduces. Option A's per-source reason map is
extended with two new keys (`diff-rename-old`, `diff-copy-old`) and
the existing rows keep their current text untouched.
