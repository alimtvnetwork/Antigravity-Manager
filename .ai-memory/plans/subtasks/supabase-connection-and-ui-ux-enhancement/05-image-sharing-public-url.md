---
plan: supabase-connection-and-ui-ux-enhancement
subtask: "05"
title: Standalone Prompt Image Sharing & Multi-Cluster Storage Rotation
domain: storage/sharing/ui
target_files:
  - src-tauri/src/modules/supabase_storage/mod.rs
  - src-tauri/src/modules/supabase_storage/uploader.rs
  - src-tauri/src/modules/supabase_storage/quota.rs
  - src-tauri/src/modules/supabase_pruner.rs
  - src-tauri/src/commands/prompt_share.rs
  - src/components/prompts/SharePromptModal.tsx
  - src/components/prompts/PromptCard.tsx
  - src/stores/usePromptStore.ts
status: pending
---

# 05 — Standalone Prompt Image Sharing & Multi-Cluster Storage Rotation

## 1. Objective

Design, implement, and verify the end-to-end **Standalone Prompt Image Sharing** capability in Antigravity-Manager. This feature enables developers to share agent prompts, visual evidence (DevTools CDP screenshots, generated architectural diagrams, multimodal inputs), and thinking snapshots via public standalone URLs while strictly enforcing a **400 MB multi-cluster storage quota boundary**:

1. **Multi-Cluster Storage Bucket Rotation**: Distribute public image assets across Primary (Lovable: `prompt-images`) and Secondary (Personal: `prompt-images-aux`) Supabase storage buckets, automatically rotating to the secondary cluster when the primary reaches the 380 MB watermark.
2. **Quota Tracking & Ephemeral Pruning**: Track consumed storage volume locally and via the Supabase Storage API, triggering automated LRU pruning when combined usage nears the 400 MB ceiling.
3. **Public & Signed URLs**: Support both high-performance public CDN URLs (with `Cache-Control: immutable`) for public prompt sharing and time-limited signed URLs for confidential sessions.
4. **Standalone Prompt Publishing**: Persist shared prompt metadata into `public.shared_prompts` and generate shareable links (`https://antigravity.live/p/{short_code}`) viewable in any modern browser.
5. **Modern Accessible UI/UX**: Provide an accessible `SharePromptModal` with image selection, privacy settings, one-click clipboard copying, and dark-glass pill styling.

---

## 2. Technical Architecture & System Design

### 2.1 Multi-Cluster Storage Bucket Topology

Supabase free-tier projects enforce a strict 500 MB storage cap. To ensure zero risk of service disruption or project pausing, Antigravity-Manager enforces a hard **400 MB operational ceiling** per project, with a proactive **380 MB rotation watermark**:

```mermaid
flowchart TD
    subgraph UI [React Frontend]
        BTN["User clicks 'Share Prompt'"] --> MODAL["SharePromptModal.tsx"]
        MODAL --> IPC_CALL["invoke('publish_shared_prompt', payload)"]
    end

    subgraph Backend [Rust Core: supabase_storage]
        IPC_CALL --> ROUTER["Storage Rotator & Quota Guard<br/>(supabase_storage::uploader)"]
        ROUTER --> QUOTA_CHECK{"Primary Usage &lt; 380 MB?"}
        
        QUOTA_CHECK -->|Yes (Normal)| P_UPLOAD["Upload to Primary (Lovable)<br/>Bucket: prompt-images<br/>Host: pezjuuddecbyfmqxytrv.supabase.co"]
        QUOTA_CHECK -->|No (Rollover Triggered)| S_UPLOAD["Upload to Secondary (Personal)<br/>Bucket: prompt-images-aux<br/>Host: ikwmurmjynhxdpmhzekt.supabase.co"]
        
        P_UPLOAD --> CDN_P["Generate Public CDN URL<br/>/storage/v1/object/public/prompt-images/{hash}.png"]
        S_UPLOAD --> CDN_S["Generate Public CDN URL<br/>/storage/v1/object/public/prompt-images-aux/{hash}.png"]
        
        CDN_P --> DB_INSERT["Insert Prompt Metadata into public.shared_prompts"]
        CDN_S --> DB_INSERT
    end

    subgraph Output [Standalone Prompt URL]
        DB_INSERT --> RESULT["Return Share Result: { short_code, share_url }"]
        RESULT --> TOAST["Render Copyable Link Pill in UI"]
    end
```

