# Issue: Nested Code Fence Rendering Broken in Docs Viewer

**Updated:** 2026-04-05
**Status:** Resolved
**Severity:** Medium

---

## Root Cause

The `specTree.json` data file stored markdown content with **3 backticks** (```` ``` ````) for fenced code blocks that should have had **4 backticks** (```` ```` ````). This caused the markdown parser's `extractCodeBlocks` to treat inner 3-backtick fences as closing markers, splitting a single code block into multiple fragments.

### Affected File

- `02-spec/01-spec-authoring-guide/06-cli-module-template.md` — contains a `````markdown` block (4 backticks) wrapping a template that itself contains ``` blocks (3 backticks).

### Why the Parser Logic Was Correct

The `codeBlockExtractor.ts` correctly handles variable-length fences:
- `isOpeningFence` captures the backtick count via `` /^(`{3,})/ ``
- `isClosingFence` requires `trimmed.length >= fence.length` — so a 3-backtick line cannot close a 4-backtick block

The bug was **data corruption**: `specTree.json` had the content with only 3 backticks on the outer fence, so the parser correctly (but undesirably) closed the block at the first inner ``` fence.

### Symptoms

- Code block showed only 24 lines instead of 60
- Folder tree structure rendered as formatted prose outside the code block
- Remaining markdown content rendered as a separate code block

---

## Fix Applied

Replaced the `specTree.json` content for `06-cli-module-template.md` with the actual file content from disk, preserving the 4-backtick fences.

---

## Prevention

When updating `specTree.json` content, always read from the source `.md` file rather than manually editing JSON strings. Backtick counts are easily lost in JSON string escaping.

---

## Related Fix: Inline Code Placeholders in Checklists (2026-04-05)

**Root Cause**: Pipeline extracted inline codes *before* checklists, so checklist items contained raw `INLINECODE_XX` placeholder text that wasn't restored.

**Fix**: Swapped pipeline order in `markdownParser.ts` — checklists now extracted before inline codes. Added inline code rendering to `checklistBuilder.ts` `applyInlineMarkdown`. Also added Export button alongside Copy on all checklists.

---

## Cross-References

| Reference | Location |
|-----------|----------|
| Code block extractor | `src/components/markdown/codeBlockExtractor.ts` |
| Markdown parser pipeline | `src/components/markdown/markdownParser.ts` |
| Checklist builder | `src/components/markdown/checklistBuilder.ts` |
| Spec data source | `src/data/specTree.json` |
| CLI module template | `02-spec/01-spec-authoring-guide/06-cli-module-template.md` |
| Visual rendering guide | `02-spec/08-docs-viewer-ui/02-features/07-visual-rendering-guide.md` |
