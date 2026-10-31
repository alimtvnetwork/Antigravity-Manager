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

export const TREE_CACHE_TTL_MS = 30_000; // 30 seconds strict minimum

export interface TreeCacheOptions {
    ttlMs: number;
    lastLoadTimestamp: number;
    isCooldownActive: boolean;
    swrCacheSize: number;
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
    if (
        normalized === 'default' ||
        normalized === '__default__' ||
        normalized === '' ||
        normalized === '#1 default' ||
        normalized === '1 default' ||
        normalized === '#1 (default)' ||
        normalized === 'default (active)' ||
        normalized === '#1 default (active)'
    ) {
        return true;
    }
    return normalized.startsWith('#1') && /\bdefault\b/.test(normalized);
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
function queuedTurnKey(p: { status: string; prompt_text: string }): string {
    return `${String(p.status).toUpperCase()}|${p.prompt_text}`;
}

export function isQueuedTurn(p: { is_queued?: boolean; status: string }): boolean {
    const s = String(p.status || '').toUpperCase();
    return Boolean(p.is_queued) || s === 'QUEUED' || s === 'ENQUEUED_IDE' || s === 'DISPATCHED' || s === 'PENDING';
}

export function queuedPromptCount(conv: { is_queued?: boolean; prompts?: Array<{ is_queued?: boolean; status: string }> } | null | undefined): number {
    if (!conv) {
        return 0;
    }
    const fromTurns = (conv.prompts || []).filter(isQueuedTurn).length;
    if (fromTurns > 0) {
        return fromTurns;
    }
    // If prompts array is hydrated and contains 0 queued turns, true queue count is 0.
    // Only fall back to conv.is_queued when prompts have not yet been loaded.
    if (!conv.prompts || conv.prompts.length === 0) {
        return conv.is_queued ? 1 : 0;
    }
    return 0;
}

/** Keep conversation QUEUED when inspecting an older completed turn. */
export function applySelectedTurnOverlay(
    conv: AgmConversationNode,
    selectedTurn?: number | null
): AgmConversationNode {
    const hasQueuedTurns = (conv.prompts || []).some(isQueuedTurn);
    if (typeof selectedTurn !== 'number' || !conv.prompts?.length) {
        if (!hasQueuedTurns) {
            return conv;
        }
        return { ...conv, is_queued: true, status: 'QUEUED' };
    }
    const matchingTurn = conv.prompts.find((t) => t.turn_number === selectedTurn);
    if (!matchingTurn) {
        if (!hasQueuedTurns) {
            return conv;
        }
        return { ...conv, is_queued: true, status: 'QUEUED' };
    }
    return {
        ...conv,
        prompt_preview_200w: matchingTurn.preview_200w || matchingTurn.prompt_text.slice(0, 200),
        full_prompt_text: matchingTurn.prompt_text,
        latest_step_summary: `Turn ${matchingTurn.turn_number} · ${matchingTurn.status}`,
        is_running: matchingTurn.is_running,
        is_queued: Boolean(matchingTurn.is_queued) || hasQueuedTurns || Boolean(conv.is_queued),
        status: hasQueuedTurns ? 'QUEUED' : matchingTurn.status,
        selected_turn_number: matchingTurn.turn_number,
    };
}

export function mergeQueuedTurns(
    incoming: AgmConversationNode,
    previous: AgmConversationNode | null | undefined
): AgmConversationNode {
    if (!previous || previous.conversation_id !== incoming.conversation_id) {
        return incoming;
    }
    const prevQueued = (previous.prompts || []).filter(isQueuedTurn);
    if (prevQueued.length === 0) {
        return incoming;
    }
    const incomingPrompts = incoming.prompts || [];
    const used = incomingPrompts.map(() => false);
    const missing: typeof prevQueued = [];
    for (const pq of prevQueued) {
        const key = queuedTurnKey(pq);
        const idx = incomingPrompts.findIndex((p, i) => !used[i] && queuedTurnKey(p) === key);
        if (idx >= 0) {
            used[idx] = true;
        } else {
            missing.push(pq);
        }
    }
    if (missing.length === 0) {
        const hasQueued = incomingPrompts.some(isQueuedTurn);
        return hasQueued ? incoming : { ...incoming, is_queued: true };
    }
    return {
        ...incoming,
        is_queued: true,
        prompts: [...incomingPrompts, ...missing],
    };
}

/** Keep in-memory SWR aligned with optimistic enqueue/rerun so instance click shows queued turns immediately. */
export function patchPromptTreeSwrCache(
    instanceId: string | undefined,
    updater: (tree: AgmProjectTreeNode[]) => AgmProjectTreeNode[]
): void {
    const cacheKey = instanceId || 'default';
    const entry = promptTreeSwrCache.get(cacheKey);
    promptTreeSwrCache.set(cacheKey, {
        data: updater(entry?.data || []),
        timestamp: Date.now(),
    });
}

function overlaySelectedQueued(
    tree: AgmProjectTreeNode[],
    selectedConv: AgmConversationNode | null | undefined,
    selectedProj: AgmProjectTreeNode | null | undefined
): AgmProjectTreeNode[] {
    if (!selectedConv || !(selectedConv.prompts || []).some(isQueuedTurn)) {
        return tree;
    }
    if (tree.length > 0) {
        return tree.map((proj) => ({
            ...proj,
            conversations: (proj.conversations || []).map((conv) =>
                conv.conversation_id === selectedConv.conversation_id
                    ? mergeQueuedTurns(conv, selectedConv)
                    : conv
            ),
        }));
    }
    if (!selectedProj) {
        return tree;
    }
    return [
        {
            ...selectedProj,
            conversations: (selectedProj.conversations || []).map((conv) =>
                conv.conversation_id === selectedConv.conversation_id ? selectedConv : conv
            ),
        },
    ];
}

export function mergeQueuedTree(
    incoming: AgmProjectTreeNode[],
    previous: AgmProjectTreeNode[]
): AgmProjectTreeNode[] {
    if (previous.length === 0) {
        return incoming;
    }
    const prevByConv = new Map<string, AgmConversationNode>();
    for (const proj of previous) {
        for (const conv of proj.conversations || []) {
            prevByConv.set(conv.conversation_id, conv);
        }
    }
    return incoming.map((proj) => ({
        ...proj,
        conversations: (proj.conversations || []).map((conv) =>
            mergeQueuedTurns(conv, prevByConv.get(conv.conversation_id))
        ),
    }));
}

export function isInstanceCompatible(
    nodeInstanceId?: string | null,
    targetInstanceId?: string | null
): boolean {
    if (nodeInstanceId === targetInstanceId) {
        return true;
    }
    const isTargetDefault = isDefaultInstanceAlias(targetInstanceId);
    const isNodeDefault = isDefaultInstanceAlias(nodeInstanceId);
    if (isTargetDefault || isNodeDefault) {
        return isTargetDefault && isNodeDefault;
    }
    if (!targetInstanceId || targetInstanceId === 'all') {
        return true;
    }
    if (!nodeInstanceId) {
        return false;
    }
    const normNode = nodeInstanceId.trim().toLowerCase();
    const normTarget = targetInstanceId.trim().toLowerCase();
    if (normNode === normTarget) {
        return true;
    }
    const cleanTarget = normTarget.replace(/^#\d+\s*/, '').trim();
    const cleanNode = normNode.replace(/^#\d+\s*/, '').trim();
    if (cleanTarget === cleanNode) {
        return true;
    }
    if (cleanTarget && cleanNode) {
        const targetParts = cleanTarget.split(/[-_]/);
        const nodeParts = cleanNode.split(/[-_]/);
        if (targetParts[0] && targetParts[0] === nodeParts[0]) {
            return true;
        }
    }
    return false;
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
    const lastLoadTimeRef = useRef<number>(0);

    const callbacksRef = useRef({
        setSelectedProject,
        setSelectedConversation,
        setActivePromptText,
        setEditedPromptText,
        setShowAllWords,
        setTreeData,
        setIsLoading,
        setIsAutoSyncing,
        setError,
        setExpandedProjects,
        setExpandedStaleGroups,
        setArchivedProjectIds,
        setPinnedProjectIds,
    });
    callbacksRef.current = {
        setSelectedProject,
        setSelectedConversation,
        setActivePromptText,
        setEditedPromptText,
        setShowAllWords,
        setTreeData,
        setIsLoading,
        setIsAutoSyncing,
        setError,
        setExpandedProjects,
        setExpandedStaleGroups,
        setArchivedProjectIds,
        setPinnedProjectIds,
    };

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
            const now = Date.now();
            // Enforce strict 30-second cooldown for background and auto-sync loads
            if (!isForce && !isInitialLoad && now - lastLoadTimeRef.current < TREE_CACHE_TTL_MS) {
                return; // Exit silently; SWR memory cache is fresh
            }
            lastLoadTimeRef.current = now;

            if (isInitialLoad) {
                setIsLoading(true);
                promptTreeLoadingEmitter.set(true);
            } else {
                setIsAutoSyncing(true);
            }
            setError(null);
            try {
                const cacheKey = instanceId || 'default';
                const previousTree = overlaySelectedQueued(
                    promptTreeSwrCache.get(cacheKey)?.data || [],
                    selectedConversationRef.current,
                    selectedProjectRef.current
                );
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

                const withQueued = mergeQueuedTree(finalData, previousTree);
                promptTreeSwrCache.set(cacheKey, { data: withQueued, timestamp: Date.now() });

                setTreeData(withQueued);

                // Retain active selection across loads and background sync cycles
                const currentConv = selectedConversationRef.current;
                const currentProj = selectedProjectRef.current;
                const currentActiveText = activePromptTextRef.current;

                let hasRetainedSelection = false;
                if (currentConv) {
                    const currentId = currentConv.conversation_id;
                    const currentTurn = currentConv.selected_turn_number;
                    const updatedProj = withQueued.find((p) => {
                        const isSameProject = Boolean(currentProj && p.project_id === currentProj.project_id);
                        const hasMatchingConversation = p.conversations.some((c) => c.conversation_id === currentId);
                        return isSameProject || hasMatchingConversation;
                    });
                    if (updatedProj) {
                        setSelectedProject(updatedProj);
                        const rawUpdatedConv = updatedProj.conversations.find(
                            (c) => c.conversation_id === currentId
                        );
                        const updatedConv = rawUpdatedConv
                            ? mergeQueuedTurns(rawUpdatedConv, currentConv)
                            : undefined;
                        if (updatedConv) {
                            hasRetainedSelection = true;
                            const turnConv = applySelectedTurnOverlay(updatedConv, currentTurn);
                            const freshText = resolveConversationPromptText(turnConv, currentTurn ?? null);
                            setSelectedConversation(turnConv);
                            setActivePromptText(freshText);
                            setEditedPromptText((prev) => (prev === currentActiveText ? freshText : prev));
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
                // Immediate SWR render, then force-refresh so enqueue/rerun
                // from this instance is visible on click (no 30s wait).
                setTreeData(cached.data);
                performAutoSelection(cached.data, latestArchived, latestPinned);
                void loadTree(false, true, latestArchived, latestPinned);
            } else {
                loadTree(true, true, latestArchived, latestPinned);
            }
        }
    }, [isOpen, instanceId, initialSelectedProjectId, loadTree, performAutoSelection, setArchivedProjectIds, setPinnedProjectIds, setTreeData]);

    // Strict 30-Second Minimum Auto-Sync Guard: Never poll faster than 30s as per user specification
    useEffect(() => {
        if (!isOpen || syncInterval === 'off') return;

        let intervalMs = TREE_CACHE_TTL_MS;
        if (syncInterval === '1m') intervalMs = 60000;
        else if (syncInterval === '2m') intervalMs = 120000;
        else intervalMs = TREE_CACHE_TTL_MS; // clamp 15s, 30s, or any other value to minimum 30s

        const effectiveInterval = Math.max(TREE_CACHE_TTL_MS, intervalMs);

        const timer = setInterval(() => {
            void loadTree(false, false);
        }, effectiveInterval);

        return () => clearInterval(timer);
    }, [isOpen, syncInterval, instanceId, loadTree]);

    return { selectConversation, loadTree };
}
