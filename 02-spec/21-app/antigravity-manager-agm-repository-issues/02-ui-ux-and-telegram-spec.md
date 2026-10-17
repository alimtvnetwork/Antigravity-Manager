# UI/UX & Telegram Specification: Antigravity Manager Repository Issues

Slug: `antigravity-manager-agm-repository-issues`
Date: 2026-10-17

## 1. Telegram Inbound & Prompt Summaries

### 1.1 Telegram Inbound Version Resolution
- Investigate why Telegram worker sent information with `v1.9.0`.
- Locate all references to version in `telegram_inbound/` and ensure it dynamically uses `crate::VERSION` or `env!("CARGO_PKG_VERSION")`.
- When querying connected instances or sending status, display true AGM host version.

### 1.2 Configurable Word Summary (100 / 200 words)
- Add setting in `TelegramConfig`: `prompt_summary_words: usize` (default: 200, selectable: 100 or 200).
- Expose selector in UI: `src/components/settings/EmailNotifications/TelegramSection.tsx`.
- Expose in CLI: `agm telegram config --summary-words <100|200>`.
- Format previous prompts in `/send` and `/history`:
  - Show short snippet clamped to `prompt_summary_words` with trailing ellipsis.
  - Provide inline callback button `[Resend #N]` to re-dispatch without re-typing.

### 1.3 Telegram Prompt Stripping
- When dispatching a user prompt from Telegram to an instance:
  - Do NOT prefix or postfix internal database/task IDs (`Id: ...`) into the prompt body sent to the IDE.
  - Send the exact sanitized text the user entered.

### 1.4 Telegram Screenshot Fix
- Fix `/shot` / `/screenshot`:
  - Capture active Antigravity IDE window or AGM UI via CDP window capture / Win32 screen capture.
  - Transmit as Telegram photo with clean caption without corrupted byte arrays.

---

## 2. Instances Table & Action Alignment

### 2.1 Unified Exe & Data Column
- Combine `IDE Executable` and `Data Directory` into a single high-density column:
  - Header: `IDE EXE & DATA`
  - Cell rendering:
    ```tsx
    <div className="flex flex-col gap-0.5">
      <span className="font-mono text-xs text-white/90 font-medium">{inst.executable_name}</span>
      <span className="font-mono text-[10px] text-white/50">{inst.data_dir_suffix}</span>
    </div>
    ```

### 2.2 Prompt Button Outside
- Move the `Prompts` button out of the `...` overflow menu.
- Place it directly into the primary actions group alongside `Switch` and `Stop`.

### 2.3 Layout Overflow Guard
- Constrain table container to `w-full max-w-full overflow-x-auto` to prevent right-side blowout.

---

## 3. Prompt Tree UI Overhaul & Synchronization

### 3.1 DetailHeader Overlap Fix
- Address screenshot Media 1 (`assets/screenshots/antigravity-manager-agm-repository-issues-01.png`):
  - Stack the top toolbar into clean vertical tiers:
    - Tier 1: Project & Conversation Title, Status Badges (`RUNNING`, `PID`, `Duration`).
    - Tier 2: Primary Actions: `[ Copy All | v ]`, `[ Send N | v ]`, `[ Rerun xN | v ]`.
    - Tier 3: Tabs Capsule: `[ Prompt Instruction | AI Results & Outputs ]` alongside `[ Preview | Raw | Edit | Details ]`.
  - Prevent relative/absolute overlapping by using standard flexbox wrapping (`flex flex-wrap items-center gap-3`).

### 3.2 Real-Time Transcript Refresh & Synchronization
- When user clicks `Refresh` or after an external prompt dispatch:
  - Invalidate in-memory tree cache (`prompt_tree_cache`).
  - Read from `transcript_full.jsonl` first, fallback to `transcript.jsonl`.
  - Push updated conversation turns directly into React state.

### 3.3 Copy All With Rich Images
- Address screenshot Media 3 (`assets/screenshots/antigravity-manager-agm-repository-issues-03.png`):
  - Standardize `copyPromptWithRichImages` in `clipboard.ts`.
  - Extract all `<img>` tags and markdown images (`![...](...)`).
  - Ensure dual-MIME `ClipboardItem` payload (`text/html` and `text/plain`).
  - Align top and bottom Copy buttons to trigger the exact same clipboard pipeline with toast notification.

---

## 4. QuotaProgressBar Multi-Segment Styling

### 4.1 Color-Segmented Track
- Address screenshot Media 2 (`assets/screenshots/antigravity-manager-agm-repository-issues-02.png`):
  - Instead of a single flat track or monochrome line, divide the bar into distinct colored segments:
    - Segment 1 (0% - 20%): Red (`#ef4444`)
    - Segment 2 (20% - 40%): Orange (`#f97316`)
    - Segment 3 (40% - 60%): Amber (`#eab308`)
    - Segment 4 (60% - 80%): Lime / Emerald (`#10b981`)
    - Segment 5 (80% - 100%): Cyan / Neon Green (`#1af18d`)
  - Highlight filled segments up to the current quota value.

---

## 5. Log & Audit Enhancements

### 5.1 Telemetry Attributes (`data-ui-name`, `data-ui-id`)
- Add `data-ui-name` and `data-ui-id` to buttons in the audit and log sections.
- Display friendly button labels (e.g., "Copy All", "Refresh", "Switch Account") in audit history rather than raw technical identifiers.

### 5.2 Audit Summary Copy Button
- Add a dedicated copy button (`📋 Copy Summary`) in `AuditPanel.tsx` and summary dialogs.

### 5.3 System Log Filtering
- Add a toggle in the AGM log viewer: `Hide System Logs` (filters out noisy `HEARTBEAT`, `TELEMETRY`, `TOKEN_USAGE` entries).
