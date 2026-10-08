# Subtask 04: Truncation Banner and Rich Copy Export

**Parent Plan:** [145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md](../../145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md)  
**Status:** `IN PROGRESS`  

---

## Objectives
1. Remove intrusive mid-panel truncation banner that disrupted the preview layout.
2. Replace raw `<truncated N bytes>` tokens with elegant inline badges (`✂ [Omitted N bytes]`) without breaking sentences.
3. Dual-source transcript ingestion: automatically read full content from `transcript_full.jsonl` if `truncated_fields` is set.
4. Implement "Copy with Image" using rich HTML clipboard with embedded image references/base64 data.
5. Provide dedicated Export dropdown (.md with images and .json with structured metadata).
