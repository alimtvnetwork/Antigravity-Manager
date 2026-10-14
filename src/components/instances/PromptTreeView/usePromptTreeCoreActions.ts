import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    setArchivedProjectsForInstance,
} from '../../../services/instanceService';
import { useErrorStore } from '../../../stores/error-store';
import { isTauri } from '../../../utils/env';
import type { AgmProjectTreeNode } from './types';
import type { PromptTreeState } from './usePromptTreeState';

export interface PromptTreeCoreActions {
    togglePinProject: (projectId: string, e: React.MouseEvent) => void;
    toggleArchiveProject: (projectId: string, e: React.MouseEvent) => void;
    handleRefreshSingleProject: (projectId: string, e: React.MouseEvent) => Promise<void>;
    handleBackup: () => Promise<void>;
    handleFileRestore: (e: React.ChangeEvent<HTMLInputElement>) => void;
    handleRestore: () => Promise<void>;
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

    const togglePinProject = useCallback(
        (projectId: string, e: React.MouseEvent) => {
            e.stopPropagation();
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
        (projectId: string, e: React.MouseEvent) => {
            e.stopPropagation();
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
            let dispatched = 0;
            let dispatchError = '';
            try {
                dispatched = await invoke<number>('dispatch_backed_up_prompts', {
                    instanceId: instanceId || 'default',
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
                : ` Restored ${restoredCount} prompt(s) and sent ${dispatched} to the instance.`;
            setActionMsg(dispatchNote.trim());
            await loadTree(true, true);
            setTimeout(() => setActionMsg(null), 5000);
        },
        [instanceId, loadTree, setActionMsg]
    );

    const handleFileRestore = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            const file = e.target.files?.[0];
            if (!file) return;
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
        [persistRestoredBackup, setActionMsg, setError]
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

    return {
        togglePinProject,
        toggleArchiveProject,
        handleRefreshSingleProject,
        handleBackup,
        handleFileRestore,
        handleRestore,
    };
}
