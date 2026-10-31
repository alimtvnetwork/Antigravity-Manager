# Subtask 01: Core Attachment Formatter Utility & Unit Test Suite

- **Subtask ID:** `49-01`
- **Slug:** `subtask-attachment-formatter`
- **Component:** Prompt Attachment Utility Subsystem (`src/utils/promptAttachment.ts`)
- **Status:** `READY_FOR_EXECUTION`
- **Author:** Spec Author 01 (Attachment Formatter & Architecture Specialist)
- **Parent Architecture Spec:** [02-spec/21-app/49-image-attachment-format-and-rerun-enqueue/01-architecture-spec.md](../../../../02-spec/21-app/49-image-attachment-format-and-rerun-enqueue/01-architecture-spec.md)
- **Primary Files Touched:**
  - `src/utils/promptAttachment.ts` (New file)
  - `src/utils/__tests__/promptAttachment.test.ts` (New test suite)

---

## 1. Subtask Objective & Context

This subtask implements the core foundation for task `49-image-attachment-format-and-rerun-enqueue`:
1. Author the pure TypeScript utility `src/utils/promptAttachment.ts` with zero `any` types and positive booleans.
2. Provide deterministic normalization of image file paths across Windows and POSIX environments (stripping `file:///`, decoding URI components, standardizing slash delimiters).
3. Implement `formatPromptWithAttachments` to replace legacy markdown image tags (`![Attached Image N](<uri>)`) with canonical Antigravity IDE attachment blocks (`Attachments:\n1. @[...]`).
4. Ensure strict `<USER_REQUEST>` tag preservation: if a prompt contains `</USER_REQUEST>`, the `Attachments:` block must be injected inside before the closing tag.
5. Create a standalone, comprehensive unit test suite in `src/utils/__tests__/promptAttachment.test.ts` verifying all normalization, formatting, deduplication, stripping, and edge case rules.

---

## 2. Technical Implementation Specifications

### Step 1: Types & Interfaces (`src/utils/promptAttachment.ts`)

Define explicit, fully documented types with zero `any`:

```typescript
/**
 * Raw attachment descriptor supporting strings, file descriptors, or media objects.
 */
export type PromptAttachmentInput =
    | string
    | {
          path?: string;
          src?: string;
          dataUri?: string;
      };

/**
 * Options controlling attachment formatting and environment resolution.
 * All flags follow positive boolean naming conventions.
 */
export interface PromptAttachmentFormatOptions {
    /**
     * Repository root path used to resolve relative file references.
     */
    repoPath?: string;

    /**
     * When true, preserves the <USER_REQUEST>...</USER_REQUEST> envelope
     * by injecting attachments inside before </USER_REQUEST>.
     * Default: true.
     */
    isUserRequestTagPreserved?: boolean;

    /**
     * When true, extracts any inline markdown images from prompt text
     * and consolidates them into the canonical Attachments: block.
     * Default: true.
     */
    isInlineImageExtractionEnabled?: boolean;

    /**
     * Optional boolean override to simulate Windows vs POSIX path separators in tests.
     * When omitted, detects runtime environment via navigator.userAgent or process.platform.
     */
    isWindowsPlatformOverride?: boolean;
}
```

### Step 2: `normalizeAttachmentPath` Implementation Contract

- **Signature:**
  ```typescript
  export function normalizeAttachmentPath(
      rawPathOrUri: string,
      repoPath?: string,
      isWindowsPlatformOverride?: boolean
  ): string
  ```
