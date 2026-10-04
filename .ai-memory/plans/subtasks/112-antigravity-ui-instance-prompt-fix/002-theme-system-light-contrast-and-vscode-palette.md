# Subtask 002 — Theme System CSS Propagation, Light Contrast & VS Code Palette

## Status: Pending
## Priority: High
## Assigned Worker: Worker 01 (Frontend Specialist — UI, Progress Bar, Themes & Cards)

---

## 1. Objectives & Root Cause Analysis

### 1.1 Root Cause of Ineffective Theme Switching
When a user switches themes in the top-level selector (e.g. from Dark to VS Code Dark+, Tokyo Night, or One Dark), the UI barely responds.
- **Root Cause**: `ThemeManager.tsx` sets CSS variables (`--bg`, `--surface`, `--primary`, `--fg`) on `document.documentElement`, but almost every major component across the codebase (such as `AccountTable.tsx`, `Instances.tsx`, `App.tsx`, modals, and sidebars) has **hardcoded Tailwind utility classes** like `dark:bg-[#071a27]`, `dark:bg-[#0c2438]`, and `dark:border-[#15334d]`.
- Because these utilities are hardcoded arbitrary hex values, changing the CSS variables does not update the card backgrounds, table backgrounds, or window surfaces.
- **Remediation**:
  1. Map theme variables into CSS classes and custom utility classes in `src/App.css` (e.g. `--app-bg: var(--bg)`, `--app-surface: var(--surface)`, and dark background override helpers).
  2. Ensure `.dark body`, `#root`, and main page wrappers inherit `var(--bg)` and `var(--surface)`.

### 1.2 Overhaul Light Theme Contrast
In the current Light theme:
- Active / current account rows (`isCurrent`) in `AccountTable.tsx` and `AccountRow.tsx` use `bg-slate-900/90`, which renders a **pitch-black row in the middle of a pure white table**.
- Low contrast for text in `QuotaItem.tsx`: `text-cyan-600` on a white background lacks sufficient WCAG contrast.
- Unfilled dots and progress bar tracks in `WaterDrainProgressBar.tsx` blend into the white table row.
- **Remediation**:
  - Replace `bg-slate-900/90` with soft blue `bg-blue-50/70 border-blue-200 text-blue-900` in light mode.
  - Calibrate `QuotaItem.tsx` text tokens for high contrast: `text-teal-700 dark:text-cyan-400` for healthy quota, `text-amber-700 dark:text-amber-400` for warning, `text-rose-700 dark:text-rose-400` for critical.
  - Enhance progress bar track contrast in light mode: `border-slate-300 bg-slate-200/90`.

### 1.3 Elimination of Unwanted Pink/Purple Accents
The user explicitly requested:
> *"I really don't like this pink color that you have everywhere. Try to have a VS Code type theme, Antigravity current theme that we have, similar type of color, not bad. There are lots of IDE colors. Try to follow this color concept... Eliminate pink/purple accents; replace with VS Code / Antigravity slate, blue, cyan, and mint design tokens."*

- **Remediation**:
  1. Refactor `THEME_PALETTES` in `src/components/common/themePalettes.ts` to replace pink/purple palettes (`dracula`, `purple`, `monokai`, `midnight`) with VS Code / Antigravity IDE palettes (Cobalt Blue, Antigravity Cyan, Nordic Mint, Slate Dark).
  2. In `src/pages/Instances.tsx`, refactor `INSTANCE_THEMES` to purge pink/purple themes (`Purple`, `Rose`, `Violet`) and replace them with IDE-grade palettes (`Slate`, `Cobalt`, `Cyan`, `Teal`, `Sky`, `Emerald`).

---

## 2. File Targets

| File Path | Description of Changes |
| :--- | :--- |
| `src/components/common/ThemeManager.tsx` | Ensure `--app-bg`, `--app-surface`, `--app-border`, `--app-primary` are propagated to `document.documentElement` alongside `--bg` and `--surface`. |
| `src/App.css` | Map global theme CSS variables to dark body, container surfaces, and provide CSS variables for surface backgrounds. |
| `src/components/common/themePalettes.ts` | Remove pink/purple palettes; replace with VS Code, Antigravity Slate, Cyan, and Mint themes. |
| `src/components/accounts/AccountTable.tsx` | Eliminate pitch-black `bg-slate-900` in light mode; use `bg-blue-50/70 border-blue-200`. |
| `src/components/accounts/AccountRow.tsx` | Mirror light mode soft blue row styling for `isCurrent`. |
| `src/components/accounts/QuotaItem.tsx` | Upgrade light mode contrast for quota labels, percentage text, and countdown ETA. |
| `src/components/common/WaterDrainProgressBar.tsx` | Increase track and empty node border contrast for light mode. |
| `src/pages/Instances.tsx` | Purge pink/purple themes from `INSTANCE_THEMES`; replace with Slate, Cobalt, Cyan, Teal, Sky, Emerald. |

