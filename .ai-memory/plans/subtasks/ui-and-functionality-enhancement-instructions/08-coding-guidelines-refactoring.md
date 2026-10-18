# Subtask 08: Coding Guidelines (CG) Refactoring & Preflight Release Gates

## Scope
- Files: Target refactored modules across `src-tauri/` and `src/`

## Implementation Steps
1. Invert negative condition guards and flatten nested if/else ladders.
2. Standardize boolean names with positive `is`/`has` prefixes.
3. Centralize constants and enums.
4. Verify CLI commands help output.
5. Ensure all function signatures have explicit return types.
6. Run pre-flight quality checks:
   - `cargo fmt -- --check`
   - `cargo clippy --all-targets --all-features`
   - `npm run build`
7. Bump minor version (`npm run bump minor`), sync changelogs (`@aukgit` attribution), and execute atomic GitMap commit/push.
