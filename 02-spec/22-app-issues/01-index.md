# App Issues

> **/goal** Master and enforce the architectural standards, specifications, and CI/CD validation rules for 22 App Issues.
> **/learn** Read the sequentially ordered specification files in this directory, follow the actionable CI/CD checklist, and apply mandatory rules before generating code.

## 🎯 Actionable CI/CD & Agent Checklist

- [ ] `/goal` Read and understand all numbered specifications under `22-app-issues/`.
- [ ] `/learn` Adhere strictly to `.ai-memory/folder-structure.md` and `.ai-memory/strictly-avoid.md`.
- [ ] `/goal` Verify zero explicit `true` boolean evaluations and no mixed-polarity conditionals.
- [ ] `/learn` Run all local verification linters via `python 03-ai-scripts/06-cicd-local-runner.py`.

. **CRITICAL AI INSTRUCTION:** This `01-index.md` file is the primary entry point for this directory. AI agents MUST read this file first before exploring other files in this folder.

**Version:** 3.2.0
**Updated:** 2026-04-16
**AI Confidence:** Production-Ready
**Ambiguity:** None

---

## Overview

App-specific issue analysis, root-cause analysis, bug documentation, and solution guidance — for whatever project this repo ships. Whether the app is a web app, Chrome extension, browser plugin, CLI tool, mobile app, WordPress plugin, or desktop app, **its bug reports and post-mortems live here.**

This folder tracks problems encountered during application development, their diagnosis, and their resolution.

---

## Placement Rule

Any content that analyzes bugs, failures, root causes, or fixes for application-level work belongs here, regardless of the app's runtime. General coding-principle violations or cross-cutting concerns belong in the core fundamentals range (`01–20`).

---

## Contents

| # | File | Title | Severity | Status |
|---|------|-------|:---:|:---:|
| 02 | [02-smart-switch-scoring-and-window-controls-rca.md](02-smart-switch-scoring-and-window-controls-rca.md) | Smart Switch Scoring Defect, Window Controls ACL & Stack Trace Telemetry | High | Fixed |
| 03 | [03-ip-security-null-and-ipc-query-fix.md](03-ip-security-null-and-ipc-query-fix.md) | IP Security Statistics Null Column Conversion & IPC Access Log Argument Mismatch | High | Fixed |
| 04 | [04-instance-delete-open-close-loop-and-focus-stealing-rca.md](04-instance-delete-open-close-loop-and-focus-stealing-rca.md) | Instance Delete Error Trace, Open/Close Restart Loop & Unwanted Focus Stealing RCA | High | Fixed |
| 05 | [05-email-html-rendering-and-smart-rotator-ide-switch-rca.md](05-email-html-rendering-and-smart-rotator-ide-switch-rca.md) | Email HTML MIME Rendering, Node Identity Prefixing & Smart Rotator IDE Switch Delegation RCA | Critical | Fixed |
| 06 | [06-blank-ui-and-taskbar-thumbnail-rca.md](06-blank-ui-and-taskbar-thumbnail-rca.md) | Blank UI & Windows Taskbar Preview Elimination via Occlusion Flag & Minimized State Exclusion | High | Fixed |
| 07 | [07-installer-click-and-update-action-rca.md](07-installer-click-and-update-action-rca.md) | Installer Click Action Wiring & Visible Detached Process Execution on Windows RCA | High | Fixed |
| 08 | [08-manifest-and-entrypoint-resolution-rca.md](08-manifest-and-entrypoint-resolution-rca.md) | Application Manifest Restoration, Entry Point Resolution & Blank Screen Elimination RCA | Critical | Fixed |
| 09 | [09-email-body-type-and-subject-version-rca.md](09-email-body-type-and-subject-version-rca.md) | Email Body HTML MIME Type & Versioned Node Telemetry Subject RCA | Critical | Fixed |
| 10 | [10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md](10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md) | Blank UI DWM Thumbnail Freeze, Workflow Version Stamping & Two-Bar Release Installation RCA | Critical | Fixed |
| 11 | [11-ci-cd-test-isolation-and-upstream-sync-rca.md](11-ci-cd-test-isolation-and-upstream-sync-rca.md) | CI/CD Unit Test Isolation, Mock Test Coverage & Upstream Sync Resolution RCA | Critical | Fixed |
| 12 | [12-auto-switcher-quota-and-instance-rotation-rca.md](12-auto-switcher-quota-and-instance-rotation-rca.md) | Auto-Switcher 98% Threshold Evaluation, Candidate Fallback & Switch Telemetry RCA | Critical | Fixed |

---

## Cross-References

| Reference | Location |
|-----------|----------|
| App Specs | [../21-app/01-index.md](../21-app/01-index.md) |
| Spec Authoring Guide | [../01-spec-authoring-guide/01-index.md](../01-spec-authoring-guide/01-index.md) |

---

## Verification

_Auto-generated section — see `02-spec/22-app-issues/97-acceptance-criteria.md` for the full criteria index._

### AC-AI-001: App issues triage conformance: Index

**Given** Audit issue write-ups for the required Reproduction / Cause / Fix / Prevention sections.
**When** Run the verification command shown below.
**Then** Every issue file contains all four sections and references at least one commit or PR.

**Verification command:**

```bash
python3 linter-scripts/check-spec-cross-links.py --root spec --repo-root .
```

**Expected:** exit 0. Any non-zero exit is a hard fail and blocks merge.

_Verification section last updated: 2026-08-30_
