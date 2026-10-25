# Subtask 01: Canonical Attachment Formatting & Parsers

**Parent Plan**: `.ai-memory/plans/pending/227-rerun-and-attachment-format-correction.md`  
**Subtask ID**: `227-01`  
**Sequence**: `227`  
**Status**: READY_FOR_EXECUTION  
**Target Branch**: `main`  
**Assigned Worker**: Worker 01  
**Author**: Spec Author 01  
**Attribution**: strictly `@aukgit`  

---

## 1. Context & Objectives

The user requirement mandates:
> "Second is that when you put the attachment, you did not put the actual format that I have given you. In the third bracket, you need to put the file, then semicolon, and then the file path. You didn't do that. You just used third bracket and file path, which is very wrong. You need to fix it for the image path."

In South Asian technical English conventions, "third brackets" designates square brackets `[...]`. The canonical format requires:
```text
@[file;<file_path>]
```
For example:
```text
Attachments:
1. @[file;assets/screenshots/image.png]
2. @[file;assets/screenshots/image2.png]
```

### Core Goals:
1. Update `src/utils/promptAttachment.ts` to output `@[file;<path>]` in `buildAttachmentsBlock` and update all parsers/extractors to strip `file;`.
2. Update `src/components/instances/PromptTreeView/clipboard.ts` to recognize `@[file;<path>]` in `extractAssets` and strip `file;` prefix during URI and asset extraction.
3. Update `src/components/instances/PromptTreeView/PromptInstructionView.tsx` to handle `@[file;<path>]` in image preview extraction and rerun prompt formatting.
4. Update `src-tauri/src/modules/repo_db/detection.rs` to parse `@[file;<path>]` and strip the `file;` prefix in `extract_image_uris_or_paths`.
5. Create comprehensive unit tests in `src/utils/__tests__/promptAttachment.test.ts` and add tests in `detection.rs`.

---

## 2. Target Files & Disjoint Boundaries

