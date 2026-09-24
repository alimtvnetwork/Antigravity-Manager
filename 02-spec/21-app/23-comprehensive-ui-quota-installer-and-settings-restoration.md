# Specification: Comprehensive UI, Quota Calculation, Installer, and Settings Restoration

> Spec ID: `21-app/23`
> Status: `active`
> Date: 2026-09-24
> Author: Antigravity Autonomous Agent

## 1. Executive Summary & Problem Classification

Following upstream PR #4 merge (`634c9fad`), several regressions and UI inconsistencies were introduced:
1. **Quota Calculation Regression**: In `src-tauri/src/modules/quota.rs`, rolling 5-hour quotas were overridden by weekly bucket statistics whenever weekly remaining fraction was lower than 5-hour, displaying days-until-reset (`6d 9h`) instead of rolling hours (`4h 56m`).
2. **Account UI Multi-Click Vanishing**: Rapid repeated clicking on refresh or quota filters caused UI state collapse due to un-debounced triple refresh calls and aggressive loading state clears.
3. **App & Shortcut Branding Inconsistency**: Process name appeared as `agm-alim` in Task Manager, and shortcut titles lacked proper clean naming. Windows Add/Remove DisplayName is verified, but Start Menu and taskbar shortcuts should be clean (`Antigravity Manager Tools by Alim`) without version suffixes.
4. **Installer Code Blocks in Releases & Docs**: Pinned and direct one-liner commands were combined in a single code block, breaking 1-click copy. Pinned URL examples had version discrepancies.
5. **Interactive CLI / PS Prefix & Prompt Injection**: Shell runner failed when passed `powershell:` prefixes or when prompt injection templates (`Project: ...`) were executed as raw commands.
6. **Email Section Polish & Tooltips**: Missing hover tooltip icons, "AGM" branding leftovers, and AI template modal visibility.
7. **Telegram Bot Guided Setup & UI Alignment**: Token ID and Chat ID form fields were misaligned, lacking a guided setup wizard.
8. **Settings & Auto-Switcher Backup/Restore**: Missing import/export dropdowns for Auto-Switcher and system-wide backup in Advanced/Supabase tabs.
9. **Instances Sequential Numbering**: Instances lacked intuitive sequence badges (`#1`, `#2`, `#3`...) for remote execution identification.
10. **Root README & Obsolete Screenshots**: Legacy Chinese screenshots and outdated v2.0.0 about sections remained in `readme.md`.

---

## 2. Technical Data Contracts & Invariants

### A. Quota Calculation Invariant
When calculating model quota for the standard 5-hour window:
- If weekly quota is completely exhausted (`remaining_fraction <= 0.001`), display weekly exhausted status (`0%`, weekly reset time).
- In ALL other cases, strictly return the 5-hour rolling bucket (`Some(h)`), preserving exact 5-hour remaining fraction and 5-hour reset time.

### B. Shell Command Prefix Normalization Invariant
- Allowed prefixes: `powershell:`, `ps:`, `bash:`, `sh:`.
- Strips any recognized prefix and whitespace before feeding to the underlying shell.
- If running on Windows, default to PowerShell (`pwsh` or `powershell.exe`) automatically when no shell is specified.
- If input matches AI prompt syntax (`Project: ...` or contains prompt directives), route to the AI prompt handler instead of raw shell execution.

### C. Application & Process Identity Invariant
- Tauri `productName`: `"Antigravity Manager Tools by Alim"`
- Process Window Title: `"Antigravity Manager Tools"`
- Start Menu / Taskbar Shortcut: `"Antigravity Manager Tools by Alim"` (no version suffix in shortcut file)
- Windows Add/Remove Programs: `"Antigravity Manager Tools <version>"` (includes version)

---

## 3. Visual & UX Specifications

1. **Accounts Table**:
   - Model Quota column renders 5-hour rolling quotas accurately (e.g. `Gemini 4h 56m 100%`).
   - Remove disruptive `hidden` classes that hide Action buttons on sub-1280px displays.
   - Debounce quota refresh calls; eliminate redundant triple-calls.
2. **Instances View**:
   - Each instance card renders a sequence pill: `#1`, `#2`, `#3` with distinct color styling.
   - Quick "+ New Instance" button placed in header.
3. **Email Settings**:
   - Tooltip hover indicators on Export/Import buttons.
   - Modernized visual export modal with clean borders and "Antigravity Manager Tools" footer.
4. **Settings & Backup**:
   - Auto-Switcher card includes Import/Export dropdown.
   - Supabase / Advanced section includes full system backup and restore.
   - Debug console button in navbar / settings header.

---

## 4. Verification & Quality Gates

- Rust compilation passes: `cargo check` in `src-tauri/`
- TypeScript lint check passes: `npx tsc --noEmit`
- Clean git status and single atomic commit.
