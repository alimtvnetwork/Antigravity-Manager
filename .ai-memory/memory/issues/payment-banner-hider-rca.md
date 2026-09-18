---
name: Payment Banner Hider — Root Cause Analysis
description: RCA for why `macro-ahk-v23/standalone-scripts/payment-banner-hider/src/index.ts` shipped with !important, error swallowing, `as unknown` casts, magic strings, no class architecture, no enums, no separate styles file, and a needless rAF chain. Captures the failure mode and the mandatory prevention checklist.
type: issue
---

# RCA — Payment Banner Hider (macro-ahk-v23)

**Date:** 2026-04-24
**Repo:** `alimtvnetwork/macro-ahk-v23`
**File:** `standalone-scripts/payment-banner-hider/src/index.ts`
**Reported by:** user, this session
**Severity:** 🔴 CODE RED — multiple zero-tolerance violations in a single 153-line file.

## 1. What was wrong

| # | Violation | Evidence (line) |
|---|-----------|-----------------|
| 1 | Pervasive `!important` in injected CSS — 12 occurrences | 35–63 |
| 2 | Error swallowing: `try { … } catch { return null; }` in `getTargetNode` | 81–83 |
| 3 | `(window as unknown as { … }).PaymentBannerHider = …` — double cast | 138–139 |
| 4 | Magic strings everywhere: `"fading"`, `"hiding"`, `"done"`, `"loading"`, `"DOMContentLoaded"`, attribute names | 19–23, 91–99, 144–148 |
| 5 | No class architecture — flat top-level functions only | entire file |
| 6 | CSS embedded in `index.ts` instead of a sibling `styles.ts` | 33–65 |
| 7 | No enums in a global `types.ts` for state / event / readyState | n/a (file missing) |
| 8 | Nested `requestAnimationFrame` to "force a paint" + `setTimeout` chain instead of a single CSS transition | 93–100 |
| 9 | Missing blank line before `return` in multiple guard clauses | 89, 107, 110, 113 |

## 2. Why it happened (root cause)

The script was generated **without first reading the project's coding-guideline spec** and **without scanning the sibling files in `standalone-scripts/`** to mirror their established structure (class + `styles.ts` + `types.ts`). The author defaulted to a quick procedural draft and used `!important` to brute-force CSS specificity instead of using a dedicated class with proper specificity. Error paths were silenced with `return null` because the author did not pull in the project's `Result` / `AppError` types.

In short: **memory of the project standards was not consulted before writing**. This is identical in shape to the README install-section regression earlier this same session.

## 3. Why memory failed

The existing memory had general "no error swallowing" and "no magic strings" rules, but no **dedicated standalone-script constraint** that names the exact violations to grep for (`!important`, `as unknown`, `catch {`, magic event-name strings). Without a check-list pinned to the file type, the AI relied on prior unrelated context.

## 4. Corrective actions taken

1. Created `mem://constraints/standalone-script-standards` with the 9 hard rules and a mandatory pre-write checklist.
2. Created this RCA so the failure is traceable.
3. Linked both into `mem://index.md` Core + Memories so they are loaded into every future session that touches a standalone script.
4. Produced a corrected reference implementation at `docs/refactors/payment-banner-hider.md` (in this repo) that the user can paste into `macro-ahk-v23`. (This sandbox cannot push to the other repo.)

## 5. Prevention — non-negotiable pre-write checklist

Before writing any standalone script (or editing one), the AI MUST:

1. `code--view` the target repo's coding-guideline spec for cross-language style + error handling.
2. `code--list_dir` the sibling `standalone-scripts/<other-script>/src/` to mirror its structure (class + `styles.ts` + `types.ts`).
3. Grep the planned source mentally for: `!important`, `as unknown`, `as any`, `catch {`, `catch (_`, raw string literals in `setAttribute` / `addEventListener` / state assignments.
4. If any of those would appear, redesign before writing.

Failure to run this checklist is itself a CODE RED.