Only Worker 01 is permitted to modify the following files:
- `src/utils/promptAttachment.ts`
- `src/utils/__tests__/promptAttachment.test.ts`
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src-tauri/src/modules/repo_db/detection.rs`

No other files may be modified in this subtask.

---

## 3. Detailed File Modifications

### 3.1 `src/utils/promptAttachment.ts`

1. **`buildAttachmentsBlock(paths: string[]): string`**:
   - Change line 262 from:
     ```ts
     const lines = paths.map((p, idx) => `${idx + 1}. @[${p}]`);
     ```
   - To canonical format:
     ```ts
     const lines = paths.map((p, idx) => `${idx + 1}. @[file;${p}]`);
     ```

2. **`collectPromptAttachmentRawTargets(prompt: string): string[]`**:
   - Update bracket extraction to handle `@[file;<path>]` and legacy `@[<path>]`:
     ```ts
     const bracketRegex = /@\[(?:file;\s*)?(.*?)\]/g;
     ```
   - When extracting `bracketMatch[1].trim()`, defensively ensure any remaining `file;` prefix (case-insensitive) is stripped:
     ```ts
     let target = bracketMatch[1].trim();
     if (/^file;/i.test(target)) {
         target = target.replace(/^file;\s*/i, '');
     }
     if (target) {
         matches.push({ index: bracketMatch.index, raw: target });
     }
     ```

3. **`extractRawTarget(input: PromptAttachmentInput): string`**:
   - Update string bracket matcher to strip `file;`:
     ```ts
     const bracketMatch = /@\[(?:file;\s*)?(.*?)\]/.exec(input);
     if (bracketMatch) {
         let target = bracketMatch[1].trim();
         if (/^file;/i.test(target)) {
             target = target.replace(/^file;\s*/i, '');
         }
         return target;
     }
     ```

4. **`stripAttachmentsFromPrompt(prompt: string): string`**:
   - Update the removal regex to match both canonical `@[file;...]` and legacy `@[...]`:
     ```ts
     const withoutAttachments = withoutLegacyImages.replace(
         /(?:\r?\n)*\bAttachments:\s*(?:\r?\n\s*\d+\.\s*@\[(?:file;\s*)?.*?\])+/gi,
         ''
     );
     ```

5. **`normalizeAttachmentPath(rawPathOrUri: string, ...)`**:
   - At the beginning of normalization, strip any accidental `file;` prefix:
     ```ts
     if (/^file;\s*/i.test(clean)) {
         clean = clean.replace(/^file;\s*/i, '');
     }
     ```

---

### 3.2 `src/components/instances/PromptTreeView/clipboard.ts`

1. **`extractAssets(text: string): { images: string[]; files: string[] }`**:
   - In bracket parsing loop (around line 179):
     ```ts
     const bracketRegex = /@\[(?:file;\s*)?(.*?)\]/g;
     let bracketMatch: RegExpExecArray | null;
     while ((bracketMatch = bracketRegex.exec(text)) !== null) {
         if (bracketMatch[1]) {
             let raw = bracketMatch[1].trim();
             if (/^file;\s*/i.test(raw)) {
                 raw = raw.replace(/^file;\s*/i, '');
             }
             const isImg = Boolean(raw.match(/\.(png|jpg|jpeg|gif|webp|svg|bmp)$/i));
             const key = (extractLocalPathFromUri(raw) || raw).replace(/\\/g, '/').toLowerCase();
             if (isImg) {
                 if (!seenImages.has(key)) {
                     seenImages.add(key);
                     images.push(raw);
                 }
             } else if (!seenFiles.has(key)) {
                 seenFiles.add(key);
                 files.push(raw);
             }
         }
     }
     ```

2. **`extractLocalPathFromUri(uri: string): string | null`**:
   - At entry, defensively strip `file;` prefix if present:
     ```ts
     if (/^file;\s*/i.test(clean)) {
         clean = clean.replace(/^file;\s*/i, '');
     }
     ```

---

### 3.3 `src/components/instances/PromptTreeView/PromptInstructionView.tsx`

1. **`extractImagesFromPrompt` helper**:
   - Ensure regex extracts paths from `@[file;<path>]` as well as legacy `@[<path>]` and markdown images `![alt](<src>)`.
   - Strip `file;` prefix before adding to image source list so that `resolveImageSrc` generates valid WebView asset URIs (`convertFileSrc`).

2. **`executeDirectRerun`**:
   - Verify that `formatPromptWithAttachments` is called to generate the formatted prompt content before enqueuing:
     ```ts
     const formattedContent = formatPromptWithAttachments(content, mediaList, { repoPath });
     await enqueuePrompt(targetInstId, repoPath, formattedContent, targetConvId, undefined, mediaList.length > 0 ? mediaList : undefined);
     ```

---

### 3.4 `src-tauri/src/modules/repo_db/detection.rs`

1. **`extract_image_uris_or_paths(content: &str) -> (Vec<String>, Vec<String>)`**:
   - Update token extraction in lines 360–381:
     ```rust
     let img_exts = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".bmp"];
     for token in content.split_whitespace() {
         let clean = token.trim_matches(|c| {
             c == '"'
                 || c == '\''
                 || c == '('
                 || c == ')'
                 || c == '['
                 || c == ']'
                 || c == '<'
                 || c == '>'
                 || c == '@'
         });
         // Strip canonical "file;" prefix if present
         let without_file_prefix = clean
             .strip_prefix("file;")
             .or_else(|| clean.strip_prefix("FILE;"))
             .unwrap_or(clean)
             .trim();

         let lower = without_file_prefix.to_lowercase();
         if img_exts.iter().any(|ext| lower.ends_with(ext)) {
             let p_str = without_file_prefix
                 .strip_prefix("file:///")
                 .unwrap_or_else(|| without_file_prefix.strip_prefix("file://").unwrap_or(without_file_prefix));
             if !found_paths.contains(&p_str.to_string()) {
                 found_paths.push(p_str.to_string());
             }
         }
     }
     ```

2. Also support explicit bracket scanning so that paths with spaces or surrounded by punctuation inside `@[file;...]` are cleanly captured.

---

## 4. Concrete Unit Test Cases

### 4.1 TypeScript Unit Tests (`src/utils/__tests__/promptAttachment.test.ts`)

Create a runnable zero-dependency test file executable with `node scripts/run-frontend-tests.mjs promptAttachment` or `npm test -- promptAttachment`:

```ts
import {
    formatPromptWithAttachments,
    extractAttachmentsFromPrompt,
    stripAttachmentsFromPrompt,
    normalizeAttachmentPath,
} from '../promptAttachment';

