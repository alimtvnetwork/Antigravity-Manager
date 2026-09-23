# Plan 64: Smart Switch Multi-Factor Scoring, Pre-Activation Refresh & UI Telemetry Hardening

Spec Reference: [02-spec/21-app/18-smart-multi-factor-scoring-pre-activation-refresh-and-ui-telemetry.md](../../../02-spec/21-app/18-smart-multi-factor-scoring-pre-activation-refresh-and-ui-telemetry.md)
App Issue RCA: [02-spec/22-app-issues/02-smart-switch-scoring-and-window-controls-rca.md](../../../02-spec/22-app-issues/02-smart-switch-scoring-and-window-controls-rca.md)
Status: COMPLETED
Slug: 64-smart-switch-scoring-and-ui-telemetry-hardening

## 1. Architectural Context & Intent

Replaced legacy additive 100k account scoring heuristics with the user-defined multiplicative formula:
$$\text{Score} = S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$$
coupled with randomized 50/50 directional tie-breaking and pre-activation live quota refresh verification. Concurrently, resolved Tauri v2 window control ACL denial via custom commands and capability manifests, enhanced click telemetry with exact XPaths and element IDs, and guaranteed non-empty stack traces in the global Error Modal.

## 2. Deliverables & Subtask Summary

| Traceable ID | Scope & Deliverable | Status |
|---|---|---|
| `Task-01` | Canonical app spec 18 & RCA 02 authoring with screenshot ingestion | COMPLETED |
| `Task-02` | Tauri v2 capabilities manifest, native window commands, button IDs & XPaths | COMPLETED |
| `Task-03` | Synthetic stack trace generation, XPath click tracking, Error Modal overview stack card | COMPLETED |
| `Task-04` | Multiplicative candidate scoring with weekly quota bucket extraction | COMPLETED |
| `Task-05` | Pre-activation live refresh probe, re-scoring & demotion fallback loop | COMPLETED |
| `Task-06` | Synthetic void local tests verifying formula and abidul.rasia vs rokixshohag1 | COMPLETED |
| `Task-07` | Bump minor version (v4.61.0), sync manifests, changelogs, atomic git push | COMPLETED |

## 3. Verification & Quality Gates

- **TypeScript Compilation:** `npx tsc --noEmit` passed with 0 errors.
- **Frontend Build:** `npm run build` passed with exit code 0.
- **Backend Cargo Check:** `cargo check` in `src-tauri` passed with exit code 0.
- **Standalone Unit Test:** `npx tsx src/services/__tests__/instanceService.test.ts` passed all 4 test suites verifying weekly quota bottleneck extraction and scoring.
- **Rust Unit Test:** `test_score_candidate_account_weekly_quota_groups_bottleneck` authored and passed in `src-tauri/src/modules/auto_switcher.rs`.
- **Privacy Assurance:** Zero real email addresses in repository test fixtures or code.
- **Version Synchronized:** Bumped to `v4.61.0` across all package manifests, `version.json`, and documentation.
