# Ambiguity: token value format (hex vs HSL-triplet vs oklch)

Status: proceeding by best judgment (plan D2).

## Question

`03-theme-variable-architecture.md` mandates HSL triplets (`--primary: 252 85% 60%`
+ `hsl(var(--primary))`), RiseUp ships oklch (`oklch(0.79 0.166 68)`), but AGM
stores hex (`themePalettes.ts`, `tailwind.config.js`, `setBackgroundColor`).

## Decision taken

Keep hex as the stored/run format; record HSL twins (from 16/17/27/40) and oklch
twins (RiseUp, reference-only) in comments/docs. Rationale: Tailwind 3.4 +
WebView2 + `setBackgroundColor(hex)` have no oklch path, and converting all 14
palettes to triplets in this overhaul risks contrast drift.

## If overturned

A full HSL-triplet migration must convert `themePalettes.ts`, `App.css`,
`tailwind.config.js`, and every `var(--…)` consumer atomically, then re-verify
AC-003/AC-005 plus acceptance T3 on all 16 palettes.
