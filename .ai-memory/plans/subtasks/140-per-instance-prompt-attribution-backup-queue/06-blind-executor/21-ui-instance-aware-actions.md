# Step 21: UI Actions Name Their Instance

Goal: the prompt tree modal sends and enqueues through the shared per-instance commands from step 20 instead of writing `.antigravity_resume_task.json` itself, never falls back to `'default'` for those actions, and shows a small instance badge on every conversation row. The Instances page loads the tree per instance and merges it, so rows of different instances are never mixed up. Subtask `../03-cli-and-ipc-parity.md` Step 6.

Read `00-start-here.md` first. This file never overrides it.

## 1. Depends on

- Step 20 (Tauri commands `send_prompt_now`, `enqueue_prompt`, `list_backed_up_prompts(instance_id)`).

## 2. Files you may edit

- `src/components/instances/PromptTreeViewModal.tsx`
- `src/pages/Instances.tsx`

No Rust file. No locale file: this step adds no translation keys. `PromptTreeViewModal.tsx` does not use `useTranslation` and all its messages are plain English, so the new messages match that file. The `Instances.tsx` change adds no visible text.

## 3. Find it

| Change | File | Place | Unique search literal | Line hint |
|---|---|---|---|---|
| A | modal | after `export interface AgmProjectTreeNode {` block, before `interface PromptTreeViewModalProps {` | `interface PromptTreeViewModalProps {` | `:81` |
| B | modal | `const handleBackup = async () => {` | `invoke<any[]>('list_backed_up_prompts')` | `:1104` |
| C | modal | inside `const handleResendPrompt = useCallback(async () => {` | `// 1. Write .antigravity_resume_task.json to project directory` | `:1233` to `:1280` |
| D | modal | `const handleEnqueuePrompt = async () => {` (whole function) | `// Handle Enqueue Prompt Action (Calls FIFO scheduler queue / writes queue task)` | `:1324` to `:1377` |
| E | modal | inside `const renderConversationNode = (conv: AgmConversationNode, project: AgmProjectTreeNode) => {` | `{conv.step_count \|\| 1} stp` | `:1577` to `:1586` |
| F | `Instances.tsx` | `const fetchRunningTasks = async () => {` (whole function) | `const fetchRunningTasks = async () => {` | `:279` to `:296` |

GitMap searches (use `--ext .tsx`):

```text
gitmap aum search "interface PromptTreeViewModalProps" src/components/instances/PromptTreeViewModal.tsx --ext .tsx
gitmap aum search "list_backed_up_prompts" src/components/instances/PromptTreeViewModal.tsx --ext .tsx
gitmap aum search "Write .antigravity_resume_task.json to project directory" src/components/instances/PromptTreeViewModal.tsx --ext .tsx
gitmap aum search "Handle Enqueue Prompt Action" src/components/instances/PromptTreeViewModal.tsx --ext .tsx
gitmap aum search "stp" src/components/instances/PromptTreeViewModal.tsx --ext .tsx
gitmap aum search "const fetchRunningTasks" src/pages/Instances.tsx --ext .tsx
gitmap aum search "interface InstancePromptRow" src --ext .tsx
```

The last search must return 0 hits. If `InstancePromptRow` already exists, STOP.

## 4. Current code

### Change A (insertion point only)

```tsx
    is_running: boolean;
    conversations: AgmConversationNode[];
}

interface PromptTreeViewModalProps {
```

### Change B (`:1104`)

```tsx
            const backedUp = await invoke<any[]>('list_backed_up_prompts');
```

### Change C (`:1233` to `:1280`, inside `handleResendPrompt`; leave the fallbacks above and the success message, `catch`, `finally` and dependency array below unchanged)

