# Specification 141: Component & UI Design System Specification

## 1. Instance Section Button Rounding & Toolbar Capsule Specification

### 1.1 Scope of Changes (`src/pages/Instances.tsx`)
In `src/pages/Instances.tsx` lines 626–846:
1. **Segmented Group 1 (Status & Maintenance)**:
   - Outer capsule container: `rounded-[5px] overflow-hidden border ...`
   - Far-left child (`Running X of Y` indicator): `rounded-l-[4px] rounded-r-none`
   - Middle children (`Refresh`, `Sync All`): `rounded-none`
   - Far-right child (`Clean & Restart`): `rounded-r-[4px] rounded-l-none`
2. **Segmented Group 2 (Automation & Settings)**:
   - Far-left child (`Auto-Switch: ON/OFF`): `rounded-l-[4px] rounded-r-none`
   - Middle children (`⏱ Next Check`, `Eval Quota`): `rounded-none`
   - Far-right child (`Settings & Sync`): `rounded-r-[4px] rounded-l-none`
3. **Segmented Group 3 (View Mode & Card Density)**:
   - Density Switcher: Normal button `rounded-l-[4px] rounded-r-none`, Compact button `rounded-r-[4px] rounded-l-none`
   - View Mode Switcher: Cards button `rounded-l-[4px] rounded-r-none`, List button `rounded-r-[4px] rounded-l-none`
4. **Standalone Action Buttons (`New Instance`)**:
   - Change `rounded-[5px]` to `rounded-[4px]`.

### 1.2 CSS3 Hover Effects & Micro-Interactions
- Smooth transitions: `transition-all duration-150 ease-out`
- Hover color mapping: use `--surface-hover` or Tailwind `hover:bg-slate-200/90 dark:hover:bg-[#15334d]` with subtle scale feedback `active:scale-[0.98]`.

---

## 2. Dynamic Theme JSON Palette & Hover Colors Specification

### 2.1 JSON Theme Schema
Each theme in `src/components/common/themePalettes.ts` supports custom middle and hover colors:
```typescript
export interface ThemePalette {
  id: string;
  label: string;
  dark: boolean;
  bg: string;
  surface: string;
  surfaceHover?: string;
  primary: string;
  primaryHover?: string;
  accent?: string;
  border?: string;
  borderHover?: string;
  fg: string;
}
```

### 2.2 CSS3 Variable Injection (`src/components/common/ThemeManager.tsx`)
In `applyTheme(theme: string)`:
```typescript
root.style.setProperty('--surface-hover', palette.surfaceHover || `color-mix(in srgb, ${palette.surface} 88%, white)`);
root.style.setProperty('--primary-hover', palette.primaryHover || `color-mix(in srgb, ${palette.primary} 85%, white)`);
root.style.setProperty('--border-hover', palette.borderHover || palette.primary);
```

---

## 3. Account Section Quota Progress Bar Specification

### 3.1 Dimensions & Proportions (`src/components/accounts/QuotaProgressBar.tsx`)
- **Width**: Reduced by 18% (`w-[82%] max-w-[82%]`) within the flex parent container.
- **Height**: Increased default height from `h-2.5` (10px) to `h-3.5` (14px).
- **Checkpoints**: Equidistant 11 points: `[100, 90, 80, 70, 60, 50, 40, 30, 20, 10, 0]`.
- **Node Dimensions**: Node circle increased from `w-3 h-3` to `w-3.5 h-3.5`, SVG checkmark icon increased to `w-2 h-2`.

### 3.2 Continuous Left-to-Right Color Blend (Red to Prominent Green)
- Progress bar track gradient:
  `bg-gradient-to-r from-rose-500 via-amber-400 via-emerald-400 to-[#1af18d]`
- The green spectrum is prominent across the upper 50% (`emerald-400` to `#1af18d`), transitioning smoothly down through amber and rose for low quota states.

---

## 4. Email Masking Tooltip & Toggle Clarification Specification

### 4.1 UI Consistency in `src/pages/Accounts.tsx`
- Replace misleading `t("accounts.show_all_quotas")` on the email masking switch with:
  `t("accounts.show_all_emails")` / `t("accounts.toggle_email_masking")`.
- Provide complete localization entries in `src/locales/en.json` and `src/locales/zh.json`.

---

## 5. Instance Settings & Sync UI & GitMap Parity Specification

### 5.1 UI Enhancements in `src/components/instances/InstanceSettingsModal.tsx`
- Clearly labeled Source Instance Card and Target Instance Card side-by-side or stacked with distinct badges.
- Explicit actions: "Copy Settings", "Move Settings", and "Export / Import as JSON".
- JSON format parity with GitMap specification: dual sequence `[AGM:P001 | GM:#1]` and conversation tags `[AGM:C001 | GM:<cid>]`.
- Tree view visualization in `src/components/instances/PromptTreeViewModal.tsx` rendering dual-sequence pill tags.
