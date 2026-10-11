# Subtask 01: Telegram HTML Card Formatting & Expandable Blocks

**File:** `.ai-memory/plans/subtasks/telegram-chat-enhancement-and-integration-solutions/01-telegram-html-card-formatting-and-expandable-blocks.md`  
**Parent Task:** `telegram-chat-enhancement-and-integration-solutions`  
**Parent Spec:** `02-spec/21-app/telegram-chat-enhancement-and-integration-solutions/01-architecture-spec.md`  
**Visual Reference:** [telegram-chat-enhancement-and-integration-solutions-01.png](file:///assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png)  
**Status:** Ready for Implementation  
**Estimated Effort:** 1 Subtask Unit  

---

## 1. Objective & Problem Analysis

### 1.1 Context & Current Limitations
As shown in the visual evidence captured in [telegram-chat-enhancement-and-integration-solutions-01.png](file:///assets/screenshots/telegram-chat-enhancement-and-integration-solutions-01.png), Telegram switch alert cards and status reports currently render prompt bodies and system logs using plain `<pre><code>...</code></pre>` blocks or unformatted raw text. When prompt lengths exceed 100–300 words, the vertical message body dominates the operator's mobile viewport, pushing previous context out of view and requiring cumbersome scrolling.

### 1.2 Target Enhancements
1. **Collapsible Prompt Blocks:** Adopt Telegram Bot API 7.2+ `<blockquote expandable>` to collapse long prompt contents into an elegant, 3-line preview with an interactive "Expand" toggle handled natively by Telegram clients.
2. **Robust Tag Balancing across Chunk Boundaries:** Enhance `formatting.rs` to guarantee that if an expandable blockquote spans a 3,800-character chunk boundary, the opening `<blockquote expandable>` is cleanly closed at the end of chunk $N$ (`</blockquote>`) and re-opened at the start of chunk $N+1$ (`<blockquote expandable>`), preventing Telegram API 400 bad request errors (`can't parse entities`).
3. **Card Visual Restructuring:** Standardize alert headers, key-value rows, balance indicators, and status badges across `telegram_alerts.rs`, `reports_a.rs`, and `reports_b.rs`.

---

## 2. Target Files & Exact Code References

### 2.1 `src-tauri/src/modules/telegram_inbound/formatting.rs`
- **Lines 9–62 (`get_unclosed_html_tags`):**
  - Current logic parses opening tags by extracting `tag_content.split_whitespace().next()`.
  - When encountering `<blockquote expandable>`, `opening` evaluates to `"blockquote"`.
  - Ensure the stack accurately tracks the `is_expandable` attribute so that carry-over tags generated across chunks correctly re-open as `<blockquote expandable>` rather than plain `<blockquote>`.
- **Lines 66–143 (`chunk_telegram_text`):**
  - When `carry_over_tags` are prepended to `chunk`, tags with attributes (such as `blockquote expandable`) must be restored with their attributes intact.
  - Slicing and char boundary logic must not slice inside `<blockquote expandable>` tags.
- **Lines 250–282 (`clean_for_telegram_html`):**
  - Verify HTML entity escaping for text contents inside expandable blocks, escaping `&`, `<`, and `>` while preserving valid enclosing tags.

### 2.2 `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
- **Lines 114–122 (`prompt_section` formatting):**
  - Replace:
    ```rust
    let prompt_section = if !prompt_preview_text.is_empty() {
        format!(
            "\n💬 <b>Active Prompt:</b> ({} words)\n<pre><code>{}</code></pre>\n",
            prompt_words_count,
            crate::modules::telegram_inbound::clean_for_telegram_html(&prompt_preview_text, 1500)
        )
    } else {
        String::new()
    };
    ```
  - With structured expandable blockquote syntax:
    ```rust
    let prompt_section = if !prompt_preview_text.is_empty() {
        format!(
            "\n💬 <b>Active Prompt:</b> ({} words)\n<blockquote expandable><code>{}</code></blockquote>\n",
            prompt_words_count,
            crate::modules::telegram_inbound::clean_for_telegram_html(&prompt_preview_text, 1500)
        )
    } else {
        String::new()
    };
    ```
- **Lines 124–159 (`text` formatting):**
  - Standardize divider bars and emojis for visual consistency with AGM design conventions.

### 2.3 `src-tauri/src/modules/telegram_inbound/reports_a.rs`
- **Lines 218–233 (`format_observe_report` prompt preview):**
  - Replace nested indentations with `<blockquote expandable>` blocks for running prompt previews across workspaces.
- **Lines 366–392 (`format_fleet_identity_report`):**
  - Ensure standalone identity cards maintain compact card layout with standardized dividers and badges.

### 2.4 `src-tauri/src/modules/telegram_inbound/reports_b.rs`
- **Lines 56–70 (`format_expand_prompt_report`):**
  - Format full prompt instructions inside `<blockquote expandable>` to give operators interactive control over reading large instructions.
- **Lines 135–147 (`format_prompts_query_report`):**
  - Render query result previews inside expandable blockquotes.

---

## 3. Data Structures & Invariants

### 3.1 Tag Stack Representation
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTagInfo {
    pub tag_name: String,
    pub is_expandable: bool,
}

impl HtmlTagInfo {
    pub fn opening_tag(&self) -> String {
        if self.tag_name == "blockquote" && self.is_expandable {
            "<blockquote expandable>".to_string()
        } else {
            format!("<{}>", self.tag_name)
        }
    }

    pub fn closing_tag(&self) -> String {
        format!("</{}>", self.tag_name)
    }
}
```

### 3.2 Invariants & Rules
1. **Positive Booleans Invariant:**
   - Use `is_expandable`, `is_active`, `is_success`, `has_prompt`, `is_auto`.
   - Never use inverted names like `is_not_collapsed` or `unclosed_tags`.
2. **Chunk Size Limit:**
   - Hard maximum of 3,800 characters per chunk (safely under Telegram's 4,096 limit).
3. **Safe Plain Text Fallback:**
   - If Telegram returns HTTP 400 on `sendMessage`, `strip_telegram_html_tags()` must strip all tags (including `<blockquote expandable>` and `</blockquote>`) and resend clean text without failing silently.

---

## 4. Step-by-Step Implementation Plan

### Step 1: Update Tag Inspection in `formatting.rs`
- Update `get_unclosed_html_tags` or implement `get_unclosed_html_tag_infos` to detect `expandable` attribute on `blockquote`.
- Ensure closing tag `</blockquote>` removes the blockquote entry regardless of whether it opened with attributes.
- Update `chunk_telegram_text` carry-over tag concatenation so chunk $N+1$ prepends `<blockquote expandable>` if an expandable block was split.

### Step 2: Refactor `telegram_alerts.rs` Alert Card
- In `dispatch_telegram_switch_alert`, update `prompt_section` to use `<blockquote expandable><code>...</code></blockquote>`.
- Validate that empty prompts produce an empty string without rendering an empty blockquote.

### Step 3: Refactor Observation & Search Reports in `reports_a.rs` & `reports_b.rs`
- In `format_observe_report`, wrap running prompt previews in `<blockquote expandable>`.
- In `format_expand_prompt_report`, wrap full prompt bodies in `<blockquote expandable>`.
- In `format_prompts_query_report`, wrap prompt previews in `<blockquote expandable>`.

### Step 4: Unit Testing & Boundary Validation
- Add unit tests in `src-tauri/src/modules/telegram_inbound/tests.rs`:
  - `test_chunk_telegram_text_with_expandable_blockquote()`
  - `test_strip_html_tags_with_expandable_blockquote()`
  - `test_clean_for_telegram_html_entities()`

---

## 5. Acceptance Criteria & Quality Gates

| Check | Acceptance Criteria | Verification Method |
| :--- | :--- | :--- |
| **AC-01** | Account switch alert cards render `<blockquote expandable><code>...</code></blockquote>` around active prompt bodies. | Automated inspection & unit tests |
| **AC-02** | Messages longer than 3,800 chars split across chunks maintain balanced `<blockquote expandable>` tags in both chunks. | `cargo test --package antigravity-manager --lib modules::telegram_inbound::tests` |
| **AC-03** | Plain text fallback (`strip_telegram_html_tags`) cleanly strips `<blockquote expandable>` tags without remnants. | Unit test assertion |
| **AC-04** | Rust codebase passes formatting and clippy gates without errors. | `cargo fmt -- --check` & `cargo clippy --all-targets --all-features` |

---

## 6. Verification Steps & Commands

```powershell
# 1. Run Telegram inbound unit tests
cd src-tauri
cargo test modules::telegram_inbound::tests

# 2. Run Rust formatting check
cargo fmt -- --check

# 3. Run Clippy lint gate
cargo clippy --all-targets --all-features
```
