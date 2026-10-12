# RCA 45: JSON Envelope Test Macro Syntax & Window Manager Error Observability

- **Date**: 2026-10-13
- **Commit**: `e23a0355` / `a80fb536`
- **Release**: `v4.195.0`
- **Component**: `src-tauri/src/modules/json_envelope/tests.rs` & `src/utils/windowManager.ts`

---

## Part 1: Symptoms & Error Signature

On GitHub Actions CI run `#38286414475` (Commit `e23a0355`), the `Check Rust Code` job failed across all matrix runners (Ubuntu, macOS, Windows) with:

```text
error: no rules expected "https://sample-mock-root.supabase.co/rest/v1/" in macro call
  --> src/modules/json_envelope/tests.rs:63:13
   |
62 |             "https://sample-mock-project.supabase.co/rest/v1/"
   |                                                               - help: missing comma here
63 |             "https://sample-mock-root.supabase.co/rest/v1/"
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no rules expected this token in macro call
   |
   = note: while trying to match sequence start
error: could not compile `agm-alim` (lib test) due to 1 previous error
Process completed with exit code 101.
```

Additionally, in `src/utils/windowManager.ts`, window resizing and decorations adjustments had uncaught promise exceptions in certain headless / Linux environments.

---

## Part 2: Root Cause

1. During the sanitization and credential scrubbing phase of `tests.rs`, an automated or merge replacement inadvertently preserved both the original mock string (`https://sample-mock-project.supabase.co/rest/v1/`) and the newly inserted replacement string (`https://sample-mock-root.supabase.co/rest/v1/`) consecutively inside the `assert_eq!` macro call without a comma or separator:
   ```rust
   assert_eq!(
       data["endpoint"].as_str().unwrap(),
       "https://sample-mock-project.supabase.co/rest/v1/"
       "https://sample-mock-root.supabase.co/rest/v1/"
   );
   ```
   In Rust macro syntax, two adjacent string literals inside an argument list trigger `error: no rules expected ... in macro call`.
2. Simultaneously, in the test JSON payload string, duplicate `"baseUrl"` keys were present.

---

## Part 3: Applied Solution & Code Changes

1. **Test Macro Correction**: In `src-tauri/src/modules/json_envelope/tests.rs`, eliminated the redundant mock line, leaving a single clean assertion:
   ```rust
   assert_eq!(
       data["endpoint"].as_str().unwrap(),
       "https://sample-mock-root.supabase.co/rest/v1/"
   );
   ```
2. **JSON Variable Deduplication**: Cleaned up the input test JSON in `tests.rs` so that only one `"baseUrl": "https://sample-mock-root.supabase.co"` is defined.
3. **Window Manager Error Observability**: Wrapped all Tauri window operations in `src/utils/windowManager.ts` with `.catch((e) => useErrorStore.getState().trackWarning(e, ...))` following the project's strict "No Silent Errors" invariant.
4. **Automated Verification**: Ran all 56 tests in `scripts/verify-supabase-and-ui-ux.ps1` and `scripts/test-supabase-ui-ux-e2e.ps1`, confirming 0 errors.

---

## Part 4: Prevention Invariant

- **Macro Call Hygiene**: Any automated string substitution inside Rust test files must be followed by local syntax validation or targeted tests check before pushing.
- **Zero Syntax Drift**: When performing multi-agent string replacements, check diffs before releasing.
- **Strict Pre-flight**: Release verification scripts (`scripts/verify-supabase-and-ui-ux.ps1`) now explicitly guard syntax integrity alongside backend endpoint sanity.
