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
- **Root Cause:** A recent edit introduced the single-line iterator expression in `cmd_instances` without applying multi-line method-chain formatting, causing `cargo fmt -- --check` to reject the commit during remote pre-flight gates.

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

---

## 3. Verification & Acceptance Criteria

1. **AC-1:** Line 8612 in `src-tauri/src/bin/agm.rs` is formatted according to standard `rustfmt` rules.
2. **AC-2:** CI/CD issue RCA 40 is documented and indexed.
3. **AC-3:** Atomic commit pushed to `main` branch.
4. **AC-4:** Remote GitHub Actions workflow run completes with green status (`success`).