- **Behavior:**
  1. Return empty string if `rawPathOrUri` is empty or whitespace.
  2. Strip `file:///` and `file://` prefixes.
  3. Decode URI components (`decodeURIComponent`) safely with try/catch fallback.
  4. If targeting Windows (detected or `isWindowsPlatformOverride === true`):
     - Strip leading slash before drive letters (e.g., `/C:/path` -> `C:/path`).
     - Replace all forward slashes `/` with backslashes `\`.
     - Resolve relative paths against `repoPath` using `\` if provided.
  5. If targeting POSIX:
     - Replace backslashes `\` with forward slashes `/`.
     - Resolve relative paths against `repoPath` using `/` if provided.
  6. Trim trailing whitespace.

### Step 3: `stripAttachmentsFromPrompt` Implementation Contract

- **Signature:**
  ```typescript
  export function stripAttachmentsFromPrompt(prompt: string): string
  ```
- **Behavior:**
  1. Return empty string if input is falsy.
  2. Strip all legacy markdown image tags:
     - Regex: `/!\[.*?\]\((?:data:image\/[^;]+;base64,[A-Za-z0-9+/=]+|https?:\/\/[^\s)]+|[^\s)]+?)\)/gi`
     - Secondary catch-all: `/!\[.*?\]\(.*?\)/gi`
  3. Strip canonical `Attachments:` blocks:
     - Regex: `/\r?\n\r?\n?Attachments:\s*(?:\r?\n\s*\d+\.\s*@\[.*?\])+/gi`
  4. Collapse redundant newlines (`\n{3,}` -> `\n\n`).
  5. Trim trailing whitespace while preserving clean structure around `<USER_REQUEST>` and `</USER_REQUEST>`.

### Step 4: `extractAttachmentsFromPrompt` Implementation Contract

- **Signature:**
  ```typescript
  export function extractAttachmentsFromPrompt(
      prompt: string,
      repoPath?: string,
      isWindowsPlatformOverride?: boolean
  ): string[]
  ```
- **Behavior:**
  1. Scan prompt for `\d+\.\s*@\[(.*?)\]` entries.
  2. Scan prompt for `!\[.*?\]\((.*?)\)` entries.
  3. Normalize each extracted path using `normalizeAttachmentPath(extracted, repoPath, isWindowsPlatformOverride)`.
  4. Deduplicate paths preserving first appearance.

### Step 5: `formatPromptWithAttachments` Implementation Contract

- **Signature:**
  ```typescript
  export function formatPromptWithAttachments(
      prompt: string,
      attachments: PromptAttachmentInput[],
      options?: PromptAttachmentFormatOptions
  ): string
  ```
- **Behavior:**
  1. Parse options with positive boolean defaults:
     - `isUserRequestTagPreserved = options?.isUserRequestTagPreserved ?? true`
     - `isInlineImageExtractionEnabled = options?.isInlineImageExtractionEnabled ?? true`
  2. Extract any inline images from prompt if `isInlineImageExtractionEnabled === true`.
  3. Clean the prompt body using `stripAttachmentsFromPrompt(prompt)`.
  4. Collect all attachment candidate strings:
     - From `attachments` array: string items or `{ path }`, `{ src }`, `{ dataUri }` properties.
     - From extracted inline images.
  5. Filter out data URIs (`data:image/...`) or non-file URLs if necessary, or preserve local paths.
  6. Normalize all paths via `normalizeAttachmentPath`.
  7. Deduplicate collected paths (case-insensitive key comparison when on Windows, preserving original case).
  8. If deduplicated list is empty, return the cleaned prompt text directly.
  9. Assemble `Attachments:` block:
     ```text
     Attachments:
     1. @[path1]
     2. @[path2]
     ```
  10. If `isUserRequestTagPreserved === true` and cleaned prompt includes `</USER_REQUEST>`:
      - Replace `</USER_REQUEST>` with `\n\n${attachmentsBlock}\n</USER_REQUEST>`.
  11. Otherwise:
      - Return `${cleanedPrompt}\n\n${attachmentsBlock}`.

---

## 3. Unit Test Specification (`src/utils/__tests__/promptAttachment.test.ts`)

The test suite must run directly using Node/tsx without browser dependencies:

```typescript
// Test Execution Matrix
// 1. normalizeAttachmentPath
//    - Windows file:/// protocol stripping: file:///C:/media/test.png -> C:\media\test.png
//    - Windows URI decoding: file:///C:/My%20Projects/pic.png -> C:\My Projects\pic.png
//    - Windows leading slash removal: /C:/folder/pic.png -> C:\folder\pic.png
//    - Windows forward slash conversion: C:/folder/pic.png -> C:\folder\pic.png
//    - Windows relative path join: assets/img.png with repoPath C:\work\repo -> C:\work\repo\assets\img.png
//    - POSIX file:/// protocol: file:///home/user/img.png -> /home/user/img.png
//    - POSIX relative path join: img.png with repoPath /home/user -> /home/user/img.png
// 2. stripAttachmentsFromPrompt
//    - Strips legacy markdown image tags: ![Alt](path) and ![Attached Image 1](...)
//    - Strips canonical Attachments: block
//    - Preserves prompt body text and <USER_REQUEST>...</USER_REQUEST> tags
// 3. formatPromptWithAttachments
//    - Formats single attachment: prompt + Attachments:\n1. @[path]
//    - Formats multiple attachments with 1-based indexing
//    - Deduplicates identical paths
//    - Injects inside </USER_REQUEST> before closing tag
//    - Strips existing legacy markdown images and converts them to Attachments:
//    - Idempotence: running format twice produces identical output
//    - Empty attachment list returns clean prompt without Attachments: header
```

---

## 4. Verification Commands

To verify subtask implementation:

```bash
# Run the unit test suite directly
npx tsx src/utils/__tests__/promptAttachment.test.ts

# Type-check TypeScript codebase
npm run build
```

---

## 5. Exit Criteria & Definition of Done

- [x] Full architecture spec approved: `02-spec/21-app/49-image-attachment-format-and-rerun-enqueue/01-architecture-spec.md`.
- [ ] `src/utils/promptAttachment.ts` authored with zero `any` and positive boolean options.
- [ ] `src/utils/__tests__/promptAttachment.test.ts` authored with 100% test pass rate.
- [ ] `stripAttachmentsFromPrompt` removes both legacy markdown images and canonical `Attachments:` blocks.
- [ ] `formatPromptWithAttachments` injects properly inside `</USER_REQUEST>` tag when present.
- [ ] No absolute workstation paths introduced into documentation or repository files.
