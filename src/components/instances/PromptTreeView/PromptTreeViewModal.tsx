import { useState, useEffect, useMemo, useCallback, useRef, Component, type ErrorInfo, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { createPortal } from 'react-dom';
import { AlertTriangle, PanelLeftOpen } from 'lucide-react';
import { useErrorStore } from '../../../stores/error-store';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { useConfigStore } from '../../../stores/useConfigStore';
import { copyToClipboard } from '../../../utils/clipboard';
import { getTruncatedText, countWords, stripImagesFromPrompt } from './formatters';
import type { PromptTreeViewModalProps } from './types';
import { usePromptTreeState } from './usePromptTreeState';
import { usePromptTreeData, invalidatePromptTreeCache } from './usePromptTreeData';
import { usePromptTreeCoreActions } from './usePromptTreeCoreActions';
import { usePromptTreePromptActions, dispatchRerunMessages, type RerunMessageOption } from './usePromptTreePromptActions';
import { isGhostConversation, isStaleOrEmptyConversation, getConversationStatus } from './conversationGuards';
import { classifyPromptTier } from './tierClassification';
import { ModalHeader } from './ModalHeader';
import { TreePanel } from './TreePanel';
import { NowRunningUpNext } from './NowRunningUpNext';
import { useRunningAndUpNext } from './useRunningAndUpNext';
import { sendPromptNow } from '../../../services/instanceService';
import { DetailHeader } from './DetailHeader';
import { DetailTabs } from './DetailTabs';
import { PromptInstructionView } from './PromptInstructionView';
import { ResultsView } from './ResultsView';
import { DetailEmptyState } from './DetailEmptyState';
import { InspectorModal } from './InspectorModal';
import { DetailsModal } from './DetailsModal';
import { PromptTreeExportModal } from './PromptTreeExportModal';
import { PromptTreeImportModal } from './PromptTreeImportModal';
import { ProjectContextMenu } from './ProjectContextMenu';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';

function formatDuration(secs: number): string {
    if (!Number.isFinite(secs) || isNaN(secs) || secs < 0) return '0s';
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    if (m === 0) return `${s}s`;
    return `${m}m ${s < 10 ? '0' : ''}${s}s`;
}

function formatByteSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    return `${(bytes / 1024).toFixed(1)} KB`;
}

interface ErrorBoundaryProps {
    children: ReactNode;
    onReset?: () => void;
}

interface ErrorBoundaryState {
    hasError: boolean;
    error: Error | null;
}

class PromptTreeErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    public override state: ErrorBoundaryState = { hasError: false, error: null };

    public static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error };
    }

    public override componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
        useErrorStore.getState().trackWarning(error, {
            source: 'PromptTreeViewModal.ErrorBoundary',
            triggerAction: 'render_conversation_transcript',
            context: { componentStack: errorInfo.componentStack?.slice(0, 500) ?? null },
        });
    }

    public override render(): ReactNode {
        if (this.state.hasError) {
            return (
                <div className="flex-1 flex flex-col items-center justify-center p-8 text-center space-y-3 bg-white dark:bg-[#171025]">
                    <div className="w-12 h-12 rounded-2xl bg-amber-500/15 text-amber-600 dark:text-amber-400 flex items-center justify-center">
                        <AlertTriangle className="w-6 h-6" />
                    </div>
                    <h4 className="text-sm font-bold text-slate-800 dark:text-slate-100">
                        Conversation Transcript Error
                    </h4>
                    <p className="text-xs text-slate-500 dark:text-slate-400 max-w-md font-mono bg-slate-50 dark:bg-[#211631] p-3 rounded-lg border border-slate-200 dark:border-[#40285e] overflow-x-auto text-left">
                        {this.state.error?.message || 'An unexpected error occurred while parsing this conversation transcript.'}
                    </p>
                    <button
                        type="button"
                        onClick={() => {
                            this.setState({ hasError: false, error: null });
                            this.props.onReset?.();
                        }}
                        className="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-xs transition-colors cursor-pointer"
                    >
                        Reset Details View
                    </button>
                </div>
            );
        }
        return this.props.children;
    }
}

