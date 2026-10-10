import { useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getArchivedProjectsForInstance } from '../../../services/instanceService';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';
import { isGhostConversation, isStaleOrEmptyConversation } from './conversationGuards';
import type { PromptTreeState } from './usePromptTreeState';

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

    // Select conversation directly (2-tier leaf)
    const selectConversation = useCallback(
        (conv: AgmConversationNode, project: AgmProjectTreeNode) => {
            setSelectedProject(project);
            setSelectedConversation(conv);
            const text = conv.full_prompt_text || conv.prompt_preview_200w || '';
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
            isForce: boolean = true,
            overrideArchivedIds?: string[],
            overridePinnedIds?: string[]
        ) => {
            if (isInitialLoad) {
                setIsLoading(true);
            } else {
                setIsAutoSyncing(true);
            }
            setError(null);
            try {
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
                setTreeData(finalData);

                if (isInitialLoad) {
                    const targetArchived = overrideArchivedIds || archivedProjectIds;
                    const targetPinned = overridePinnedIds || pinnedProjectIds;
                    performAutoSelection(finalData, targetArchived, targetPinned);
                } else {
                    // Background sync: match existing selected conversation in place
                    const currentConv = selectedConversationRef.current;
                    const currentProj = selectedProjectRef.current;
                    const currentActiveText = activePromptTextRef.current;

                    if (currentConv && currentProj) {
                        const updatedProj = finalData.find((p) => p.project_id === currentProj.project_id);
                        if (updatedProj) {
                            setSelectedProject(updatedProj);
                            const updatedConv = updatedProj.conversations.find(
                                (c) => c.conversation_id === currentConv.conversation_id
                            );
                            if (updatedConv) {
                                setSelectedConversation(updatedConv);
                                const freshText = updatedConv.full_prompt_text || updatedConv.prompt_preview_200w || '';
                                setActivePromptText(freshText);
                                // Do not clobber user's dirty textarea edits
                                setEditedPromptText((prev) => (prev === currentActiveText ? freshText : prev));
                            }
                        }
                    }
                }
            } catch (err: unknown) {
                if (isInitialLoad) {
                    setError(err instanceof Error ? err.message : String(err) || 'Failed to load project and conversation tree');
                } else {
                    console.warn('Background auto-sync failed:', err);
                }
            } finally {
                if (isInitialLoad) {
                    setIsLoading(false);
                } else {
                    setIsAutoSyncing(false);
                }
            }
        },
        [
            instanceId,
            archivedProjectIds,
            pinnedProjectIds,
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

    // Modal open effect
    useEffect(() => {
        if (isOpen) {
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
            loadTree(true, true, latestArchived, latestPinned);
        }
    }, [isOpen, instanceId, initialSelectedProjectId, loadTree, setArchivedProjectIds, setPinnedProjectIds]);

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
            loadTree(false, true);
        }, safeIntervalMs);

        return () => clearInterval(timer);
    }, [isOpen, syncInterval, instanceId, loadTree]);

    return { selectConversation, loadTree };
}
