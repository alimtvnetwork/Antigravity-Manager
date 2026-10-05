# RCA 44: AccountRow JSX Closing Tag Mismatch & instance.rs Rustfmt Drift

**Date:** 2026-10-05  
**Trigger:** CI/CD Run Failure on GitHub Actions (Run ID: 37259074384)  
**Affecting Jobs:** `Build Tauri App (windows-2022, macos-latest, ubuntu-latest)`, `Check Rust Code (ubuntu-latest, windows-2022)`

---

## Part 1: Defect Description & Observed Failure

1. **Frontend Build Failure (`Build Tauri app (debug)`):**
   ```text
   src/components/accounts/AccountRow.tsx(163,15): error TS17002: Expected corresponding JSX closing tag for 'tr'.
   src/components/accounts/AccountRow.tsx(162,19): error TS17002: Expected corresponding JSX closing tag for 'td'.
   src/components/accounts/AccountRow.tsx(165,13): error TS1005: ')' expected.
   src/components/accounts/AccountRow.tsx(328,9): error TS1128: Declaration or statement expected.
   src/components/accounts/AccountRow.tsx(329,5): error TS1109: Expression expected.
   beforeBuildCommand `npm run build` failed with exit code 2
   ```

2. **Rust Code Formatting Failure (`Check Rust formatting`):**
   ```text
   Diff in src-tauri/src/modules/instance.rs:
   - Import order: crate::error vs crate::models
   - Line 672: is_default_candidate multi-condition line break
   - Line 691: has_ide_markers long method chaining line breaks
   Process completed with exit code 1.
   ```

---

## Part 2: Root Cause Analysis

1. **AccountRow.tsx JSX Tag Imbalance:**
   During the Accounts table styling refactor, a redundant closing `</div>` tag was left on line 162 inside the first column (`<td>`), closing the parent container prematurely and breaking the JSX element tree balance.
2. **Rustfmt Line Wrapping Standards:**
   In `src-tauri/src/modules/instance.rs`, newly introduced conditions in `is_default_candidate` and `has_ide_markers` exceeded rustfmt standard column width rules and import statement alphabetical sorting conventions.

---

## Part 3: Surgical Remediation

1. **AccountRow.tsx:**
   Removed the extraneous `</div>` tag on line 162, restoring perfect 1:1 matching between opened and closed JSX elements across the component.
2. **instance.rs:**
   - Reordered imports to place `use crate::error::{AppError, AppResult};` before `pub use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};`.
   - Broken long conditional expressions across lines matching rustfmt standard formatting specifications.

---

## Part 4: Verification & Preventive Measures

1. Validated JSX tree structure in `AccountRow.tsx`.
2. Cleaned rustfmt line breaks in `instance.rs`.
3. Executed version bump and verified via `gitmap pe -t` telemetry.
