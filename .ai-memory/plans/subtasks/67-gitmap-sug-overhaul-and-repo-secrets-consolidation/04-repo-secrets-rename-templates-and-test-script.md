# Subtask 04: Repo Secrets 01-gitmap Rename, SIO Templates & D:\test-gitmap Script

Traceability ID: Task-06, Task-07
Spec Reference: [02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md](../../../02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md)
Target Files: D:/work/repo-secrets, D:/work/gitmap/cli/cmd/commitin/config_json.go
Action:
- In `D:\work\repo-secrets`: Rename directory `01-git-map` to `01-gitmap`.
- Update `seo-templates.json` and `seo-templates-cli.json`:
  - Convert all variable keys to PascalCase (`Company`, `CompanyUrl`, `CompanyAlt`, `Regions`, `Marek`, `MarekRole`, `Alim`, `AlimUrl`, etc.).
  - Enforce company name: `RISEUP ASIA LLC`.
  - Enforce URL: `alimkarim.com`, portfolio: `alim-portfolio`, aliases (`MD. Alim Ul Karim`, `AKA MD Alim Karim`).
  - Add array variable for regions with random/indexed selection support.
- In `commit-pull-config.json`:
  - Standardize boolean flags with positive `is*` prefixes (`isApplyTree` / `isTree`, `isApplyFinalSync`, `isApplyCd`, `isRecreate`, `isPushImmediate`).
  - Add `workDir` variable and deduplicate repeating `imports` path.
- In `cli/cmd/commitin/config_json.go`:
  - Ensure Go unmarshaling accepts both new `isApply*` / `is*` PascalCase tags and legacy tags.
  - Support array variable expansion (e.g. `${Region[0]}` or random element pick) and `${Variable}` syntax.
- Author `test-gitmap-recreate.ps1` in `01-gitmap` to create, test, and remove `D:\test-gitmap` strictly outside `D:\work`.
- Commit and push changes in `D:\work\repo-secrets`.

Acceptance Criteria:
- Directory `01-git-map` successfully renamed to `01-gitmap`.
- Templates updated to PascalCase with `${Variable}` format, `RISEUP ASIA LLC`, `alimkarim.com`, and array support.
- `test-gitmap-recreate.ps1` operates strictly on `D:\test-gitmap` and passes execution.

Targeted Verification:
- Run `test-gitmap-recreate.ps1` and verify clean creation and teardown of `D:\test-gitmap`.
