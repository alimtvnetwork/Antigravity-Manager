# Subtask 02: Two-Bar Release Notes & README

> Status: DONE
> Owner: Antigravity Agent
> Spec: 02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md
> RCA: 02-spec/22-app-issues/10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md

## Objectives
1. Update `.github/workflows/release.yml` `Extract Release Notes` step to generate two distinct markdown code blocks ("bars") for Windows (Latest vs Version-Based) and two distinct code blocks ("bars") for Linux/macOS.
2. In Version-Based installation bars, provide exact command strings with `-Version "<VER>"` (PowerShell) and `--version "<VER>"` (Bash).
3. Update `README.md` and `README_EN.md` installation sections to mirror the clear two-bar layout.
