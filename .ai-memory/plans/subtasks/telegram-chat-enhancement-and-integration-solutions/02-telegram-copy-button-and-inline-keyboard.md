# Subtask 02: Telegram Copy Button & Inline Keyboard

**File:** `.ai-memory/plans/subtasks/telegram-chat-enhancement-and-integration-solutions/02-telegram-copy-button-and-inline-keyboard.md`  
**Parent Task:** `telegram-chat-enhancement-and-integration-solutions`  
**Parent Spec:** `02-spec/21-app/telegram-chat-enhancement-and-integration-solutions/01-architecture-spec.md`  
**Visual Reference:** [telegram-chat-enhancement-and-integration-solutions-01.png](file:///assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png)  
**Status:** Ready for Implementation  
**Estimated Effort:** 1 Subtask Unit  

---

## 1. Objective & Problem Analysis

### 1.1 Context & Current Limitations
When an automated account switch or prompt backup event occurs, operators frequently need to inspect or reuse the prompt instructions that were executing on the workspace. In the current system (as shown in [telegram-chat-enhancement-and-integration-solutions-01.png](file:///assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png)):
- Operators must manually highlight and copy text across mobile screens, which is error-prone when text is mixed with metadata headers and timestamps.
- Although Bot API 7.8 introduced direct clipboard copy buttons (`copy_text`), payloads exceeding 4,096 characters trigger immediate Telegram HTTP 400 Bad Request errors (`BUTTON_DATA_INVALID`).
- Switch alerts lack quick follow-up navigation buttons (e.g. checking live observation or account list), forcing operators to switch to keyboards and type `/observe` or `/accounts` manually.

### 1.2 Target Enhancements
1. **Clamped Native Clipboard Button:** Implement a robust `copy_text` button builder that strictly clamps payload text to $\le 4,000$ characters, safely avoiding the 4,096-character limit while preserving the operator's active prompt.
2. **Interactive Quick Actions Keyboard:** Attach secondary navigation buttons (`[📊 Live Observe]`, `[👥 Accounts]`) to alert messages and prompt inspection outputs.
3. **Multi-Chunk Button Deduping:** Guarantee that `reply_markup` is mounted strictly on the final message chunk when a message exceeds 3,800 characters, preventing duplicate or ghost button rows across multi-part messages.

---

## 2. Target Files & Exact Code References

### 2.1 `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
- **Lines 160–175 (`reply_markup` assembly in `dispatch_telegram_switch_alert`):**
  - Current logic:
    ```rust
    let reply_markup = if !prompt_preview_text.is_empty() {
        Some(serde_json::json!({
            "inline_keyboard": [
                [
                    {
                        "text": "📋 Copy Prompt",
                        "copy_text": {
                            "text": prompt_preview_text
                        }
                    }
                ]
            ]
        }))
    } else {
        None
    };
    ```
  - Defect: If `prompt_preview_text` exceeds 4,096 characters (or if full prompt content is passed), Telegram API rejects the entire request with 400 Bad Request.
  - Enhancement: Pass through `clamp_telegram_copy_text(&prompt_preview_text, 4000)` and add a second row with quick navigation buttons:
    ```rust
    let reply_markup = build_alert_inline_keyboard(
        if !prompt_preview_text.is_empty() { Some(&prompt_preview_text) } else { None },
        true // is_interactive
    );
    ```

### 2.2 `src-tauri/src/modules/telegram_inbound/formatting.rs`
- **Lines 146–229 (`send_telegram_message_with_markup`):**
  - Lines 171–181: Ensure `is_last` gate correctly attaches `reply_markup` only to chunk `idx + 1 == total`.
  - Lines 193–197: Ensure retry fallback on 400 Bad Request preserves `reply_markup` on the plain text payload.
- **Helper Function Addition:**
  - Implement `clamp_telegram_copy_text(text: &str, max_chars: usize) -> String` ensuring UTF-8 character boundary safety:
    ```rust
    pub fn clamp_telegram_copy_text(text: &str, max_chars: usize) -> String {
        let limit = if max_chars == 0 || max_chars > 4000 { 4000 } else { max_chars };
        if text.chars().count() <= limit {
            text.to_string()
        } else {
            let clamped: String = text.chars().take(limit - 32).collect();
            format!("{}\n\n... [Truncated for clipboard]", clamped)
        }
    }
    ```

### 2.3 `src-tauri/src/modules/telegram_inbound/reports_b.rs`
- **Lines 56–70 (`format_expand_prompt_report`):**
  - Expose a companion helper `build_expand_prompt_reply_markup(prompt_content: &str) -> Option<Value>` so that expanded prompt responses also offer the `📋 Copy Prompt` button.

---

## 3. Data Structures & Invariants

### 3.1 Button Builder Types & Invariants
```rust
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct InlineKeyboardConfig {
    pub has_copy_button: bool,
    pub has_observe_shortcut: bool,
    pub has_accounts_shortcut: bool,
    pub copy_payload: Option<String>,
}

pub fn build_inline_keyboard(config: &InlineKeyboardConfig) -> Option<Value> {
    let mut rows: Vec<Vec<Value>> = Vec::new();

    if config.has_copy_button {
        if let Some(ref text) = config.copy_payload {
            let clamped = clamp_telegram_copy_text(text, 4000);
            rows.push(vec![json!({
                "text": "📋 Copy Prompt",
                "copy_text": {
                    "text": clamped
                }
            })]);
        }
    }

    let mut action_row: Vec<Value> = Vec::new();
    if config.has_observe_shortcut {
        action_row.push(json!({
            "text": "📊 Live Observe",
            "callback_data": "/observe"
        }));
    }
    if config.has_accounts_shortcut {
        action_row.push(json!({
            "text": "👥 Accounts",
            "callback_data": "/accounts"
        }));
    }

    if !action_row.is_empty() {
        rows.push(action_row);
    }

    if rows.is_empty() {
        None
    } else {
        Some(json!({ "inline_keyboard": rows }))
    }
}
```

### 3.2 Invariants & Rules
1. **Positive Booleans Invariant:**
   - Field names must be `has_copy_button`, `has_observe_shortcut`, `has_accounts_shortcut`, `is_clamped`, `is_last`.
   - Never use `disable_copy` or `no_shortcuts`.
2. **Clipboard Character Cap Invariant:**
   - `copy_text.text` length must never exceed 4,000 characters under any circumstance.
3. **No Callback Overhead for Copy:**
   - The `copy_text` button is completely client-side in Telegram clients (Bot API 7.8+). It does not generate server network callbacks or database operations.

---

## 4. Step-by-Step Implementation Plan

### Step 1: Implement `clamp_telegram_copy_text` in `formatting.rs`
- Add character boundary-safe clamping function in `src-tauri/src/modules/telegram_inbound/formatting.rs`.
- Verify behavior with multi-byte Unicode strings (e.g. emojis, Chinese, Japanese, Korean characters).

### Step 2: Implement Inline Keyboard Builders
- Create structured keyboard generator helper in `formatting.rs` or `telegram_alerts.rs`.
- Guarantee that `copy_text` is formatted according to Bot API 7.8 specification.

### Step 3: Wire Copy Buttons into Alert and Expand Dispatches
- Update `dispatch_telegram_switch_alert` in `telegram_alerts.rs` to use the clamped keyboard builder.
- Update `format_expand_prompt_report` in `reports_b.rs` to support companion copy buttons when sent via `send_telegram_message_with_markup`.

### Step 4: Unit Testing & Clamping Assertions
- Add unit tests in `src-tauri/src/modules/telegram_inbound/tests.rs`:
  - `test_clamp_telegram_copy_text_short()`: Verifies short text remains unchanged.
  - `test_clamp_telegram_copy_text_boundary()`: Verifies 5,000-char text is clamped to $\le 4,000$ chars with truncation marker.
  - `test_clamp_telegram_copy_text_unicode()`: Verifies multi-byte UTF-8 does not panic on slicing.

---

## 5. Acceptance Criteria & Quality Gates

| Check | Acceptance Criteria | Verification Method |
| :--- | :--- | :--- |
| **AC-01** | `copy_text.text` payload never exceeds 4,000 characters regardless of prompt length. | Unit test with 10,000-char input |
| **AC-02** | Inline keyboard with `copy_text` renders cleanly on Telegram Bot API 7.8+ clients without 400 errors. | API response code verification |
| **AC-03** | Multi-chunk messages attach inline keyboards only to the final message chunk. | `send_telegram_message_with_markup` chunk test |
| **AC-04** | All new structs and functions adhere to positive boolean naming conventions. | Code review check |
| **AC-05** | Pre-flight checks (`cargo fmt -- --check`, `cargo clippy`) pass cleanly. | Pre-flight CI gates |

---

## 6. Verification Steps & Commands

```powershell
# 1. Run Telegram inbound unit tests including clamping tests
cd src-tauri
cargo test modules::telegram_inbound::tests

# 2. Run Rust formatting check
cargo fmt -- --check

# 3. Run Clippy lint gate
cargo clippy --all-targets --all-features
```
