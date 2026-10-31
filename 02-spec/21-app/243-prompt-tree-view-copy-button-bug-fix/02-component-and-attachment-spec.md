# Component & Attachment Specification: Prompt Tree View Copy & Popovers

**Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Sequence**: `243`  
**Spec Path**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/02-component-and-attachment-spec.md`  
**Status**: APPROVED_FOR_EXECUTION  
**Target Branch**: `main`  
**Attribution**: strictly `@aukgit`  

---

## 1. Attachment Generator Refactor (`src/utils/promptAttachment.ts`)

### 1.1 Helper Function: `formatAttachmentLabel`
Generates human-readable 3-tier display strings:
```typescript
export function formatAttachmentLabel(path: string, index: number, customDesc?: string): string {
    const seq = String(index + 1).padStart(2, '0');
    const normalized = path.replace(/\\/g, '/');
    const filename = normalized.split('/').pop() || `image_${seq}.png`;
    
    let desc = customDesc?.trim();
    if (!desc) {
        const base = filename.replace(/\.[a-zA-Z0-9]+$/, '');
        if (base.startsWith('media_')) {
            desc = 'Screenshot Attachment';
        } else {
            desc = base.replace(/[-_]+/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()) + ' Screenshot';
        }
    }
    return `[Attachment ${seq} | ${desc} | ${filename}]`;
}
```

### 1.2 `buildAttachmentsBlock` Update
```typescript
function buildAttachmentsBlock(paths: string[]): string {
    const lines = paths.map((p, idx) => {
        const label = formatAttachmentLabel(p, idx);
        return `${idx + 1}. ${label}(file;${p})`;
    });
    return `Attachments:\n${lines.join('\n')}`;
}
```

---

## 2. Copy Flow Refactor (`src/components/instances/PromptTreeView/DetailHeader.tsx`)

### 2.1 Media Resolution Invariant
When copying, resolve all available turn-level and conversation-level media:
```typescript
const resolveActiveMediaPaths = (): string[] => {
    const turnMedia = selectedTurnNumber != null
        ? selectedConversation.prompts?.find((p) => p.turn_number === selectedTurnNumber)?.media_paths
        : undefined;
    const convMedia = selectedConversation.media_paths || [];
    return Array.from(new Set([...(turnMedia || []), ...convMedia])).filter(Boolean);
};
```

### 2.2 Primary Copy Button Refactor (`handleCopyAll`)
```typescript
const handleCopyAll = async () => {
    try {
        const source = resolvePromptBody(selectedConversation, activePromptText);
        if (!source) {
            setActionMsg?.('No prompt content to copy');
            setTimeout(() => setActionMsg?.(null), 2000);
            return;
        }
        const media = resolveActiveMediaPaths();
        const effectivePrompt = media.length > 0
            ? formatPromptWithAttachments(source, media, { repoPath: rerunRepoPath })
            : source;
        const ok = await copyToClipboard(effectivePrompt);
        if (ok) {
            markCopiedAll();
            return;
        }
        reportPromptActionError(new Error('Failed to copy prompt to clipboard'), 'DetailHeader.copyAll');
    } catch (err) {
        reportPromptActionError(err, 'DetailHeader.copyAll');
    }
};
```

---

## 3. Popover Click-Outside Implementation (`DetailHeader.tsx`)

Mount three distinct container refs using `useClickOutside`:
```typescript
const copyMenuContainerRef = useRef<HTMLDivElement>(null);
const dispatchMenuContainerRef = useRef<HTMLDivElement>(null);
const breakdownMenuContainerRef = useRef<HTMLDivElement>(null);

useClickOutside(copyMenuContainerRef, () => {
    if (isCopyMenuOpen) setIsCopyMenuOpen(false);
}, isCopyMenuOpen);

useClickOutside(dispatchMenuContainerRef, () => {
    if (isMenuOpen) handleCloseDispatchMenu();
}, isMenuOpen);

useClickOutside(breakdownMenuContainerRef, () => {
    if (isBreakdownDropdownOpen) setIsBreakdownDropdownOpen(false);
}, isBreakdownDropdownOpen);
```
