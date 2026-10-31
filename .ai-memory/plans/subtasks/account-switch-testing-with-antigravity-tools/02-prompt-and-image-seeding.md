# Subtask 02: Prompt and Image Seeding

## Status: COMPLETED
- **Completed At**: 2026-10-20T02:07:20+08:00
- **Worker**: Worker 02 (`account-switch-testing-with-antigravity-tools`)
- **Verification Result**: 100% Passed (Pre-switch backup, SQLite schema verification, multimodal payload preservation)

---

## Objective
Register and seed active running prompts containing deep analytical instructions and multimodal Base64 image payloads into the isolated test instance environment. Verify pre-switch persistence across `repo_prompts.db`, `backup-prompts.db`, and workspace resumption files.

---

## Pre-requisites & Sandbox Isolation
- Transient test sandbox provisioned via `agm instances test-create test-switch-verify --json`.
- Dedicated test instance identifier: `test-switch-verify`.
- Isolated test workspace root: relative path `tests/sandboxes/test-switch-verify`.
- Caller IDE self-preservation active: caller instance PID (`default-copy-5597`) protected against mutation.
- Verified binary: `src-tauri/target/debug/agm.exe`.

---

## Execution Steps & Results

### 1. Multimodal Test Image Payload Preparation
1. **Base64 Payload**: Valid 1x1 test PNG data URI synthesized:
   ```text
   data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==
   ```
2. **Local File Fixture**: Verified local image fixture exists at relative path `tests/fixtures/test_diagram.png` (70 bytes, valid PNG signature `\x89PNG\r\n\x1a\n`).
3. **Screenshot ID Token**: Upstream screenshot tokens (`screenshot_id=SS_ID_987654321` and `screenshot_id=SS_ID_112233445`) appended to validate regex extraction and composite payload tagging.
4. **Rich Analytical Prompt Text**:
   - Prompt 1 (`p-switch-001`):
     ```text
     Deep analysis: inspect neural layer latency graph and layout fidelity with attached diagram: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_987654321
     ```
   - Prompt 2 (`p-switch-002`):
     ```text
     Secondary verification: audit restored token telemetry and visual fidelity: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_112233445
     ```

---

### 2. Schema Verification (`repo_prompts.db` & `backup-prompts.db`)
Inspected database table columns via SQLite PRAGMA table_info:
- `repo_prompts.db: active_prompts`:
  - `id` (TEXT, PK)
  - `project_id` (TEXT)
  - `instance_id` (TEXT)
  - `repo_path` (TEXT)
  - `prompt_content` (TEXT)
  - `model` (TEXT)
  - `session_id` (TEXT)
  - `status` (TEXT)
  - `created_at` (INTEGER)
  - `updated_at` (INTEGER)
  - `image_payload` (TEXT) -> Confirmed column present.
- `backup-prompts.db: prompt_backups`:
  - `has_images` (BOOLEAN) -> Confirmed column present (`1`).
  - `images_payload` (TEXT) -> Confirmed column present.

---

### 3. Active Prompt Seeding into `repo_prompts.db`
Seeded project and active prompt rows into `repo_prompts.db`:

#### A. Seed `running_projects`
```sql
INSERT INTO running_projects (
    id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
) VALUES (
    'test-switch-verify__test-switch-verify',
    'test-switch-verify',
    'test-switch-verify',
    'tests/sandboxes/test-switch-verify',
    'tests/sandboxes/test-switch-verify/.workspaceStorage',
    1,
    strftime('%s', 'now'),
    strftime('%s', 'now')
);
```

#### B. Seed `active_prompts`
Seeded two prompts to test multi-prompt handling and sequence preservation:
```sql
INSERT INTO active_prompts (
    id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
) VALUES (
    'p-switch-001',
    'test-switch-verify__test-switch-verify',
    'test-switch-verify',
    'tests/sandboxes/test-switch-verify',
    'Deep analysis: inspect neural layer latency graph and layout fidelity with attached diagram: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_987654321',
    'gemini-2.5-pro',
    'conv-switch-test-8899',
    'running',
    1792433022,
    1792433090,
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_987654321'
);

INSERT INTO active_prompts (
    id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
) VALUES (
    'p-switch-002',
    'test-switch-verify__test-switch-verify',
    'test-switch-verify',
    'tests/sandboxes/test-switch-verify',
    'Secondary verification: audit restored token telemetry and visual fidelity: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_112233445',
    'gemini-2.5-pro',
    'conv-switch-test-8899',
    'running',
    1792433052,
    1792433090,
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_112233445'
);
```

