import { useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getArchivedProjectsForInstance } from '../../../services/instanceService';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';
import { isGhostConversation, isStaleOrEmptyConversation } from './conversationGuards';
import type { PromptTreeState } from './usePromptTreeState';
import { useErrorStore } from '../../../stores/error-store';

export interface PromptTreeData {
    selectConversation: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    loadTree: (
        isInitialLoad?: boolean,
        isForce?: boolean,
        overrideArchivedIds?: string[],
        overridePinnedIds?: string[]
    ) => Promise<void>;
}

interface UsePromptTreeDataProps {
    state: PromptTreeState;
    instanceId: string;
    isOpen: boolean;
    initialSelectedProjectId?: string;
}

interface RunningCand {
    conv: AgmConversationNode;
    proj: AgmProjectTreeNode;
    timestamp: number;
}

interface PromptTreeSwrEntry {
    data: AgmProjectTreeNode[];
    timestamp: number;
}

// Module-level in-memory SWR cache across modal mount/unmount lifecycles
const promptTreeSwrCache = new Map<string, PromptTreeSwrEntry>();

/** Invalidate in-memory SWR cache across instances or for a specific instance */
export function invalidatePromptTreeCache(targetInstanceId?: string): void {
    if (targetInstanceId) {
        promptTreeSwrCache.delete(targetInstanceId);
        if (targetInstanceId === 'default' || targetInstanceId === '__default__') {
            promptTreeSwrCache.delete('default');
            promptTreeSwrCache.delete('__default__');
        }
    } else {
        promptTreeSwrCache.clear();
    }
}

/**
 * Optimistically update conversation title in the in-memory SWR cache across instances
 */
export function renameConversationOptimistic(
    conversationId: string,
    newTitle: string,
    targetInstanceId?: string
): void {
    const keys = targetInstanceId
        ? [targetInstanceId, ...(targetInstanceId === 'default' || targetInstanceId === '__default__' ? ['default', '__default__'] : [])]
        : Array.from(promptTreeSwrCache.keys());

    for (const key of keys) {
        const entry = promptTreeSwrCache.get(key);
        if (!entry) continue;

        let modified = false;
        const updatedProjects = entry.data.map((proj) => {
            const hasConv = proj.conversations.some((c) => c.conversation_id === conversationId);
            if (!hasConv) return proj;

            modified = true;
            return {
                ...proj,
                conversations: proj.conversations.map((conv) => {
                    if (conv.conversation_id === conversationId) {
                        return { ...conv, title: newTitle };
                    }
                    return conv;
                }),
            };
        });

        if (modified) {
            promptTreeSwrCache.set(key, {
                ...entry,
                data: updatedProjects,
            });
        }
    }
}


// Shared loading state emitter so TreePanel can observe initial load and render skeletons
export const promptTreeLoadingEmitter = {
    isLoading: false,
    listeners: new Set<(loading: boolean) => void>(),
    set(loading: boolean) {
        this.isLoading = loading;
        this.listeners.forEach((fn) => fn(loading));
    },
    subscribe(fn: (loading: boolean) => void) {
        this.listeners.add(fn);
        return () => this.listeners.delete(fn);
    },
};

function isValidConversation(c: AgmConversationNode): boolean {
    return (
        !isGhostConversation(c) &&
        !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
    );
}

function sortByLastModifiedDesc(convs: AgmConversationNode[]): AgmConversationNode[] {
    return [...convs].sort((a, b) => {
        const aTime = new Date(a.last_modified).getTime() || 0;
        const bTime = new Date(b.last_modified).getTime() || 0;
        return bTime - aTime;
    });
}

/**
 * Resolves the appropriate prompt text for a conversation node, respecting an explicitly
 * selected turn, any running turn, or falling back to the last turn with substantial text.
 */
export function resolveConversationPromptText(
    conv: AgmConversationNode,
    selectedTurnNumber?: number | null
): string {
    let text = conv.full_prompt_text || conv.prompt_preview_200w || '';
    if (conv.prompts && conv.prompts.length > 0) {
        if (typeof selectedTurnNumber === 'number') {
            const matchedTurn = conv.prompts.find((p) => p.turn_number === selectedTurnNumber);
            if (matchedTurn?.prompt_text) {
                return matchedTurn.prompt_text;
            }
        }
        const runningTurn = conv.prompts.find((p) => p.is_running && p.prompt_text && p.prompt_text.trim());
        if (runningTurn?.prompt_text) {
            return runningTurn.prompt_text;
        }
        const lastTurnWithText = [...conv.prompts]
            .reverse()
            .find((p) => p.prompt_text && p.prompt_text.trim().length > text.trim().length);
        if (lastTurnWithText?.prompt_text) {
            return lastTurnWithText.prompt_text;
        }
    }
    return text;
}

