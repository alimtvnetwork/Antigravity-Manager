# Subtask 03: GitMap PE Multi-Target Support & Help Menu Overhaul

Traceability ID: Task-04, Task-05
Spec Reference: [02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md](../../../02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md)
Target Files: D:/work/gitmap/cli/cmd/help.go, D:/work/gitmap/cli/cmdpipeline/pipeline_errors_cmd.go, D:/work/gitmap/cli/cmdpipeline/pipeline_flags.go
Action:
- Enhance `gitmap pe` / pipeline execution commands to accept direct folder paths, project aliases, or git URLs.
- Resolve target folder paths to project roots automatically.
- Implement group-based help filtering in `gitmap help <group>` (e.g. `sug`, `aum`, `pipeline`, `ssh`).
- Provide help suggestions on `gitmap help `.
- Document explicit differences between `gitmap search` (filesystem multi-core text search) and `gitmap aum search` (symbolic/AST index search).
- Clarify AGM commands and integrations.

Acceptance Criteria:
- `gitmap pe <path>` and `gitmap pe <alias>` resolve targets properly.
- `gitmap help <group>` filters commands by category.
- Help documentation clearly contrasts search vs AUM search.

Targeted Verification:
- Run `gitmap help sug` and verify filtered help output.
