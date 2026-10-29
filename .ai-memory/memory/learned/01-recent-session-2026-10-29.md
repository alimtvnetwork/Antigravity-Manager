# Recent session — 2026-10-29

This note records what was created and shipped on 2026-10-29. Paths below are relative git paths.

## go-email-reader

- Specifications for warm-up sequences 08 through 13 are in `02-spec/21-app/08-ui-and-warm-up-algorithm-improvements/` through `02-spec/21-app/13-ui-and-warm-up-algorithm-improvements/`.
- Plans are in `.ai-memory/plans/pending/08-ui-and-warm-up-algorithm-improvements.md` through `.ai-memory/plans/pending/13-ui-and-warm-up-algorithm-improvements.md`.
- Sequence 08 is the connector and gate map. Sequence 09 is brakes. Sequence 10 is the send lease and cool-down. Sequence 11 is the shadow compare. Sequence 12 is the pace envelope. Sequence 13 is the shared quiet window.
- The live tick now consults `internal/warmup/gatefeed.go`. A peer can tighten the remaining-send cap and cannot raise it. An absent or stale peer does not stop local warm-up. Local quiet hours hold. A quiet skip does not send. A dry run does not spend the cap. The gate order is health, cadence, connector, theme.
- The reader UI gate map reads `GET /api/warmup/gates`.
- Commit `f3864ee` is on `main` and is pushed. The remote uses SSH.
- Sequences 08 through 13 stay pending as catalogs. Only the gate engine is implemented.
- `user.email` is `local@gitmap`. The name is `gitmap automation`.

## gitmap-v28

- Cursor-like IDE setup landed in `cli/cmdide/ide_setup.go`, `cli/cmdide/setup/`, and `cli/cmdui/ui_server_ide_setup.go`.
- The spec is `02-spec/21-app/352-cursor-like-ide-setup-from-gitmap/`.
- Pushed commits include `3229df9b4`, `864012968`, and `04690b952`.
- `cli/cov` is a coverage profile and was not committed.

## repo-cache

- Commit `f474298` stores heatmap databases, including `.heatmap/06-cursor/go-email-reader/heatmap.db`.
- SQLite `-shm` and `-wal` sidecars were not committed.

## Push denied

These local commits are still unpushed because GitHub denied `alimtvnetwork`:

- `alim.karim.profile` (19 commits)
- `kubernetes-training-v1` (19 commits)
- `common-linux-installer` (1 commit)

## Rules learned

- Do not put a personal access token in a git remote URL. Use SSH.
- Do not commit coverage profiles or SQLite sidecar files.

## Last 30 commits in this repo

```text
aa47e6596 docs(memory) - record the 2026-10-29 warm-up and git session.
df8202752 docs(memory): persist copy, telegram 236 e2e, and json-switch lie
86877cf53 Bug: prompt-tree - copy full prompt text without rewriting skill refs
ebc55ae2e chore(version) - bump to v4.277.4
89844a0cb feat(sync) - sync canonical v6 execute suite, gitmap heatmap telemetry, skills, and prompts (29-Oct-26 06:30 pm)
1d492ed50 chore(version) - bump to v4.277.3
ea08a878e feat(sync) - sync canonical v6 execute suite, gitmap heatmap telemetry, skills, and prompts (29-Oct-26 04:22 pm)
8c0607d44 chore(version) - bump to v4.277.2
9ef78584b feat(sync) - sync canonical v6 execute suite, gitmap heatmap telemetry, skills, and prompts (29-Oct-26 03:15 pm)
f455960c4 chore(version) - bump to v4.277.1
76fa45598 feat(sync) - sync canonical v6 execute suite, gitmap heatmap telemetry, skills, and prompts (29-Oct-26 02:52 pm)
a421917d9 chore(release): bump version to 4.277.0 and update changelog
59f65a222 instances ui - restyle create profile modal and surface 100-step prompt breakdown
b2cfc5596 chore(release): bump version to 4.276.0 and update changelog
1ce618741 telegram inbound - stop screenshot self-focus hang so status csv and copy work
01b7e1840 improvements - add a shared plan-100 / next-10 flywheel on CLI, UI, and Telegram so operators can run 01-prompts/30-improvements without opening prompt files by hand.
4c25d84e0 chore(release): bump version to 4.275.3 and update changelog
982e3e284 ci - drop macOS from required CI so cancelled hosted runners cannot block the Release gate
95e25f43d chore(release): bump version to 4.275.2 and update changelog
010881d4f telegram - CLI observe fan-out and non-blocking macOS CI so instance cards can ship
b23bca5f3 telegram - screenshot mosaic of IDEs, Instances, and Accounts for v4.275.1
ded2505cd telegram - per-instance status cards, downloadable CSV, hover shade, and v4.275.0
32a94e681 instances - elevate card and switch styling, idle notification hub, step trace logging and release v4.274.0
29f81e56f Bug: instances - resolve switch account dialog color contrast, search bar and ui ux
a0baa068a docs(plans): close and move completed plans to completed directory
3913e9367 revert: remove warmup and emailing ui features, specs, and plans
8de61477c chore(version) - bump to v5.0.2
efad28e4d feat(sync) - sync canonical v6 execute suite, gitmap heatmap telemetry, skills, and prompts (28-Oct-26 10:16 am)
2dff6fad2 feat(sync) - sync canonical v6 execute suite, gitmap heatmap telemetry, skills, and prompts
a59390275 Release: e2e - complete test suites and release minor bump
```
