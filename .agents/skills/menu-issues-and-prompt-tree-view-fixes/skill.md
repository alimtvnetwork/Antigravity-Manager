---
name: menu-issues-and-prompt-tree-view-fixes
description: Specialized skill for managing Prompt Tree View menu dismissals, eliminating context menu flickering and ghost duplicate portals, ensuring FIFO enqueued prompt turn visibility and auto-expansion, and guaranteeing robust dual-MIME image and event clipboard copying in Antigravity-Manager.
---

# Menu Issues & Prompt Tree View Fixes — Architecture & Maintenance Guide

## Overview
This skill governs the presentation, context menu lifecycle, queue visibility, and clipboard media extraction in Antigravity-Manager's Prompt Tree View (`src/components/instances/PromptTreeView/`).

## Core Invariants

### Invariant 1: Centralized Singleton Context Menu
Multiple context menus must **NEVER** coexist in the DOM.
- Context menu state (`activeContextMenu`) is hoisted to the root container (`PromptTreeViewModal.tsx` / `TreePanel.tsx`).
- Individual tree nodes (`ConversationNode.tsx`, `ProjectNode.tsx`) must **NOT** maintain independent `contextMenuPos` or `turnMenuPos` states.
- Exactly ONE portal renders into `document.body` with a full-screen transparent capture backdrop (`fixed inset-0 z-[99998] bg-transparent`) ensuring any outside click, right-click, window scroll, or `Escape` key immediately dismisses the active menu.

### Invariant 2: Auto-Expansion and Top-Tier Sorting for Enqueued Prompts
Any conversation containing queued prompt turns must remain immediately discoverable.
- Conversations with pending FIFO queue items (`conv.is_queued === true` or child turn `p.is_queued === true` or `status` containing `'queue'`) must be sorted to Tier 2 (directly beneath active running sessions).
- When `activeFilter === 'queued'` or when viewing an active session, conversations with queued turns must automatically expand (`isExpanded = true`) to display turn chips (`T1`, `T2`) with amber clock badges (`QUEUED`).
- Queued filters must never discard parent conversations that contain queued child turns.

### Invariant 3: Taint-Free Native Image and Event Clipboard Copying
Clipboard copying of screenshots, attached media, and transcript events must never fail due to CORS or tainted canvases.
- Local filesystem image paths (e.g., `file:///...`, `C:\...`) are read via backend Tauri IPC (`read_file_as_base64`), returning direct Base64 data URIs.
- Binary PNG Blobs are constructed directly from decoded Base64 buffers via `base64ToBlob('image/png')` and inserted into `ClipboardItem` with dual/triple MIME types (`image/png`, `text/html`, `text/plain`).
- Document focus guards (`ensureDocumentFocus`) and fallback ladders guarantee that if rich binary insertion is rejected, Markdown text is copied to clipboard without silent failure.

### Invariant 4: Caller IDE Self-Preservation & Zero-Storage Hygiene
- Never terminate active caller instance `default-copy-5597`.
- All documentation, commit messages, and changelogs must strictly use relative Git paths.
- Release attribution must strictly and exclusively attribute `@aukgit` (`(Thanks to @aukgit)`).