---

## 3. Exact Implementation Specifications

### 3.1 `ThemeManager.tsx` & `App.css` CSS Variable Propagation

In `src/components/common/ThemeManager.tsx`:
```tsx
root.style.setProperty('--bg', palette.bg);
root.style.setProperty('--surface', palette.surface);
root.style.setProperty('--primary', palette.primary);
root.style.setProperty('--fg', palette.fg);

// Additional semantic mappings for components
root.style.setProperty('--app-bg', palette.bg);
root.style.setProperty('--app-surface', palette.surface);
root.style.setProperty('--app-primary', palette.primary);
root.style.setProperty('--app-border', isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.08)');
```

In `src/App.css`:
```css
/* Dynamic theme variable binding */
html,
body,
#root {
  background-color: var(--app-bg, #f5faf9);
  color: var(--fg, #111827);
}

.dark html,
.dark body,
.dark #root {
  background-color: var(--app-bg, #071a27);
  color: var(--fg, #e5e7eb);
}

/* Dynamic surface containers */
.theme-surface {
  background-color: var(--app-surface, #ffffff);
}
.dark .theme-surface {
  background-color: var(--app-surface, #0e2433);
}

/* Smooth theme transition */
*,
*::before,
*::after {
  transition-property: color, background-color, border-color, text-decoration-color, fill, stroke;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 200ms;
}
```

---

### 3.2 `themePalettes.ts` VS Code & Antigravity Palettes Overhaul

Replace undesirable pink/purple palettes in `src/components/common/themePalettes.ts`:

```typescript
export const THEME_PALETTES: ThemePalette[] = [
    { id: 'light', label: 'Light Clean', bg: '#F8FAFC', surface: '#FFFFFF', primary: '#2563EB', fg: '#0F172A', dark: false },
    { id: 'dark', label: 'Antigravity Dark', bg: '#071A27', surface: '#0E2433', primary: '#38BDF8', fg: '#E5E7EB', dark: true },
    { id: 'system', label: 'System Automatic', bg: '#071A27', surface: '#0E2433', primary: '#38BDF8', fg: '#E5E7EB', dark: true },
    { id: 'vscode-dark', label: 'VS Code Dark+', bg: '#1E1E1E', surface: '#252526', primary: '#007ACC', fg: '#D4D4D4', dark: true },
    { id: 'tokyo-night', label: 'Tokyo Night Slate', bg: '#1A1B26', surface: '#24283B', primary: '#7AA2F7', fg: '#C0CAF5', dark: true },
    { id: 'one-dark', label: 'One Dark Pro', bg: '#21252B', surface: '#282C34', primary: '#61AFEF', fg: '#ABB2BF', dark: true },
    { id: 'github-dark', label: 'GitHub Dark HC', bg: '#0D1117', surface: '#161B22', primary: '#58A6FF', fg: '#F0F6FC', dark: true },
    { id: 'obsidian', label: 'Obsidian Cyan', bg: '#0B0F19', surface: '#111827', primary: '#06B6D4', fg: '#F3F4F6', dark: true },
    { id: 'mint-terminal', label: 'Nordic Mint', bg: '#0B1614', surface: '#122320', primary: '#10B981', fg: '#ECFDF5', dark: true },
    { id: 'green-choice', label: 'Green Choice Light', bg: '#F4F8F5', surface: '#FFFFFF', primary: '#16A34A', fg: '#13201B', dark: false },
    { id: 'green-choice-dark', label: 'Green Choice Dark', bg: '#0D1713', surface: '#14231E', primary: '#22C55E', fg: '#F0FDF4', dark: true },
    { id: 'clean-wide', label: 'Clean Studio Light', bg: '#FFFFFF', surface: '#F8FAFC', primary: '#0284C7', fg: '#0F172A', dark: false },
    { id: 'cobalt', label: 'Cobalt Blue', bg: '#0B1528', surface: '#13223E', primary: '#3B82F6', fg: '#F1F5F9', dark: true },
    { id: 'vscode-navy-gold', label: 'Navy Amber', bg: '#0A121E', surface: '#111F33', primary: '#F59E0B', fg: '#F8FAFC', dark: true },
];
```

---

### 3.3 Fixing Light Mode Active Row in `AccountTable.tsx` & `AccountRow.tsx`

#### In `AccountTable.tsx` (`SortableAccountRow` line 270):
**Before (Problematic Pitch-Black in Light Mode):**
```tsx
isCurrent
    ? "bg-slate-900/90 dark:bg-[#091b2c] border-l-amber-400 dark:border-l-amber-400 border-amber-400/50 dark:border-amber-400/40 font-semibold text-amber-300 dark:text-amber-300 shadow-sm ring-1 ring-amber-400/30 hover:bg-slate-800/90 dark:hover:bg-[#0c2438]"
```