---

### 4. Resumption Manifest Mirroring in Workspace Root
Written atomic resumption manifests to `tests/sandboxes/test-switch-verify`:
1. `tests/sandboxes/test-switch-verify/.antigravity_resume_task.json` (canonical pointer):
```json
{
  "prompt_id": "p-switch-001",
  "project_id": "test-switch-verify__test-switch-verify",
  "workspace_id": "test-switch-verify__test-switch-verify",
  "sequence_code": "[AGM:P001 | GM:#1]",
  "seq_code": "[AGM:P001 | GM:#1]",
  "instance_id": "test-switch-verify",
  "repo_path": "tests/sandboxes/test-switch-verify",
  "prompt_content": "Deep analysis: inspect neural layer latency graph and layout fidelity with attached diagram: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_987654321",
  "model": "gemini-2.5-pro",
  "session_id": "conv-switch-test-8899",
  "conversation_id": "conv-switch-test-8899",
  "image_payload": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==;screenshot_id=SS_ID_987654321",
  "image_paths": [
    "tests/fixtures/test_diagram.png"
  ],
  "has_image": true,
  "auto_boot": true,
  "status": "running",
  "backed_up_at": 1792433090
}
```
2. `tests/sandboxes/test-switch-verify/.antigravity_resume_task.test-switch-verify.json` (instance-scoped fallback).

---

### 5. Trigger Pre-Switch Snapshot via CLI
Executed pre-switch prompt capture command:
```powershell
.\src-tauri\target\debug\agm.exe backup-prompts -i test-switch-verify --json
```

**CLI Execution Output**:
```json
{
  "success": true,
  "status": "success",
  "command": "prompts backup",
  "instance_id": "test-switch-verify",
  "timestamp": 1792433090,
  "data": {
    "status": "backed_up",
    "instance_id": "test-switch-verify"
  },
  "meta": {
    "command": "prompts backup",
    "instance_id": "test-switch-verify",
    "timestamp": "2026-10-19T18:04:50.711217800+00:00",
    "version": "4.238.0"
  }
}
```

**Tracing Telemetry**:
```text
02:04:50  INFO [RepoDB] Backed up 0 running prompts for instance 'test-switch-verify' across Antigravity core and workspaces
02:04:50  INFO [PROMPT_BACKUP] Backed up prompt p-switch-001 for instance 'test-switch-verify' (has_images: true, payload_bytes: 148)
02:04:50  INFO [PROMPT_BACKUP] Backed up prompt p-switch-002 for instance 'test-switch-verify' (has_images: true, payload_bytes: 148)
```

---

### 6. Verification & Integrity Assertions
Verified that pre-switch persistence is 100% intact before proceeding to account switch:

1. **Database Records in `prompt_backups`**:
   ```sql
   SELECT prompt_id, sequence_id, has_images, length(images_payload), status, is_restored, instance_id 
   FROM prompt_backups 
   WHERE instance_id = 'test-switch-verify'
   ORDER BY created_at ASC, sequence_id ASC;
   ```
   - Record 1: `p-switch-001 | seq=1 | has_images=1 | payload_len=148 | status=backed_up | is_restored=0 | instance_id=test-switch-verify`
   - Record 2: `p-switch-002 | seq=2 | has_images=1 | payload_len=148 | status=backed_up | is_restored=0 | instance_id=test-switch-verify`
   - **Assertions Passed**:
     - `has_images == 1`
     - `length(images_payload) == 148` (complete un-truncated Base64 payload preserved)
     - `is_restored == 0`
     - `instance_id == 'test-switch-verify'`
     - `sequence_id == 1` and `2`

2. **Transition in `repo_prompts.db:active_prompts`**:
   - Both rows transitioned to `status = 'backed_up'`.
   - Payload length remained strictly 148 bytes without truncation.

3. **Workspace Manifests**:
   - `tests/sandboxes/test-switch-verify/.antigravity_resume_task.json` confirmed on disk.
   - `tests/sandboxes/test-switch-verify/.antigravity_resume_task.test-switch-verify.json` confirmed on disk.
   - Contains `has_image: true` and identical 148-byte Base64 data URI string.
