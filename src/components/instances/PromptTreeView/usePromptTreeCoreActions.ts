import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    setArchivedProjectsForInstance,
} from '../../../services/instanceService';
import { useErrorStore } from '../../../stores/error-store';
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
                // Best-effort: backup proceeds with tree data even if DB read fails. Tracked.
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
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `prompts-backup-${instanceId || 'all'}-${Date.now()}.json`;
            a.click();
            URL.revokeObjectURL(url);
            setActionMsg(`Backed up ${fullBackupPayload.totalProjects} project(s) and ${fullBackupPayload.totalConversations} conversation(s)!`);
            setTimeout(() => setActionMsg(null), 3000);
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : String(err) || 'Failed to backup prompts');
        }
    }, [instanceId, instanceName, treeData, setActionMsg, setError]);

    const handleFileRestore = useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            const file = e.target.files?.[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = async (event) => {
                try {
                    setActionMsg('Importing and persisting prompts backup into SQLite...');
                    const content = event.target?.result as string;
                    JSON.parse(content);
                    const restoredCount = await invoke<number>('restore_prompts_backup', { backupJson: content });
                    setActionMsg(`Successfully restored and persisted ${restoredCount} prompts in strict FIFO order!`);
                    await loadTree(true, true);
                    setTimeout(() => {
                        setActionMsg(null);
                    }, 3500);
                } catch (err: unknown) {
                    const msg = err instanceof Error ? err.message : String(err);
                    setError(`Restore failed: ${msg || 'Invalid backup format'}`);
                }
            };
            reader.readAsText(file);
            e.target.value = '';
        },
        [loadTree, setActionMsg, setError]
    );

    const handleRestore = useCallback(async () => {
        if (restoreFileInputRef.current) {
            restoreFileInputRef.current.click();
        } else {
            try {
                setActionMsg('Restoring prompts with 7s channel stabilization...');
                await invoke('resume_recent_project_prompts', {
                    instanceId: instanceId || 'default',
                    maxAgeSeconds: 3600,
                });
                setActionMsg('Prompts restoration dispatched!');
                setTimeout(() => {
                    setActionMsg(null);
                    loadTree(true, true);
                }, 3000);
            } catch (err: unknown) {
                setError(err instanceof Error ? err.message : String(err) || 'Failed to restore prompts');
            }
        }
    }, [restoreFileInputRef, instanceId, loadTree, setActionMsg, setError]);

    return {
        togglePinProject,
        toggleArchiveProject,
        handleRefreshSingleProject,
        handleBackup,
        handleFileRestore,
        handleRestore,
    };
}