**After (Soft Blue Contrast in Light Mode, Dark Navy in Dark Mode):**
```tsx
isCurrent
    ? "bg-blue-50/70 dark:bg-[#091b2c] border-l-blue-600 dark:border-l-amber-400 border-blue-200 dark:border-amber-400/40 font-semibold text-blue-900 dark:text-amber-300 shadow-xs ring-1 ring-blue-400/30 dark:ring-amber-400/30 hover:bg-blue-100/60 dark:hover:bg-[#0c2438]"
```

#### Apply the same update in `src/components/accounts/AccountRow.tsx` line 112.

---

### 3.4 Calibrating `QuotaItem.tsx` High-Contrast Text

In `src/components/accounts/QuotaItem.tsx`:
```tsx
const getTextColorClass = (p: number) => {
    // Light mode: text-teal-700 / Dark mode: text-cyan-400
    if (p >= 50) return 'text-teal-700 dark:text-cyan-400';
    // Light mode: text-amber-700 / Dark mode: text-amber-400
    if (p >= 20) return 'text-amber-700 dark:text-amber-400';
    // Light mode: text-rose-700 / Dark mode: text-rose-400
    return 'text-rose-700 dark:text-rose-400';
};

const getTimeColorClass = (time?: string) => {
    if (!time) return 'text-gray-400 dark:text-gray-500';
    const color = getTimeRemainingColor(time);
    switch (color) {
        case 'success': return 'text-teal-700 dark:text-cyan-400';
        case 'warning': return 'text-amber-700 dark:text-amber-400';
        default: return 'text-blue-700 dark:text-blue-400';
    }
};
```

---

### 3.5 Purging Pink/Purple Themes from `INSTANCE_THEMES` in `Instances.tsx`

In `src/pages/Instances.tsx`, lines 68-118, replace `INSTANCE_THEMES`:
```tsx
const INSTANCE_THEMES = [
    {
        name: 'Indigo',
        accentBar: 'from-indigo-500 via-blue-500 to-indigo-600',
        badge: 'bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border-indigo-400/30',
        emailPill: 'bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800/60',
        dot: 'bg-indigo-500',
    },
    {
        name: 'Emerald',
        accentBar: 'from-emerald-500 via-teal-500 to-emerald-600',
        badge: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-400/30',
        emailPill: 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/60',
        dot: 'bg-emerald-500',
    },
    {
        name: 'Cyan',
        accentBar: 'from-cyan-500 via-sky-500 to-blue-500',
        badge: 'bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border-cyan-400/30',
        emailPill: 'bg-cyan-50 dark:bg-cyan-950/40 text-cyan-700 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800/60',
        dot: 'bg-cyan-500',
    },
    {
        name: 'Amber',
        accentBar: 'from-amber-500 via-yellow-500 to-orange-500',
        badge: 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-400/30',
        emailPill: 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800/60',
        dot: 'bg-amber-500',
    },
    {
        name: 'Sky',
        accentBar: 'from-sky-500 via-blue-500 to-cyan-600',
        badge: 'bg-sky-500/10 text-sky-700 dark:text-sky-300 border-sky-400/30',
        emailPill: 'bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 border-sky-200 dark:border-sky-800/60',
        dot: 'bg-sky-500',
    },
    {
        name: 'Teal',
        accentBar: 'from-teal-500 via-emerald-500 to-teal-600',
        badge: 'bg-teal-500/10 text-teal-700 dark:text-teal-300 border-teal-400/30',
        emailPill: 'bg-teal-50 dark:bg-teal-950/40 text-teal-700 dark:text-teal-300 border-teal-200 dark:border-teal-800/60',
        dot: 'bg-teal-500',
    },
    {
        name: 'Slate',
        accentBar: 'from-slate-500 via-gray-500 to-slate-600',
        badge: 'bg-slate-500/10 text-slate-700 dark:text-slate-300 border-slate-400/30',
        emailPill: 'bg-slate-100 dark:bg-slate-900/60 text-slate-700 dark:text-slate-300 border-slate-300 dark:border-slate-800',
        dot: 'bg-slate-500',
    },
];
```

---

## 4. Acceptance Criteria & Verification

1. **Theme Switching Verification**:
   - Change theme from Dark -> VS Code Dark+ -> One Dark -> Light Clean.
   - Verify application window background, body background, and root container dynamically change to match the selected theme's hex color.
2. **Light Theme Table Contrast**:
   - Switch to Light Clean theme.
   - Inspect the currently active account row.
   - Verify the row renders as soft blue (`bg-blue-50/70`) with subtle border (`border-blue-200`), with zero pitch-black blocks.
   - Verify all quota text, progress bars, and email labels have strong contrast.
3. **No Pink/Purple Pollution**:
   - Audit `themePalettes.ts` and `INSTANCE_THEMES`.
   - Confirm all hot pink and vibrant purple tokens are eliminated.
   - Confirm VS Code slate, cobalt, cyan, and mint themes are rendered cleanly.
