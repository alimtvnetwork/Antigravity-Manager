---
plan: auto-switch-and-ui-functionality-issues
subtask: "03"
title: Audit Log JSON/JSONL Direct Clipboard Copy and Call Stack Trace Enrichment
domain: frontend-ui-and-audit
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/auto-switch-and-ui-functionality-issues/02-audit-log-windows-and-ui-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/audit/AuditPanel.tsx
  - src/components/audit/audit-client.ts
  - src/components/audit/audit-types.ts
  - src/hooks/useAuditRecorder.ts
  - src-tauri/src/modules/audit/types.rs
  - src-tauri/src/modules/audit/commands.rs
  - src-tauri/src/modules/audit/recorder.rs
status: pending
---

# Subtask 03: Audit Log JSON/JSONL Direct Clipboard Copy and Call Stack Trace Enrichment

## 1. Context & Motivation

Users reported critical usability flaws in the Interaction Audit Log:
1. **Broken Export Buttons**: Clicking `JSON` or `JSONL` fails to copy or download anything. The legacy implementation synthesizes an `<a download>` link and invokes `.click()`, which is silently dropped in Tauri's WebView2 sandbox on Windows. Users need direct system clipboard copying (`copyToClipboard`) with visual `Copied!` tooltip/button states and confirmation toasts.
2. **Empty Method & Target Cells**: The audit table renders blank `METHOD` and `TARGET` cells across all entries. Investigation revealed that Rust's `InteractionEntry` struct serializes as `snake_case` (`method_name`, `ui_target`), whereas the frontend `AuditEntry` interface expects `camelCase` (`methodName`, `uiTarget`).
3. **Broken Tauri IPC Invocation**: `audit-client.ts` passes `{ entry: payload }`, but the backend command `record_ui_interaction` expects `{ input: RecordInteractionInput }`.
4. **Lack of DOM Climbing & Stack Traces**: Clicking SVG icons inside buttons attributes interactions to raw `<path>` or `<span>` nodes instead of the semantic button/anchor, and lacks JavaScript caller function names and stack trace line numbers.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/audit/types.rs` | `InteractionEntry`, `RecordInteractionInput` | Add `#[serde(rename_all = "camelCase")]` to both structs. |
| `src/components/audit/audit-client.ts` | `auditRecord` | Fix invoke argument key from `{ entry: payload }` to `{ input: payload }`. |
| `src/hooks/useAuditRecorder.ts` | `useAuditRecorder`, `describeElement`, `extractStackTrace` | Add DOM climbing to `findInteractiveTarget` (`closest('button, a, ...')`) and capture call stack trace via `new Error().stack`. |
| `src/components/audit/AuditPanel.tsx` | `AuditPanel`, `handleCopyJson`, `handleCopyJsonl`, `EntryRow` | Replace `downloadFile` with `copyToClipboard`, introduce 2s `copiedFormat` visual state with check icon, and render formatted call stack in expanded rows. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Apply `camelCase` Serde to Rust Audit Types
In `src-tauri/src/modules/audit/types.rs`:
1. Add `#[serde(rename_all = "camelCase")]` to `InteractionEntry`:
   ```rust
   #[derive(Serialize, Deserialize, Clone, Debug)]
   #[serde(rename_all = "camelCase")]
   pub struct InteractionEntry {
       pub id: i64,
       pub timestamp: i64,
       pub actor: String,
       pub method_name: String,
       pub ui_target: String,
       pub trigger: String,
       pub params: String,
       pub result: String,
       pub message: String,
   }
   ```
2. Add `#[serde(rename_all = "camelCase")]` to `RecordInteractionInput`:
   ```rust
   #[derive(Deserialize, Debug)]
   #[serde(rename_all = "camelCase")]
   pub struct RecordInteractionInput {
       pub actor: InteractionActor,
       pub method_name: String,
       pub route: String,
       pub component: Option<String>,
       pub element_id: Option<String>,
       pub trigger: InteractionTrigger,
       pub params: Option<serde_json::Value>,
       pub result: String,
       pub message: Option<String>,
   }
   ```