---

## 3. Detailed Component Specifications

### 3.1 Backend Storage Module (`src-tauri/src/modules/supabase_storage/`)

#### 1. Uploader (`uploader.rs`)
- **Content-Addressable Hashing**: Calculate SHA-256 checksum of raw image bytes. The object key is formatted as `{sha256[0..16]}_{timestamp}.{ext}`.
- **Deduplication Check**: Perform a lightweight `HEAD` request to `/storage/v1/object/{bucket}/{key}` before uploading. If the file already exists on the CDN, reuse the existing URL without re-uploading bytes.
- **MIME Type Validation**: Strictly permit standard image MIME types: `image/png`, `image/jpeg`, `image/webp`, `image/svg+xml`. Reject unrecognized binaries with `AppError::Config`.
- **Upload Request**:
  ```http
  POST /storage/v1/object/{bucket}/{filename}
  apikey: {endpoint.api_key}
  Authorization: Bearer {endpoint.api_key}
  Content-Type: {mime_type}
  x-upsert: true
  ```

#### 2. Quota Guard & Multi-Cluster Rotator (`quota.rs`)
- Maintain local cache of bucket utilization (`StorageQuotaCache`) updated every 10 minutes or after every 5 uploads.
- **Rotation Rules**:
  - If `primary_used_bytes < 380 * 1024 * 1024`: Route to Primary Cluster (`pezjuuddecbyfmqxytrv`).
  - If `primary_used_bytes >= 380 * 1024 * 1024` and `secondary_used_bytes < 380 * 1024 * 1024`: Route to Secondary Cluster (`ikwmurmjynhxdpmhzekt`).
  - If both exceed 380 MB: Trigger synchronous LRU prune of assets older than 14 days before attempting upload.

#### 3. Storage Pruner Extension (`src-tauri/src/modules/supabase_pruner.rs`)
- Add `prune_storage_buckets(endpoint: &SupabaseEndpoint, max_age_days: u32) -> Result<usize, AppError>`.
- Query `/storage/v1/object/list/{bucket}` sorted by `created_at ASC`.
- Cross-reference object keys against active prompts in `public.shared_prompts`. Delete unreferenced objects or expired prompts (`expires_at < NOW()`).

---

### 3.2 Database Schema: `public.shared_prompts`

Shared prompt records are stored in PostgreSQL on the primary cluster:

```sql
CREATE TABLE IF NOT EXISTS public.shared_prompts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    short_code VARCHAR(16) UNIQUE NOT NULL,
    title TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    origin_tier VARCHAR(32) NOT NULL DEFAULT 'USER_PROMPT',
    model_name VARCHAR(64),
    thinking_content TEXT,
    image_urls JSONB DEFAULT '[]'::jsonb,
    is_public BOOLEAN NOT NULL DEFAULT true,
    cluster_id VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days')
);

CREATE INDEX IF NOT EXISTS idx_shared_prompts_code ON public.shared_prompts(short_code);
CREATE INDEX IF NOT EXISTS idx_shared_prompts_expires ON public.shared_prompts(expires_at);
```

---

### 3.3 Tauri IPC Interface (`src-tauri/src/commands/prompt_share.rs`)

Define IPC request/response envelopes:

```rust
#[derive(Debug, Deserialize)]
pub struct SharePromptRequest {
    pub prompt_id: String,
    pub title: String,
    pub prompt_text: String,
    pub origin_tier: String,
    pub model_name: Option<String>,
    pub thinking_content: Option<String>,
    pub local_image_paths: Vec<String>,
    pub is_public: bool,
    pub ttl_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SharePromptResponse {
    pub is_success: bool,
    pub short_code: String,
    pub share_url: String,
    pub uploaded_images_count: usize,
    pub primary_cluster_used_mb: f64,
    pub secondary_cluster_used_mb: f64,
}

#[tauri::command]
pub async fn publish_shared_prompt(
    request: SharePromptRequest,
) -> Result<SharePromptResponse, AppError> {
    // Implementation coordinates uploader, quota guard, and PostgREST insert
}
```

---

### 3.4 Frontend User Experience (`src/components/prompts/SharePromptModal.tsx`)

