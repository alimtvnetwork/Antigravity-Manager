---
name: Standalone Script Standards
description: Hard rules for any standalone DOM/browser script (e.g. payment-banner-hider). NO !important, NO error swallowing, NO `as unknown` casts, NO magic strings, class-based architecture, enums in global types, blank line before every return, no requestAnimationFrame for simple hides — use a `.hide` class with a CSS transition.
type: constraint
---

# Standalone Script Standards (Browser / DOM Userscripts)

These rules apply to **every** standalone script in any repo (e.g. `macro-ahk-v23/standalone-scripts/*`, `payment-banner-hider`, future hider/injector scripts). The user has corrected each of these violations explicitly. Do not regress.

## 1. CSS — never use `!important`

- ❌ FORBIDDEN: `display: none !important;`, `opacity: 0 !important;`, any `!important` declaration.
- ✅ REQUIRED: Use selector specificity (a `data-` attribute or single dedicated class on `<html>` / target element) and put rules in a **dedicated CSS section / `styles.ts` file**, not inline.
- ✅ REQUIRED: For hide/show, define **one** class (`is-hidden`) with `display: none;` plus a transition class (`is-fading`) with `opacity: 0; transition: opacity 250ms ease-out;`. Toggle classes — do not mutate inline style.

## 2. File layout — separate CSS from logic

Every standalone script directory MUST have:

```
src/
  index.ts                # entry: wire-up + auto-run only
  PaymentBannerHider.ts   # the main class (PascalCase, one class per file)
  styles.ts               # exports the CSS string; injected once
  types.ts                # global enums + branded types
```

Do not stuff the CSS into a template literal in `index.ts`.

## 3. Architecture — class-based, dependency-injected

- ❌ FORBIDDEN: a flat list of top-level functions (`injectStyles`, `getTargetNode`, `hideBanner`, `checkAndHide`, `startObserver`).
- ✅ REQUIRED: One **main class** named after the feature (`PaymentBannerHider`). Every helper is either a method on that class or another class **injected via the constructor**.
- ✅ REQUIRED: The entry file does exactly two things:
  1. Construct the class with its dependencies.
  2. Call `.start()` (or equivalent) once.

Example skeleton:

```ts
export class PaymentBannerHider {
  constructor(
    private readonly dom: DomQuery,
    private readonly styles: StyleInjector,
    private readonly observer: BannerObserver,
  ) {}

  start(): Result<void> { /* ... */ }
}
```

## 4. Error handling — never swallow

- ❌ FORBIDDEN: `try { ... } catch { return null; }`, `catch (_) {}`, empty `catch` blocks, returning `null`/`undefined` from a `catch`.
- ✅ REQUIRED: Return a `Result<T, AppError>` (or the project's equivalent discriminated union). Wrap the caught error with a typed `AppError` (`apperror.WrapType(apperrtype.XPathEvaluationFailed, caught)` style) and propagate.
- ✅ REQUIRED: The caller decides what to do with the failure — the helper never decides silently.

## 5. Magic strings — use enums in global types

- ❌ FORBIDDEN: inline string literals for selectors, attribute names, attribute values, event names, `readyState` values, observer options, etc.
- ✅ REQUIRED: Declare an enum in `types.ts` (or a shared `global-types.ts` if reused across scripts):

```ts
export enum BannerState {
  Idle    = "idle",
  Fading  = "fading",
  Hiding  = "hiding",
  Done    = "done",
}

export enum DocReadyState {
  Loading     = "loading",
  Interactive = "interactive",
  Complete    = "complete",
}

export enum DomEventName {
  DomContentLoaded = "DOMContentLoaded",
}
```

State transitions, event registrations, and attribute writes use these enum values — never the literal string.

## 6. Type casting — never `as unknown`, never double-cast

- ❌ FORBIDDEN: `(window as unknown as { Foo: typeof Foo }).Foo = Foo;`
- ❌ FORBIDDEN: any `as unknown`, `as any`, or chained casts.
- ✅ REQUIRED: Augment the global type once in `types.ts`:

```ts
declare global {
  interface Window {
    PaymentBannerHider: PaymentBannerHider;
  }
}
```

Then assign with no cast: `window.PaymentBannerHider = instance;`

If the value comes from `document.evaluate`, narrow with a **user-defined type guard** function (`isHTMLElement(node)`), not a cast.

## 7. Blank line before every `return`

When a `return` statement is preceded by other statements in the same block, insert exactly **one** blank line before it. (This matches `release-artifacts/coding-guidelines-v24.4.0/02-spec/02-coding-guidelines/01-cross-language/04-code-style/04-blank-lines-and-spacing.md`, Rule 4.)

## 8. Hiding pattern — class toggle + CSS transition only

- ❌ FORBIDDEN: nested `requestAnimationFrame` calls to "force a paint then trigger collapse".
- ❌ FORBIDDEN: setting attributes through three states (`fading` → `hiding` → `done`) with `setTimeout` chains.
- ✅ REQUIRED: One class. One transition. The CSS does the timing:

```css
.payment-banner.is-hiding { opacity: 0; transition: opacity 250ms ease-out; }
.payment-banner.is-hidden { display: none; }
```

```ts
banner.classList.add(BannerClass.IsHiding);
banner.addEventListener("transitionend", () => banner.classList.add(BannerClass.IsHidden), { once: true });
```

That is the entire hide sequence. No `rAF`, no `setTimeout` fallback chains.

## 9. Pre-write checklist (mandatory before producing standalone-script code)

Before writing or editing any standalone script, the AI MUST:

1. Read the coding-guideline spec for the target repo (e.g. `02-spec/02-coding-guidelines/01-cross-language/04-code-style/`).
2. Read the existing files in the same `standalone-scripts/` directory to confirm the established structure (`src/index.ts` + `src/<Feature>.ts` + `src/styles.ts` + `src/types.ts`).
3. Verify none of the rules above are violated — explicitly grep the produced source for `!important`, `as unknown`, `catch {`, `catch (_)`, and bare string literals in attribute/event positions.
4. Only then write the file.

Skipping this checklist is the documented root cause of the `payment-banner-hider/src/index.ts` failure — see `mem://issues/payment-banner-hider-rca`.
