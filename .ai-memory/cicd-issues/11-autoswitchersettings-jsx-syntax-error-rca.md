# CI/CD Issue RCA: TypeScript TS1005 and TS1381 in AutoSwitcherSettings.tsx

> **Version:** 1.0.0
> **Date:** 2026-09-17
> **Failed Run:** [#35236829827](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35236829827)
> **Trigger Commit:** `8cb2331a`
> **Fixed In:** `HEAD`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom
In GitHub Actions CI pipeline [#35236829827](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35236829827), four jobs failed (`Build Frontend`, `Build Tauri App (macos-latest)`, `Build Tauri App (ubuntu-latest)`, and `Build Tauri App (windows-2025)`) during the `TypeScript check` (`npx tsc --noEmit`) and `Build frontend` (`npm run build`) steps with exit code 2:

```text
src/components/settings/AutoSwitcherSettings.tsx(217,15): error TS1005: '}' expected.
src/components/settings/AutoSwitcherSettings.tsx(217,15): error TS1005: '}' expected.
src/components/settings/AutoSwitcherSettings.tsx(217,21): error TS1381: Unexpected token. Did you mean `{'}'}` or `&rbrace;`?
Process completed with exit code 2.
```

### 2. Root Cause
In `src/components/settings/AutoSwitcherSettings.tsx`, the conditional container at line 103 opened with logical AND syntax:
```tsx
{currentConfig.is_enabled && (
    <div className="...">
```
When the visual explanation card was inserted at line 219, the closing expression at line 217 was erroneously written using ternary operator syntax:
```tsx
    </div>
) : null}
```
Because no matching ternary condition (`?`) had preceded the block, the TypeScript compiler encountered `: null}` and failed with syntax error `TS1005: '}' expected` and `TS1381: Unexpected token`.

### 3. Resolution
In `src/components/settings/AutoSwitcherSettings.tsx`, replaced `) : null}` at line 217 with the matching closing parenthesis and brace `)}`:

```diff
-                     </div>
-                 </div>
-             ) : null}
+                     </div>
+                 </div>
+             )}
```
Verified that both curly braces and parentheses across `AutoSwitcherSettings.tsx` and all modified components are 100% balanced with 0 discrepancy.

### 4. Prevention & Learnings
- **Strict Syntax Symmetry in Conditional JSX:** Ensure that conditional JSX blocks strictly align with their opening construct. If opened with `{condition && (...)`, it MUST close with `)}`. If opened with `{condition ? (...)`, it MUST close with `) : null}`.
- **Brace & Parenthesis Validation:** Automated lint scripts must verify total balance of curly braces and parentheses across modified `.tsx` files prior to committing.
- **Zero-Bypass CI/CD Fix:** Resolved legitimately by correcting the underlying syntax rather than suppressing TypeScript compiler checks.
