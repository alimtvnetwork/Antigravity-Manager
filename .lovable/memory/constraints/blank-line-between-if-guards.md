---
name: Blank line between consecutive if-guards (Rule 5)
description: Always insert one blank line after a closing `}` when followed by more code — including consecutive `if` guards and the final `return`. Applies to all spec example snippets and production code in PHP/TypeScript/Go.
type: constraint
---

# Blank Line After Closing `}` — Rule 5

**Source spec:** `02-spec/02-coding-guidelines/01-cross-language/04-code-style/04-blank-lines-and-spacing.md`

## The Rule

If code continues after `}` (and the next token is NOT another `}`, `else`,
`catch`, or `finally`), insert exactly one blank line.

## What This Forbids

```ts
// ❌ FORBIDDEN — no blank line between consecutive if guards
if (isUserMissing(user)) {
  return Failure(...);
}
if (isUserSuspended(user)) {
  return Failure(...);
}
return ok();
```

## What This Requires

```ts
// ✅ REQUIRED
if (isUserMissing(user)) {
  return Failure(...);
}

if (isUserSuspended(user)) {
  return Failure(...);
}

return ok();
```

## Why This Was Saved

A `processUser` example in `readme.md` was committed without the blank
lines (2026-04-24). Spec example code MUST follow the same style rules
production code follows — readers copy-paste these snippets.

## How to Apply

- When writing or reviewing any code block in markdown (readme, docs, spec
  examples), apply Rule 5 before saving.
- When writing production code, same rule applies.
- The CI linter validates `spec/` directory code patterns but does NOT lint
  markdown prose — manual diligence required for examples in `readme.md`,
  `docs/`, and release-artifacts.

## Related Rules

- Rule 4: blank line before `return`/`throw` when preceded by statements
- Rule 10: blank line before control structures when preceded by statements
