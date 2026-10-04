# Application Specification: 107-wp-exam-themes-and-header-switcher

## User Request (Verbatim)
```text
please follow skill @[.agents/skills/execute-parent-task-with-n-steps-v6] add more themes are colors from wp-exam adn try to have the themes in the top level header besides the dropdown menu so that can be changed to any theme on the fly, clear??
and bump the minor please
```

## Architectural Overview
This specification defines:
1. Ingestion of 8 rich, high-authority themes from `wp-exam` into AGM's theming system (`THEME_PALETTES`), extending the existing 10 themes to a total of 18 distinctive presets.
2. Architecture of the on-the-fly Top-Level Header Theme Switcher (`ThemeSwitcherDropdown`), positioned directly beside the central route dropdown menu (`NavMenu`) in `Navbar.tsx`.
3. Instant DOM attribute, CSS custom property (`--bg`, `--surface`, `--primary`, `--fg`), and native Tauri window color synchronization without requiring page reloads.
4. Seamless integration with `useConfigStore.updateTheme()` and persistence in `gui_config.json`.
5. Minor version bump ceremony (`v4.134.0` -> `v4.135.0`) with strict `@aukgit` attribution.

## Theme Catalogue Ingestion Matrix (wp-exam -> AGM)
1. **Green Choice** (`green-choice`, Light): `bg: "#F4F8F5"`, `surface: "#FFFFFF"`, `primary: "#16A34A"`, `fg: "#13201B"`, `dark: false`.
2. **Green Choice Dark** (`green-choice-dark`, Dark): `bg: "#0E1613"`, `surface: "#16221E"`, `primary: "#22C55E"`, `fg: "#F0FDF4"`, `dark: true`.
3. **Clean Wide** (`clean-wide`, Light): `bg: "#FFFFFF"`, `surface: "#F8FAFC"`, `primary: "#4F46E5"`, `fg: "#0F172A"`, `dark: false`.
4. **Riseup Gold** (`riseup`, Dark): `bg: "#0A0A14"`, `surface: "#141424"`, `primary: "#E8C547"`, `fg: "#FFF1D6"`, `dark: true`.
5. **Dracula** (`dracula`, Dark): `bg: "#191A21"`, `surface: "#282A36"`, `primary: "#BD93F9"`, `fg: "#F8F8F2"`, `dark: true`.
6. **Purple Violet** (`purple`, Dark): `bg: "#0F0E1E"`, `surface: "#18162F"`, `primary: "#5C45FD"`, `fg: "#FFFFFF"`, `dark: true`.
7. **Obsidian Cyan** (`obsidian`, Dark): `bg: "#0D1117"`, `surface: "#161B22"`, `primary: "#38BDF8"`, `fg: "#F0F6FC"`, `dark: true`.
8. **Navy Gold** (`vscode-navy-gold`, Dark): `bg: "#0D1117"`, `surface: "#161B22"`, `primary: "#E8C547"`, `fg: "#F0F6FC"`, `dark: true`.

## Header Theme Switcher Specifications
- **Positioning**: Mounted in `Navbar.tsx` directly adjacent to `<NavMenu navItems={navItems} />` within the central header zone.
- **Trigger Button**:
  - Compact rounded pill capsule (`h-8 px-2.5 rounded-full flex items-center gap-1.5`).
  - Active theme gradient swatch dot (`w-3.5 h-3.5 rounded-full border border-black/10 dark:border-white/20`).
  - Lucide `Palette` icon or current theme name.
  - `ChevronDown` arrow with smooth transition rotation when open.
- **Dropdown Popover**:
  - Glassmorphic card (`w-80 p-2.5 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-2xl z-[9999]`).
  - Coordinated through `agm:dropdown-open` event bus to auto-close other dropdowns.
  - Active checkmark and border highlight for current theme.
  - Clean 5-6px corner radius (`rounded-[5px]`) on all theme cards.
  - Instant on-the-fly theme switching via `updateTheme(palette.id)`.

## Verification Criteria
- [ ] All 18 themes accessible from both Top Header Switcher and Settings ThemePicker.
- [ ] On-the-fly switching immediately updates document element classes, styles, and background.
- [ ] No regression in navbar layout or window controls draggable region.
- [ ] `npm run build` passes with zero errors.
- [ ] Cargo formatting and clippy pass cleanly.