1. **Trigger Context**:
   - Mounted in `PromptCard.tsx` and Prompt Tree view alongside existing actions (Resend, Inspect, Copy).
   - Styled as a dark-glass segmented pill button (`rounded-full`, Share icon).
2. **Modal Features**:
   - **Visual Asset Grid**: Displays thumbnails of screenshots, visual evidence cards, and diagrams associated with the prompt run. Checkboxes allow the user to select which assets to include.
   - **Thinking Snapshot Option**: Pill toggle to include/exclude agent chain-of-thought blocks.
   - **Privacy Selection**: Toggle between "Public Link (30 Days)" and "Restricted / Signed URL (24 Hours)".
   - **Result Banner**: When published, generates a shareable URL (`https://antigravity.live/p/{code}`) with a one-click "Copy Link" button and toast feedback.
3. **Accessibility & Keyboard Navigation**:
   - Implements `Escape` key dismissal.
   - Traps focus within modal while open.
   - Accessible ARIA labels on all interactive controls.

---

## 4. Implementation Roadmap

### Phase 1: Storage Uploader & Client (Rust Backend)
- [ ] Create `src-tauri/src/modules/supabase_storage/mod.rs` and `uploader.rs`.
- [ ] Implement SHA-256 content addressing and deduplication check.
- [ ] Support binary and base64 image uploading to Supabase Storage REST endpoint `/storage/v1/object/{bucket}/{filename}`.

### Phase 2: Multi-Bucket Quota Guard & Rotator (Rust Backend)
- [ ] Implement `src-tauri/src/modules/supabase_storage/quota.rs`.
- [ ] Query bucket usage and enforce the 380 MB rotation watermark between Lovable and Personal clusters.
- [ ] Integrate bucket auto-pruning in `src-tauri/src/modules/supabase_pruner.rs`.

### Phase 3: PostgREST Shared Prompt Persistence (Rust Backend)
- [ ] Author migration SQL for `public.shared_prompts`.
- [ ] Implement database insertion routine using `SupabaseClient::insert()`.
- [ ] Generate 8-character URL-safe random alphanumeric short codes (`nanoid` or `rand`).

### Phase 4: Tauri IPC Bridge & Command Registration
- [ ] Implement `src-tauri/src/commands/prompt_share.rs`.
- [ ] Register `publish_shared_prompt` in `src-tauri/src/lib.rs` / `main.rs`.
- [ ] Wire error propagation into `AppError` without swallowing network exceptions.

### Phase 5: React Frontend UI & Store Integration
- [ ] Create `src/components/prompts/SharePromptModal.tsx`.
- [ ] Mount Share action button in `src/components/prompts/PromptCard.tsx` and Prompt Tree.
- [ ] Add state management hooks in `src/stores/usePromptStore.ts`.
- [ ] Handle offline/unconfigured Supabase fallback with graceful warning toast.

### Phase 6: End-to-End Verification & Quality Gates
- [ ] Unit test: Verify SHA-256 deduplication and quota calculation.
- [ ] Integration test: Test image upload against live Supabase storage bucket.
- [ ] Frontend build: Verify TypeScript type checks and component styling.
- [ ] Linting gate: Run `cargo fmt -- --check` and `cargo clippy`.

---

## 5. Verification & Testing Matrix

| Test ID | Target Component | Scenario | Expected Outcome |
| :--- | :--- | :--- | :--- |
| **TEST-01** | `uploader::hash_image` | Hash duplicate image bytes. | Generates identical key; skips upload via HEAD request. |
| **TEST-02** | `quota::check_rotation` | Simulate Primary bucket at 385 MB. | Seamlessly routes upload to Secondary bucket (`prompt-images-aux`). |
| **TEST-03** | `pruner::prune_storage` | Run storage prune against expired prompts. | Deletes orphan objects from storage and frees quota. |
| **TEST-04** | `cmd::publish_shared_prompt`| Publish prompt with 2 PNG screenshots. | Returns valid `share_url` and records in `public.shared_prompts`. |
| **TEST-05** | `SharePromptModal.tsx` | Open modal, select images, click Copy Link. | Copies link to clipboard; displays toast; closes on Escape. |
| **TEST-06** | Error Boundary | Attempt share when network disconnected. | Displays non-blocking warning toast; records failure via `trackWarning`. |