export function usePromptTreeData({
    state,
    instanceId,
    isOpen,
    initialSelectedProjectId,
}: UsePromptTreeDataProps): PromptTreeData {
    const {
        setTreeData,
        setIsLoading,
        setIsAutoSyncing,
        setError,
        archivedProjectIds,
        pinnedProjectIds,
        setArchivedProjectIds,
        setPinnedProjectIds,
        setExpandedProjects,
        setExpandedStaleGroups,
        setSelectedProject,
        setSelectedConversation,
        setActivePromptText,
        setEditedPromptText,
        setShowAllWords,
        selectedConversationRef,
        selectedProjectRef,
        activePromptTextRef,
        syncInterval,
    } = state;

    const archivedProjectIdsRef = useRef(archivedProjectIds);
    archivedProjectIdsRef.current = archivedProjectIds;
    const pinnedProjectIdsRef = useRef(pinnedProjectIds);
    pinnedProjectIdsRef.current = pinnedProjectIds;

    // Select conversation directly (2-tier leaf)
    const selectConversation = useCallback(
        (conv: AgmConversationNode, project: AgmProjectTreeNode) => {
            setSelectedProject(project);
            setSelectedConversation(conv);
            const text = resolveConversationPromptText(conv, conv.selected_turn_number);
            setActivePromptText(text);
            setEditedPromptText(text);
            setShowAllWords(false);
        },
        [setSelectedProject, setSelectedConversation, setActivePromptText, setEditedPromptText, setShowAllWords]
    );

    // Auto-selection algorithm on modal open
    const performAutoSelection = useCallback(
        (data: AgmProjectTreeNode[], currentArchivedIds: string[], currentPinnedIds: string[]) => {
            if (data.length === 0) {
                setSelectedProject(null);
                setSelectedConversation(null);
                return;
            }

            // If selectedConversationRef.current exists and its conversation_id is still present in data,
            // preserve that conversation selection rather than forcing winning[0]! This eliminates selection flapping.
            if (selectedConversationRef.current) {
                const currentConvId = selectedConversationRef.current.conversation_id;
                for (const proj of data) {
                    const matchedConv = proj.conversations.find((c) => c.conversation_id === currentConvId);
                    if (matchedConv) {
                        setExpandedProjects((prev) => ({ ...prev, [proj.project_id]: true }));
                        selectConversation(matchedConv, proj);
                        return;
                    }
                }
            }

            // Direct deep-link focus if initialSelectedProjectId is provided
            if (initialSelectedProjectId) {
                const target = data.find((p) => p.project_id === initialSelectedProjectId);
                if (target) {
                    setExpandedProjects({ [target.project_id]: true });
                    setSelectedProject(target);
                    if (target.conversations && target.conversations.length > 0) {
                        const runningConv = target.conversations.find(
                            (c) => Boolean(c.is_running) && isValidConversation(c)
                        );
                        if (runningConv) {
                            selectConversation(runningConv, target);
                            setExpandedStaleGroups(
                                isStaleOrEmptyConversation(runningConv) ? { [target.project_id]: true } : {}
                            );
                            return;
                        }
                        const candidateConvs = target.conversations.filter(isValidConversation);
                        if (candidateConvs.length > 0) {
                            const winningConv = sortByLastModifiedDesc(candidateConvs)[0];
                            selectConversation(winningConv, target);
                            setExpandedStaleGroups(
                                isStaleOrEmptyConversation(winningConv) ? { [target.project_id]: true } : {}
                            );
                        } else {
                            setSelectedConversation(null);
                        }
                    } else {
                        setSelectedConversation(null);
                    }
                    return;
                }
            }

            // Target non-archived projects in prioritized order
            const nonArchived = data.filter((p) => !currentArchivedIds.includes(p.project_id));
            const targetPool = nonArchived.length > 0 ? nonArchived : data;

            const prioritized = [...targetPool].sort((a, b) => {
                const aPinned = currentPinnedIds.includes(a.project_id);
                const bPinned = currentPinnedIds.includes(b.project_id);
                if (aPinned !== bPinned) return aPinned ? -1 : 1;

                const aRunning = Boolean(a.is_running) || a.conversations.some((c) => Boolean(c.is_running));
                const bRunning = Boolean(b.is_running) || b.conversations.some((c) => Boolean(c.is_running));
                if (aRunning !== bRunning) return aRunning ? -1 : 1;

                const aLatest = Math.max(0, ...a.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
                const bLatest = Math.max(0, ...b.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
                if (bLatest !== aLatest) return bLatest - aLatest;

                return a.repo_name.localeCompare(b.repo_name);
            });

            // 1. Look for actively running conversation across non-archived projects
            const runningCandidates: RunningCand[] = [];
            for (const proj of prioritized) {
                for (const conv of proj.conversations) {
                    if (Boolean(conv.is_running) && isValidConversation(conv)) {
                        runningCandidates.push({
                            conv,
                            proj,
                            timestamp: new Date(conv.last_modified).getTime() || 0,
                        });
                    }
                }
            }

            if (runningCandidates.length > 0) {
                // Pick newest running conversation
                runningCandidates.sort((a, b) => b.timestamp - a.timestamp);
                const winning = runningCandidates[0];
                setExpandedProjects({ [winning.proj.project_id]: true });
                selectConversation(winning.conv, winning.proj);
                setExpandedStaleGroups(
                    isStaleOrEmptyConversation(winning.conv) ? { [winning.proj.project_id]: true } : {}
                );
                return;
            }

            // 2. If no conversation is running, check if any non-archived project has Boolean(p.is_running)
            const runningProj = prioritized.find((p) => Boolean(p.is_running) && p.conversations.length > 0);
            if (runningProj) {
                const candidateConvs = runningProj.conversations.filter(isValidConversation);
                if (candidateConvs.length > 0) {
                    const winningConv = sortByLastModifiedDesc(candidateConvs)[0];
                    setExpandedProjects({ [runningProj.project_id]: true });
                    selectConversation(winningConv, runningProj);
                    setExpandedStaleGroups(
                        isStaleOrEmptyConversation(winningConv) ? { [runningProj.project_id]: true } : {}
                    );
                    return;
                }
            }

            // 3. Fallback: Auto-expand the first non-archived project in prioritized order and select its newest conversation
            const firstProj = prioritized[0];
            if (firstProj) {
                setExpandedProjects({ [firstProj.project_id]: true });
                const candidateConvs = firstProj.conversations.filter(isValidConversation);
                if (candidateConvs.length > 0) {
                    const winningConv = sortByLastModifiedDesc(candidateConvs)[0];
                    selectConversation(winningConv, firstProj);
                    setExpandedStaleGroups(
                        isStaleOrEmptyConversation(winningConv) ? { [firstProj.project_id]: true } : {}
                    );
                } else {
                    setSelectedProject(firstProj);
                    setSelectedConversation(null);
                }
            }
        },
        [selectConversation, initialSelectedProjectId, setSelectedProject, setSelectedConversation, setExpandedProjects, setExpandedStaleGroups]
    );

    const loadTree = useCallback(
        async (
            isInitialLoad: boolean = true,
            isForce: boolean = false,
            overrideArchivedIds?: string[],
            overridePinnedIds?: string[]
        ) => {
            if (isInitialLoad) {
                setIsLoading(true);
                promptTreeLoadingEmitter.set(true);
            } else {
                setIsAutoSyncing(true);
            }
            setError(null);
            try {
                if (isForce) {
                    invalidatePromptTreeCache(instanceId);
                    try {
                        await invoke('reset_prompt_tree_cache', { instanceId: instanceId || null });
                    } catch (resetErr: unknown) {
                        useErrorStore.getState().trackWarning(resetErr, {
                            source: 'usePromptTreeData.loadTree.resetCache',
                            triggerAction: 'reset_prompt_tree_cache',
                        });
                    }
                }
                const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                    instanceId: instanceId || undefined,
                    maxWords: 2000,
                    onlyRunning: false,
                    force: isForce,
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

                // Cache in memory for immediate SWR retrieval on subsequent opens
                const cacheKey = instanceId || 'default';
                if (isForce) {
                    promptTreeSwrCache.delete(cacheKey);
                }
                promptTreeSwrCache.set(cacheKey, { data: finalData, timestamp: Date.now() });

                setTreeData(finalData);

                // Retain active selection across loads and background sync cycles
                const currentConv = selectedConversationRef.current;
                const currentProj = selectedProjectRef.current;
                const currentActiveText = activePromptTextRef.current;

                let hasRetainedSelection = false;
                if (currentConv) {
                    const currentId = currentConv.conversation_id;
                    const currentTurn = currentConv.selected_turn_number;
                    const updatedProj = finalData.find((p) =>
                        (currentProj && p.project_id === currentProj.project_id) ||
                        p.conversations.some((c) => c.conversation_id === currentId)
                    );
                    if (updatedProj) {
                        setSelectedProject(updatedProj);
                        const updatedConv = updatedProj.conversations.find(
                            (c) => c.conversation_id === currentId
                        );
                        if (updatedConv) {
                            hasRetainedSelection = true;
                            if (typeof currentTurn === 'number' && updatedConv.prompts?.length) {
                                const matchingTurn = updatedConv.prompts.find(
                                    (t) => t.turn_number === currentTurn
                                );
                                if (matchingTurn) {
                                    const turnConv: AgmConversationNode = {
                                        ...updatedConv,
                                        prompt_preview_200w: matchingTurn.preview_200w || matchingTurn.prompt_text.slice(0, 200),
                                        full_prompt_text: matchingTurn.prompt_text,
                                        latest_step_summary: `Turn ${matchingTurn.turn_number} · ${matchingTurn.status}`,
                                        is_running: matchingTurn.is_running,
                                        is_queued: matchingTurn.is_queued,
                                        status: matchingTurn.status,
                                        selected_turn_number: matchingTurn.turn_number,
                                    };
                                    setSelectedConversation(turnConv);
                                    const freshText = matchingTurn.prompt_text || '';
                                    setActivePromptText(freshText);
                                    setEditedPromptText((prev) => (prev === currentActiveText ? freshText : prev));
                                } else {
                                    const freshText = resolveConversationPromptText(updatedConv, null);
                                    setSelectedConversation(updatedConv);
                                    setActivePromptText(freshText);
                                    setEditedPromptText((prev) => (prev === currentActiveText ? freshText : prev));
                                }
                            } else {
                                const freshText = resolveConversationPromptText(updatedConv, null);
                                setSelectedConversation(updatedConv);
                                setActivePromptText(freshText);
                                // Do not clobber user's dirty textarea edits
                                setEditedPromptText((prev) => (prev === currentActiveText ? freshText : prev));
                            }
                        }
                    }
                }

                if (isInitialLoad && !hasRetainedSelection) {
                    const targetArchived = overrideArchivedIds || archivedProjectIdsRef.current;
                    const targetPinned = overridePinnedIds || pinnedProjectIdsRef.current;
                    performAutoSelection(finalData, targetArchived, targetPinned);
                }
            } catch (err: unknown) {
                if (isInitialLoad) {
                    setError(err instanceof Error ? err.message : String(err) || 'Failed to load project and conversation tree');
                    useErrorStore.getState().trackWarning(err, { source: 'usePromptTreeData.loadTree', triggerAction: 'initial_load' });
                } else {
                    useErrorStore.getState().trackWarning(err, { source: 'usePromptTreeData.loadTree', triggerAction: 'auto_sync' });
                }
            } finally {
                if (isInitialLoad) {
                    setIsLoading(false);
                    promptTreeLoadingEmitter.set(false);
                } else {
                    setIsAutoSyncing(false);
                }
            }
        },
        [
            instanceId,
            performAutoSelection,
            selectedConversationRef,
            selectedProjectRef,
            activePromptTextRef,
            setTreeData,
            setIsLoading,
            setIsAutoSyncing,
            setError,
            setSelectedProject,
            setSelectedConversation,
            setActivePromptText,
            setEditedPromptText,
        ]
    );

    const hasInitializedRef = useRef(false);

    // Modal open effect: Check SWR in-memory cache first for instant 0ms perceived render
    useEffect(() => {
        if (!isOpen) {
            hasInitializedRef.current = false;
            return;
        }

        const cacheKey = instanceId || 'default';
        const cached = promptTreeSwrCache.get(cacheKey);
        const latestArchived = getArchivedProjectsForInstance(instanceId);
        const latestPinned = (() => {
            try {
                const stored = localStorage.getItem(`agm_pinned_projects_${instanceId || 'default'}`);
                return stored ? JSON.parse(stored) : [];
            } catch {
                return [];
            }
        })();
        setArchivedProjectIds(latestArchived);
        setPinnedProjectIds(latestPinned);

        if (!hasInitializedRef.current) {
            hasInitializedRef.current = true;
            if (cached && cached.data.length > 0) {
                // Immediate SWR render
                setTreeData(cached.data);
                performAutoSelection(cached.data, latestArchived, latestPinned);
                // Revalidate in background using force: false to leverage backend SQLite cache
                loadTree(false, false, latestArchived, latestPinned);
            } else {
                // Initial load using force: false to leverage backend SQLite cache
                loadTree(true, false, latestArchived, latestPinned);
            }
        }
    }, [isOpen, instanceId, initialSelectedProjectId, loadTree, performAutoSelection, setArchivedProjectIds, setPinnedProjectIds, setTreeData]);

    // Configurable auto-sync interval timer (minimum 15s floor)
    useEffect(() => {
        if (!isOpen || syncInterval === 'off') return;

        let intervalMs = 30000;
        if (syncInterval === '15s') intervalMs = 15000;
        else if (syncInterval === '30s') intervalMs = 30000;
        else if (syncInterval === '1m') intervalMs = 60000;
        else if (syncInterval === '2m') intervalMs = 120000;

        const safeIntervalMs = Math.max(intervalMs, 15000);

        const timer = setInterval(() => {
            loadTree(false, false);
        }, safeIntervalMs);

        return () => clearInterval(timer);
    }, [isOpen, syncInterval, instanceId, loadTree]);

    return { selectConversation, loadTree };
}
