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

/**
 * Evaluates whether an instance identifier is empty or matches default instance aliases.
 */
export function isDefaultInstanceAlias(instanceId?: string | null): boolean {
    if (!instanceId) {
        return true;
    }
    const normalized = instanceId.trim().toLowerCase();
    const isDefaultExact = normalized === 'default' || normalized === '__default__';
    const hasDefaultSubstring = normalized.includes('default');
    return isDefaultExact || hasDefaultSubstring;
}

/**
 * Resolves cache keys for SWR invalidation and optimistic updates.
 */
export function resolveInstanceCacheKeys(targetInstanceId?: string | null): string[] {
    if (!targetInstanceId) {
        return Array.from(promptTreeSwrCache.keys());
    }
    if (isDefaultInstanceAlias(targetInstanceId)) {
        const defaultKeys = new Set([targetInstanceId, 'default', '__default__']);
        return Array.from(defaultKeys);
    }
    return [targetInstanceId];
}

/** Invalidate in-memory SWR cache across instances or for a specific instance */
export function invalidatePromptTreeCache(targetInstanceId?: string): void {
    if (!targetInstanceId) {
        promptTreeSwrCache.clear();
        return;
    }
    promptTreeSwrCache.delete(targetInstanceId);
    if (isDefaultInstanceAlias(targetInstanceId)) {
        promptTreeSwrCache.delete('default');
        promptTreeSwrCache.delete('__default__');
    }
}

/**
 * Optimistically update conversation title in the in-memory SWR cache across instances
 */
