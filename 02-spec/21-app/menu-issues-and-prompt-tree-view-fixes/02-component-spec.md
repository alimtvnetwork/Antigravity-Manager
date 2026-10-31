# Component Specification: Enqueued Items Visibility & Clipboard Media/Event Copying

## 1. Problem Classification & Root Cause Analysis

### 1.1 Enqueued Items Hidden in Tree ("I do not see the enqueue items")
- **Symptom**: User states "I do not see the enqueue items". In screenshot `media_1792342419562_c0815c8c.png`, the conversation header displays `FIFO: 1 pending`, but in the sidebar tree:
  1. The conversation row appears collapsed because `isExpanded` initializes to `Boolean(conv.is_running)`. If the conversation is idle with queued turns, `isExpanded` remains false.
  2. In `src/components/instances/PromptTreeView/ProjectNode.tsx`, filtering for `'queued'` executes:
     ```tsx
     if (activeFilter === 'queued' && !conv.is_queued && !conv.status.toLowerCase().includes('queue')) {
         return;
     }
     ```
     If the root conversation record `is_queued` is false, but individual child turns (`conv.prompts`) are queued, the entire conversation is omitted from the view.
  3. In sorting (`sortConversations`), conversations with queued turns do not float above idle conversations because child prompts are not checked for queue status.

### 1.2 Image and Event Copy Failures ("image or event does not get copied")
- **Symptom**: Right-clicking or clicking copy on images in `PromptInstructionView.tsx` ("Attached Images (5)") fails to write binary image data to the clipboard, falling back to plaintext or failing silently.
- **Root Cause**:
  `convertImageUriToPngBlob` in `src/components/instances/PromptTreeView/clipboard.ts` attempts to load local `convertFileSrc` asset URLs into an offscreen HTML `Image` element with `img.crossOrigin = 'anonymous'`. In Tauri WebView2, local asset protocol URLs (`http://asset.localhost/...` or `asset://localhost/...`) lack CORS response headers. Setting `crossOrigin` causes WebView2 to abort the load with a network CORS error or taints the HTML canvas, throwing `SecurityError` upon `canvas.toBlob(...)`.
  Additionally, when copying event or transcript steps, UI lacked immediate toast feedback and fallbacks.

---

## 2. Technical Remediation

### 2.1 Enqueued Items Auto-Expansion & Sorting
1. **Dynamic Queue Detection**:
   A conversation is defined as queued if:
   ```typescript
   export function isConversationOrTurnQueued(conv: AgmConversationNode): boolean {
       if (Boolean(conv.is_queued)) return true;
       if (conv.status && conv.status.toLowerCase().includes('queue')) return true;
       if (conv.prompts && conv.prompts.some((p) => Boolean(p.is_queued) || (p.status && p.status.toLowerCase().includes('queue')))) {
           return true;
       }
       return false;
   }
   ```
2. **Auto-Expand Queued Sessions**:
   In `ConversationNode.tsx`:
   - Initialize `isExpanded` to `Boolean(conv.is_running) || isConversationOrTurnQueued(conv) || activeFilter === 'queued'`.
   - Re-expand when `isConversationOrTurnQueued(conv)` becomes true during polling.
3. **Filter & Sort Inclusion**:
   - In `ProjectNode.tsx`, allow conversations where `isConversationOrTurnQueued(conv)` is true to pass `activeFilter === 'queued'`.
   - In `sortConversations` and `tierClassification.ts`, sort queued conversations into Tier 2 (above idle/completed sessions).

### 2.2 Taint-Free Native Image & Event Copying
1. **Backend Base64 Reader**:
   In `src-tauri/src/commands/` or `src-tauri/src/modules/`, expose:
   ```rust
   #[tauri::command]
   pub async fn read_file_as_base64(path: String) -> Result<String, AppError> { ... }
   ```
   Reads file binary directly from disk, verifies image signature, and returns `data:image/png;base64,...`.
2. **Frontend Dual-MIME Clipboard Pipeline**:
   In `clipboard.ts`:
   - If URI is a local path (starts with `file://`, drive letter `C:\`, `/`, or `http://asset.localhost/`), bypass offscreen HTML `Image` loading.
   - Invoke `read_file_as_base64(cleanPath)` to obtain the base64 data URI.
   - Decode directly using `base64ToBlob(b64, 'image/png')`.
   - Write to `navigator.clipboard.write([new ClipboardItem({ 'image/png': blob, 'text/html': htmlBlob, 'text/plain': mdText })])`.
   - Provide immediate visual toast feedback (`showToast({ message: 'Image copied to clipboard as native PNG!', type: 'success' })`).
