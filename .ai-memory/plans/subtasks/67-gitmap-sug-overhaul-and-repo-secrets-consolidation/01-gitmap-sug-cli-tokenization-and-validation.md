# Subtask 01: GitMap SUG CLI Tokenization, Target Validation & Help Interception

Traceability ID: Task-02
Spec Reference: [02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md](../../../02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md)
Target Files: D:/work/gitmap/cli/cmdagy/agy_sug.go, D:/work/gitmap/cli/cmdagy/agy_sug_executor.go
Action:
- Fix subcommand routing in `routeSUGSubcommand` to handle space-separated args such as `agy-running projects` and `running projects`.
- Detect direct folder paths / aliases passed directly to `gitmap sug <path>` or `gitmap sug <alias>` to add or target them.
- Intercept `help`, `-h`, `--help` on all subcommands (e.g. `add-projects help`) to display usage rather than adding "help" to the watch list.
- Add aliases for `add-projects`: `add`, `/add`, `ap`. Add aliases for `rm`: `remove`, `del`, `/rm`, `rp`.
- Implement strict target validation: verify if target is an existing directory, a known project in GitMap database, or a valid git URL.
- On invalid target, output descriptive error with suggested commands (`gitmap list`, `gitmap lf`, `gitmap sc`).
- Add clear explanation of what `<target>` means in help text.

Acceptance Criteria:
- `gitmap agy sug agy-running projects` routes correctly without validation error.
- `gitmap sug add-projects help` renders help menu; "help" is never stored in watch list.
- Invalid targets trigger informative guidance with recovery commands.

Targeted Verification:
- Run `gitmap sug add-projects help` and verify help output and clean watch list via `gitmap sug ls`.