### 3.2 Step 2: Correct Tauri IPC Contract in `src/components/audit/audit-client.ts`
1. In `auditRecord(payload: AuditRecordPayload)`, replace `{ entry: payload }` with `{ input: payload }`:
   ```ts
   export async function auditRecord(payload: AuditRecordPayload): Promise<boolean> {
       try {
           await invoke(AUDIT_COMMANDS.record, { input: payload });
           return true;
       } catch (error) {
           warn(error, AUDIT_COMMANDS.record, { trigger: payload.trigger, methodName: payload.methodName });
           return false;
       }
   }
   ```

### 3.3 Step 3: Implement DOM Climbing & Call Stack Capture in `src/hooks/useAuditRecorder.ts`
1. Implement interactive target resolution:
   ```ts
   function findInteractiveTarget(el: Element | null): HTMLElement | null {
       if (!el) return null;
       const interactive = el.closest('button, a, [role="button"], input, select, textarea');
       return (interactive as HTMLElement) || (el as HTMLElement);
   }
   ```
2. Update `describeElement` to use `findInteractiveTarget(el)` so that clicks on inner SVG paths or spans resolve to their parent `<button>` or `<a>`.
3. Add stack trace generator:
   ```ts
   function extractStackTrace(): { callerInfo: string; stackTrace: string } {
       try {
           const stack = new Error().stack || '';
           const lines = stack.split('\n').map((l) => l.trim()).filter(Boolean);
           const filtered = lines.filter(
               (l) =>
                   !l.includes('useAuditRecorder') &&
                   !l.includes('extractStackTrace') &&
                   !l.includes('Error') &&
                   !l.includes('report')
           );
           const callerInfo = filtered[0] || 'anonymous';
           const stackTrace = filtered.slice(0, 8).join('\n');
           return { callerInfo, stackTrace };
       } catch {
           return { callerInfo: 'unknown', stackTrace: '' };
       }
   }
   ```
4. Attach `caller` and `stack` into interaction payload in `handleClick` and `handleInputCommit`.

### 3.4 Step 4: Direct Clipboard Copy & UI Polish in `src/components/audit/AuditPanel.tsx`
1. Import `copyToClipboard` from `../../utils/clipboard` and `showToast` from `../common/ToastContainer`.
2. Replace `handleExportJson` and `handleExportJsonl` with asynchronous clipboard write handlers:
   ```tsx
   const [copiedFormat, setCopiedFormat] = useState<'json' | 'jsonl' | null>(null);

   const handleCopyJson = async () => {
       try {
           const payload = JSON.stringify(filtered, null, 2);
           const success = await copyToClipboard(payload);
           if (success) {
               setCopiedFormat('json');
               showToast(t('audit.copied_json', 'Audit log (JSON) copied to clipboard!'), 'success');
               window.setTimeout(() => setCopiedFormat(null), 2000);
           } else {
               showToast(t('audit.copy_failed', 'Failed to copy audit log to clipboard'), 'error');
           }
       } catch (err) {
           useErrorStore.getState().captureError(err, {
               source: 'AuditPanel.handleCopyJson',
               triggerAction: 'copyToClipboard',
           });
       }
   };
   ```
3. Update buttons in toolbar to render `Copied!` with check icon when active.
4. In `EntryRow`, render a dedicated Call Stack inspection section when expanded if `params.stack` is present.

---

## 4. Verification & Quality Gates

- [ ] Run `npm run build` and ensure zero TypeScript compilation errors.
- [ ] Open Audit Log modal; verify `METHOD` and `TARGET` cells contain genuine populated strings.
- [ ] Click `JSON` button; verify button displays `Copied!`, toast appears, and clipboard contains valid JSON array.
- [ ] Click `JSONL` button; verify clipboard contains newline-delimited JSON.
- [ ] Click an interactive element and expand its audit row; verify call stack trace and caller function are visible.
