# Subtask Implementation Plan: Standalone Telegram Navigation Route & Interactive Setup Guide

> **Subtask ID:** `05-standalone-telegram-hub-and-guide`  
> **Parent Slug:** `ssh-and-prompt-management-testing`  
> **Spec References:**
> - `02-spec/21-app/ssh-and-prompt-management-testing/01-architecture-spec.md`
> - `02-spec/21-app/ssh-and-prompt-management-testing/03-migration-and-verification-spec.md`  
> **Target Files:**
> - `src/components/navbar/Navbar.tsx`
> - `src/components/navbar/NavDropdowns.tsx`
> - `src/components/navbar/NavSettings.tsx`
> - `src/components/settings/TelegramSettings.tsx`
> - `src/components/telegram/TelegramGuideModal.tsx`
> - `src/pages/Telegram.tsx`
> - `src-tauri/src/bin/agm/telegram_dispatch.rs`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

The user explicitly requested:
> *"Focus on the Telegram thing. Add a Telegram guide, and in the drop-down, besides the instance, you have the drop-down, right? Settings and other stuff. This is where you should also add the Telegram section and take out the Telegram section from the settings. Telegram, keep it as its standalone section, have some guide. Also share some CLI commands that a user can use to achieve and do this."*

Previously, Telegram bot configuration was embedded deeply inside the generic settings modal, creating clutter and hindering accessibility. Furthermore, users lacked an integrated, visual step-by-step guide explaining how to create a bot via `@BotFather`, obtain the bot token, detect their authorized chat ID, and utilize the native `agm telegram` CLI suite for headless remote control.

### Core Objectives:
1. **Navbar Dropdown Integration**: Extract Telegram management from the settings modal and place a dedicated Telegram launcher in the top header dropdown directly beside the instance selector.
2. **Dedicated Route & View**: Maintain the dedicated `/telegram` route with full bot lifecycle controls, daemon status monitors, and inbound command permissions.
3. **Interactive Setup Guide Modal**: Provide an intuitive, multi-step interactive wizard (`TelegramGuideModal.tsx`) detailing `@BotFather` setup, token provisioning, chat ID detection, and permission lockdown.
4. **CLI Command Reference**: Document and expose clear CLI commands (`agm telegram status`, `agm telegram send`, `agm telegram listen`, `agm telegram config`, `agm telegram setup`) with copy-to-clipboard functionality.

---

## 2. Concrete Implementation Steps

### Step 1: Add Telegram Launcher to Header & Navbar Dropdowns
- In `src/components/navbar/NavSettings.tsx` and `src/components/navbar/NavDropdowns.tsx`:
  - Add a dedicated "Telegram Remote Control" entry into the preferences dropdown popover adjacent to the instance selector.
  - Render a distinct Telegram brand icon (`MessageSquare` or Telegram paper plane) with dark-glass pill styling.
  - Clicking this action opens either the `TelegramGuideModal` or directly routes the user to the `/telegram` management hub.
- In `src/components/navbar/Navbar.tsx`:
  - Ensure the high-priority `/telegram` navigation item (`path: '/telegram'`, `label: t('nav.telegram', 'Telegram Remote')`) is cleanly rendered in the compact responsive navigation menu.

### Step 2: Enhance Interactive Setup Guide (`src/components/telegram/TelegramGuideModal.tsx`)
- Structure the guide modal into progressive, easily digestible steps:
  1. **Step 1: Create Bot with @BotFather**:
     - Direct link: `https://t.me/BotFather`.
     - Instructions: Send `/newbot`, provide a display name, and set a username ending in `bot` (e.g. `MyAgmRemoteBot`).
     - Copyable command: `/newbot`.
  2. **Step 2: Obtain API Token**:
     - Explain HTTP API token format (`123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ`).
     - Provide single-click input field to test or paste the token directly.
  3. **Step 3: Authorize Chat ID**:
     - Explain sending a test message to the newly created bot.
     - Provide "Detect Chat ID" trigger button leveraging the daemon's getUpdates probe.
  4. **Step 4: Command Reference & CLI Execution**:
     - Render structured cards with one-click copy buttons for key AGM CLI commands:
       - `agm telegram status`: Inspect daemon connection and webhook health.
       - `agm telegram send "Hello AGM"`: Test sending message to authorized chat.
       - `agm telegram listen`: Run background polling daemon.
       - `agm telegram setup`: Launch interactive CLI configuration wizard.
       - `agm prompt --instance default "Run tests"`: Dispatch execution prompt remotely.

### Step 3: Decouple Telegram from General Settings Modal
- In `src/components/settings/`:
  - Remove redundant Telegram configuration inputs from `src/components/settings/EmailNotifications/TelegramSection.tsx` and settings tabs.
  - Redirect any legacy settings references directly to `/telegram` or the standalone modal.
  - Ensure `TelegramSettings.tsx` acts purely as the command reference and palette selector, avoiding configuration duplication.

### Step 4: Expand CLI Telegram Command Suite (`src-tauri/src/bin/agm/telegram_dispatch.rs`)
- Verify that `cmd_telegram(args: &[String])` handles all documented subcommands:
  - `agm telegram status`: Inspects bot token presence, allowed chat ID, and background daemon polling status.
  - `agm telegram send "<message>"`: Dispatches message to the configured `allowed_chat_id`.
  - `agm telegram listen`: Starts polling loop with configurable interval.
  - `agm telegram config [--summary-words <100|200>]`: Sets summary preview word bounds.
  - `agm telegram palette <name>`: Customizes rich HTML message styling cards.
- Ensure all commands output clean machine-readable JSON when `--json` flag is provided.

---

## 3. Invariants & Guardrails

- **Strict Relative Paths**: All path references in documentation, tooltips, and code must be relative to the repository root (e.g. `src/components/telegram/`). Zero absolute filesystem paths or `file:///` URIs.
- **Positive Booleans**: Use positive naming conventions (`is_enabled`, `is_bot_running`, `is_guide_open`, `is_testing`).
- **Zero Silent Errors**: Catch blocks in `TelegramGuideModal.tsx` and `Telegram.tsx` must track warnings via `useErrorStore.getState().trackWarning(err, meta)`. Never swallow exceptions with bare `catch {}`.
- **UI Capsule Styling**: Navbar buttons and segmented capsules must use `rounded-full`, shared border, subtle divider lines, and dark-glass styling (`bg-white dark:bg-[#171025]`).
- **Attribution**: All changes and documentation must strictly attribute `@aukgit`.

---

## 4. Acceptance Criteria & Verification

### Acceptance Criteria:
1. The Telegram section is completely separated from the general settings modal and accessible as a top-level item in the navbar dropdown next to the instance selector.
2. Clicking the Telegram item in the dropdown smoothly navigates to the dedicated `/telegram` view and allows launching the `TelegramGuideModal`.
3. The setup guide modal displays clear, step-by-step instructions for `@BotFather`, token acquisition, and chat ID detection.
4. Each CLI command in the guide modal features an operational "Copy" button that provides instant clipboard feedback.
5. Running `agm telegram status` in the terminal outputs daemon health without panics or crashes.

### Verification Commands:
```powershell
# 1. Verify frontend build and TypeScript compilation
npm run build

# 2. Verify CLI telegram dispatch compilation
cd src-tauri
cargo check --bin agm
cd ..

# 3. Test CLI telegram command execution
cargo run --bin agm -- telegram config
cargo run --bin agm -- telegram status
cargo run --bin agm -- telegram help
```

---

*Maintained by @aukgit*
