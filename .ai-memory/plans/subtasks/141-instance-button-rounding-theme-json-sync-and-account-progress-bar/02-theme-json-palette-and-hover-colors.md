# Subtask 02: Dynamic Theme JSON Palette & Hover States

## Objective
Support configurable middle/surface and hover colors via JSON palette schema in `src/components/common/themePalettes.ts` and `src/components/common/ThemeManager.tsx`, injecting CSS3 variables `--surface-hover`, `--primary-hover`, and `--border-hover`.

## Target Files
- `src/components/common/themePalettes.ts`
- `src/components/common/ThemeManager.tsx`
- `src/App.css`

## Verification
- Theme definitions support optional `surfaceHover`, `primaryHover`, and `borderHover`.
- CSS variables `--surface-hover`, `--primary-hover`, `--border-hover` are dynamically set on `:root` / `html`.
