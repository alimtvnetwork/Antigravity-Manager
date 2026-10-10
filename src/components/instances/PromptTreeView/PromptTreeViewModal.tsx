import { useEffect, useMemo } from 'react';
import { createPortal } from 'react-dom';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { getTruncatedText, countWords } from './formatters';
import type { PromptTreeViewModalProps } from './types';
import { usePromptTreeState } from './usePromptTreeState';
import { usePromptTreeData } from './usePromptTreeData';
import { usePromptTreeCoreActions } from './usePromptTreeCoreActions';
import { usePromptTreePromptActions } from './usePromptTreePromptActions';
import { ModalHeader } from './ModalHeader';
import { TreePanel } from './TreePanel';
import { DetailHeader } from './DetailHeader';
import { DetailTabs } from './DetailTabs';
import { PromptInstructionView } from './PromptInstructionView';
import { ResultsView } from './ResultsView';
import { DetailEmptyState } from './DetailEmptyState';
import { InspectorModal } from './InspectorModal';
import { DetailsModal } from './DetailsModal';

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
    const state = usePromptTreeState(instanceId);
    const {
        treeData,
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
        selectedProjectScope, setSelectedProjectScope,
        isDetailsModalOpen, setIsDetailsModalOpen,
        copiedField, setCopiedField,
        expandedProjects, setExpandedProjects,
        expandedStaleGroups, setExpandedStaleGroups,
        selectedProject, setSelectedProject,
        selectedConversation,
        activePromptText, setActivePromptText,
        editedPromptText, setEditedPromptText,
        viewMode, setViewMode,
        showAllWords, setShowAllWords,
        previewTab, setPreviewTab,
        isCopiedResults, setIsCopiedResults,
        isCopied, setIsCopied,
        isResending,
        isEnqueueing,
        isExportMenuOpen, setIsExportMenuOpen,
        actionMsg, setActionMsg,
        elapsedSeconds, setElapsedSeconds,
        inspectorPrompt, setInspectorPrompt,
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
    });

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
            list = list.filter((p) => Boolean(p.is_running) || p.conversations.some((c) => Boolean(c.is_running)));
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

        return { activeProjects: active, archivedProjects: archived };
    }, [treeData, searchQuery, selectedProjectScope, activeFilter, archivedProjectIds, pinnedProjectIds]);

    // Active prompt truncation for Preview mode (default preview word threshold: 120 words)
    const { displayText: previewDisplayText, isTruncated, totalWords } = useMemo(() => {
        return getTruncatedText(activePromptText, 120);
    }, [activePromptText]);

    const displayedMarkdown = showAllWords || !isTruncated ? activePromptText : previewDisplayText;
    const activeWordCount = totalWords || countWords(activePromptText);
    const activeByteCount = useMemo(() => {
        return new TextEncoder().encode(activePromptText).length;
    }, [activePromptText]);

    const hasImages = useMemo(() => {
        const text = activePromptText || selectedConversation?.prompt_preview_200w || '';
        return /!\[.*?\]\((?:https?:\/\/.*?|data:image\/.*?;base64,.*?|[^\s)]+)\)/.test(text);
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
            className={`fixed inset-0 z-[300] flex items-center justify-center bg-black/60 backdrop-blur-xs animate-in fade-in duration-200 ${isFullscreen ? "p-0" : "p-4 sm:p-6"}`}
        >
            <div
                className={`flex flex-col overflow-hidden bg-white dark:bg-[#0c2438] shadow-2xl transition-all ${isFullscreen
                    ? "h-full w-full rounded-none border-0"
                    : "h-[88vh] w-full max-w-6xl rounded-2xl border border-slate-200 dark:border-[#15334d]"
                    }`}
            >
                <ModalHeader
                    instanceName={instanceName}
                    sequenceNumber={sequenceNumber}
                    executablePath={executablePath}
                    isFullscreen={isFullscreen}
                    setIsFullscreen={setIsFullscreen}
                    isLoading={isLoading}
                    syncInterval={syncInterval}
                    setSyncInterval={setSyncInterval}
                    restoreFileInputRef={restoreFileInputRef}
                    onClose={onClose}
                    onBackup={coreActions.handleBackup}
                    onRestore={coreActions.handleRestore}
                    onFileRestore={coreActions.handleFileRestore}
                    onRefreshAll={() => loadTree(true, true)}
                />

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
                    />

                    {/* Right Panel: Prompt Details */}
                    <div className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-[#0c2438]">
                        {selectedConversation ? (
                            <div className="flex-1 flex flex-col px-5 py-3.5 overflow-y-auto space-y-3">
                                <DetailHeader
                                    selectedConversation={selectedConversation}
                                    activePromptText={activePromptText}
                                    confirmationSuffix={confirmationSuffix}
                                    setConfirmationSuffix={setConfirmationSuffix}
                                    isCopiedText={false}
                                    setIsCopiedText={() => {}}
                                    isCopiedRaw={false}
                                    isCopied={isCopied}
                                    setIsCopied={setIsCopied}
                                    isEnqueueing={isEnqueueing}
                                    isResending={isResending}
                                    isFocusing={false}
                                    isExportMenuOpen={isExportMenuOpen}
                                    setIsExportMenuOpen={setIsExportMenuOpen}
                                    hasImages={hasImages}
                                    instancePid={instancePid}
                                    instanceSeqNum={instanceSeqNum}
                                    instanceExeName={instanceExeName}
                                    instanceNameDisplay={instanceNameDisplay}
                                    formatDuration={formatDuration}
                                    elapsedSeconds={elapsedSeconds}
                                    onCopyWithImages={promptActions.handleCopyWithImagesRich}
                                    onSaveImages={promptActions.handleSaveImages}
                                    onExport={promptActions.handleExport}
                                    onFocusIde={promptActions.handleFocusIde}
                                    onResend={promptActions.handleResendPrompt}
                                    onEnqueue={promptActions.handleEnqueuePrompt}
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
                                        openInspector={promptActions.openInspector}
                                        selectedProject={selectedProject}
                                        activeByteCount={activeByteCount}
                                        setActivePromptText={setActivePromptText}
                                        setActionMsg={setActionMsg}
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
                                    />
                                )}
                            </div>
                        ) : (
                            <DetailEmptyState
                                selectedProject={selectedProject}
                                isResending={isResending}
                                onResend={promptActions.handleResendPrompt}
                            />
                        )}
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
            />

            {isAutoSyncing && (
                <div className="absolute bottom-4 right-4 flex items-center gap-2 px-3 py-1.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-xs text-blue-600 dark:text-cyan-400">
                    <span className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
                    Syncing...
                </div>
            )}
        </div>
    );

    return typeof document !== 'undefined' ? createPortal(modalContent, document.body) : modalContent;
}
