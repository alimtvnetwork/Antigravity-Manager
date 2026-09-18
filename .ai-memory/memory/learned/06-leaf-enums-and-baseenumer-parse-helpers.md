# Learned Memory 06: Leaf Enums & Baseenumer Parse Helpers

> **Date:** 2026-09-09
> **Status:** Active
> **Domain:** Golang Enums & Dependency Architecture

---

## 1. Context & Architectural Root Cause

Previously, several enum packages under `04-code/golang/pkg/enum/` (`processstatetype`, `bytetype`, `fileoptype`, `filepermtype`, `filewritemodetype`, `logleveltype`, `openfiletype`) imported:
- `coding-guidelines/common/pkg/errtype`
- `coding-guidelines/common/pkg/result`

This created a severe architectural import cycle risk:
- `pkg/result` imports `pkg/appfault`
- `pkg/appfault` imports enums (`severitytype`, `prioritytype`)
- Any enum importing `result` risks causing an unresolvable circular dependency in Go (`appfault` -> `enum` -> `result` -> `appfault`).

Additionally, each enum package manually exported `variantMap = basicEnum.Map()` and repeated 15 lines of verbose string-matching and error-wrapping boilerplate in `Parse(s string) Result`.

---

## 2. Standard Leaf Enum Architecture

All enum packages are strictly designated as foundational leaf packages:
1. **Zero High-Level Imports:** Enums under `pkg/enum/` must never import `pkg/result`, `pkg/errtype`, or `pkg/appfault`.
2. **Standard Signature:** `Parse(s string) (Variant, bool)` across all enums.
3. **Safe Defaults:** `ParseOrZero(s string) Variant`, `ParseOrInvalid(s string) Variant`, `ParseOrUnknown(s string) Variant`.
4. **Baseenumer Encapsulation:** `BasicIntegerEnum` and `BasicStringEnum` in `pkg/baseenumer` provide direct methods:
   - `Parse(s string) (V, bool)`
   - `ParseOrZero(s string) V`
   - `ParseErr(s string) (V, error)`
   - `ParseLookup(s string) (V, string, bool)`
   Enums no longer need to export or maintain a separate `variantMap`.
5. **High-Level Consumers:** If higher-level packages (like `pkg/fileutil`) need to provide wrapped results, they wrap the enum value in `result.Wrap[T]` in their own package scope.

---

## 3. Verification & Compliance

- 24/24 Go packages pass tests (`go test ./pkg/... -count=1`).
- 36/36 CI/CD quality gates pass in `python 03-ai-scripts/06-cicd-local-runner.py`.
