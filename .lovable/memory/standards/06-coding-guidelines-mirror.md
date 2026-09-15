# Grounded Coding Guidelines Mirror Standard

**Date:** 2026-09-09
**Status:** Canonical Standard
**Scope:** `.lovable/coding-guidelines.md`, `02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md`, `.cursorrules`

---

## 1. Single Source of Truth & Tri-Location Sync

To ensure 100% deterministic compliance across human developers, IDE extensions, and AI agents, coding guidelines follow a strict tri-location mirror architecture:

1. **Canonical Authoritative Source:**
   `02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md`
2. **Compact AI Context Root:**
   `.lovable/coding-guidelines.md` (distilled ruleset for system prompts).
3. **IDE Agent Instructions:**
   `.cursorrules` (contains Hard Rules between `<!-- BEGIN:SYNC-HARD-RULES -->` and `<!-- END:SYNC-HARD-RULES -->`).

### Automated Mirroring
- Manual editing of `.lovable/coding-guidelines.md` or `.cursorrules` hard rules is strictly prohibited.
- All updates originate in `34-compiled-simple-coding-guidelines.md` and are propagated via `scripts/sync-guidelines.mjs` (run via `npm run sync`).
- Verified in CI via `npm run sync:check`.

---

## 2. Core Hard Rules Summary (Zero Tolerance)

1. **Explicit `== true` TOTAL BAN:** Never compare booleans to `true`. Always use implicit evaluation (`if isReady { ... }`).
2. **Positive Boolean Prefixes:** Boolean identifiers must carry positive prefixes (`is`, `has`, `can`, `should`).
3. **No Mixed Polarity:** Do not combine positive and negative conditions in the same `if` statement.
4. **Zero-Nesting:** Flatten nested conditionals using guard clauses and early returns.
5. **Function & File Metrics:** Functions 8–15 lines, files < 300 lines, React components < 100 lines.
6. **Go Error Standard:** All Go functions returning failure metadata must use `*appfault.AppError`.
7. **Strict Relative Paths:** Absolute paths and `file:///` URIs are totally banned.
8. **Strict Lowercase File Naming:** All repository files must use lowercase naming without uppercase characters.
