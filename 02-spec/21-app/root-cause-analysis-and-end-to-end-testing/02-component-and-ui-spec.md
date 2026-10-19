# Component and UI Spec: Root Cause Analysis and End-to-End Testing

## 1. Prompt Tree Component Hierarchy

```text
PromptTreeView/
├── TreePanel.tsx
│   ├── SearchBar
│   ├── ConversationListNode
│   │   └── TurnSubtreeNode (T1..TN)
│   │       ├── TurnBadge (RUNNING / QUEUED / COMPLETED)
│   │       ├── TurnTimestamp (e.g. 5:41 PM, 16h 18m)
│   │       └── TurnTitlePreview
├── DetailHeader.tsx
│   ├── ConversationStatusBadge
│   ├── TimestampRangeSummary (Start -> Current / End)
│   └── PromptActionCapsule (Send Now, Enqueue, Rerun, Copy All)
└── PromptInstructionView.tsx
    ├── MediaGallery (Attached Images)
    │   ├── GalleryActionBar (Copy Pure PNG, Batch Copy N Images)
    │   └── PromptThumbnailCard
    │       ├── ImagePreview (Webview2 Asset URL / Base64 fallback)
    │       ├── SingleClickCopyOverlay [ 📋 ]
    │       └── PathTooltip & Metadata
    └── RichMarkdownRenderer (Prompt Text Viewport)
```

---

## 2. UI Component Specifications

### 2.1 TurnSubtreeNode & TreePanel
- **Auto-Scroll Behavior**: When a conversation is loaded or refreshed, the viewport must auto-scroll to the currently active (`RUNNING`) or most recently enqueued (`QUEUED`) turn.
- **Turn Badges**:
  - `RUNNING`: Emerald pulsing badge (`bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 animate-pulse`).
  - `QUEUED`: Amber clock badge (`bg-amber-500/20 text-amber-400 border border-amber-500/40`).
  - `COMPLETED`: Muted slate badge (`bg-slate-700/20 text-slate-400 border border-slate-700/30`).
- **Timestamp Formatting**:
  - Absolute Start Time: `h:mm A` (e.g. `5:41 PM`).
  - Relative Duration: `Xm Ys` or `Xh Ym`.

### 2.2 PromptThumbnailCard & Attached Media Gallery
- **Preloading & Base64 Fallback**:
  - Proactively queries `read_image_as_base64` on mount.
  - While loading, displays a subtle shimmer placeholder.
  - If loading succeeds, renders the image using high-definition pixel rendering.
  - If loading fails, displays error banner with the exact filesystem path and an interactive retry button.
- **Copy Actions**:
  - Single-click hover copy overlay `[ 📋 ]` copies pure PNG directly to system clipboard via `writePureImageToClipboard`.
  - Dedicated gallery button `[ 📋 Copy Image (PNG) ]` copies the first image.
  - Multi-image button `[ 📋 Copy Images (N) ]` copies all images sequentially for OneNote and Word pasting.

### 2.3 Elimination of Redundant UI Artifacts
- **Double Expand Button Removal**: Ensure `PromptInstructionView.tsx` does not render a duplicate `Expand full text` button beneath `RichMarkdownRenderer`.
