/**
 * Shared selected-state style tokens (Option A: dark slate with amber rail).
 *
 * Implements Plan 85 Subtask 007 & Ambiguity 05 resolution:
 * Option A provides a calm, high-contrast visual identity using a slate base,
 * a 6px solid amber-400 rail, and an amber ACTIVE pill.
 *
 * Contrast ratios calculated per WCAG 2.1 relative luminance algorithm:
 * 1. Dark Theme Row/Card:
 *    - Background: Tailwind slate-900 (#0f172a, relative luminance L = 0.0094)
 *    - Foreground: White (#ffffff, relative luminance L = 1.0000)
 *    - Contrast Ratio: (1.0000 + 0.05) / (0.0094 + 0.05) = 17.67:1 (exceeds WCAG AAA requirement 7.0:1)
 *
 * 2. Light Theme Row/Card:
 *    - Background: Tailwind slate-100 (#f1f5f9, relative luminance L = 0.9081)
 *    - Foreground: Slate-900 (#0f172a, relative luminance L = 0.0094)
 *    - Contrast Ratio: (0.9081 + 0.05) / (0.0094 + 0.05) = 16.12:1 (exceeds WCAG AAA requirement 7.0:1)
 *
 * 3. Amber Left Rail:
 *    - Color: Tailwind amber-400 (#fbbf24, relative luminance L = 0.5824)
 *    - Left rail width: 6px solid border on both light and dark backgrounds.
 *
 * 4. Active Pill (Dark Mode):
 *    - Background: Amber-950/80 (#451a03 with 80% opacity, L ~ 0.0208)
 *    - Foreground: Amber-300 (#fcd34d, L = 0.6777)
 *    - Contrast Ratio: (0.6777 + 0.05) / (0.0208 + 0.05) = 10.27:1 (exceeds WCAG AAA requirement 7.0:1)
 *
 * 5. Active Pill (Light Mode):
 *    - Background: Amber-100 (#fef3c7, L = 0.8931)
 *    - Foreground: Amber-900 (#78350f, L = 0.0664)
 *    - Contrast Ratio: (0.8931 + 0.05) / (0.0664 + 0.05) = 8.10:1 (exceeds WCAG AAA requirement 7.0:1)
 */

export const SELECTED_ROW_CLASSES =
  "bg-slate-100 dark:bg-slate-900 text-slate-900 dark:text-white border-l-[6px] border-l-amber-400 dark:border-l-amber-400 font-semibold shadow-xs";

export const SELECTED_CARD_CLASSES =
  "bg-slate-100 dark:bg-slate-900 text-slate-900 dark:text-white border-l-[6px] border-l-amber-400 dark:border-l-amber-400 font-semibold shadow-sm ring-1 ring-amber-400/30";

export const ACTIVE_PILL_CLASSES =
  "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-bold bg-amber-100 text-amber-900 border border-amber-300 dark:bg-amber-950/80 dark:text-amber-300 dark:border-amber-400/50 shadow-xs";
