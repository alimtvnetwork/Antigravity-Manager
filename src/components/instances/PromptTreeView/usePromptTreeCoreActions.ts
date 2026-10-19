import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    setArchivedProjectsForInstance,
} from '../../../services/instanceService';
import { useErrorStore } from '../../../stores/error-store';
import { isTauri } from '../../../utils/env';
import { copyPromptWithRichImages, copyPromptCleanText } from './clipboard';
import type { AgmProjectTreeNode } from './types';
import type { PromptTreeState } from './usePromptTreeState';
import type { ExportScopeParams } from './PromptTreeExportModal';

export interface PromptTreeCoreActions {
    togglePinProject: (projectId: string, e?: React.MouseEvent) => void;
    toggleArchiveProject: (projectId: string, e?: React.MouseEvent) => void;
    isProjectPinned: (projectId: string) => boolean;
    handleRefreshSingleProject: (projectId: string, e: React.MouseEvent) => Promise<void>;
    handleBackup: () => Promise<void>;
    handleFileRestore: (e: React.ChangeEvent<HTMLInputElement>) => void;
    handleRestore: () => Promise<void>;
    handleCopyAllTreePrompts: () => Promise<void>;
    handleCopyPromptWithImages: (markdownText: string, mediaPaths?: string[]) => Promise<boolean>;
    handleCopyPromptAsText: (markdownText: string) => Promise<boolean>;
    handleExportSqlite: () => Promise<void>;
    handleImportSqlite: () => Promise<void>;
    handleExportRunningJson: () => Promise<void>;
    handleExportQueuedJson: () => Promise<void>;
    handleExportRunningAndQueuedJson: () => Promise<void>;
    handleResendQueued: () => Promise<void>;
    isResendingQueued: boolean;

    // Scoped Export / Import Modal Handlers
    isExportModalOpen: boolean;
    setIsExportModalOpen: (open: boolean) => void;
    exportScopeTarget?: ExportScopeParams;
    setExportScopeTarget: (scope?: ExportScopeParams) => void;
    exportInitialFormat: 'json' | 'sqlite' | 'yaml';
    setExportInitialFormat: (format: 'json' | 'sqlite' | 'yaml') => void;
    handleOpenExportModal: (scope?: ExportScopeParams, initialFormat?: 'json' | 'sqlite' | 'yaml') => void;
    handleCloseExportModal: () => void;
    executeSectionExport: (params: {
        scope: ExportScopeParams;
        format: 'json' | 'sqlite' | 'yaml';
        embed_images: boolean;
        export_mode?: 'standalone' | 'lean';
    }) => Promise<void>;

    isImportModalOpen: boolean;
    setIsImportModalOpen: (open: boolean) => void;
    handleOpenImportModal: () => void;
    handleCloseImportModal: () => void;
    executeSectionImport: (params: {
        filePath?: string;
        dataB64?: string;
        conflictMode: 'merge' | 'replace';
        materializeImages: boolean;
        targetInstanceId?: string;
    }) => Promise<number>;
}

interface ResendQueuedOutcome {
    instance_id: string;
    backup_batch_id?: string | null;
    backed_up_queued: number;
    reinjected: number;
    dispatched: number;
    dispatched_from_backup: number;
    skipped_duplicates: number;
    failed: number;
    failures: string[];
}

export type PromptTreeStatusExport = 'running' | 'queued' | 'running_and_queued';

interface LoadedPromptTreeBinding {
    tree: AgmProjectTreeNode[];
    notify: (message: string | null) => void;
}

const loadedPromptTree: LoadedPromptTreeBinding = {
    tree: [],
    notify: () => {},
};

export function bindLoadedPromptTree(
    tree: AgmProjectTreeNode[],
    notify: (message: string | null) => void
): void {
    loadedPromptTree.tree = tree;
    loadedPromptTree.notify = notify;
}

function conversationMatchesStatus(
    conv: AgmProjectTreeNode['conversations'][number],
    kind: PromptTreeStatusExport
): boolean {
    const status = (conv.status || '').toLowerCase();
    const running = Boolean(conv.is_running) || status === 'running' || status.includes('running');
    if (kind === 'running') return running;
    const queued = (Boolean(conv.is_queued) || status === 'queued' || status.includes('queue')) && !running;
    if (kind === 'queued') return queued;
    if (kind === 'running_and_queued') return running || queued;
    return false;
}

function collectStatusPrompts(
    projects: AgmProjectTreeNode[],
    kind: PromptTreeStatusExport
): AgmProjectTreeNode[] {
    return projects
        .map((project) => {
            const conversations: AgmProjectTreeNode['conversations'] = [];
            const seen = new Set<string>();
            const walk = (nodes: AgmProjectTreeNode['conversations'] | undefined) => {
                for (const conv of nodes || []) {
                    if (conversationMatchesStatus(conv, kind) && !seen.has(conv.conversation_id)) {
                        seen.add(conv.conversation_id);
                        conversations.push({ ...conv, sub_runs: undefined });
                    }
                    walk(conv.sub_runs);
                }
            };
            walk(project.conversations);
            return { ...project, conversations };
        })
        .filter((project) => project.conversations.length > 0);
}

