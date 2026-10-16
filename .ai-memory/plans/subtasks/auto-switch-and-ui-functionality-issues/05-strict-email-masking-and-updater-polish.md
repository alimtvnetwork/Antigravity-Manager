---
plan: auto-switch-and-ui-functionality-issues
subtask: "05"
title: Strict Email Masking Across UI/Backend and Desktop Updater Visual Polish
domain: privacy-security-and-ui
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/auto-switch-and-ui-functionality-issues/02-audit-log-windows-and-ui-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/pages/instances/InstanceCard.tsx
  - src/utils/maskEmail.ts
  - src-tauri/src/modules/logger.rs
  - src-tauri/src/modules/notification_hub/telegram_alerts.rs
  - src-tauri/src/modules/auto_switcher/rotation.rs
  - src/components/navbar/NavMenu.tsx
  - src-tauri/src/modules/delegate_updater/run.rs
status: pending
---

# Subtask 05: Strict Email Masking Across UI/Backend and Desktop Updater Visual Polish

## 1. Context & Motivation

Users identified three critical privacy, ergonomic, and aesthetic defects:
1. **Unwanted Email Exposure**: The account email (`riseup.asia.team@gmail.com`) was accidentally unmasked whenever users hovered their cursor over an instance card, and the unmasked address was embedded in the DOM `title` tooltip. Furthermore, backend `tracing` logs and Telegram alert notifications broadcasted raw plaintext email addresses.
2. **Jittery Navbar Update Icon**: In `NavMenu.tsx`, the "Check for Updates" icon used `group-hover:animate-bounce`, causing erratic visual jumping during menu navigation.
3. **Broken Windows Updater Console**: When running `delegate_updater/run.rs`, Windows spawned a console with tiny 8x12 raster font and disabled ANSI processing, causing escape sequence corruption and unreadable output.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/pages/instances/InstanceCard.tsx` | `InstanceCard` | Remove `isEmailHovered` state; prevent hover unmasking; scrub plaintext email from `title` tooltip. |
| `src/utils/maskEmail.ts` | `maskEmail` | Standardize email masking to `rise***@***`. |
| `src-tauri/src/modules/logger.rs` | `mask_email` | Add backend email masking utility for logging and telemetry. |
| `src-tauri/src/modules/notification_hub/telegram_alerts.rs` | `format_switch_notification` | Apply `mask_email` to `selected_email` and `previous_email` in HTML messages. |
| `src-tauri/src/modules/auto_switcher/rotation.rs` | `execute_profile_rotation_with_context` | Mask email before writing to `logger::log_info`. |
| `src/components/navbar/NavMenu.tsx` | `NavMenu` | Replace `group-hover:animate-bounce` with smooth upward translation and scale. |
| `src-tauri/src/modules/delegate_updater/run.rs` | `run`, `win32_console::setup_modern_console` | Set UTF-8 code page (65001), enable `ENABLE_VIRTUAL_TERMINAL_PROCESSING`, configure 18pt Consolas font. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Remove Email Hover Leaks in `src/pages/instances/InstanceCard.tsx`
1. Remove `const [isEmailHovered, setIsEmailHovered] = useState(false);`.
2. Remove `onMouseEnter={() => setIsEmailHovered(true)}` and `onMouseLeave={() => setIsEmailHovered(false)}`.
3. Update `title` attribute so unmasked email is never injected into DOM attributes:
   ```tsx
   title={
       isEmailRevealed
           ? `${maskEmail(displayEmail)} (Click to mask)`
           : `${maskEmail(displayEmail)} (Click to reveal)`
   }
   ```
4. Render:
   ```tsx
   <span className="truncate">
       {isEmailRevealed ? displayEmail : maskEmail(displayEmail)}
   </span>
   ```

### 3.2 Step 2: Standardize `src/utils/maskEmail.ts`
```ts
export function maskEmail(email: string): string {
    const trimmed = (email || '').trim();
    if (!trimmed) return '***';
    const parts = trimmed.split('@');
    const local = parts[0] || '';
    if (local.length <= 2) {
        return `${local.slice(0, 1)}***@***`;
    }
    const shown = Math.min(4, Math.max(1, local.length - 2));
    return `${local.slice(0, shown)}***@***`;
}
```

### 3.3 Step 3: Implement Backend `mask_email` in Rust
In `src-tauri/src/modules/logger.rs`:
```rust
pub fn mask_email(email: &str) -> String {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return "***".to_string();
    }
    if let Some((local, _domain)) = trimmed.split_once('@') {
        if local.len() <= 2 {
            format!("{}***@***", &local[..1.min(local.len())])
        } else {
            let shown = 4.min(local.len().saturating_sub(2)).max(1);
            format!("{}***@***", &local[..shown])
        }
    } else if trimmed.len() <= 3 {
        "***".to_string()
    } else {
        format!("{}***", &trimmed[..3])
    }
}
```
Apply in `telegram_alerts.rs` and `rotation.rs` so all outgoing alerts and log lines mask emails.

### 3.4 Step 4: Smooth Transition on NavMenu Update Icon
In `src/components/navbar/NavMenu.tsx`:
Replace:
```tsx
<Download className="w-4 h-4 shrink-0 text-blue-600 dark:text-blue-400 group-hover:animate-bounce" />
```
With:
```tsx
<Download className="w-4 h-4 shrink-0 text-blue-600 dark:text-blue-400 transition-transform duration-200 ease-out group-hover:-translate-y-0.5 group-hover:scale-110" />
```

### 3.5 Step 5: Win32 Console 18pt Font & ANSI Processing in `delegate_updater/run.rs`
In `src-tauri/src/modules/delegate_updater/run.rs`:
1. Implement `win32_console::setup_modern_console`:
   - Calls `SetConsoleOutputCP(65001)`.
   - Calls `SetConsoleMode` with `ENABLE_VIRTUAL_TERMINAL_PROCESSING` (`0x0004`).
   - Calls `SetCurrentConsoleFontEx` with 18pt Consolas font.
2. Invoke `win32_console::setup_modern_console()` immediately after `AllocConsole()`.

---

## 4. Verification & Quality Gates

- [ ] Inspect DOM in browser DevTools: verify `InstanceCard` account button `title` does NOT contain raw email address.
- [ ] Hover cursor over account button: verify email remains masked and does not flicker into plaintext.
- [ ] Inspect `logs/app.log`: verify account rotation messages show `rise***@***`.
- [ ] Hover over "Check for Updates" in NavMenu: verify smooth upward float without jerky bounce.
- [ ] Verify Win32 console setup compiles cleanly on Windows.
