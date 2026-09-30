# 89. CI/CD Rustfmt `cmd_instances` Formatting Drift & Pipeline Recovery

> **/goal** Master and enforce CI/CD rustfmt compliance, Root Cause Analysis documentation, and remote pipeline green gate verification.
> **/learn** Read the root cause analysis in `.ai-memory/cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md` and enforce pre-commit rustfmt checks.

## User Request (Verbatim)

```markdown
Can you please check that if the CI/CD is working fine and if anything is breaking in the CI/CD, try to fix that. Yeah, I think things are breaking in the CI/CD. We need to fix the CI/CD first. Make sure that nothing remains behind. Right, the root cause of this failure. Okay, and based on that, your job is to fix the CI/CD, make the minor bug remedies and check until it gets green. Okay? Also, when you fix the CI/CD, you find the right and root cause properly so that these issues never repeat again. Find the CI/CD issues folder. Try to understand previous ones and new ones, why it is happening, root cause of it. Update its memory in the AI memory so that anytime any AI model reads the AI memory folder, it knows why it failed, how it failed, and it shouldn't do it again.

adn make sure issues are listed in theissues and rerrun so that mistakesa rte not repeated agian ,clear??
```

---

## 1. Problem Classification & Root Cause Summary

In GitHub Actions CI workflow `CI` on run `36738909573` (commit `2fe82117`), the job `Check Rust Code` failed across all 3 matrix operating systems (`windows-2022`, `ubuntu-latest`, and `macos-latest`):
- **Failed Step:** `Check Rust formatting` executing `cd src-tauri && cargo fmt -- --check`.
- **Target File:** `src-tauri/src/bin/agm.rs:8612`.
- **Symptom:** Single-line method chain `let non_flag_args: Vec<String> = args.iter().filter(|a| !a.starts_with('-')).cloned().collect();` exceeds rustfmt's maximum line length column width threshold.
- **Direct cause:** Release-bump commit `d606ae20` (`chore(release): bump version to 4.109.0`) also carried hand-edited Rust; `git blame` attributes `agm.rs:8612` to it.
- **Root cause (one sentence):** The rustfmt pre-flight rule was documentation-only (no git hook, no release gate), so a bump commit carrying hand-edited Rust landed unformatted code, and each new push cancelled the in-flight CI run, so the red went unseen while `v4.109.1`–`v4.109.4` published.
- **Recurrence:** 11th rustfmt RCA (02, 12, 16, 18, 20, 22, 27, 28, 29, 34, 40). Every earlier remediation was a written rule only.
- **Masked failures:** `Run Clippy` sits after `Check Rust formatting` in the same job, so Clippy did not execute on any commit from `bf491cfd` until the fix. Earlier red runs (`db5a3787` … `a72f9e9a`) also had TypeScript and Tauri build failures, fixed in `c4983761`.

---

## 2. Technical Remediation Plan

### 2.1 Formatting Correction in `src-tauri/src/bin/agm.rs`
Format the iterator chain across multiple lines:
```rust
    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();
```

### 2.2 Institutional Memory & RCA Registration
- Record Root Cause Analysis in `.ai-memory/cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md`.
- Register entry in `.ai-memory/cicd-index.md` and `.ai-memory/cicd-issues/readme.md`.
- Append hard prohibition in `.ai-memory/strictly-avoid.md`.

### 2.3 Enforced Gates (root-cause fix)
- **Local:** tracked `.githooks/pre-commit` runs `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` whenever staged files include `src-tauri/*.rs`, and blocks the commit. `scripts/install-git-hooks.mjs` sets `core.hooksPath=.githooks`, wired to `npm install` via `prepare` and to `npm run hooks:install`.
- **Release:** `release.yml` → `verify-release-target` runs `cargo fmt -- --check` before any build job. Committed locally; push blocked by token scope ([issue 56](../../.ai-memory/issues/56-release-fmt-gate-blocked-by-token-workflow-scope.md)).

### 2.4 Memory Accuracy Audit (2026-10-01 re-run)
- `.ai-memory/cicd-index.md`: added missing rows 35–39, marked the dead row-32 link, set RCA 40 to partial, added recurring-failure-class table.
- `.ai-memory/strictly-avoid.md`: appended process bans (no `--no-verify`, no code in bump commits, no tag before exact-SHA green, no bare `gh` in this clone).
- `.ai-memory/what-to-read.md`: corrected the entry that claimed the `release.yml` gate was enforced.
- `.ai-memory/issues/`: opened issue 56; moved fixed issue 55 to resolved.
- RCA 40: added masked-failure analysis, current gate status, and announced CI deadlines (Node 24 action majors; `ubuntu-latest` → Ubuntu 26 on 2026-10-19).
- `cicd-index.md`: repointed the seven dead `resolved-issues/01–07` links to their renumbered files (`02–08`).

### 2.5 Tooling Bug Remedy (RCA 41)
- `03-ai-scripts/33-test-inventory-generator.py --record` crashed (`'list' object has no attribute 'items'`) and `--check-age` failed on the synced `.ai-memory/test-inventory.json` schema (`tests` list, epoch `updated_at`). Added `normalize_inventory_tests()` and epoch parsing.

---

## 3. Verification & Acceptance Criteria

| AC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| AC-1 | `agm.rs:8612` rustfmt-compliant | PASS | `fb86a734`; `cargo fmt -- --check` exit 0 |
| AC-2 | RCA 40 documented and indexed | PASS | `cicd-index.md` row 40, `cicd-issues/readme.md` |
| AC-3 | Fix pushed to `main` | PASS | `fb86a734`, `09338155` on `origin/main` |
| AC-4 | Remote CI green | PASS | CI `success` on `09338155`, `ade10b48`, `9b9b8dc7`, `bd60e569` |
| AC-5 | Pre-commit hook blocks unformatted Rust | PASS | local test: drifted file exit 1, clean file exit 0 |
| AC-6 | Release refuses unformatted tags | BLOCKED | issue 56 (needs `workflow` token scope) |
| AC-7 | Change-recording tool works on committed manifest | PASS | `--record` exit 0 (11 files); `--check-age --json` returns `age_days: 13.21` |
| AC-8 | Memory indexes have no dead links | PASS | `cicd-index.md` 23 links, 0 broken; changed files 86 links, 0 broken |