function downloadJsonBlob(fileName: string, json: string): void {
    const blob = new Blob([json], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

/**
 * Download JSON for running or queued prompts from the tree already loaded
 * in the prompt view, or fetching the current tree on demand. Uses the existing
 * backup file writer in the desktop app and a browser download otherwise.
 */
export async function downloadPromptTreeStatusJson(
    kind: PromptTreeStatusExport,
    customTree?: AgmProjectTreeNode[],
    notify?: (msg: string | null) => void
): Promise<void> {
    const notifier = notify || loadedPromptTree.notify;
    let tree = customTree || loadedPromptTree.tree;
    if (!tree || tree.length === 0) {
        try {
            tree = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                maxWords: 2000,
                onlyRunning: false,
                force: false,
            });
        } catch {
            tree = [];
        }
    }
    const projects = collectStatusPrompts(tree, kind);
    const promptCount = projects.reduce((sum, project) => sum + project.conversations.length, 0);
    const label = kind === 'running_and_queued' ? 'running & queued' : kind;
    const payload = {
        version: '1.0',
        filter: kind,
        exportedAt: new Date().toISOString(),
        totalProjects: projects.length,
        totalConversations: promptCount,
        projects,
    };
    const jsonStr = JSON.stringify(payload, null, 2);
    const fileSlug = kind === 'running_and_queued' ? 'running-and-queued' : kind;
    const fileName = `prompts-${fileSlug}-${Date.now()}.json`;
    try {
        notifier(`Exporting ${label} prompts...`);
        if (!isTauri()) {
            downloadJsonBlob(fileName, jsonStr);
            notifier(`Exported ${promptCount} ${label} prompt(s) as JSON.`);
            window.setTimeout(() => notifier(null), 3000);
            return;
        }
        const { save } = await import('@tauri-apps/plugin-dialog');
        const selected = await save({
            defaultPath: fileName,
            filters: [{ name: 'JSON', extensions: ['json'] }],
        });
        if (!selected) {
            notifier(null);
            return;
        }
        await invoke('write_prompt_backup_file', { path: selected, content: jsonStr });
        notifier(`Exported ${promptCount} ${label} prompt(s) to ${selected}`);
        window.setTimeout(() => notifier(null), 4000);
    } catch (err: unknown) {
        useErrorStore.getState().captureError(err, {
            source: 'PromptTreeViewModal.downloadPromptTreeStatusJson',
            triggerAction: `export_${kind}_prompts_json`,
        });
        notifier(null);
    }
}

/**
 * Restores prompts from raw JSON text in strict FIFO order, with an interactive
 * 3-second countdown banner before dispatching restored prompts to the target instance.
 */
export async function restorePromptBackupFromJsonString(
    content: string,
    targetInstanceId?: string,
    notify?: (msg: string | null) => void
): Promise<{ restoredCount: number; dispatchedCount: number }> {
    JSON.parse(content);
    notify?.('Importing prompts backup...');
    const restoredCount = await invoke<number>('restore_prompts_backup', { backupJson: content });
    const targetInst = targetInstanceId || 'default';
    for (let remaining = 3; remaining > 0; remaining--) {
        notify?.(`⏳ Waiting time: ${remaining}s remaining · Next action: Resume and dispatch restored prompt(s) to instance ${targetInst}`);
        await new Promise((r) => setTimeout(r, 1000));
    }
    let dispatched = 0;
    try {
        dispatched = await invoke<number>('dispatch_backed_up_prompts', {
            instanceId: targetInst,
        });
    } catch (err: unknown) {
        useErrorStore.getState().captureError(err, {
            source: 'restorePromptBackupFromJsonString',
            triggerAction: 'dispatch_backed_up_prompts',
        });
    }
    notify?.(`Restored ${restoredCount} prompt(s) and dispatched ${dispatched} to instance ${targetInst}.`);
    window.setTimeout(() => notify?.(null), 5000);
    return { restoredCount, dispatchedCount: dispatched };
}

/**
 * Opens the native file picker to select a .json prompts backup and restores it.
 */
export async function openAndRestorePromptBackupFile(
    targetInstanceId?: string,
    notify?: (msg: string | null) => void
): Promise<{ restoredCount: number; dispatchedCount: number } | null> {
    if (!isTauri()) {
        return new Promise((resolve) => {
            const input = document.createElement('input');
            input.type = 'file';
            input.accept = '.json,application/json';
            input.onchange = async () => {
                const file = input.files?.[0];
                if (!file) {
                    resolve(null);
                    return;
                }
                const text = await file.text();
                const res = await restorePromptBackupFromJsonString(text, targetInstanceId, notify);
                resolve(res);
            };
            input.click();
        });
    }
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON Backup', extensions: ['json'] }],
    });
    if (!selected || typeof selected !== 'string') return null;
    const content = await invoke<string>('read_prompt_backup_file', { path: selected });
    return await restorePromptBackupFromJsonString(content, targetInstanceId, notify);
}

/**
 * Re-sends all running and queued prompts for an instance or all instances in FIFO order.
 */
