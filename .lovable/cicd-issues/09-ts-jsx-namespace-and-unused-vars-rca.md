# CI/CD Issue RCA: TypeScript TS2503 'JSX' Namespace and TS6133 Unused Variables

> **Version:** 1.0.0
> **Date:** 2026-09-16
> **Failed Run:** [#35084037371](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35084037371)
> **Trigger Commit:** `8e9fc102`
> **Fixed In:** `c5b71ca7`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom
In GitHub Actions CI pipeline `#35084037371`, jobs `Build Frontend`, `Build Tauri App (windows-2025)`, `Build Tauri App (ubuntu-latest)`, and `Build Tauri App (macos-latest)` failed during the `tsc && vite build` step with exit code 2:
```text
src/components/errors/error-modal.tsx(27,31): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(106,23): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(170,56): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(202,60): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(251,59): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(292,57): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(342,59): error TS2503: Cannot find namespace 'JSX'.
src/components/errors/error-modal.tsx(375,23): error TS2503: Cannot find namespace 'JSX'.
src/lib/error-listener.ts(4,3): error TS6133: 'buildCapturedError' is declared but its value is never read.
src/lib/error-listener.ts(36,9): error TS6133: 'route' is declared but its value is never read.
```

### 2. Root Cause
1. **TS2503 (`Cannot find namespace 'JSX'`):**
   In React 19 with `"jsx": "react-jsx"` in `tsconfig.json`, the global `JSX` namespace is not automatically injected into the global scope. Using `JSX.Element` directly without importing `JSX` causes compilation failure. In React 19, the standard and portable component return type is `React.ReactNode`.
2. **TS6133 (`declared but its value is never read`):**
   `tsconfig.json` enforces `"noUnusedLocals": true` and `"noUnusedParameters": true`. In `src/lib/error-listener.ts`, `buildCapturedError` was imported but unused, and `route` was computed in `handleWindowError` but not referenced.

### 3. Resolution
1. In `src/components/errors/error-modal.tsx`, replaced all explicit `: JSX.Element` annotations with `: React.ReactNode`.
2. In `src/lib/error-listener.ts`:
   - Removed unused import `buildCapturedError`.
   - Removed unused local variable `const route = ...` from `handleWindowError`.

### 4. Prevention & Learnings
- **Use `React.ReactNode` for Component Return Types:** Under modern React (`react-jsx` transform), never annotate functional components with raw `JSX.Element`. Always use `React.ReactNode` or omit explicit return annotations when inference is unambiguous.
- **Strict Tree Hygiene on New Files:** Verify all imported symbols and local constants against `"noUnusedLocals": true` before committing new TypeScript files.
