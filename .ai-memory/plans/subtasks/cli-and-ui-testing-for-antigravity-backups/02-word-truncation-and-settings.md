# Subtask 02: Configurable Word Truncation Engine & Settings Synchronization

**Parent Plan**: `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md`  
**Target Files**:  
- `src-tauri/src/models/config.rs`  
- `src/types/config.ts`  
- `src-tauri/src/modules/repo_db/text_utils.rs`  

---

## 1. Scope & Objectives
1. Extend `AppConfig` in `src-tauri/src/models/config.rs` with the `prompt_preview_max_words` setting (allowed range: 100–200 words, default: 100).
2. Synchronize TypeScript frontend types in `src/types/config.ts` to expose `prompt_preview_max_words?: number`.
3. Implement `truncate_words_bounded` and bounded configuration helpers in `src-tauri/src/modules/repo_db/text_utils.rs` to enforce word-boundary truncation across CLI prompt inspection, Telegram telemetry, and UI tree views.
4. Provide comprehensive unit tests verifying Serde backward compatibility, default fallback, out-of-range clamping, and whitespace normalization.

---

## 2. Invariants & Guardrails
- **Strict Relative Git Paths Invariant**: All file references in plans, code, and comments must be strictly repository-relative (e.g., `src-tauri/src/...`, `src/...`). Never output absolute paths (`D:\...`, `C:\...`).
- **No Silent Errors Invariant**: Config loading and deserialization errors must never be swallowed with `.unwrap_or_default()` without routing or tracking warnings. Fallbacks must log via `tracing` or error store.
- **Attribution Invariant**: Attribution must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).

---

## 3. Detailed Implementation Steps

### Step 1: Rust Model Extension (`src-tauri/src/models/config.rs`)
- In struct `AppConfig`:
  ```rust
  #[serde(default = "default_prompt_preview_max_words")]
  pub prompt_preview_max_words: usize,
  ```
- Define default constructor function:
  ```rust
  fn default_prompt_preview_max_words() -> usize {
      100
  }
  ```
- Update `AppConfig::new()`:
  ```rust
  prompt_preview_max_words: 100,
  ```
- Add clamping utility method on `AppConfig`:
  ```rust
  impl AppConfig {
      /// Get the effective prompt preview word limit clamped to [100, 200]
      pub fn effective_prompt_preview_max_words(&self) -> usize {
          self.prompt_preview_max_words.clamp(100, 200)
      }
  }
  ```
- In `mod tests`:
  - Add test `test_prompt_preview_max_words_default()` verifying that omitting `prompt_preview_max_words` in JSON deserializes to `100`.
  - Add test `test_prompt_preview_max_words_clamping()` verifying values < 100 clamp to 100 and values > 200 clamp to 200.

### Step 2: TypeScript Interface Synchronization (`src/types/config.ts`)
- In `interface AppConfig`:
  ```typescript
  export interface AppConfig {
      // ... existing fields ...
      max_backup_prompts?: number; // Maximum prompt snapshots retained in SQLite backup database (default 100, range 10-1000)
      prompt_preview_max_words?: number; // Configurable prompt word preview limit (100-200, default 100)
  }
  ```
- Ensure comments indicate the valid range (`100-200`) and default value (`100`).

### Step 3: Word Truncation Engine (`src-tauri/src/modules/repo_db/text_utils.rs`)
- Implement `truncate_words_bounded`:
  ```rust
  /// Truncate text to a maximum number of whitespace-delimited words.
  /// Returns the truncated string (with trailing " ..." if truncated) and the original word count.
  pub fn truncate_words_bounded(text: &str, max_words: usize) -> (String, usize) {
      let words: Vec<&str> = text.split_whitespace().collect();
      let total = words.len();
      let limit = max_words.max(1);
      if total <= limit {
          (words.join(" "), total)
      } else {
          (format!("{} ...", words[..limit].join(" ")), total)
      }
  }

  /// Resolve prompt preview using configured word limit or explicit override.
  /// Enforces clamping to [100, 200] unless explicit CLI override is provided.
  pub fn truncate_prompt_with_config(
      text: &str,
      explicit_words: Option<usize>,
      configured_limit: usize,
  ) -> (String, usize) {
      let limit = explicit_words.unwrap_or_else(|| configured_limit.clamp(100, 200));
      truncate_words_bounded(text, limit)
  }
  ```
- In `src-tauri/src/modules/repo_db/text_utils.rs` `mod tests`:
  - Test 1: Short text under limit returns identical words without ellipsis, correct count.
  - Test 2: Text matching exact word count limit returns joined words without ellipsis.
  - Test 3: Text exceeding limit truncates cleanly at word boundary with `" ..."` appended.
  - Test 4: Empty string returns `("", 0)`.
  - Test 5: Excessive whitespace and newlines are collapsed cleanly into single spaces.

---

## 4. Verification & Validation Steps
1. Run targeted Rust unit tests:
   `cargo test --lib -- modules::repo_db::text_utils::tests`
   `cargo test --lib -- models::config::tests`
2. Validate frontend TypeScript compilation:
   `npm run build` or `npx tsc --noEmit` to verify `src/types/config.ts` consistency.
3. Run `cargo clippy --lib -- -D warnings` to verify zero compiler warnings.