export async function resendPromptsForInstance(
    instanceId?: string | null,
    notify?: (msg: string | null) => void
): Promise<number> {
    notify?.('Re-sending prompts & queued messages in FIFO order...');
    try {
        const count = await invoke<number>('resend_all_prompts', {
            instanceId: instanceId || undefined,
            limit: 20,
        });
        notify?.(`Successfully resent ${count} prompt(s) in FIFO order.`);
        window.setTimeout(() => notify?.(null), 4000);
        return count;
    } catch (err: unknown) {
        useErrorStore.getState().captureError(err, {
            source: 'resendPromptsForInstance',
            triggerAction: 'resend_all_prompts',
        });
        notify?.(null);
        throw err;
    }
}

interface TreeExportStats {
    path: string;
    project_count: number;
    prompt_count: number;
    sequence_count: number;
}

interface TreeExportPayload {
    data_b64: string;
    file_name: string;
    stats: TreeExportStats;
}

interface UsePromptTreeCoreActionsProps {
    state: PromptTreeState;
    instanceId: string;
    instanceName: string;
    loadTree: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>;
}

export function usePromptTreeCoreActions({
    state,
    instanceId,
    instanceName,
    loadTree,
}: UsePromptTreeCoreActionsProps): PromptTreeCoreActions {
    const {
        treeData,
        setTreeData,
        setError,
        setActionMsg,
        pinnedProjectIds,
        setPinnedProjectIds,
        archivedProjectIds,
        setArchivedProjectIds,
        setRefreshingProjectId,
        selectedProject,
        setSelectedProject,
        restoreFileInputRef,
    } = state;

    useEffect(() => {
        bindLoadedPromptTree(treeData, (message) => {
            setActionMsg(message);
        });
    }, [treeData, setActionMsg]);

    const isProjectPinned = useCallback(
        (projectId: string): boolean => pinnedProjectIds.includes(projectId),
        [pinnedProjectIds]
    );

    const togglePinProject = useCallback(
        (projectId: string, e?: React.MouseEvent) => {
            e?.stopPropagation?.();
            setPinnedProjectIds((prev) => {
                const next = prev.includes(projectId) ? prev.filter((id) => id !== projectId) : [...prev, projectId];
                try {
                    localStorage.setItem(`agm_pinned_projects_${instanceId || 'default'}`, JSON.stringify(next));
                } catch (err: unknown) {
                    // Best-effort pin persist; localStorage may throw in restricted contexts. Tracked, in-memory pins still applied.
                    useErrorStore.getState().trackWarning(err, {
                        source: 'PromptTreeViewModal.togglePinProject',
                        triggerAction: 'persist_pinned_projects',
                    });
                }
                return next;
            });
        },
        [instanceId, setPinnedProjectIds]
    );

    const toggleArchiveProject = useCallback(
        (projectId: string, e?: React.MouseEvent) => {
            e?.stopPropagation?.();
            const isCurrentlyArchived = archivedProjectIds.includes(projectId);
            let nextArchived: string[];
            if (isCurrentlyArchived) {
                nextArchived = archivedProjectIds.filter((id) => id !== projectId);
            } else {
                nextArchived = [...archivedProjectIds, projectId];
                // Automatically unpin the project when archived
                if (pinnedProjectIds.includes(projectId)) {
                    const nextPinned = pinnedProjectIds.filter((id) => id !== projectId);
                    setPinnedProjectIds(nextPinned);
                    try {
                        localStorage.setItem(`agm_pinned_projects_${instanceId || 'default'}`, JSON.stringify(nextPinned));
                    } catch (err: unknown) {
                        // Best-effort pin persist on archive; localStorage may throw in restricted contexts. Tracked, in-memory state still applied.
                        useErrorStore.getState().trackWarning(err, {
                            source: 'PromptTreeViewModal.toggleArchiveProject',
                            triggerAction: 'persist_pinned_projects',
                        });
                    }
                }
            }
            setArchivedProjectIds(nextArchived);
            setArchivedProjectsForInstance(instanceId, nextArchived);
        },
        [archivedProjectIds, pinnedProjectIds, instanceId, setPinnedProjectIds, setArchivedProjectIds]
    );

    const handleRefreshSingleProject = useCallback(
        async (projectId: string, e: React.MouseEvent) => {
            e.stopPropagation();
            setRefreshingProjectId(projectId);
            try {
                const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                    instanceId: instanceId || undefined,
                    maxWords: 2000,
                    onlyRunning: false,
                    force: true,
                });
                const isTargetDefault = instanceId === 'default' || instanceId === '__default__';
                const relevant = instanceId
                    ? data
                          .filter((p) => {
                              if (p.instance_id === instanceId) return true;
                              if (isTargetDefault && (!p.instance_id || p.instance_id === 'default' || p.instance_id === '__default__')) {
                                  return true;
                              }
                              return false;
                          })
                          .map((p) => ({
                              ...p,
                              conversations: (p.conversations || []).filter((c) => {
                                  if (c.instance_id === instanceId) return true;
                                  if (isTargetDefault && (!c.instance_id || c.instance_id === 'default' || c.instance_id === '__default__')) {
                                      return true;
                                  }
                                  return false;
                              }),
                          }))
                    : data;
                const finalData = relevant;
                setTreeData(finalData);

                const updatedProject = finalData.find((p) => p.project_id === projectId);
                if (updatedProject && selectedProject?.project_id === projectId) {
                    setSelectedProject(updatedProject);
                }
                setActionMsg(`Project ${updatedProject?.repo_name || ''} refreshed!`);
                setTimeout(() => setActionMsg(null), 2500);
            } catch (err: unknown) {
                setError(err instanceof Error ? err.message : String(err) || 'Failed to refresh project');
            } finally {
                setRefreshingProjectId(null);
            }
        },
        [instanceId, selectedProject, setTreeData, setSelectedProject, setActionMsg, setError, setRefreshingProjectId]
    );

    const handleBackup = useCallback(async () => {
        try {
            setActionMsg('Backing up prompts...');
            let backedUpDb: Record<string, unknown>[] = [];
            try {
                backedUpDb = await invoke<Record<string, unknown>[]>('list_backed_up_prompts');
            } catch (err: unknown) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'PromptTreeViewModal.handleBackup',
                    triggerAction: 'list_backed_up_prompts',
                });
            }

            const fullBackupPayload = {
                version: '1.0',
                exportedAt: new Date().toISOString(),
                instanceId: instanceId || 'all',
                instanceName: instanceName || 'All Instances',
                projects: treeData,
                activePrompts: backedUpDb,
                totalProjects: treeData.length,
                totalConversations: treeData.reduce((sum, p) => sum + p.conversations.length, 0),
            };

            const jsonStr = JSON.stringify(fullBackupPayload, null, 2);
            const fileName = `prompts-backup-${instanceId || 'all'}-${Date.now()}.json`;

            if (!isTauri()) {
                const blob = new Blob([jsonStr], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = fileName;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                setActionMsg(`Backed up ${fullBackupPayload.totalProjects} project(s).`);
                setTimeout(() => setActionMsg(null), 3000);
                return;
            }

            const { save } = await import('@tauri-apps/plugin-dialog');
            const selected = await save({
                defaultPath: fileName,
                filters: [{ name: 'JSON Backup', extensions: ['json'] }],
            });
            if (!selected) {
                setActionMsg(null);
                return;
            }
            await invoke('write_prompt_backup_file', { path: selected, content: jsonStr });
            setActionMsg(`Backed up ${fullBackupPayload.totalProjects} project(s) to ${selected}`);
            setTimeout(() => setActionMsg(null), 4000);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeViewModal.handleBackup',
                triggerAction: 'write_prompt_backup_file',
            });
            setError(err instanceof Error ? err.message : String(err) || 'Failed to backup prompts');
            setActionMsg(null);
        }
    }, [instanceId, instanceName, treeData, setActionMsg, setError]);

    const persistRestoredBackup = useCallback(
        async (content: string) => {
            JSON.parse(content);
            setActionMsg('Importing prompts backup...');
            const restoredCount = await invoke<number>('restore_prompts_backup', { backupJson: content });
            
            const targetInst = instanceId || 'default';
            for (let remaining = 3; remaining > 0; remaining--) {
                setActionMsg(`⏳ Waiting time: ${remaining}s remaining · Next action: Resume and dispatch restored prompt(s) to instance ${targetInst}`);
                await new Promise((r) => setTimeout(r, 1000));
            }

            let dispatched = 0;
            let dispatchError = '';
            try {
                dispatched = await invoke<number>('dispatch_backed_up_prompts', {
                    instanceId: targetInst,
                });
            } catch (err: unknown) {
                dispatchError = err instanceof Error ? err.message : String(err);
                useErrorStore.getState().captureError(err, {
                    source: 'PromptTreeViewModal.handleRestore',
                    triggerAction: 'dispatch_backed_up_prompts',
                });
            }
            const dispatchNote = dispatchError
                ? ` Import saved ${restoredCount} prompt(s), but sending them failed: ${dispatchError}`
                : ` Restored ${restoredCount} prompt(s) and sent ${dispatched} to instance ${targetInst}.`;
            setActionMsg(dispatchNote.trim());
            await loadTree(true, true);
            setTimeout(() => setActionMsg(null), 5000);
        },
        [instanceId, loadTree, setActionMsg]
    );

    const persistRestoredSqliteBytes = useCallback(
        async (dataB64: string) => {
            setActionMsg('Importing SQLite tree export...');
            const restored = await invoke<number>('import_prompt_tree_sqlite_bytes', { dataB64 });
            setActionMsg(`Imported ${restored} prompt(s) from SQLite export.`);
            await loadTree(true, true);
            setTimeout(() => setActionMsg(null), 5000);
        },
        [loadTree, setActionMsg]
    );

    const handleFileRestore = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            const file = e.target.files?.[0];
            if (!file) return;

            // SQLite tree exports arrive here in web (non-Tauri) mode: read the
            // raw bytes, base64-encode them, and run the validated byte import.
            if (/\.(db|sqlite)$/i.test(file.name)) {
                const reader = new FileReader();
                reader.onload = async (event) => {
                    try {
                        const buf = event.target?.result;
                        if (!(buf instanceof ArrayBuffer)) {
                            throw new Error('database file could not be read');
                        }
                        const bytes = new Uint8Array(buf);
                        let binary = '';
                        const chunk = 0x8000;
                        for (let i = 0; i < bytes.length; i += chunk) {
                            binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + chunk)));
                        }
                        await persistRestoredSqliteBytes(btoa(binary));
                    } catch (err: unknown) {
                        const msg = err instanceof Error ? err.message : String(err);
                        useErrorStore.getState().captureError(err, {
                            source: 'PromptTreeViewModal.handleFileRestore',
                            triggerAction: 'import_prompt_tree_sqlite_bytes',
                        });
                        setError(`SQLite import failed: ${msg || 'Invalid database file'}`);
                        setActionMsg(null);
                    }
                };
                reader.onerror = () => {
                    const readErr = reader.error ?? new Error('failed to read database file');
                    useErrorStore.getState().captureError(readErr, {
                        source: 'PromptTreeViewModal.handleFileRestore',
                        triggerAction: 'read_sqlite_file',
                    });
                    setError(`SQLite import failed: ${readErr.message}`);
                    setActionMsg(null);
                };
                reader.readAsArrayBuffer(file);
                e.target.value = '';
                return;
            }

            const reader = new FileReader();
            reader.onload = async (event) => {
                try {
                    const content = event.target?.result;
                    if (typeof content !== 'string') {
                        throw new Error('backup file was empty');
                    }
                    await persistRestoredBackup(content);
                } catch (err: unknown) {
                    const msg = err instanceof Error ? err.message : String(err);
                    useErrorStore.getState().captureError(err, {
                        source: 'PromptTreeViewModal.handleFileRestore',
                        triggerAction: 'restore_prompts_backup',
                    });
                    setError(`Restore failed: ${msg || 'Invalid backup format'}`);
                    setActionMsg(null);
                }
            };
            reader.onerror = () => {
                const readErr = reader.error ?? new Error('failed to read backup file');
                useErrorStore.getState().captureError(readErr, {
                    source: 'PromptTreeViewModal.handleFileRestore',
                    triggerAction: 'read_backup_file',
                });
                setError(`Restore failed: ${readErr.message}`);
                setActionMsg(null);
            };
            reader.readAsText(file);
            e.target.value = '';
        },
        [persistRestoredBackup, persistRestoredSqliteBytes, setActionMsg, setError]
    );

    const handleRestore = useCallback(async () => {
        try {
            if (!isTauri()) {
                restoreFileInputRef.current?.click();
                return;
            }
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({
                multiple: false,
                filters: [{ name: 'JSON Backup', extensions: ['json'] }],
            });
            if (!selected || typeof selected !== 'string') {
                setActionMsg(null);
                return;
            }
            setActionMsg('Reading backup...');
            const content = await invoke<string>('read_prompt_backup_file', { path: selected });
            await persistRestoredBackup(content);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeViewModal.handleRestore',
                triggerAction: 'read_prompt_backup_file',
            });
            setError(err instanceof Error ? err.message : String(err) || 'Failed to restore prompts');
            setActionMsg(null);
        }
    }, [restoreFileInputRef, persistRestoredBackup, setActionMsg, setError]);

    /**
     * Copy ALL prompts across every project in the tree in one shot.
     * Reuses the existing rich-copy path (`copyPromptWithRichImages`) with
     * per-project / per-conversation markdown separators.
     */
    const handleCopyAllTreePrompts = useCallback(async () => {
        try {
            const sections: string[] = [];
            let convCount = 0;
            let projectCount = 0;
            for (const proj of treeData) {
                const convs = (proj.conversations || []).filter((c) =>
                    (c.full_prompt_text || c.prompt_preview_200w || '').trim().length > 0
                );
                if (convs.length === 0) continue;
                projectCount++;
                sections.push(`\n\n---\n\n# ${proj.repo_name || proj.project_id}\n`);
                for (const conv of convs) {
                    const text = (conv.full_prompt_text || conv.prompt_preview_200w || '').trim();
                    const heading = conv.title || conv.short_id || conv.conversation_id;
                    const seq = conv.seq_code ? ` (${conv.seq_code})` : '';
                    sections.push(`## ${heading}${seq}\n\n${text}`);
                    convCount++;
                }
            }
            if (convCount === 0) {
                setActionMsg('No prompts in the tree to copy.');
                setTimeout(() => setActionMsg(null), 2500);
                return;
            }
            const combined = sections.join('\n\n').trim();
            const ok = await copyPromptWithRichImages(
                combined,
                (msg) => {
                    setActionMsg(`Copied ${convCount} prompt(s) from ${projectCount} project(s) — ${msg}`);
                    setTimeout(() => setActionMsg(null), 3000);
                },
                (err) => {
                    useErrorStore.getState().captureException(
                        err instanceof Error ? err : new Error(String(err)),
                        {
                            source: 'PromptTreeViewModal.handleCopyAllTreePrompts',
                            triggerAction: 'copyPromptWithRichImages',
                        }
                    );
                    setError('Failed to copy tree prompts to clipboard');
                }
            );
            if (!ok) {
                setError('Failed to copy tree prompts to clipboard');
            }
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeViewModal.handleCopyAllTreePrompts',
                triggerAction: 'copy_all_tree_prompts',
            });
            setError(err instanceof Error ? err.message : String(err) || 'Failed to copy tree prompts');
        }
    }, [treeData, setActionMsg, setError]);

    const handleCopyPromptWithImages = useCallback(
        async (markdownText: string, mediaPaths?: string[]): Promise<boolean> => {
            try {
                return await copyPromptWithRichImages(
                    markdownText,
                    (msg) => {
                        setActionMsg(msg);
                        setTimeout(() => setActionMsg(null), 2500);
                    },
                    (err) => {
                        useErrorStore.getState().captureException(
                            err instanceof Error ? err : new Error(String(err)),
                            {
                                source: 'PromptTreeViewModal.handleCopyPromptWithImages',
                                triggerAction: 'copyPromptWithRichImages',
                            }
                        );
                        setError('Failed to copy prompt with images to clipboard');
                    },
                    mediaPaths
                );
            } catch (err: unknown) {
                useErrorStore.getState().captureError(err, {
                    source: 'PromptTreeViewModal.handleCopyPromptWithImages',
                    triggerAction: 'copy_prompt_with_images',
                });
                setError(err instanceof Error ? err.message : String(err) || 'Failed to copy prompt with images');
                return false;
            }
        },
        [setActionMsg, setError]
    );

    const handleCopyPromptAsText = useCallback(
        async (markdownText: string): Promise<boolean> => {
            try {
                return await copyPromptCleanText(
                    markdownText,
                    (msg) => {
                        setActionMsg(msg);
                        setTimeout(() => setActionMsg(null), 2500);
                    },
                    (err) => {
                        useErrorStore.getState().captureException(
                            err instanceof Error ? err : new Error(String(err)),
                            {
                                source: 'PromptTreeViewModal.handleCopyPromptAsText',
                                triggerAction: 'copyPromptCleanText',
                            }
                        );
                        setError('Failed to copy prompt as clean text to clipboard');
                    }
                );
            } catch (err: unknown) {
                useErrorStore.getState().captureError(err, {
                    source: 'PromptTreeViewModal.handleCopyPromptAsText',
                    triggerAction: 'copy_prompt_as_text',
                });
                setError(err instanceof Error ? err.message : String(err) || 'Failed to copy prompt as text');
                return false;
            }
        },
        [setActionMsg, setError]
    );

    /**
     * Export the full prompt tree (prompts, structure, metadata) to a
     * portable SQLite `.db` file. Tauri: native save dialog. Web: byte
     * payload downloaded as a blob.
     */
    const handleExportSqlite = useCallback(async () => {
        try {
            setActionMsg('Exporting prompt tree to SQLite...');
            const fileName = `prompt-tree-${instanceId || 'all'}-${Date.now()}.db`;

            if (!isTauri()) {
                const payload = await invoke<TreeExportPayload>('export_prompt_tree_sqlite_bytes');
                const bytes = Uint8Array.from(atob(payload.data_b64), (c) => c.charCodeAt(0));
                const blob = new Blob([bytes as BlobPart], { type: 'application/x-sqlite3' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = payload.file_name || fileName;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                setActionMsg(
                    `Exported ${payload.stats.project_count} project(s), ${payload.stats.prompt_count} prompt(s) to SQLite.`
                );
                setTimeout(() => setActionMsg(null), 4000);
                return;
            }

            const { save } = await import('@tauri-apps/plugin-dialog');
            const selected = await save({
                defaultPath: fileName,
                filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite'] }],
            });
            if (!selected) {
                setActionMsg(null);
                return;
            }
            const stats = await invoke<TreeExportStats>('export_prompt_tree_sqlite', {
                path: selected,
            });
            setActionMsg(
                `Exported ${stats.project_count} project(s), ${stats.prompt_count} prompt(s) to ${stats.path}`
            );
            setTimeout(() => setActionMsg(null), 4000);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeViewModal.handleExportSqlite',
                triggerAction: 'export_prompt_tree_sqlite',
            });
            setError(err instanceof Error ? err.message : String(err) || 'Failed to export prompt tree to SQLite');
            setActionMsg(null);
        }
    }, [instanceId, setActionMsg, setError]);

    /**
     * Import a portable SQLite `.db` tree export into the live database.
     * The backend validates the file (SQLite magic, required tables, export
     * kind) and reports real errors. Tauri: native open dialog. Web: hidden
     * file input (handled by handleFileRestore).
     */
    const handleImportSqlite = useCallback(async () => {
        try {
            if (!isTauri()) {
                restoreFileInputRef.current?.click();
                return;
            }
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({
                multiple: false,
                filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite'] }],
            });
            if (!selected || typeof selected !== 'string') {
                setActionMsg(null);
                return;
            }
            setActionMsg('Importing prompt tree from SQLite...');
            const restored = await invoke<number>('import_prompt_tree_sqlite', {
                path: selected,
            });
            setActionMsg(`Imported ${restored} prompt(s) from SQLite export.`);
            await loadTree(true, true);
            setTimeout(() => setActionMsg(null), 5000);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeViewModal.handleImportSqlite',
                triggerAction: 'import_prompt_tree_sqlite',
            });
            setError(err instanceof Error ? err.message : String(err) || 'Failed to import prompt tree from SQLite');
            setActionMsg(null);
        }
    }, [restoreFileInputRef, loadTree, setActionMsg, setError]);

    const handleExportRunningJson = useCallback(async () => {
        await downloadPromptTreeStatusJson('running');
    }, []);

    const handleExportQueuedJson = useCallback(async () => {
        await downloadPromptTreeStatusJson('queued');
    }, []);

    const handleExportRunningAndQueuedJson = useCallback(async () => {
        await downloadPromptTreeStatusJson('running_and_queued');
    }, []);

    const [isResendingQueued, setIsResendingQueued] = useState(false);

    /**
     * Re-send all enqueued prompts in FIFO order (Task-03).
     * GUI twin of `agm prompts resend-queued`: takes a durable FIFO snapshot
     * of the queue first, then re-sends via a single dispatch pass.
     * Real counts (dispatched / skipped / failed) — never silent success.
     */
    const handleResendQueued = useCallback(async () => {
        if (!isTauri()) {
            setActionMsg('Re-send queued prompts is available in the desktop app.');
            setTimeout(() => setActionMsg(null), 3000);
            return;
        }
        if (isResendingQueued) return;
        setIsResendingQueued(true);
        try {
            setActionMsg('Re-sending queued prompts in FIFO order…');
            const outcome = await invoke<ResendQueuedOutcome>('resend_queued_prompts', {
                instanceId: instanceId || 'default',
                limit: 20,
            });
            const summary =
                `Re-sent queued prompts: ${outcome.dispatched} dispatched` +
                ` (${outcome.dispatched_from_backup} from backup),` +
                ` ${outcome.skipped_duplicates} skipped (already sent/queued),` +
                ` ${outcome.failed} failed.`;
            setActionMsg(
                outcome.failed > 0 && outcome.failures.length > 0
                    ? `${summary} First failure: ${outcome.failures[0]}`
                    : summary
            );
            await loadTree(true, true);
            setTimeout(() => setActionMsg(null), 6000);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeViewModal.handleResendQueued',
                triggerAction: 'resend_queued_prompts',
            });
            setError(err instanceof Error ? err.message : String(err) || 'Failed to re-send queued prompts');
            setActionMsg(null);
        } finally {
            setIsResendingQueued(false);
        }
    }, [instanceId, isResendingQueued, loadTree, setActionMsg, setError]);

    const [isExportModalOpen, setIsExportModalOpen] = useState(false);
    const [exportScopeTarget, setExportScopeTarget] = useState<ExportScopeParams | undefined>(undefined);
    const [exportInitialFormat, setExportInitialFormat] = useState<'json' | 'sqlite' | 'yaml'>('json');

    const [isImportModalOpen, setIsImportModalOpen] = useState(false);

    const handleOpenExportModal = useCallback((scope?: ExportScopeParams, initialFormat?: 'json' | 'sqlite' | 'yaml') => {
        setExportScopeTarget(scope);
        if (initialFormat) {
            setExportInitialFormat(initialFormat);
        }
        setIsExportModalOpen(true);
    }, []);

    const handleCloseExportModal = useCallback(() => {
        setIsExportModalOpen(false);
    }, []);

    const handleOpenImportModal = useCallback(() => {
        setIsImportModalOpen(true);
    }, []);

    const handleCloseImportModal = useCallback(() => {
        setIsImportModalOpen(false);
    }, []);

    const executeSectionExport = useCallback(
        async (params: {
            scope: ExportScopeParams;
            format: 'json' | 'sqlite' | 'yaml';
            embed_images: boolean;
            export_mode?: 'standalone' | 'lean';
        }) => {
            const { scope, format, embed_images, export_mode } = params;
            const ext = format === 'sqlite' ? 'db' : format === 'yaml' ? 'yaml' : 'json';
            const targetSlug = scope.project_name || scope.project_id || scope.conversation_id || scope.target_type;
            const mode = export_mode || (embed_images ? 'standalone' : 'lean');
            const modeLabel = mode === 'standalone' ? 'Standalone' : 'Clean';
            const extSuffix = format === 'json'
                ? (mode === 'standalone' ? 'standalone.json' : 'lean.json')
                : ext;
            const fileName = `prompt-tree-${targetSlug}-${Date.now()}.${extSuffix}`;

            try {
                if (!isTauri()) {
                    setActionMsg(`Exporting ${modeLabel} section as ${format.toUpperCase()}...`);
                    const [dataB64, returnedFileName, stats] = await invoke<[string, string, TreeExportStats]>('export_prompt_tree_section_bytes', {
                        scope,
                        format,
                        options: {
                            format,
                            embed_images: mode === 'standalone',
                            export_mode: mode,
                        },
                    });
                    const binary = atob(dataB64);
                    const bytes = new Uint8Array(binary.length);
                    for (let i = 0; i < binary.length; i++) {
                        bytes[i] = binary.charCodeAt(i);
                    }
                    const mimeType = format === 'sqlite' ? 'application/x-sqlite3' : format === 'yaml' ? 'text/yaml' : 'application/json';
                    const blob = new Blob([bytes], { type: mimeType });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = returnedFileName || fileName;
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);

                    setActionMsg(`Exported ${stats.prompt_count} prompt(s) (${stats.project_count} projects) as ${modeLabel} ${format.toUpperCase()}`);
                    setTimeout(() => setActionMsg(null), 4000);
                    return;
                }

                const { save } = await import('@tauri-apps/plugin-dialog');
                const filters = format === 'sqlite'
                    ? [{ name: 'SQLite Database', extensions: ['db', 'sqlite'] }]
                    : format === 'yaml'
                    ? [{ name: 'YAML Document', extensions: ['yaml', 'yml'] }]
                    : mode === 'standalone'
                    ? [
                        { name: 'Standalone JSON (.standalone.json)', extensions: ['standalone.json', 'json'] },
                        { name: 'JSON Document (.json)', extensions: ['json'] },
                      ]
                    : [
                        { name: 'Clean JSON (.lean.json)', extensions: ['lean.json', 'json'] },
                        { name: 'JSON Document (.json)', extensions: ['json'] },
                      ];

                const selected = await save({
                    defaultPath: fileName,
                    filters,
                });
                if (!selected) {
                    return;
                }

                setActionMsg(`Exporting ${modeLabel} section to ${selected}...`);
                const stats = await invoke<TreeExportStats>('export_prompt_tree_section', {
                    scope,
                    format,
                    options: {
                        format,
                        embed_images: mode === 'standalone',
                        export_mode: mode,
                    },
                    destinationPath: selected,
                });

                setActionMsg(`Exported ${stats.prompt_count} prompt(s) across ${stats.project_count} project(s) as ${modeLabel} ${format.toUpperCase()} to ${stats.path || selected}`);
                setTimeout(() => setActionMsg(null), 4000);
            } catch (err: unknown) {
                useErrorStore.getState().captureError(err, {
                    source: 'PromptTreeViewModal.executeSectionExport',
                    triggerAction: 'export_prompt_tree_section',
                });
                setError(err instanceof Error ? err.message : String(err) || 'Failed to export prompt tree section');
                setActionMsg(null);
                throw err;
            }
        },
        [setActionMsg, setError]
    );

    const executeSectionImport = useCallback(
        async (params: {
            filePath?: string;
            dataB64?: string;
            conflictMode: 'merge' | 'replace';
            materializeImages: boolean;
            targetInstanceId?: string;
        }): Promise<number> => {
            try {
                setActionMsg('Importing prompt tree section...');
                const res = await invoke<{ success: boolean; imported_count: number; materialized_images_count: number; message: string } | number>('import_prompt_tree_section', {
                    filePath: params.filePath || null,
                    sourcePath: params.filePath || null,
                    dataB64: params.dataB64 || null,
                    options: {
                        conflict_mode: params.conflictMode,
                        materialize_images: params.materializeImages,
                        target_instance_id: params.targetInstanceId || instanceId || null,
                    },
                });

                const count = typeof res === 'number' ? res : (res.imported_count ?? 0);
                const imgCount = typeof res === 'object' && res.materialized_images_count ? res.materialized_images_count : 0;
                setActionMsg(`Imported ${count} prompt(s) with ${imgCount} materialized image(s).`);
                await loadTree(false, true);
                setTimeout(() => setActionMsg(null), 5000);
                return count;
            } catch (err: unknown) {
                useErrorStore.getState().captureError(err, {
                    source: 'PromptTreeViewModal.executeSectionImport',
                    triggerAction: 'import_prompt_tree_section',
                });
                setError(err instanceof Error ? err.message : String(err) || 'Failed to import prompt tree section');
                setActionMsg(null);
                throw err;
            }
        },
        [instanceId, loadTree, setActionMsg, setError]
    );

    return {
        togglePinProject,
        toggleArchiveProject,
        isProjectPinned,
        handleRefreshSingleProject,
        handleBackup,
        handleFileRestore,
        handleRestore,
        handleCopyAllTreePrompts,
        handleCopyPromptWithImages,
        handleCopyPromptAsText,
        handleExportSqlite,
        handleImportSqlite,
        handleExportRunningJson,
        handleExportQueuedJson,
        handleExportRunningAndQueuedJson,
        handleResendQueued,
        isResendingQueued,
        isExportModalOpen,
        setIsExportModalOpen,
        exportScopeTarget,
        setExportScopeTarget,
        exportInitialFormat,
        setExportInitialFormat,
        handleOpenExportModal,
        handleCloseExportModal,
        executeSectionExport,
        isImportModalOpen,
        setIsImportModalOpen,
        handleOpenImportModal,
        handleCloseImportModal,
        executeSectionImport,
    };
}
