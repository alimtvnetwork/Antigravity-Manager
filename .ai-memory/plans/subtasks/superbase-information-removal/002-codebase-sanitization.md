# Subtask 002: Codebase & Working Tree Credential Sanitization

- **Code:** `Task-02`
- **Parent Plan:** `.ai-memory/plans/superbase-information-removal.md`
- **Status:** `COMPLETED`
- **Owner:** Worker 01

## Objectives
1. Replace hardcoded API keys (`sb_publishable_...`) and project URLs in `scripts/supabase-endpoints.json` and `scripts/supabase-accounts.json` with standard placeholder structures.
2. Sanitize default parameter values in `scripts/supabase-setup.ps1` and `scripts/setup-supabase.ps1`.
3. Scrub documentation files `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`, `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`, `changelog.md`, and `changelog_en.md`.
4. Sanitize `src-tauri/src/modules/json_envelope/tests.rs` with mock URLs.

## Execution Summary
- `scripts/supabase-endpoints.json`: Sanitized with `https://<ROOT_PROJECT_REF>.supabase.co/rest/v1/`, `sb_publishable_<ROOT_KEY_PLACEHOLDER>`, `https://<SECONDARY_PROJECT_REF>.supabase.co/rest/v1/`, `sb_publishable_<SECONDARY_KEY_PLACEHOLDER>`.
- `scripts/supabase-accounts.json`: Sanitized with the same standard placeholder structures.
- `scripts/setup-supabase.ps1`: Default parameters for `$RootUrl`, `$RootKey`, `$SecondaryUrl`, `$SecondaryKey` bound to `$env:AGM_SUPABASE_ROOT_URL`, `$env:AGM_SUPABASE_ROOT_KEY`, `$env:AGM_SUPABASE_SECONDARY_URL`, `$env:AGM_SUPABASE_SECONDARY_KEY`.
- `scripts/supabase-setup.ps1`: Default parameters for `$RootUrl`, `$RootKey`, `$SecondaryUrl`, `$SecondaryKey` bound to `$env:AGM_SUPABASE_ROOT_URL`, `$env:AGM_SUPABASE_ROOT_KEY`, `$env:AGM_SUPABASE_SECONDARY_URL`, `$env:AGM_SUPABASE_SECONDARY_KEY`.
- `src-tauri/src/modules/json_envelope/tests.rs`: Replaced real Supabase URLs with `https://sample-mock-project.supabase.co`.
- `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`: Replaced all live credentials and project URLs with placeholders.
- `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`: Replaced all live credentials with placeholders.
- `changelog.md`: Replaced root and secondary URLs in release notes with `<ROOT_SUPABASE_URL>` and `<SECONDARY_SUPABASE_URL>`.
- `changelog_en.md`: Replaced root and secondary URLs in release notes with `<ROOT_SUPABASE_URL>` and `<SECONDARY_SUPABASE_URL>`.
- Verification: Executed `gitmap aum search` across the entire working tree; confirmed 0 remaining occurrences of credentials across all 9 target files.
