## Quick Install v4.65.3

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install (v4.65.3)
```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1))) -Version 4.65.3
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

#### Pinned Version Install (v4.65.3)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --version 4.65.3
```

---

-   **[Release v4.65.3: Mailbox Multi-Format Visual Export, Single-Account Import/Export & Embedded JSON AI Syntax] Interactive Export Preview Modal, Native OS File Saving, Single Account Form Drawer & Embedded JSON AI Instructions**:
    -   **Interactive Visual Export Modal & WebView2 Download Fix**: Introduced `MailboxExportModal.tsx` to eliminate WebView2 silent download failures where blob URL clicks were dropped without file or UI response. Clicking any export action immediately launches a syntax-highlighted dark modal with live switching between **JSON**, **YAML**, and **CSV**, instant clipboard copying, and native OS file saving.
    -   **Tauri Backend File Save Gate Expansion**: Updated `validate_user_json_path` in `src-tauri/src/commands/mod.rs` to permit saving `.json`, `.csv`, `.yaml`, `.yml`, `.txt`, `.xlsx`, and `.xls` files via Tauri's native `@tauri-apps/plugin-dialog` save handler.
    -   **Single-Account Individual Export**: Added dedicated `Export` buttons to each account row in the Email Accounts table and added an `Account IO` header toolbar in the Add/Edit Mailbox modal for one-click JSON, YAML, or CSV export of single configurations.
    -   **Single-Account Quick Import Drawer**: Added an expandable quick import drawer to the Add/Edit Mailbox dialog to paste or browse (`.json`, `.yaml`, `.csv`, `.txt`) configurations and automatically populate all form fields.
    -   **Embedded JSON Syntax in AI Configuration Segment**: Refactored the AI Mailbox Configuration banner to embed the exact JSON schema syntax directly inside the prompt text, accompanied by a `View JSON Format` toggle to preview the active JSON structure.
    -   **Global AI Templates & YAML Bulk Ingestion**: Embedded JSON schemas and added a dedicated YAML Schema tab in `ai-sample-templates-modal.tsx`, while introducing YAML radio selection and fallback parsing to the general Import dialog.
