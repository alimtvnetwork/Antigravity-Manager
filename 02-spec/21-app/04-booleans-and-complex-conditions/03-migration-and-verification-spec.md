# Verification & Release Specification: Boolean Modernization & Minor Bump

- **Specification Slug:** `04-booleans-and-complex-conditions`
- **Specification Path:** `02-spec/21-app/04-booleans-and-complex-conditions/03-migration-and-verification-spec.md`
- **Target Release Version:** `v4.244.0` (Minor Bump from `v4.243.0`)
- **Author:** `@aukgit`
- **Status:** APPROVED & MANDATED

---

## 1. Quality Gates & Test Invariants

1. **Unit Testing**:
   - Run `npx tsx src/pages/instances/__tests__/instanceRunningPrompts.test.ts`.
   - Expected: 32/32 tests pass (exit code 0).
2. **Type Safety & Compilation**:
   - Run `npx tsc --noEmit`.
   - Expected: 0 errors (exit code 0).
   - Run `npm run build`.
   - Expected: Vite production bundle builds successfully (exit code 0).
3. **Rust Code Formatting**:
   - Run `cd src-tauri && cargo fmt -- --check`.
   - Expected: 0 formatting issues (exit code 0).

---

## 2. Release Synchronization Protocol

1. **Version Calculation**:
   - Previous: `4.243.0`
   - Target: `4.244.0` (`npm run bump minor`)
2. **Manifest Locations**:
   - `package.json`, `package-lock.json`, `version.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/hooks.nsh`, `Casks/antigravity-tools.rb`, `releases-manifest.json`.
3. **Attribution & Documentation**:
   - Update `CHANGELOG.md` (`changelog.md`) and `CHANGELOG_EN.md` (`changelog_en.md`) with strict `@aukgit` attribution.
   - Update `README.md` and `README_EN.md` header badges and `## 📝 Changelog` section.