export function renameConversationOptimistic(
    conversationId: string,
    newTitle: string,
    targetInstanceId?: string,
    onSelectionUpdate?: (conv: AgmConversationNode) => void
): void {
    const keys = resolveInstanceCacheKeys(targetInstanceId);

    for (const key of keys) {
        const entry = promptTreeSwrCache.get(key);
        if (!entry) continue;

        let modified = false;
        let updatedConvNode: AgmConversationNode | null = null;
        const updatedProjects = entry.data.map((proj) => {
            const hasConv = proj.conversations.some((c) => c.conversation_id === conversationId);
            if (!hasConv) return proj;

            modified = true;
            return {
                ...proj,
                conversations: proj.conversations.map((conv) => {
                    if (conv.conversation_id === conversationId) {
                        const updated = { ...conv, title: newTitle };
                        updatedConvNode = updated;
                        return updated;
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
            if (updatedConvNode && onSelectionUpdate) {
                onSelectionUpdate(updatedConvNode);
            }
        }
    }
}

// Shared image filter toggle emitter across header toolbar and tree panel
export const imageFilterEmitter = {
    isActive: false,
    listeners: new Set<(active: boolean) => void>(),
    get(): boolean {
        return this.isActive;
    },
    set(active: boolean) {
        this.isActive = active;
        this.listeners.forEach((fn) => fn(active));
    },
    toggle() {
        this.set(!this.isActive);
    },
    subscribe(fn: (active: boolean) => void) {
        this.listeners.add(fn);
        return () => {
            this.listeners.delete(fn);
        };
    },
};

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
        return () => {
            this.listeners.delete(fn);
        };
    },
};

/**
 * Evaluates whether a conversation node has a non-empty prompt or preview.
 */
export function hasMeaningfulPrompt(c: AgmConversationNode): boolean {
    const hasWordCount = c.prompt_word_count > 0;
    const hasPreview = Boolean(c.prompt_preview_200w && c.prompt_preview_200w.trim());
    return hasWordCount || hasPreview;
}

/**
 * Evaluates whether a conversation node is valid and not a ghost or empty node.
 */
export function isValidConversation(c: AgmConversationNode): boolean {
    if (isGhostConversation(c)) {
        return false;
    }
    return hasMeaningfulPrompt(c);
}

/**
 * Evaluates whether the instance filter targets all instances without profile scoping.
 */
export function isAllInstancesFilter(instanceId?: string | null): boolean {
    if (!instanceId) {
        return true;
    }
    return instanceId.trim().toLowerCase() === 'all';
}

/**
 * Determines whether a node's instance identifier matches the active instance target.
 * Resolves exact matches and cross-alias default compatibility.
 */
export function isInstanceCompatible(
    nodeInstanceId?: string | null,
    targetInstanceId?: string | null
): boolean {
    if (nodeInstanceId === targetInstanceId) {
        return true;
    }
    const isTargetDefault = isDefaultInstanceAlias(targetInstanceId);
    const isNodeDefault = isDefaultInstanceAlias(nodeInstanceId);
    return isTargetDefault && isNodeDefault;
}

/**
 * Returns true if a project is marked running or contains any active conversations.
 */
export function isProjectRunning(project: AgmProjectTreeNode): boolean {
    const isNodeRunning = Boolean(project.is_running);
    const hasRunningConversation = (project.conversations || []).some((c) => Boolean(c.is_running));
    return isNodeRunning || hasRunningConversation;
}

/**
 * Resolves the newest modification timestamp among conversations belonging to a project.
 */
export function getProjectLatestTimestamp(project: AgmProjectTreeNode): number {
    if (!project.conversations || project.conversations.length === 0) {
        return 0;
    }
    return Math.max(0, ...project.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
}

/**
 * Comparator sorting projects by pinned status, running activity, last modified time, and name.
 */
export function compareProjectsPriority(
    a: AgmProjectTreeNode,
    b: AgmProjectTreeNode,
    pinnedProjectIds: string[]
): number {
    const isAPinned = pinnedProjectIds.includes(a.project_id);
    const isBPinned = pinnedProjectIds.includes(b.project_id);
    if (isAPinned !== isBPinned) {
        return isAPinned ? -1 : 1;
    }

    const isARunning = isProjectRunning(a);
    const isBRunning = isProjectRunning(b);
    if (isARunning !== isBRunning) {
        return isARunning ? -1 : 1;
    }

    const aLatest = getProjectLatestTimestamp(a);
    const bLatest = getProjectLatestTimestamp(b);
    if (bLatest !== aLatest) {
        return bLatest - aLatest;
    }

    return a.repo_name.localeCompare(b.repo_name);
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
        const runningTurn = conv.prompts.find((p) => {
            const isRunning = Boolean(p.is_running);
            const hasText = Boolean(p.prompt_text && p.prompt_text.trim());
            return isRunning && hasText;
        });
        if (runningTurn?.prompt_text) {
            return runningTurn.prompt_text;
        }
        const lastTurnWithText = [...conv.prompts]
            .reverse()
            .find((p) => {
                const textLength = p.prompt_text ? p.prompt_text.trim().length : 0;
                return textLength > text.trim().length;
            });
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
            let effectiveConv = conv;
            let effectiveTurnNum = conv.selected_turn_number;
            const hasNoTurn = effectiveTurnNum == null;
            const hasPrompts = Boolean(conv.prompts && conv.prompts.length > 0);
            if (hasNoTurn && hasPrompts && conv.prompts) {
                const runningTurn = conv.prompts.find((p) => Boolean(p.is_running)) || conv.prompts[conv.prompts.length - 1];
                if (runningTurn) {
                    effectiveTurnNum = runningTurn.turn_number;
                    effectiveConv = {
                        ...conv,
                        selected_turn_number: effectiveTurnNum,
                        full_prompt_text: runningTurn.prompt_text,
                        prompt_preview_200w: runningTurn.preview_200w || runningTurn.prompt_text.slice(0, 200),
                    };
                }
            }
            setSelectedConversation(effectiveConv);
            const text = resolveConversationPromptText(effectiveConv, effectiveTurnNum);
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
                        const runningConv = target.conversations.find((c) => {
                            const isRunning = Boolean(c.is_running);
                            return isRunning && isValidConversation(c);
                        });
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

            const prioritized = [...targetPool].sort((a, b) =>
                compareProjectsPriority(a, b, currentPinnedIds)
            );

            // 1. Look for actively running conversation across non-archived projects
            const runningCandidates: RunningCand[] = [];
            for (const proj of prioritized) {
                for (const conv of proj.conversations) {
                    const isRunning = Boolean(conv.is_running);
                    if (isRunning && isValidConversation(conv)) {
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

            // 2. If no conversation is running, check if any non-archived project is running
            const runningProj = prioritized.find((p) => {
                const isRunning = isProjectRunning(p);
                const hasConversations = p.conversations.length > 0;
                return isRunning && hasConversations;
            });
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
                const isFilterAll = isAllInstancesFilter(instanceId);
                const relevant = isFilterAll
                    ? data
                    : data
                          .filter((p) => isInstanceCompatible(p.instance_id, instanceId))
                          .map((p) => ({
                              ...p,
                              conversations: (p.conversations || []).filter((c) =>
                                  isInstanceCompatible(c.instance_id, instanceId)
                              ),
                          }));
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
                    const updatedProj = finalData.find((p) => {
                        const isSameProject = Boolean(currentProj && p.project_id === currentProj.project_id);
                        const hasMatchingConversation = p.conversations.some((c) => c.conversation_id === currentId);
                        return isSameProject || hasMatchingConversation;
                    });
                    if (updatedProj) {
                        setSelectedProject(updatedProj);
                        const updatedConv = updatedProj.conversations.find(
                            (c) => c.conversation_id === currentId
                        );
                        if (updatedConv) {
                            hasRetainedSelection = true;
                            const hasSelectedTurn = typeof currentTurn === 'number';
                            const hasPrompts = Boolean(updatedConv.prompts && updatedConv.prompts.length > 0);
                            if (hasSelectedTurn && hasPrompts && updatedConv.prompts) {
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