export default function PromptTreeViewModal(props: PromptTreeViewModalProps) {
    const {
        isOpen,
        onClose,
        instanceId,
        instanceName,
        sequenceNumber,
        executablePath,
        initialSelectedProjectId,
    } = props;

    const { instances } = useInstanceStore();
    const { config } = useConfigStore();
    const state = usePromptTreeState(instanceId);
    const [appVersion, setAppVersion] = useState<string>('');
    const [isResetting, setIsResetting] = useState<boolean>(false);

    useEffect(() => {
        let isMounted = true;
        import('@tauri-apps/api/app')
            .then(({ getVersion }) => {
                getVersion().then((v) => {
                    if (isMounted) {
                        setAppVersion(v);
                        const lastSeen = localStorage.getItem('agm_prompt_tree_version');
                        if (lastSeen && lastSeen !== v) {
                            localStorage.setItem('agm_prompt_tree_version', v);
                            invalidatePromptTreeCache(instanceId);
                            void invoke('reset_prompt_tree_cache', { instanceId: instanceId || null });
                        } else if (!lastSeen) {
                            localStorage.setItem('agm_prompt_tree_version', v);
                        }
                    }
                });
            })
            .catch((err: unknown) => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'PromptTreeViewModal.getVersion',
                    triggerAction: 'getVersion',
                });
            });
        return () => {
            isMounted = false;
        };
    }, [instanceId]);
    const {
        treeData,
        setTreeData,
        isLoading,
        isAutoSyncing,
        syncInterval, setSyncInterval,
        error,
        searchQuery, setSearchQuery,
        activeFilter, setActiveFilter,
        categoryFilter, setCategoryFilter,
        restoreFileInputRef,
        isFullscreen, setIsFullscreen,
        pinnedProjectIds,
        archivedProjectIds,
        isArchivedProjectsExpanded, setIsArchivedProjectsExpanded,
        refreshingProjectId,
        confirmationSuffix, setConfirmationSuffix,
        conversationTargetMode, setConversationTargetMode,
        selectedProjectScope, setSelectedProjectScope,
        isDetailsModalOpen, setIsDetailsModalOpen,
        copiedField, setCopiedField,
        expandedProjects, setExpandedProjects,
        expandedStaleGroups, setExpandedStaleGroups,
        selectedProject, setSelectedProject,
        selectedConversation, setSelectedConversation,
        activePromptText, setActivePromptText,
        editedPromptText, setEditedPromptText,
        viewMode, setViewMode,
        showAllWords, setShowAllWords,
        previewTab, setPreviewTab,
        isCopiedResults, setIsCopiedResults,
        isCopied, setIsCopied,
        isCopiedAll, setIsCopiedAll,
        isResending,
        isEnqueueing,
        isCopyMenuOpen, setIsCopyMenuOpen,
        isSendMenuOpen, setIsSendMenuOpen,
        isEnqueueMenuOpen, setIsEnqueueMenuOpen,
        isExportMenuOpen, setIsExportMenuOpen,
        actionMsg, setActionMsg,
        elapsedSeconds, setElapsedSeconds,
        inspectorPrompt, setInspectorPrompt,
        isTreeCollapsed, setIsTreeCollapsed,
        isAutomatedHidden, setIsAutomatedHidden,
    } = state;

    const { selectConversation, loadTree } = usePromptTreeData({
        state,
        instanceId,
        isOpen,
        initialSelectedProjectId,
    });

    const coreActions = usePromptTreeCoreActions({
        state,
        instanceId,
        instanceName,
        loadTree,
    });

    const promptActions = usePromptTreePromptActions({
        state,
        instanceId,
        selectConversation,
        loadTree,
    });

    const [projectContextMenu, setProjectContextMenu] = useState<{
        x: number;
        y: number;
        project: AgmProjectTreeNode;
    } | null>(null);

    const handleProjectContextMenu = (e: React.MouseEvent, repoName: string, project?: AgmProjectTreeNode) => {
        e.preventDefault();
        e.stopPropagation();
        const proj = project || treeData.find((p) => p.repo_name === repoName);
        if (proj) {
            setProjectContextMenu({
                x: e.clientX,
                y: e.clientY,
                project: proj,
            });
        }
    };

    const handleRenameConversation = async (conversationId: string, newTitle: string, instId?: string) => {
        const targetInst = instId || selectedConversation?.instance_id || selectedProject?.instance_id || instanceId || 'default';
        await invoke('rename_conversation', {
            instanceId: targetInst,
            conversationId,
            newTitle,
        });
        setTreeData((prev) =>
            prev.map((p) => ({
                ...p,
                conversations: p.conversations.map((c) =>
                    c.conversation_id === conversationId ? { ...c, title: newTitle } : c
                ),
            }))
        );
        setSelectedConversation((prev) =>
            prev?.conversation_id === conversationId ? { ...prev, title: newTitle } : prev
        );
        setSelectedProject((prev) => {
            if (!prev) return prev;
            return {
                ...prev,
                conversations: prev.conversations.map((c) =>
                    c.conversation_id === conversationId ? { ...c, title: newTitle } : c
                ),
            };
        });
    };

    const handleResendPrompts = () => {
        void promptActions.handleResendPrompt();
    };

    const handleEnqueue = () => {
        void promptActions.handleEnqueuePrompt();
    };

    const handleRerunConversation = (convoId: string, nudgeType: string) => {
        const targetInstId = selectedProject?.instance_id || instanceId || 'default';
        const targetRepoPath = selectedProject?.repo_path || '';
        const msgOption: RerunMessageOption = nudgeType === 'is-it-published' ? 'is-it-published' : 'is-it-done';
        void dispatchRerunMessages({
            targetInstId,
            repoPath: targetRepoPath,
            convId: convoId,
            count: 1,
            messageOption: msgOption,
            source: 'PromptTreeViewModal.onRerunConversation',
        });
    };

    // "Now Running + Up Next" strip (spec 154 §1): data hook + row-action handlers.
    const runningUpNext = useRunningAndUpNext(instanceId, isOpen);

    const findStripConversation = (promptId: string): { conv: AgmConversationNode; proj: AgmProjectTreeNode } | null => {
        for (const proj of treeData) {
            const conv = (proj.conversations || []).find((c) => c.conversation_id === promptId);
            if (conv) return { conv, proj };
        }
        return null;
    };

    const handleStripSelectPrompt = (promptId: string) => {
        const found = findStripConversation(promptId);
        if (!found) {
            useErrorStore.getState().trackWarning(`NowRunningUpNext: prompt ${promptId} not found in tree`, {
                source: 'PromptTreeViewModal.stripSelectPrompt',
                triggerAction: 'select_prompt',
            });
            return;
        }
        selectConversation(found.conv, found.proj);
    };

    const handleStripRerun = (promptId: string) => {
        handleRerunConversation(promptId, 'is-it-done');
    };

    const handleStripResendWithImage = (promptId: string) => {
        const found = findStripConversation(promptId);
        if (!found) {
            useErrorStore.getState().trackWarning(`NowRunningUpNext: prompt ${promptId} not found in tree`, {
                source: 'PromptTreeViewModal.stripResendWithImage',
                triggerAction: 'resend_with_image',
            });
            setActionMsg('Resend failed: prompt not found in tree.');
            return;
        }
        const { conv, proj } = found;
        const text = (conv.full_prompt_text || conv.prompt_preview_200w || '').trim();
        if (!text) {
            setActionMsg('Resend failed: prompt has no text content.');
            return;
        }
        const targetInstId = conv.instance_id || proj.instance_id || instanceId || 'default';
        const repoPath = proj.repo_path || '';
        // Send raw text (no base64→attachment rewrite): embedded data URIs /
        // image refs travel verbatim so the backend captures them into the
        // side-channel image_payload at dispatch time.
        setActionMsg('Resending prompt with images...');
        void sendPromptNow(targetInstId, repoPath, text, conv.conversation_id)
            .then(() => {
                setActionMsg('Prompt resent with images!');
                setTimeout(() => setActionMsg(null), 3000);
            })
            .catch((err: unknown) => {
                // sendPromptNow already routed this into the error store; surface user text.
                const msg = err instanceof Error ? err.message : String(err) || 'dispatch rejected';
                setActionMsg(`Resend failed: ${msg}`);
            });
    };

    const currentInstance = useMemo(() => {
        const targetId = selectedProject?.instance_id || instanceId || 'default';
        return instances.find((i) => i.config.id === targetId);
    }, [instances, selectedProject?.instance_id, instanceId]);

    const instancePid = currentInstance?.pid;

    // Elapsed timer for running conversations
    useEffect(() => {
        if (!selectedConversation?.is_running) {
            setElapsedSeconds(0);
            return;
        }
        let startTime = Date.now();
        if (selectedConversation.last_modified) {
            const raw = selectedConversation.last_modified;
            if (typeof raw === 'number') {
                startTime = raw < 1e11 ? raw * 1000 : raw;
            } else {
                const trimmed = String(raw).trim();
                if (/^\d+$/.test(trimmed)) {
                    const num = Number(trimmed);
                    startTime = num < 1e11 ? num * 1000 : num;
                } else {
                    const parsed = Date.parse(trimmed);
                    if (Number.isFinite(parsed) && parsed > 0) {
                        startTime = parsed;
                    }
                }
            }
        }
        const updateElapsed = () => {
            const now = Date.now();
            const diff = Math.max(0, Math.floor((now - startTime) / 1000));
            setElapsedSeconds(Number.isFinite(diff) ? diff : 0);
        };
        updateElapsed();
        const timer = setInterval(updateElapsed, 1000);
        return () => clearInterval(timer);
    }, [selectedConversation?.conversation_id, selectedConversation?.is_running, selectedConversation?.last_modified, setElapsedSeconds]);

    // Deep-link auto-expansion & active running conversation selection for initialSelectedProjectId
    const lastHandledInitialProjectIdRef = useRef<string | null>(null);
    useEffect(() => {
        if (!isOpen) {
            lastHandledInitialProjectIdRef.current = null;
            return;
        }
        if (!initialSelectedProjectId || treeData.length === 0) return;
        if (lastHandledInitialProjectIdRef.current === initialSelectedProjectId) return;

        const target = treeData.find((p) => p.project_id === initialSelectedProjectId);
        if (!target) return;

        lastHandledInitialProjectIdRef.current = initialSelectedProjectId;
        setExpandedProjects((prev) => ({ ...prev, [target.project_id]: true }));
        setSelectedProject(target);

        if (target.conversations && target.conversations.length > 0) {
            // Find genuine active running conversation first
            const runningConv = target.conversations.find(
                (c) =>
                    Boolean(c.is_running) &&
                    !isGhostConversation(c) &&
                    !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            if (runningConv) {
                selectConversation(runningConv, target);
                if (isStaleOrEmptyConversation(runningConv)) {
                    setExpandedStaleGroups((prev) => ({ ...prev, [target.project_id]: true }));
                }
                return;
            }

            // Fallback to latest valid conversation
            const validConvs = target.conversations.filter(
                (c) =>
                    !isGhostConversation(c) &&
                    !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            if (validConvs.length > 0) {
                const sorted = [...validConvs].sort((a, b) => {
                    const aTime = new Date(a.last_modified).getTime() || 0;
                    const bTime = new Date(b.last_modified).getTime() || 0;
                    return bTime - aTime;
                });
                selectConversation(sorted[0], target);
                if (isStaleOrEmptyConversation(sorted[0])) {
                    setExpandedStaleGroups((prev) => ({ ...prev, [target.project_id]: true }));
                }
            } else {
                setSelectedConversation(null);
            }
        } else {
            setSelectedConversation(null);
        }
    }, [
        isOpen,
        initialSelectedProjectId,
        treeData,
        selectConversation,
        setExpandedProjects,
        setSelectedProject,
        setSelectedConversation,
        setExpandedStaleGroups,
    ]);

    // Synchronize selected project and first conversation when project scope dropdown changes
    useEffect(() => {
        if (!isOpen || selectedProjectScope === 'all' || treeData.length === 0) return;
        const target = treeData.find((p) => p.project_id === selectedProjectScope);
        if (!target) return;
        setSelectedProject(target);
        setExpandedProjects((prev) => ({ ...prev, [target.project_id]: true }));

        // If currently selected conversation already belongs to this target project, keep it
        const hasCurrentInProject = selectedConversation && target.conversations.some((c) => c.conversation_id === selectedConversation.conversation_id);
        if (hasCurrentInProject) return;

        if (target.conversations && target.conversations.length > 0) {
            const validConvs = target.conversations.filter(
                (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            const pool = validConvs.length > 0 ? validConvs : target.conversations;
            const sorted = [...pool].sort((a, b) => {
                const aRunning = Boolean(a.is_running);
                const bRunning = Boolean(b.is_running);
                if (aRunning !== bRunning) return aRunning ? -1 : 1;
                const aTime = new Date(a.last_modified).getTime() || 0;
                const bTime = new Date(b.last_modified).getTime() || 0;
                return bTime - aTime;
            });
            if (sorted[0]) {
                selectConversation(sorted[0], target);
                if (isStaleOrEmptyConversation(sorted[0])) {
                    setExpandedStaleGroups((prev) => ({ ...prev, [target.project_id]: true }));
                }
            }
        }
    }, [isOpen, selectedProjectScope, treeData, selectedConversation, selectConversation, setSelectedProject, setExpandedProjects, setExpandedStaleGroups]);

    // Hotkey: 'N' triggers Send Now
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.ctrlKey || e.altKey || e.metaKey) return;

            const target = e.target as HTMLElement | null;
            if (target) {
                const tagName = target.tagName?.toLowerCase();
                if (
                    tagName === 'input' ||
                    tagName === 'textarea' ||
                    tagName === 'select' ||
                    target.isContentEditable ||
                    target.getAttribute('contenteditable') === 'true'
                ) {
                    return;
                }
            }

            if (e.key === 'n' || e.key === 'N') {
                e.preventDefault();
                promptActions.handleResendPrompt();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, promptActions]);

    const archivedCount = useMemo(() => {
        return treeData.filter((p) => archivedProjectIds.includes(p.project_id)).length;
    }, [treeData, archivedProjectIds]);

    const isAllExpanded = useMemo(() => {
        return treeData.length > 0 && treeData.every((p) => expandedProjects[p.project_id]);
    }, [treeData, expandedProjects]);

    const handleToggleExpandAll = useCallback(() => {
        if (isAllExpanded) {
            setExpandedProjects({});
        } else {
            const next: Record<string, boolean> = {};
            for (const p of treeData) {
                next[p.project_id] = true;
            }
            setExpandedProjects(next);
        }
    }, [isAllExpanded, treeData, setExpandedProjects]);

    // Search filter & multi-tier sorting across projects, partitioned into active & archived
    const { activeProjects, archivedProjects } = useMemo(() => {
        let list = [...treeData];

        // 1. Search filter
        if (searchQuery.trim()) {
            const q = searchQuery.toLowerCase();
            list = list.filter(
                (p) =>
                    p.repo_name.toLowerCase().includes(q) ||
                    p.repo_path.toLowerCase().includes(q) ||
                    p.conversations.some(
                        (c) =>
                            c.title.toLowerCase().includes(q) ||
                            c.conversation_id.toLowerCase().includes(q) ||
                            c.prompt_preview_200w.toLowerCase().includes(q)
                    )
            );
        }

        // 1.5 Project Scope Filter
        if (selectedProjectScope !== 'all') {
            list = list.filter((p) => p.project_id === selectedProjectScope);
        }

        // 2. Filter Pills
        if (activeFilter === 'running') {
            list = list
                .filter(
                    (p) =>
                        Boolean(p.is_running) ||
                        p.conversations.some(
                            (c) =>
                                Boolean(c.is_running) &&
                                !isGhostConversation(c)
                        )
                )
                .map((p) => {
                    const hasRunningSubagents = p.conversations.some((c) => {
                        if (!c.is_running) return false;
                        const cat = classifyPromptTier(c.prompt_preview_200w, c.title, c);
                        return cat.tier === 'SUBAGENT_INSTRUCTION';
                    });

                    if (!hasRunningSubagents) {
                        return p;
                    }

                    // Preserve parent user prompts for running subagents so that ProjectNode filter
                    // does not drop the parent or orphan active subagents
                    const convs = [...p.conversations];
                    const chronologicalIndices = convs
                        .map((c, idx) => ({ c, idx, time: new Date(c.last_modified).getTime() || 0 }))
                        .sort((a, b) => a.time - b.time);

                    let currentParentIdx: number | null = null;
                    const parentIndicesWithRunningSubagents = new Set<number>();

                    for (const { c, idx } of chronologicalIndices) {
                        const cat = classifyPromptTier(c.prompt_preview_200w, c.title, c);
                        if (cat.tier === 'USER_PROMPT') {
                            currentParentIdx = idx;
                        } else if (cat.tier === 'SUBAGENT_INSTRUCTION' && Boolean(c.is_running) && !isGhostConversation(c)) {
                            if (currentParentIdx !== null) {
                                parentIndicesWithRunningSubagents.add(currentParentIdx);
                            }
                        }
                    }

                    if (parentIndicesWithRunningSubagents.size === 0) {
                        return p;
                    }

                    const updatedConvs = convs.map((c, idx) => {
                        if (parentIndicesWithRunningSubagents.has(idx) && !c.is_running) {
                            return { ...c, is_running: true };
                        }
                        return c;
                    });

                    return { ...p, conversations: updatedConvs };
                });
        } else if (activeFilter === 'queued') {
            list = list.filter((p) => p.conversations.some((c) => Boolean(c.is_queued) || c.status.toLowerCase().includes('queue')));
        } else if (activeFilter === 'pinned') {
            list = list.filter((p) => pinnedProjectIds.includes(p.project_id));
        } else if (activeFilter === 'latest_conv') {
            list = list.filter((p) => p.conversations.length > 0);
        } else if (activeFilter === 'latest_prompt') {
            list = list.filter((p) => p.conversations.some((c) => c.step_count > 0 || Boolean(c.prompt_preview_200w)));
        } else if (activeFilter === 'archived') {
            list = list.filter((p) => archivedProjectIds.includes(p.project_id));
        }

        // Partition into active & archived
        const archived: typeof list = [];
        const active: typeof list = [];
        for (const p of list) {
            if (archivedProjectIds.includes(p.project_id)) {
                archived.push(p);
            } else {
                active.push(p);
            }
        }

        // Elevate running projects to top of active list
        active.sort((a, b) => {
            const aPinned = pinnedProjectIds.includes(a.project_id);
            const bPinned = pinnedProjectIds.includes(b.project_id);
            if (aPinned !== bPinned) return aPinned ? -1 : 1;

            const aRunning =
                Boolean(a.is_running) ||
                a.conversations.some((c) => getConversationStatus(c).isRunning);
            const bRunning =
                Boolean(b.is_running) ||
                b.conversations.some((c) => getConversationStatus(c).isRunning);
            if (aRunning !== bRunning) return aRunning ? -1 : 1;

            const aQueued = a.conversations.some((c) => getConversationStatus(c).isQueued);
            const bQueued = b.conversations.some((c) => getConversationStatus(c).isQueued);
            if (aQueued !== bQueued) return aQueued ? -1 : 1;

            const aLatest = Math.max(0, ...a.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
            const bLatest = Math.max(0, ...b.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
            if (bLatest !== aLatest) return bLatest - aLatest;

            return a.repo_name.localeCompare(b.repo_name);
        });

        return { activeProjects: active, archivedProjects: archived };
    }, [treeData, searchQuery, selectedProjectScope, activeFilter, archivedProjectIds, pinnedProjectIds]);

    // Active prompt truncation for Preview mode (configured preview word threshold: default 200 words)
    const promptPreviewWords = config?.prompt_preview_words ?? 200;
    const { displayText: previewDisplayText, isTruncated, totalWords } = useMemo(() => {
        return getTruncatedText(activePromptText, promptPreviewWords);
    }, [activePromptText, promptPreviewWords]);

    const displayedMarkdown = showAllWords || !isTruncated ? activePromptText : previewDisplayText;
    const activeWordCount = totalWords || countWords(activePromptText);
    const activeByteCount = useMemo(() => {
        return new TextEncoder().encode(activePromptText).length;
    }, [activePromptText]);

    const hasImages = useMemo(() => {
        if (selectedConversation?.media_paths && selectedConversation.media_paths.length > 0) {
            return true;
        }
        if (selectedConversation?.prompts && selectedConversation.prompts.some((p) => p.media_paths && p.media_paths.length > 0)) {
            return true;
        }
        if (selectedConversation?.sub_runs && selectedConversation.sub_runs.some((s) => s.media_paths && s.media_paths.length > 0)) {
            return true;
        }
        const text = activePromptText || selectedConversation?.prompt_preview_200w || '';
        return /!\[.*?\]\((?:https?:\/\/.*?|data:image\/.*?;base64,.*?|[^\s)]+)\)/.test(text) ||
            /<img\s+[^>]*?src=["'].*?["']/i.test(text) ||
            /data:image\/[a-zA-Z0-9.+_-]+;base64,/.test(text);
    }, [activePromptText, selectedConversation]);

    const instanceSeqNum = selectedConversation?.instance_seq_num
        ?? selectedProject?.instance_seq_num
        ?? 1;

    const instanceExeName = selectedConversation?.instance_exe_name
        ?? selectedProject?.instance_exe_name
        ?? 'Antigravity.exe';

    const instanceNameDisplay = selectedConversation?.instance_name
        ?? selectedProject?.instance_name
        ?? instanceName
        ?? 'Default';

    if (!isOpen) return null;

    const modalContent = (
        <div
            onClick={(e) => {
                if (e.target === e.currentTarget) {
                    onClose();
                }
            }}
            onDoubleClick={(e) => {
                if (e.target === e.currentTarget) {
                    onClose();
                }
            }}
            className={`fixed inset-0 z-[99999] flex items-center justify-center bg-black/75 backdrop-blur-md animate-in fade-in duration-200 ${isFullscreen ? "p-0" : "p-4 sm:p-6"}`}
        >
            {/* Draggable Top Region */}
            <div data-tauri-drag-region className="fixed top-0 left-0 right-0 h-8 z-[100000] pointer-events-none" />

            <div
                onClick={(e) => e.stopPropagation()}
                onDoubleClick={(e) => e.stopPropagation()}
                className={`flex flex-col overflow-hidden bg-white dark:bg-[#0c0817]/95 backdrop-blur-2xl shadow-[0_0_50px_rgba(0,0,0,0.8)] border border-slate-200 dark:border-purple-900/40 transition-all relative z-10 ${isFullscreen
                    ? "h-full w-full rounded-none border-0"
                    : "h-[90vh] w-full max-w-7xl rounded-2xl"
                    }`}
            >
                <ModalHeader
                    instanceName={instanceName}
                    appVersion={appVersion}
                    sequenceNumber={sequenceNumber}
                    executablePath={executablePath}
                    isFullscreen={isFullscreen}
                    setIsFullscreen={setIsFullscreen}
                    isLoading={isLoading}
                    isTreeCollapsed={isTreeCollapsed}
                    onToggleTree={() => setIsTreeCollapsed((prev) => !prev)}
                    syncInterval={syncInterval}
                    setSyncInterval={setSyncInterval}
                    restoreFileInputRef={restoreFileInputRef}
                    onClose={onClose}
                    onBackup={coreActions.handleBackup}
                    onRestore={coreActions.handleRestore}
                    onExportSqlite={() => coreActions.handleOpenExportModal({ target_type: 'full_tree' }, 'sqlite')}
                    onImportSqlite={coreActions.handleOpenImportModal}
                    onCopyAllTreePrompts={coreActions.handleCopyAllTreePrompts}
                    onFileRestore={coreActions.handleFileRestore}
                    isAutoSyncing={isAutoSyncing}
                    onRefreshAll={async () => {
                        setActionMsg('Refreshing prompt tree...');
                        try {
                            await loadTree(false, true);
                            setActionMsg('Prompt tree refreshed');
                        } catch (err) {
                            useErrorStore.getState().trackWarning(err, {
                                source: 'PromptTreeViewModal.onRefreshAll',
                                triggerAction: 'refresh_prompt_tree',
                            });
                            setActionMsg('Prompt tree refresh failed');
                        } finally {
                            setTimeout(() => {
                                setActionMsg(null);
                            }, 2500);
                        }
                    }}
                    isResetting={isResetting}
                    onResetTree={async () => {
                        setIsResetting(true);
                        setActionMsg('Wiping database & regenerating prompt tree from disk transcripts...');
                        try {
                            invalidatePromptTreeCache();
                            setSearchQuery('');
                            setActiveFilter('all');
                            setCategoryFilter('all');
                            await invoke('wipe_and_rescan_repo_db', { instanceId: instanceId || null });
                            await loadTree(true, true);
                            setActionMsg('Prompt tree wiped & regenerated successfully');
                        } catch (err) {
                            useErrorStore.getState().captureException(
                                err instanceof Error ? err : new Error(String(err)),
                                { source: 'PromptTreeViewModal.onResetTree', triggerAction: 'wipe_and_rescan_repo_db' }
                            );
                            setActionMsg('Failed to reset & regenerate prompt tree');
                        } finally {
                            setIsResetting(false);
                            setTimeout(() => {
                                setActionMsg(null);
                            }, 3000);
                        }
                    }}
                />

                {/* Top Indeterminate Progress Bar */}
                {(isLoading || isAutoSyncing || isResetting) && (
                    <div className="w-full h-0.5 bg-purple-950/40 dark:bg-[#40285e]/40 overflow-hidden shrink-0">
                        <div className="h-full bg-[#1af18d] shadow-[0_0_8px_#1af18d] animate-pulse w-full" />
                    </div>
                )}

                {/* Notification Banner */}
                {actionMsg && (
                    <div className="bg-emerald-500/10 border-b border-emerald-500/20 px-6 py-2 text-xs font-medium text-emerald-700 dark:text-emerald-300 flex items-center gap-2 shrink-0">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                        {actionMsg}
                    </div>
                )}
                {error && (
                    <div className="bg-rose-500/10 border-b border-rose-500/20 px-6 py-2 text-xs font-medium text-rose-700 dark:text-rose-300 shrink-0">
                        {error}
                    </div>
                )}

                {/* Main Split View */}
                <div className="flex flex-1 overflow-hidden">
                    {!isTreeCollapsed ? (
                        <div className="w-[420px] sm:w-[450px] lg:w-[480px] shrink-0 flex flex-col border-r border-slate-200 dark:border-[#40285e] bg-slate-50/50 dark:bg-[#090611]/60 overflow-hidden">
                            <NowRunningUpNext
                                items={runningUpNext.items}
                                isLoading={runningUpNext.isLoading}
                                lastSyncAt={runningUpNext.lastSyncAt}
                                hasError={runningUpNext.hasError}
                                upNextTotal={runningUpNext.upNextTotal}
                                onSelectPrompt={handleStripSelectPrompt}
                                onRerun={handleStripRerun}
                                onResendWithImage={handleStripResendWithImage}
                                onRefresh={runningUpNext.onRefresh}
                            />
                            <div className="flex-1 min-h-0 flex flex-col [&>div]:h-full [&>div]:border-r-0">
                                <TreePanel
                            treeData={treeData}
                            activeProjects={activeProjects}
                            archivedProjects={archivedProjects}
                            archivedCount={archivedCount}
                            searchQuery={searchQuery}
                            setSearchQuery={setSearchQuery}
                            activeFilter={activeFilter}
                            setActiveFilter={setActiveFilter}
                            categoryFilter={categoryFilter}
                            setCategoryFilter={setCategoryFilter}
                            selectedProjectScope={selectedProjectScope}
                            setSelectedProjectScope={setSelectedProjectScope}
                            isArchivedProjectsExpanded={isArchivedProjectsExpanded}
                            setIsArchivedProjectsExpanded={setIsArchivedProjectsExpanded}
                            isAutomatedHidden={isAutomatedHidden}
                            setIsAutomatedHidden={setIsAutomatedHidden}
                            isCollapsed={isTreeCollapsed}
                            onToggleCollapse={() => setIsTreeCollapsed(true)}
                            onToggleExpandAll={handleToggleExpandAll}
                            isAllExpanded={isAllExpanded}
                            instanceName={instanceName}
                            instanceId={instanceId}
                            expandedProjects={expandedProjects}
                            expandedStaleGroups={expandedStaleGroups}
                            pinnedProjectIds={pinnedProjectIds}
                            archivedProjectIds={archivedProjectIds}
                            refreshingProjectId={refreshingProjectId}
                            selectedProjectId={selectedProject?.project_id}
                            selectedConversationId={selectedConversation?.conversation_id}
                            onSelectProject={(p) => {
                                setSelectedProject(p);
                                setShowAllWords(false);
                            }}
                            onToggleProjectExpand={(pid) => setExpandedProjects((prev) => ({ ...prev, [pid]: !prev[pid] }))}
                            onToggleStaleGroup={(pid) => setExpandedStaleGroups((prev) => ({ ...prev, [pid]: !prev[pid] }))}
                            onSelectConversation={selectConversation}
                            onOpenInspector={promptActions.openInspector}
                            onRefreshProject={coreActions.handleRefreshSingleProject}
                            onTogglePin={coreActions.togglePinProject}
                            onToggleArchive={coreActions.toggleArchiveProject}
                            onResendQueued={coreActions.handleResendQueued}
                            isResendingQueued={coreActions.isResendingQueued}
                            onProjectContextMenu={handleProjectContextMenu}
                            onRenameConversation={handleRenameConversation}
                            onResendPrompts={handleResendPrompts}
                            onEnqueue={handleEnqueue}
                            onRerunConversation={handleRerunConversation}
                                />
                            </div>
                        </div>
                    ) : (
                        <div className="w-10 shrink-0 border-r border-slate-200 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#090611]/70 flex flex-col items-center py-3 gap-3">
                            <button
                                type="button"
                                onClick={() => setIsTreeCollapsed(false)}
                                className="p-1.5 rounded-lg text-slate-500 hover:text-blue-500 dark:text-slate-400 dark:hover:text-blue-400 hover:bg-slate-200/60 dark:hover:bg-[#201538] transition-colors cursor-pointer"
                                title="Expand Tree Panel"
                            >
                                <PanelLeftOpen className="w-4 h-4" />
                            </button>
                            <span
                                className="text-[10px] font-semibold text-slate-400 dark:text-slate-500 tracking-wider uppercase cursor-pointer select-none [writing-mode:vertical-rl] rotate-180 hover:text-slate-700 dark:hover:text-slate-300 transition-colors"
                                onClick={() => setIsTreeCollapsed(false)}
                            >
                                Show Tree
                            </span>
                        </div>
                    )}

                    {/* Right Panel: Prompt Details */}
                    <div className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-[#171025]">
                        <PromptTreeErrorBoundary onReset={() => setSelectedConversation(null)}>
                            {selectedConversation ? (
                                <div className="flex-1 flex flex-col px-5 py-3.5 overflow-y-auto space-y-1.5">
                                    <DetailHeader
                                        previewTab={previewTab}
                                        setPreviewTab={setPreviewTab}
                                        showAllWords={showAllWords}
                                        setShowAllWords={setShowAllWords}
                                        isTruncated={isTruncated}
                                        setIsDetailsModalOpen={setIsDetailsModalOpen}
                                        selectedConversation={selectedConversation}
                                        activePromptText={activePromptText}
                                        confirmationSuffix={confirmationSuffix}
                                        setConfirmationSuffix={setConfirmationSuffix}
                                        conversationTargetMode={conversationTargetMode}
                                        setConversationTargetMode={setConversationTargetMode}
                                        isCopiedText={false}
                                        setIsCopiedText={() => {}}
                                        isCopiedRaw={false}
                                        isCopied={isCopied}
                                        setIsCopied={setIsCopied}
                                        isCopiedAll={isCopiedAll}
                                        setIsCopiedAll={setIsCopiedAll}
                                        isEnqueueing={isEnqueueing}
                                        isResending={isResending}
                                        isFocusing={false}
                                        isCopyMenuOpen={isCopyMenuOpen}
                                        setIsCopyMenuOpen={setIsCopyMenuOpen}
                                        isSendMenuOpen={isSendMenuOpen}
                                        setIsSendMenuOpen={setIsSendMenuOpen}
                                        isEnqueueMenuOpen={isEnqueueMenuOpen}
                                        setIsEnqueueMenuOpen={setIsEnqueueMenuOpen}
                                        isExportMenuOpen={isExportMenuOpen}
                                        setIsExportMenuOpen={setIsExportMenuOpen}
                                        hasImages={hasImages}
                                        instancePid={instancePid}
                                        instanceSeqNum={instanceSeqNum}
                                        instanceExeName={instanceExeName}
                                        instanceNameDisplay={instanceNameDisplay}
                                        formatDuration={formatDuration}
                                        elapsedSeconds={elapsedSeconds}
                                        onCopyAllWithRichImages={promptActions.handleCopyAllWithRichImages}
                                        onCopyCleanText={() => {
                                            const cleanText = stripImagesFromPrompt(activePromptText);
                                            copyToClipboard(cleanText);
                                        }}
                                        onCopyWithImages={promptActions.handleCopyWithImagesRich}
                                        onSaveImages={promptActions.handleSaveImages}
                                        onExport={promptActions.handleExport}
                                        onFocusIde={promptActions.handleFocusIde}
                                        onResend={promptActions.handleResendPrompt}
                                        onEnqueue={promptActions.handleEnqueuePrompt}
                                        rerunRepoPath={selectedProject?.repo_path || ''}
                                        onRenameConversation={handleRenameConversation}
                                        onOpenInspector={(conv) => promptActions.openInspector(conv, selectedProject?.repo_path || '')}
                                    />

                                    <DetailTabs
                                        previewTab={previewTab}
                                        setPreviewTab={setPreviewTab}
                                        selectedConversation={selectedConversation}
                                        activeWordCount={activeWordCount}
                                        activeByteCount={activeByteCount}
                                        formatByteSize={formatByteSize}
                                        isTruncated={isTruncated}
                                        activePromptText={activePromptText}
                                        showAllWords={showAllWords}
                                        setShowAllWords={setShowAllWords}
                                        viewMode={viewMode}
                                        setViewMode={setViewMode}
                                        setIsDetailsModalOpen={setIsDetailsModalOpen}
                                    />

                                    {previewTab === 'instruction' ? (
                                        <PromptInstructionView
                                            selectedConversation={selectedConversation}
                                            previewTab={previewTab}
                                            viewMode={viewMode}
                                            activePromptText={activePromptText}
                                            editedPromptText={editedPromptText}
                                            setEditedPromptText={setEditedPromptText}
                                            displayedMarkdown={displayedMarkdown}
                                            showAllWords={showAllWords}
                                            setShowAllWords={setShowAllWords}
                                            isTruncated={isTruncated}
                                            totalWords={totalWords}
                                            activeWordCount={activeWordCount}
                                            formatByteSize={formatByteSize}
                                            isCopied={isCopied}
                                            setIsCopied={setIsCopied}
                                            isCopiedAll={isCopiedAll}
                                            onCopyAllWithRichImages={promptActions.handleCopyAllWithRichImages}
                                            openInspector={promptActions.openInspector}
                                            selectedProject={selectedProject}
                                            activeByteCount={activeByteCount}
                                            setActivePromptText={setActivePromptText}
                                            setActionMsg={setActionMsg}
                                            promptPreviewWords={promptPreviewWords}
                                        />
                                    ) : (
                                        <ResultsView
                                            selectedConversation={selectedConversation}
                                            isCopiedResults={isCopiedResults}
                                            setIsCopiedResults={setIsCopiedResults}
                                            isFocusing={false}
                                            instancePid={instancePid}
                                            formatDuration={formatDuration}
                                            onFocusIde={promptActions.handleFocusIde}
                                            elapsedSeconds={elapsedSeconds}
                                            showAllWords={showAllWords}
                                        />
                                    )}
                                </div>
                            ) : selectedConversation === null && treeData.length === 0 ? (
                                <div className="flex-1 flex flex-col px-6 py-5 space-y-4 animate-pulse overflow-hidden bg-white dark:bg-[#171025]">
                                    {/* Skeleton Header */}
                                    <div className="space-y-3 pb-3 border-b border-slate-200/70 dark:border-[#40285e]/60">
                                        <div className="flex items-center justify-between gap-3">
                                            <div className="flex items-center gap-2 flex-1">
                                                <div className="h-5 w-16 bg-blue-500/15 rounded-[5px]" />
                                                <div className="h-5 w-24 bg-slate-200 dark:bg-[#211631] rounded-md" />
                                                <div className="h-5 w-44 bg-slate-200 dark:bg-[#211631] rounded-md" />
                                            </div>
                                            <div className="h-7 w-28 bg-slate-200 dark:bg-[#211631] rounded-md" />
                                        </div>
                                    </div>
                                    {/* Skeleton Tabs */}
                                    <div className="flex items-center gap-2">
                                        <div className="h-6 w-24 bg-slate-200 dark:bg-[#211631] rounded-md" />
                                        <div className="h-6 w-20 bg-slate-100 dark:bg-[#211631]/60 rounded-md" />
                                        <div className="h-6 w-16 bg-slate-100 dark:bg-[#211631]/60 rounded-md" />
                                    </div>
                                    {/* Skeleton Content Area */}
                                    <div className="flex-1 rounded-xl bg-slate-50/80 dark:bg-[#211631]/40 border border-slate-200/60 dark:border-[#40285e]/40 p-5 space-y-3">
                                        <div className="h-4 w-3/4 bg-slate-200/80 dark:bg-[#2a1c3f] rounded" />
                                        <div className="h-4 w-full bg-slate-200/60 dark:bg-[#2a1c3f]/80 rounded" />
                                        <div className="h-4 w-5/6 bg-slate-200/60 dark:bg-[#2a1c3f]/80 rounded" />
                                        <div className="h-4 w-2/3 bg-slate-200/40 dark:bg-[#2a1c3f]/60 rounded" />
                                        <div className="pt-4 space-y-2">
                                            <div className="h-24 w-full bg-slate-200/30 dark:bg-[#2a1c3f]/40 rounded-lg" />
                                        </div>
                                    </div>
                                    {/* Skeleton Footer */}
                                    <div className="flex items-center justify-between pt-1">
                                        <div className="h-3 w-32 bg-slate-200/60 dark:bg-[#211631] rounded" />
                                        <div className="h-3 w-20 bg-slate-200/60 dark:bg-[#211631] rounded" />
                                    </div>
                                </div>
                            ) : (
                                <DetailEmptyState
                                    selectedProject={selectedProject}
                                    isResending={isResending}
                                    onResend={promptActions.handleResendPrompt}
                                />
                            )}
                        </PromptTreeErrorBoundary>
                    </div>
                </div>
            </div>

            {/* Full-Screen Prompt Inspector Modal */}
            {inspectorPrompt && (
                <InspectorModal
                    inspectorPrompt={inspectorPrompt}
                    showAllWords={showAllWords}
                    setShowAllWords={setShowAllWords}
                    isCopied={isCopied}
                    setIsCopied={setIsCopied}
                    isCopiedRaw={false}
                    onClose={() => setInspectorPrompt(null)}
                    onCopyWithImages={promptActions.handleCopyWithImagesRich}
                />
            )}

            {/* Conversation & Project Details Modal */}
            <DetailsModal
                isOpen={isDetailsModalOpen}
                onClose={() => setIsDetailsModalOpen(false)}
                selectedConversation={selectedConversation}
                selectedProject={selectedProject}
                copiedField={copiedField}
                setCopiedField={setCopiedField}
                instanceId={instanceId}
                instanceExeName={instanceExeName}
                instancePid={instancePid}
                instanceSeqNum={instanceSeqNum}
                instanceNameDisplay={instanceNameDisplay}
                executablePath={executablePath}
            />

            {/* Project Right-Click Context Menu */}
            {projectContextMenu && (
                <ProjectContextMenu
                    x={projectContextMenu.x}
                    y={projectContextMenu.y}
                    projectId={projectContextMenu.project.project_id}
                    projectName={projectContextMenu.project.repo_name}
                    isPinned={pinnedProjectIds.includes(projectContextMenu.project.project_id)}
                    isArchived={archivedProjectIds.includes(projectContextMenu.project.project_id)}
                    onClose={() => setProjectContextMenu(null)}
                    onRefresh={() => {
                        const fakeEvent = { stopPropagation: () => {} } as React.MouseEvent;
                        void coreActions.handleRefreshSingleProject(projectContextMenu.project.project_id, fakeEvent);
                    }}
                    onTogglePin={() => {
                        const fakeEvent = { stopPropagation: () => {} } as React.MouseEvent;
                        coreActions.togglePinProject(projectContextMenu.project.project_id, fakeEvent);
                    }}
                    onToggleArchive={() => {
                        const fakeEvent = { stopPropagation: () => {} } as React.MouseEvent;
                        coreActions.toggleArchiveProject(projectContextMenu.project.project_id, fakeEvent);
                    }}
                    onExportJson={() => {
                        coreActions.handleOpenExportModal({
                            target_type: 'project',
                            project_id: projectContextMenu.project.project_id,
                            project_name: projectContextMenu.project.repo_name,
                        }, 'json');
                    }}
                    onExportSqlite={() => {
                        coreActions.handleOpenExportModal({
                            target_type: 'project',
                            project_id: projectContextMenu.project.project_id,
                            project_name: projectContextMenu.project.repo_name,
                        }, 'sqlite');
                    }}
                    onExportYaml={() => {
                        coreActions.handleOpenExportModal({
                            target_type: 'project',
                            project_id: projectContextMenu.project.project_id,
                            project_name: projectContextMenu.project.repo_name,
                        }, 'yaml');
                    }}
                    onImportProject={() => coreActions.handleOpenImportModal()}
                    onRerunProject={() => void coreActions.handleResendQueued()}
                />
            )}

            {/* Scoped Section Export Modal */}
            <PromptTreeExportModal
                isOpen={coreActions.isExportModalOpen}
                onClose={coreActions.handleCloseExportModal}
                preselectedScope={coreActions.exportScopeTarget}
                initialFormat={coreActions.exportInitialFormat}
                treeData={treeData}
                selectedProject={selectedProject || undefined}
                selectedConversation={selectedConversation || undefined}
                onExecuteExport={coreActions.executeSectionExport}
            />

            {/* Scoped Section Import Modal */}
            <PromptTreeImportModal
                isOpen={coreActions.isImportModalOpen}
                onClose={coreActions.handleCloseImportModal}
                instanceId={instanceId || 'default'}
                instanceName={instanceName || 'Default Instance'}
                onExecuteImport={coreActions.executeSectionImport}
                onImportComplete={() => {
                    void loadTree(false, true);
                }}
            />

            {isAutoSyncing && (
                <div className="absolute bottom-4 right-4 flex items-center gap-2 px-3 py-1.5 rounded-md bg-blue-500/10 border border-blue-500/20 text-xs text-blue-600 dark:text-cyan-400">
                    <span className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
                    Syncing...
                </div>
            )}
        </div>
    );

    return typeof document !== 'undefined' ? createPortal(modalContent, document.body) : modalContent;
}
