# Subtask 05: Multimodal Image Tracing and Configurable Word Limits

**Module:** `cli-and-ui-testing-for-antigravity-backups`
**Version:** 4.239.0
**Updated:** 2026-10-20
**Status:** Ready for Execution

---

## 1. Context & Objectives

This subtask implements robust multimodal image and sub-image tracing within the prompt detection engine, and introduces a configurable word limit setting (`prompt_preview_words`) spanning the Rust backend, TypeScript types, Settings UI, and Prompt Tree View components.

### Core Targets:
- Image & Sub-Image Tracing: `src-tauri/src/modules/repo_db/detection.rs`
- Config Model: `src-tauri/src/models/config.rs`
- TypeScript Config: `src/types/config.ts`
- Settings UI: `src/pages/Settings/tabs/AdvancedMaintenanceSection.tsx`
- Tree View Components: `src/components/instances/PromptTreeViewModal.tsx` & `src/components/instances/PromptTreeView/ConversationNode.tsx`

---

## 2. Technical Invariants & Guardrails

1. **Multimodal Payload Invariant**:
   - Extracted images must distinguish between inline Base64 data URIs (`data:image/...`) and local filesystem paths (`file://...` or direct paths).
   - Multi-image prompts must serialize into a stable JSON array format (`["uri1", "uri2"]`), preserving 100% of attached media.
   - Upstream screenshot tokens (`SS_ID_*` or `screenshot_id=*`) must be extracted and preserved without corrupting JSON structures.
2. **Word Limit Clamping Invariant**:
   - `prompt_preview_words` must strictly clamp between `100` and `200` words.
   - Default value is `100` words when absent from existing configurations.
3. **Strict Relative Git Paths**:
   - All code comments, logs, and documentation must reference relative Git paths only (`src/...`, `src-tauri/...`).

---

## 3. Step-by-Step Implementation Plan

### Step 1: Image & Sub-Image Tracing Engine (`src-tauri/src/modules/repo_db/detection.rs`)
Verify and ensure complete coverage for `extract_image_uris_or_paths`, `serialize_image_uris`, `parse_image_uris`, and `extract_screenshot_id_token`:
- Parse `data:image/` URIs until terminating whitespace or closing quotes/brackets.
- Parse local image references with extensions `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.svg`, `.bmp`.
- Strip `file:///` and `file://` prefixes for normalized disk paths.
- Attach `;screenshot_id=<token>` to single-string payloads or embed appropriately in composite payloads.

### Step 2: Rust Backend Config Extension (`src-tauri/src/models/config.rs`)
Add `prompt_preview_words` to `AppConfig`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ... existing fields ...
    #[serde(default = "default_prompt_preview_words")]
    pub prompt_preview_words: usize,
}

fn default_prompt_preview_words() -> usize {
    100
}

impl AppConfig {
    pub fn get_clamped_prompt_preview_words(&self) -> usize {
        self.prompt_preview_words.clamp(100, 200)
    }
}
```

### Step 3: TypeScript Type Synchronizations (`src/types/config.ts`)
Add `prompt_preview_words` to `AppConfig`:
```typescript
export interface AppConfig {
    // ... existing fields ...
    prompt_preview_words?: number; // [NEW] Prompt preview word limit (100-200, default 100)
}
```

### Step 4: Settings UI Control (`src/pages/Settings/tabs/AdvancedMaintenanceSection.tsx`)
Mount the configurable slider and numeric input under the Prompt Backup Retention section:
- Label: `Prompt Preview Word Limit`
- Slider: `min="100" max="200" step="10"`
- Input: `type="number" min="100" max="200"`
- Visual badges: `100 min (default)`, `150 balanced`, `200 max`
- Bound change handlers enforcing `Math.min(200, Math.max(100, val))`

### Step 5: Frontend Prompt Tree View Truncation (`src/components/instances/PromptTreeView/`)
- In `ConversationNode.tsx` and `TreePanel.tsx`, read `config.prompt_preview_words ?? 100`.
- Implement word-boundary truncation helper:
```typescript
export function truncateToWordLimit(text: string, wordLimit: number): { text: string; isTruncated: boolean; totalWords: number } {
    if (!text) return { text: '', isTruncated: false, totalWords: 0 };
    const words = text.trim().split(/\s+/);
    if (words.length <= wordLimit) {
        return { text, isTruncated: false, totalWords: words.length };
    }
    return {
        text: words.slice(0, wordLimit).join(' ') + '...',
        isTruncated: true,
        totalWords: words.length,
    };
}
```
- When `isTruncated` is true, render the interactive dark-glass callout displaying omitted word counts.

---

## 4. Verification & Testing Protocol

1. **Rust Unit Tests**:
   - Author unit test `test_multimodal_image_extraction_and_sub_image_tracing` in `src-tauri/src/modules/repo_db/detection.rs`.
   - Test cases:
     - Single `data:image/png;base64,...` URI.
     - Multiple `data:image/` URIs serializing to JSON array.
     - Local file paths with `file:///` stripped.
     - Screenshot token extraction `SS_ID_10203040`.
2. **Word Limit Clamping Tests**:
   - Verify `get_clamped_prompt_preview_words()` returns 100 for 50, 150 for 150, and 200 for 300.
3. **Execution Gate**:
   ```bash
   cd src-tauri && cargo test modules::repo_db::detection::tests -- --nocapture
   ```