```tsx
            const repoPath = proj?.repo_path || '';

            // 1. Write .antigravity_resume_task.json to project directory
            if (repoPath) {
                const taskPath = `${repoPath.replace(/[\\/]+$/, '')}/.antigravity_resume_task.json`;
                const payload = {
                    prompt_id: conv?.conversation_id || `prompt-${Date.now()}`,
                    project_id: proj?.project_id || '',
                    instance_id: proj?.instance_id || instanceId || 'default',
                    repo_path: repoPath,
                    prompt_content: promptContent,
                    model: 'gemini-2.5-pro',
                    auto_boot: true,
                    status: 'dispatched',
                    resumed_at: Math.floor(Date.now() / 1000),
                };
                try {
                    await invoke('save_text_file', {
                        path: taskPath,
                        content: JSON.stringify(payload, null, 2),
                    });
                } catch (fsErr) {
                    console.warn('save_text_file resume error', fsErr);
                }
            }

            // 2. Trigger auto resume command
            const targetInstId = proj?.instance_id || instanceId || 'default';
            try {
                await invoke('resume_recent_project_prompts', {
                    instanceId: targetInstId,
                    maxAgeSeconds: 3600,
                });
            } catch {
                // non-fatal fallback
            }

            // 3. Copy prompt content to clipboard so user can paste immediately
            try {
                await navigator.clipboard.writeText(promptContent);
            } catch {}

            // 4. Focus or launch IDE instance window so user immediately sees it ("goes there")
            try {
                await focusOrLaunchInstance(targetInstId);
            } catch (focusErr) {
                console.warn('focusOrLaunchInstance error', focusErr);
            }
```

The lines directly after this block must be:

```tsx

            setActionMsg("Prompt Dispatched & Focused IDE (via Hotkey 'N' / Send Now)!");
            setTimeout(() => setActionMsg(null), 3500);
```

### Change D (`:1324` to `:1377`, whole function)

```tsx
    // Handle Enqueue Prompt Action (Calls FIFO scheduler queue / writes queue task)
    const handleEnqueuePrompt = async () => {
        if (!selectedConversation) return;
        try {
            setIsEnqueueing(true);
            setActionMsg('Enqueueing prompt into FIFO scheduler queue...');
            const promptContent = editedPromptText.trim() || activePromptText || selectedConversation.prompt_preview_200w || '';
            const repoPath = selectedProject?.repo_path || '';

            let enqueued = false;
            try {
                await invoke('enqueue_prompt', {
                    conversationId: selectedConversation.conversation_id,
                    projectId: selectedProject?.project_id,
                    instanceId: selectedProject?.instance_id || instanceId || 'default',
                    repoPath,
                    promptContent,
                });
                enqueued = true;
            } catch {
                // Fallback: write .antigravity_resume_task.json with queued status
                if (repoPath) {
                    const taskPath = `${repoPath.replace(/[\\/]+$/, '')}/.antigravity_resume_task.json`;
                    const payload = {
                        prompt_id: selectedConversation.conversation_id || `prompt-${Date.now()}`,
                        project_id: selectedProject?.project_id || '',
                        instance_id: selectedProject?.instance_id || instanceId || 'default',
                        repo_path: repoPath,
                        prompt_content: promptContent,
                        model: 'gemini-2.5-pro',
                        auto_boot: false,
                        status: 'queued',
                        queued_at: Math.floor(Date.now() / 1000),
                    };
                    try {
                        await invoke('save_text_file', {
                            path: taskPath,
                            content: JSON.stringify(payload, null, 2),
                        });
                        enqueued = true;
                    } catch (fsErr) {
                        console.warn('save_text_file queue error', fsErr);
                    }
                }
            }

            setActionMsg(enqueued ? 'Prompt enqueued into FIFO scheduler queue!' : 'Prompt recorded for queue scheduler!');
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to enqueue prompt');
        } finally {
            setIsEnqueueing(false);
        }
    };
```

### Change E (`:1577` to `:1586`, inside `renderConversationNode`)

```tsx
                <div className="flex items-center gap-1.5 shrink-0">
                    <span
                        className={cn(
                            'text-[9px] font-mono px-1 rounded-[3px]',
                            isConvSelected
                                ? 'bg-blue-700/80 text-white'
                                : 'bg-slate-200 dark:bg-[#15334d] text-slate-500 dark:text-slate-400'
                        )}
                    >
                        {conv.step_count || 1} stp
                    </span>
```

### Change F (`Instances.tsx:279` to `:296`)

