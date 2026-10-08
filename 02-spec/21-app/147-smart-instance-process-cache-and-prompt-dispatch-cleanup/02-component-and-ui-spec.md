# Component & UI Specification: Task 147 - Smart Instance Process Cache & Prompt Dispatch Cleanup

## 1. Frontend Component Contracts

### 1.1 `src/components/instances/PromptTreeViewModal.tsx`
- **Compact Sequence Badge Formatter**:
  ```ts
  // Clean, non-intrusive badge formatter replacing verbose [AGM:P001 | GM:#1]
  function formatCompactBadge(code: string | undefined, fallback: string): string {
      if (!code) return fallback;
      const clean = code.replace(/^(AGM:|GM:)/i, '').trim();
      return clean.startsWith('#') || clean.startsWith('P') || clean.startsWith('C') ? clean : `#${clean}`;
  }
  ```
- **Duration Calculation Guard**:
  ```ts
  const formatDuration = (secs: number): string => {
      if (!Number.isFinite(secs) || isNaN(secs) || secs < 0) return '0s';
      const m = Math.floor(secs / 60);
      const s = Math.floor(secs % 60);
      if (m === 0) return `${s}s`;
      return `${m}m ${s < 10 ? '0' : ''}${s}s`;
  };
  ```
- **Timestamp Ingestion**:
  ```ts
  const parseTimestampSafe = (raw: string | number | undefined): number => {
      if (!raw) return Date.now();
      if (typeof raw === 'number') {
          return raw < 1e11 ? raw * 1000 : raw;
      }
      const str = String(raw).trim();
      if (/^\d+$/.test(str)) {
          const num = Number(str);
          return num < 1e11 ? num * 1000 : num;
      }
      const parsed = Date.parse(str);
      return Number.isFinite(parsed) ? parsed : Date.now();
  };
  ```
- **Direct Dispatch Call (`handleResendPrompt` / Send Now)**:
  - Invokes `invoke('send_prompt_now', { instanceId, repoPath, promptContent, conversationId })`.
  - Replaces previous indirect `resume_recent_project_prompts` call.
- **Enqueue Prompt Call (`handleEnqueuePrompt`)**:
  - Invokes `invoke('enqueue_prompt', { instanceId, repoPath, promptContent, conversationId })`.

### 1.2 `src/services/instanceService.ts`
- Export functions:
  ```ts
  export async function sendPromptNow(
      instanceId: string,
      repoPath: string,
      promptContent: string,
      conversationId?: string
  ): Promise<any>;

  export async function enqueuePrompt(
      instanceId: string,
      repoPath: string,
      promptContent: string,
      conversationId?: string
  ): Promise<any>;

  export async function getRunningInstancesProcessCount(): Promise<number>;
  ```

## 2. Backend Component Contracts

### 2.1 `src-tauri/src/modules/instance.rs`
- **Smart Instance Process Cache API**:
  ```rust
  pub fn get_instance_running_process_count() -> usize;
  pub fn is_instance_process_running_cached(instance_id: &str) -> bool;
  pub fn ensure_instance_running_smart(instance_id: &str, workspace_path: Option<&str>) -> crate::error::AppResult<bool>;
  pub fn invalidate_instance_process_cache(instance_id: &str);
  ```

### 2.2 `src-tauri/src/commands/instance.rs`
- **Tauri IPC Command Registrations**:
  ```rust
  #[tauri::command]
  pub async fn send_prompt_now(
      instance_id: String,
      repo_path: String,
      prompt_content: String,
      conversation_id: Option<String>,
  ) -> Result<crate::modules::repo_db::ActivePrompt, String>;

  #[tauri::command]
  pub async fn enqueue_prompt(
      instance_id: String,
      repo_path: String,
      prompt_content: String,
      conversation_id: Option<String>,
  ) -> Result<crate::modules::repo_db::ActivePrompt, String>;

  #[tauri::command]
  pub fn get_running_instances_process_count() -> Result<usize, String>;
  ```

### 2.3 `src-tauri/src/modules/repo_db.rs`
- **Direct Dispatch Implementations**:
  ```rust
  pub fn send_prompt_now_for_instance(
      instance_id: &str,
      repo_path: &str,
      prompt_content: &str,
      conversation_id: Option<&str>,
  ) -> Result<ActivePrompt, String>;

  pub fn enqueue_prompt_for_instance(
      instance_id: &str,
      repo_path: &str,
      prompt_content: &str,
      conversation_id: Option<&str>,
  ) -> Result<ActivePrompt, String>;
  ```
- **Ghost Running Status Filter**:
  - Tighten `is_conv_running`: verify if the conversation step status is actually running.
  - Automatically prune or mark `completed` on prompts in `active_prompts` where the `agy` worker process has exited or the prompt is older than 5 minutes.