// Test 1: Canonical formatting uses @[file;<path>]
test('formatPromptWithAttachments outputs canonical @[file;<path>] format', () => {
    const prompt = 'Investigate error in billing component';
    const attachments = ['assets/screenshots/billing-error.png', 'assets/screenshots/network-trace.png'];
    const result = formatPromptWithAttachments(prompt, attachments, { isWindowsPlatformOverride: false });

    assert(result.includes('Attachments:'));
    assert(result.includes('1. @[file;assets/screenshots/billing-error.png]'));
    assert(result.includes('2. @[file;assets/screenshots/network-trace.png]'));
});

// Test 2: Extraction handles canonical @[file;<path>]
test('extractAttachmentsFromPrompt extracts clean path from canonical @[file;<path>]', () => {
    const text = 'Check UI:\nAttachments:\n1. @[file;assets/screenshot.png]';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/screenshot.png');
});

// Test 3: Backward compatibility for legacy @[<path>]
test('extractAttachmentsFromPrompt maintains backward compatibility with legacy @[<path>]', () => {
    const text = 'Check UI:\nAttachments:\n1. @[assets/legacy.png]';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/legacy.png');
});

// Test 4: Tolerates whitespace surrounding semicolon
test('extractAttachmentsFromPrompt tolerates whitespace after file semicolon', () => {
    const text = 'Check UI:\nAttachments:\n1. @[file;   assets/spaced.png]';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/spaced.png');
});

// Test 5: Mixed formats deduplicate cleanly
test('extractAttachmentsFromPrompt deduplicates mixed canonical and legacy inputs', () => {
    const text = '1. @[file;assets/dup.png]\n2. @[assets/dup.png]';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/dup.png');
});

// Test 6: stripAttachmentsFromPrompt removes canonical @[file;...] blocks
test('stripAttachmentsFromPrompt removes canonical and legacy attachment blocks', () => {
    const text = 'Some prompt\n\nAttachments:\n1. @[file;assets/shot.png]\n2. @[legacy.png]';
    const cleaned = stripAttachmentsFromPrompt(text);
    assertEqual(cleaned, 'Some prompt');
});

// Test 7: Preserves </USER_REQUEST> tag position
test('formatPromptWithAttachments inserts attachments before </USER_REQUEST>', () => {
    const prompt = 'Please fix this bug\n</USER_REQUEST>';
    const attachments = ['assets/bug.png'];
    const result = formatPromptWithAttachments(prompt, attachments, { isWindowsPlatformOverride: false });

    assert(result.indexOf('Attachments:') < result.indexOf('</USER_REQUEST>'));
    assert(result.includes('1. @[file;assets/bug.png]'));
});
```

---

### 4.2 Rust Unit Tests (`src-tauri/src/modules/repo_db/detection.rs`)

Add test cases in `detection.rs::tests`:

```rust
#[test]
fn test_extract_canonical_bracketed_image_attachment() {
    let content = "Please check this bug:\nAttachments:\n1. @[file;fixtures/screenshots/image.png]\n2. @[file;file:///path/to/test.webp]\n3. @[file;relative/shot.png]";
    let (data_uris, paths) = extract_image_uris_or_paths(content);
    assert!(data_uris.is_empty());
    assert_eq!(
        paths,
        vec![
            "fixtures/screenshots/image.png".to_string(),
            "path/to/test.webp".to_string(),
            "relative/shot.png".to_string()
        ]
    );
}

#[test]
fn test_extract_mixed_bracketed_image_attachments() {
    let content = "Check:\n1. @[file;assets/canonical.png]\n2. @[assets/legacy.png]";
    let (data_uris, paths) = extract_image_uris_or_paths(content);
    assert!(data_uris.is_empty());
    assert_eq!(
        paths,
        vec![
            "assets/canonical.png".to_string(),
            "assets/legacy.png".to_string()
        ]
    );
}
```

---

## 5. Pre-Flight Verification & Quality Checklist

Before completing this subtask:
1. `npm test -- promptAttachment` runs with 100% green pass.
2. `cd src-tauri && cargo test --lib modules::repo_db::detection::tests` passes all tests.
3. Strict relative path verification: confirm zero `C:\` or `D:\` absolute paths exist in code or comments.
4. Attribution: ensure strictly `@aukgit` attribution without extraneous handles.
5. Error discipline: zero swallowed errors.