```tsx
    const fetchRunningTasks = async () => {
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                maxWords: 50,
                onlyRunning: false,
                force: false,
            });
            if (Array.isArray(data)) {
                setProjectTreeNodes(data);
                const running = data.filter(
                    (node) => Boolean(node.is_running)
                );
                setRunningTreeNodes(running);
            }
        } catch {
            // Silently ignore background polling error
        }
    };
```

## 5. New code

### Change A: insert between the closing `}` of `AgmProjectTreeNode` and `interface PromptTreeViewModalProps {`

```tsx
    is_running: boolean;
    conversations: AgmConversationNode[];
}

interface InstancePromptRow {
    id: string;
    instance_id: string;
    instance_name: string;
    repo_path: string;
    session_id: string | null;
    status: string;
    status_reason: string | null;
    attempts: number;
    source_dir: string | null;
}

interface PromptTreeViewModalProps {
```

### Change B: becomes

```tsx
            const backedUp = await invoke<any[]>('list_backed_up_prompts', { instanceId: instanceId || null });
```

### Change C: replace the block from section 4 with

`targetInstId` is still used by the focus call at the end of the block. The dependency array of `handleResendPrompt` already contains `instanceId`; do not change it.

```tsx
            const repoPath = proj?.repo_path || '';
            const targetInstId = proj?.instance_id || instanceId;
            if (!targetInstId) {
                setActionMsg(null);
                setError('Instance unknown; open the tree from an instance row');
                return;
            }
            if (!repoPath) {
                setActionMsg(null);
                setError('Project folder unknown; select a project first');
                return;
            }

            // 1. Send through the shared per-instance path (row first, then agy)
            const sentRow = await invoke<InstancePromptRow>('send_prompt_now', {
                instanceId: targetInstId,
                repoPath,
                promptContent,
                conversationId: conv?.conversation_id ?? null,
            });
            if (sentRow.status === 'failed') {
                setActionMsg(null);
                setError(`Send failed for ${sentRow.instance_name}: ${sentRow.status_reason || 'unknown reason'}`);
                return;
            }

            // 2. Copy prompt content to clipboard so user can paste immediately
            try {
                await navigator.clipboard.writeText(promptContent);
            } catch {}

            // 3. Focus or launch IDE instance window so user immediately sees it ("goes there")
            try {
                await focusOrLaunchInstance(targetInstId);
            } catch (focusErr) {
                console.warn('focusOrLaunchInstance error', focusErr);
            }
```

### Change D: the whole function becomes

```tsx
    // Handle Enqueue Prompt Action (Calls FIFO scheduler queue for the selected instance)
    const handleEnqueuePrompt = async () => {
        if (!selectedConversation) return;
        const targetInstId = selectedProject?.instance_id || instanceId;
        if (!targetInstId) {
            setError('Instance unknown; open the tree from an instance row');
            return;
        }
        try {
            setIsEnqueueing(true);
            setActionMsg('Enqueueing prompt into FIFO scheduler queue...');
            const promptContent = editedPromptText.trim() || activePromptText || selectedConversation.prompt_preview_200w || '';
            const repoPath = selectedProject?.repo_path || '';

            const queuedRow = await invoke<InstancePromptRow>('enqueue_prompt', {
                instanceId: targetInstId,
                repoPath,
                promptContent,
                conversationId: selectedConversation.conversation_id,
            });

            setActionMsg(`Prompt enqueued for ${queuedRow.instance_name} (FIFO scheduler queue)!`);
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: any) {
            setActionMsg(null);
            setError(err?.toString() || 'Failed to enqueue prompt');
        } finally {
            setIsEnqueueing(false);
        }
    };
```

### Change E: insert the badge directly above the step-count `<span`

The badge reuses the pill size of the step counter and the purple instance colors already used in this file. `#seq` falls back to the project's sequence number when the conversation node has none.

