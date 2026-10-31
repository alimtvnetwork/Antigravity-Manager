# Subtask 04: Full Verification & Release Pre-Flight

**Task Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Subtask**: `04-subtask-verification-and-release`  
**Sequence**: `243-04`  
**Spec Reference**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/03-migration-and-verification-spec.md`  

## Objectives
1. Run targeted unit tests:
   `node scripts/run-frontend-tests.mjs promptAttachment`
   `node scripts/run-frontend-tests.mjs rerunActions`
   `node scripts/run-frontend-tests.mjs promptCopy`
2. Run pre-flight quality gates:
   `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
   `npm run build`
3. Minor bump version to `4.287.0`, synchronize manifests, update `changelog.md` and readmes with `@aukgit` attribution and strictly relative Git paths.
4. Stage, commit, tag `v4.287.0`, push to `origin/main`, and verify with `gitmap pe`.
