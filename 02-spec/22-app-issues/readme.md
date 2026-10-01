# Application Issues & 4-Part RCA Index (`02-spec/22-app-issues/`)

| # | Issue / RCA File | Summary |
| :--- | :--- | :--- |
| `01` | [01-rca-switch-close-reopen-and-scoring-normalization.md](./01-rca-switch-close-reopen-and-scoring-normalization.md) | 4-Part RCA for `ls_running` hot-switch misclassification skipping IDE close/reopen, unnormalized `+1000` candidate scoring, and stale 6h–10h instance/Supabase binding lockouts. |
| `25` | [25-duplicate-telemetry-and-switch-prompt-lifecycle.md](./25-duplicate-telemetry-and-switch-prompt-lifecycle.md) | 4-Part RCA for duplicate `previous_email`, `predicted_email`, and `selected_email` telemetry, candidate pool exclusions, and enforcement of the 5-step prompt backup/close/switch/rerun/re-inject lifecycle. |
| `26` | [26-previous-selected-predicted-email-same-collision-rca.md](./26-previous-selected-predicted-email-same-collision-rca.md) | 4-Part RCA for previous, selected, and predicted email collision bug, case-insensitive exclusion matching, and tri-field mutual exclusivity invariant enforcement. |
| `27` | [27-taskbar-pin-preservation-install-script-rca.md](./27-taskbar-pin-preservation-install-script-rca.md) | 4-Part RCA for Taskbar pin removal on reinstallation/update in `install.ps1`, unquoted registry path comparison fix, taskbar directory deletion ban, and pre-flight pin backup/restoration. |
| `28` | [28-cdn-releases-manifest-stale-version-rca.md](./28-cdn-releases-manifest-stale-version-rca.md) | 4-Part RCA for stale CDN releases manifest (`v4.85.0` lockout), atomic version bump integration in `scripts/bump-version.mjs`, and CI dynamic manifest generation in `.github/workflows/release.yml`. |
