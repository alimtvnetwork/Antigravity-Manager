# InstallSection.tsx — 337 Lines, STYLE-005 File Length

**Status:** ✅ Solved
**Date:** 2026-04-23
**Rule:** STYLE-005 (files < 300 lines, React components < 100 lines)

## Description

`src/components/landing/InstallSection.tsx` had grown to 337 lines, mixing tokenization, clipboard logic, and bundle card UI in a single component.

## Solution

Decomposed into `src/components/landing/install/`:
- `HighlightedCommand.tsx` — shell command tokenization + syntax highlighting.
- `CopyButton.tsx` — clipboard logic with `useState` feedback.
- `BundleCard.tsx` — display logic for installation bundles.

`InstallSection.tsx` now orchestrates these children only.

## Learning

When a component nears 250 lines, extract child components by responsibility before crossing 300.

## What NOT to repeat

Never inline tokenization or clipboard hooks in a section component.