```tsx
                <div className="flex items-center gap-1.5 shrink-0">
                    {conv.instance_id && (
                        <span
                            className={cn(
                                'text-[9px] font-mono px-1 rounded-[3px] border',
                                isConvSelected
                                    ? 'bg-blue-700/80 text-white border-blue-400/40'
                                    : 'bg-purple-500/10 text-purple-700 dark:text-purple-300 border-purple-500/20'
                            )}
                            title={`Instance ${conv.instance_id}`}
                        >
                            #{conv.instance_seq_num ?? project.instance_seq_num ?? '-'} {conv.instance_name || project.instance_name || conv.instance_id}
                        </span>
                    )}
                    <span
                        className={cn(
                            'text-[9px] font-mono px-1 rounded-[3px]',
                            isConvSelected
                                ? 'bg-blue-700/80 text-white'
                                : 'bg-slate-200 dark:bg-[#15334d] text-slate-500 dark:text-slate-400'
                        )}
                    >
                        {conv.step_count || 1} stp
                    </span>
```

### Change F (`Instances.tsx`): the whole function becomes

The function runs from a `setInterval` created once (`useEffect` with `[]`), so it must read the instance list from the store at call time, not from the render closure.

```tsx
    const fetchRunningTasks = async () => {
        try {
            const instanceIds = useInstanceStore.getState().instances.map((inst) => inst.config.id);
            if (instanceIds.length === 0) {
                return;
            }
            const perInstance = await Promise.all(
                instanceIds.map((id) =>
                    invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                        instanceId: id,
                        maxWords: 50,
                        onlyRunning: false,
                        force: false,
                    }).catch(() => [] as AgmProjectTreeNode[])
                )
            );
            const data = perInstance.flatMap((nodes, idx) =>
                (Array.isArray(nodes) ? nodes : []).map((node) => ({
                    ...node,
                    instance_id: node.instance_id || instanceIds[idx],
                }))
            );
            setProjectTreeNodes(data);
            const running = data.filter(
                (node) => Boolean(node.is_running)
            );
            setRunningTreeNodes(running);
        } catch {
            // Silently ignore background polling error
        }
    };
```

`useInstanceStore` is already imported in `Instances.tsx` (line 42). `AgmProjectTreeNode` is already imported there (it is used by the current code).

### Left unchanged on purpose (do not edit)

- `handleRestore` (`:1121`) still calls `resume_recent_project_prompts` with `instanceId || 'default'`.
- The `'default'` fallbacks at `:683`, `:1382` (`handleFocusIde`) and `:2810`.

They are not send or enqueue actions and are outside this step's scope.

## 6. Tests

There is no frontend unit test runner for these components. Verification is the TypeScript build (strict mode, `noUnusedLocals`, `noUnusedParameters` in `tsconfig.json`) plus the Rust gate as a regression check. The live UI check (E2E-09) is a human case in step 22.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 ipc_enqueue_matches_cli_enqueue_row_shape; cd ..
npm run build
```

`npm run build` must exit 0 with no TypeScript error. If it reports an unused variable in a line you changed, remove only that variable.

## 8. Commit

```text
Fix: prompts - UI send and enqueue name their instance
```

## 9. Done when

- [ ] `gitmap aum search "save_text_file" src/components/instances/PromptTreeViewModal.tsx --ext .tsx` shows no hit inside `handleResendPrompt` or `handleEnqueuePrompt`.
- [ ] `handleResendPrompt` calls `invoke<InstancePromptRow>('send_prompt_now', ...)` and no longer calls `resume_recent_project_prompts`.
- [ ] Neither `handleResendPrompt` nor `handleEnqueuePrompt` contains `|| 'default'`.
- [ ] Every conversation row shows a `#<seq> <name>` badge when `conv.instance_id` is set.
- [ ] `fetchRunningTasks` calls `get_project_conversation_tree` once per instance with `instanceId`.
- [ ] Callers searched in `src` (`.ts` and `.tsx`) for every IPC command this step invokes (`send_prompt_now`, `enqueue_prompt`, `list_running_projects`, `list_backed_up_prompts`, `get_project_conversation_tree`), and in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. This step changes no Rust signature, so the Rust side needs no caller edit. The zero-argument `invoke('list_running_projects')` and `invoke('list_backed_up_prompts')` in `src/services/instanceService.ts:443` and `:447` stay valid, because the command's `instance_id` is an `Option<String>` from step 20.
- [ ] `npm run build` exits 0; the Rust gate exits 0.
- [ ] Only the two files from section 2 are staged.
