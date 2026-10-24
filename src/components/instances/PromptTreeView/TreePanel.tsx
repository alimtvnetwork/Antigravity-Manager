import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import {
    Folder,
    FolderPlus,
    FolderMinus,
    Search,
    ChevronRight,
    ChevronDown,
    Archive,
    User,
    Bot,
    Terminal,
    Pin,
    RotateCcw,
    Eye,
    EyeOff,
    PanelLeftClose,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';
import { ProjectNode, type ProjectNodeProps } from './ProjectNode';
import { PromptTreeContextMenu, type PromptTreeContextMenuProps } from './PromptTreeContextMenu';
import { promptTreeLoadingEmitter, imageFilterEmitter } from './usePromptTreeData';

export type ActiveFilter = 'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived';
export type CategoryFilter = 'all' | 'user' | 'subagent' | 'system';

export interface TreePanelProps {
    isLoading?: boolean;
    isImageFilterActive?: boolean;
    treeData: AgmProjectTreeNode[];
    activeProjects: AgmProjectTreeNode[];
    archivedProjects: AgmProjectTreeNode[];
    archivedCount: number;
    searchQuery: string;
    setSearchQuery: (v: string) => void;
    activeFilter: ActiveFilter;
    setActiveFilter: (v: ActiveFilter) => void;
    categoryFilter: CategoryFilter;
    setCategoryFilter: (v: CategoryFilter) => void;
    selectedProjectScope: string;
    setSelectedProjectScope: (v: string) => void;
    isArchivedProjectsExpanded: boolean;
    setIsArchivedProjectsExpanded: (v: boolean | ((prev: boolean) => boolean)) => void;
    instanceName: string;
    instanceId: string;
    // Project node props
    expandedProjects: Record<string, boolean>;
    expandedStaleGroups: Record<string, boolean>;
    pinnedProjectIds: string[];
    archivedProjectIds: string[];
    refreshingProjectId: string | null;
    selectedProjectId: string | undefined;
    selectedConversationId: string | undefined;
    onSelectProject: (project: AgmProjectTreeNode) => void;
    onToggleProjectExpand: (projectId: string) => void;
    onToggleStaleGroup: (projectId: string) => void;
    onSelectConversation: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    onOpenInspector: (conv: AgmConversationNode, repoPath: string) => void;
    onRefreshProject: (projectId: string, e: React.MouseEvent) => void;
    onTogglePin: (projectId: string, e: React.MouseEvent) => void;
    onToggleArchive: (projectId: string, e: React.MouseEvent) => void;
    // Re-send queued prompts (Task-03): shown next to the Queued filter.
    onResendQueued?: () => void;
    isResendingQueued?: boolean;
    isAutomatedHidden?: boolean;
    setIsAutomatedHidden?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isCollapsed?: boolean;
    onToggleCollapse?: () => void;
    onToggleExpandAll?: () => void;
    isAllExpanded?: boolean;
    // Context menu handlers forwarded to ConversationNode
    onRenameConversation?: (conversationId: string, newTitle: string) => Promise<void> | void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
    onProjectContextMenu?: (e: React.MouseEvent, repoName: string, project?: AgmProjectTreeNode) => void;
}

const ProjectNodeWithMenu = ProjectNode as React.ComponentType<
    ProjectNodeProps & { onOpenContextMenu?: (menuProps: PromptTreeContextMenuProps) => void }
>;

export function TreePanel(props: TreePanelProps) {
    const {
        isLoading,
        treeData,
        activeProjects,
        archivedProjects,
        archivedCount,
        searchQuery,
        setSearchQuery,
        activeFilter,
        setActiveFilter,
        categoryFilter,
        setCategoryFilter,
        selectedProjectScope,
        setSelectedProjectScope,
        isArchivedProjectsExpanded,
        setIsArchivedProjectsExpanded,
        isAutomatedHidden = true,
        setIsAutomatedHidden,
        onToggleCollapse,
        onToggleExpandAll,
        isAllExpanded = false,
        instanceName,
        instanceId,
        expandedProjects,
        expandedStaleGroups,
        pinnedProjectIds,
        archivedProjectIds,
        refreshingProjectId,
        selectedProjectId,
        selectedConversationId,
        onSelectProject,
        onToggleProjectExpand,
        onToggleStaleGroup,
        onSelectConversation,
        onOpenInspector,
        onRefreshProject,
        onTogglePin,
        onToggleArchive,
        onResendQueued,
        isResendingQueued,
        onRenameConversation,
        onResendPrompts,
        onEnqueue,
        onRerunConversation,
        onProjectContextMenu,
    } = props;

    const [subLoading, setSubLoading] = useState(promptTreeLoadingEmitter.isLoading);
    const [internalAutomatedHidden, setInternalAutomatedHidden] = useState(isAutomatedHidden);
    const [activeContextMenu, setActiveContextMenu] = useState<PromptTreeContextMenuProps | null>(null);
    const [internalImageFilter, setInternalImageFilter] = useState(() => imageFilterEmitter.get());

    useEffect(() => {
        const unsub = imageFilterEmitter.subscribe((active) => setInternalImageFilter(active));
        return () => {
            unsub();
        };
    }, []);

    // Imp-60: Keyboard navigation across conversation nodes (ArrowUp / ArrowDown)
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            const target = e.target as HTMLElement | null;
            if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) {
                return;
            }

            if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') {
                return;
            }

            const flatConvs: { conv: AgmConversationNode; proj: AgmProjectTreeNode }[] = [];
            for (const proj of activeProjects) {
                if (expandedProjects[proj.project_id] !== false) {
                    for (const conv of proj.conversations) {
                        flatConvs.push({ conv, proj });
                    }
                }
            }

            if (flatConvs.length === 0) return;

            const currentIndex = flatConvs.findIndex(item => item.conv.conversation_id === selectedConversationId);
            let nextIndex = currentIndex;

            if (e.key === 'ArrowDown') {
                e.preventDefault();
                nextIndex = currentIndex < flatConvs.length - 1 ? currentIndex + 1 : 0;
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                nextIndex = currentIndex > 0 ? currentIndex - 1 : flatConvs.length - 1;
            }

            if (nextIndex >= 0 && nextIndex < flatConvs.length && nextIndex !== currentIndex) {
                const targetItem = flatConvs[nextIndex];
                onSelectConversation(targetItem.conv, targetItem.proj);
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [activeProjects, expandedProjects, selectedConversationId, onSelectConversation]);

    const effectiveImageFilter = props.isImageFilterActive !== undefined ? props.isImageFilterActive : internalImageFilter;

    const handleOpenContextMenu = (menuProps: PromptTreeContextMenuProps) => {
        setActiveContextMenu(menuProps);
    };

    const handleCloseContextMenu = () => {
        setActiveContextMenu(null);
    };

    useEffect(() => {
        setInternalAutomatedHidden(isAutomatedHidden);
    }, [isAutomatedHidden]);

    const effectiveAutomatedHidden = isAutomatedHidden !== undefined ? isAutomatedHidden : internalAutomatedHidden;
    const handleToggleAutomated = () => {
        if (setIsAutomatedHidden) {
            setIsAutomatedHidden(!effectiveAutomatedHidden);
        } else {
            setInternalAutomatedHidden((prev) => !prev);
        }
    };

    useEffect(() => {
        const unsubscribe = promptTreeLoadingEmitter.subscribe((v) => setSubLoading(v));
        return () => {
            unsubscribe();
        };
    }, []);

    const isEffectiveLoading = isLoading ?? subLoading;

    const isScopedSingleProject = selectedProjectScope !== 'all';

    const isFilterActive = searchQuery !== '' || activeFilter !== 'all' || categoryFilter !== 'all' || selectedProjectScope !== 'all' || effectiveImageFilter;
    const isQueuedView = activeFilter === 'queued' && Boolean(onResendQueued);

    const handleResetFilters = () => {
        setSearchQuery('');
        setActiveFilter('all');
        setCategoryFilter('all');
        setSelectedProjectScope('all');
        imageFilterEmitter.set(false);
    };

    const conversationHasImages = (c: AgmConversationNode): boolean => {
        const hasDirect = Boolean((c.media_paths && c.media_paths.length > 0) || (c.media_thumbnails && c.media_thumbnails.length > 0));
        if (hasDirect) return true;
        return Boolean(
            c.prompts?.some((turn) =>
                Boolean((turn.media_paths && turn.media_paths.length > 0) || (turn.media_thumbnails && turn.media_thumbnails.length > 0))
            )
        );
    };

    const filterByImage = (p: AgmProjectTreeNode) => {
        if (!effectiveImageFilter) return true;
        return (p.conversations || []).some(conversationHasImages);
    };

    const effectiveActiveProjects = activeProjects.filter(filterByImage);
    const effectiveArchivedProjects = archivedProjects.filter(filterByImage);

    const handleProjectContextMenu = onProjectContextMenu
        ? (e: React.MouseEvent, repoName: string, project?: AgmProjectTreeNode) => {
              onProjectContextMenu(e, repoName, project);
          }
        : undefined;

    const renderProject = (project: AgmProjectTreeNode) => {
        let displayedProject = effectiveImageFilter
            ? {
                  ...project,
                  conversations: (project.conversations || []).filter(conversationHasImages),
              }
            : project;

        // Imp-86: Virtualized Tree Node Threshold (>50 conversations)
        const totalConvs = (displayedProject.conversations || []).length;
        const convThreshold = 50;
        if (totalConvs > convThreshold && !isScopedSingleProject) {
            displayedProject = {
                ...displayedProject,
                conversations: displayedProject.conversations.slice(0, convThreshold),
            };
        }

        return (
            <ProjectNodeWithMenu
                key={displayedProject.project_id}
                project={displayedProject}
                isExpanded={isScopedSingleProject || Boolean(expandedProjects[displayedProject.project_id])}
                isScopedSingleProject={isScopedSingleProject}
                isSelected={selectedProjectId === displayedProject.project_id}
                isPinned={pinnedProjectIds.includes(displayedProject.project_id)}
                isArchived={archivedProjectIds.includes(displayedProject.project_id)}
                isRefreshing={refreshingProjectId === displayedProject.project_id}
                isStaleGroupExpanded={Boolean(expandedStaleGroups[displayedProject.project_id])}
                activeFilter={activeFilter}
                categoryFilter={categoryFilter}
                searchQuery={searchQuery}
                isAutomatedHidden={effectiveAutomatedHidden}
                selectedConversationId={selectedConversationId}
                onSelectProject={onSelectProject}
                onToggleExpand={onToggleProjectExpand}
                onToggleStaleGroup={onToggleStaleGroup}
                onSelectConversation={onSelectConversation}
                onOpenInspector={onOpenInspector}
                onRefresh={onRefreshProject}
                onTogglePin={onTogglePin}
                onToggleArchive={onToggleArchive}
                onProjectContextMenu={handleProjectContextMenu}
                onOpenContextMenu={handleOpenContextMenu}
                onRenameConversation={onRenameConversation}
                onResendPrompts={onResendPrompts}
                onEnqueue={onEnqueue}
                onRerunConversation={onRerunConversation}
            />
        );
    };

    const effectiveIsAllExpanded = typeof isAllExpanded === 'boolean'
        ? isAllExpanded
        : treeData.length > 0 && treeData.every((p) => Boolean(expandedProjects[p.project_id]));

    const handleExpandCollapseAll = () => {
        if (onToggleExpandAll) {
            onToggleExpandAll();
            return;
        }
        if (onToggleProjectExpand) {
            const projectsToToggle = effectiveIsAllExpanded
                ? treeData.filter((p) => Boolean(expandedProjects[p.project_id]))
                : treeData.filter((p) => !Boolean(expandedProjects[p.project_id]));
            projectsToToggle.forEach((p) => onToggleProjectExpand(p.project_id));
        }
    };

    return (
        <div className="w-full flex-1 min-h-0 flex flex-col overflow-hidden">
            {/* Search Input & Filter Controls */}
            <div className="relative z-10 bg-slate-50/95 dark:bg-[#0c0817]/95 backdrop-blur-md px-2 py-1 border-b border-slate-200 dark:border-[#40285e] shrink-0 space-y-1">
                {/* Project Scope Dropdown & Quick Pin Header */}
                <div className="flex items-center justify-between gap-1.5 bg-white dark:bg-[#171025] px-2 py-1 rounded-lg border border-slate-200 dark:border-[#40285e]">
                    <div className="flex items-center gap-1.5 min-w-0 flex-1">
                        <Folder className="h-3.5 w-3.5 text-blue-500 shrink-0" />
                        <select
                            value={selectedProjectScope}
                            onChange={(e) => setSelectedProjectScope(e.target.value)}
                            className="w-full text-xs font-medium bg-transparent border-0 focus:ring-0 focus:outline-none text-slate-800 dark:text-slate-200 cursor-pointer truncate"
                            title="Filter tree by project scope"
                        >
                            <option value="all" className="dark:bg-[#171025]">All Projects ({treeData.length})</option>
                            {treeData.map((p) => (
                                <option key={p.project_id} value={p.project_id} className="dark:bg-[#171025]">
                                    {p.repo_name} {p.is_running ? '●' : ''} ({p.conversations.length} convs)
                                </option>
                            ))}
                        </select>
                    </div>
                    {selectedProjectScope !== 'all' && (
                        <button
                            type="button"
                            onClick={(e) => onTogglePin(selectedProjectScope, e)}
                            className={cn(
                                "p-1 rounded-md transition-colors cursor-pointer shrink-0",
                                pinnedProjectIds.includes(selectedProjectScope)
                                    ? "text-amber-500 bg-amber-50 dark:bg-amber-950/40"
                                    : "text-slate-400 hover:text-amber-500 hover:bg-slate-200 dark:hover:bg-[#1f1730]"
                            )}
                            title={pinnedProjectIds.includes(selectedProjectScope) ? "Unpin project" : "Pin project to prompt view"}
                        >
                            <Pin className={cn("w-3.5 h-3.5", pinnedProjectIds.includes(selectedProjectScope) && "fill-current")} />
                        </button>
                    )}
                    {(typeof onToggleExpandAll === 'function' || typeof onToggleProjectExpand === 'function') && (
                        <button
                            type="button"
                            onClick={handleExpandCollapseAll}
                            className="p-1 rounded-md text-slate-400 hover:text-purple-400 hover:bg-slate-100 dark:hover:bg-[#1f1730] transition-colors cursor-pointer shrink-0"
                            title={effectiveIsAllExpanded ? "Collapse all projects" : "Expand all projects"}
                            aria-label={effectiveIsAllExpanded ? "Collapse all projects" : "Expand all projects"}
                        >
                            {effectiveIsAllExpanded ? (
                                <FolderMinus className="w-3.5 h-3.5 text-purple-400" />
                            ) : (
                                <FolderPlus className="w-3.5 h-3.5 text-slate-400" />
                            )}
                        </button>
                    )}
                    {onToggleCollapse && (
                        <button
                            type="button"
                            onClick={onToggleCollapse}
                            className="p-1 rounded-md text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#1f1730] transition-colors cursor-pointer shrink-0"
                            title="Collapse tree panel"
                        >
                            <PanelLeftClose className="w-3.5 h-3.5" />
                        </button>
                    )}
                </div>

                {/* Search Input */}
                <div className="relative">
                    <Search className="absolute left-2.5 top-2 h-3.5 w-3.5 text-slate-400" />
                    <input
                        type="text"
                        placeholder="Search projects, convos, or prompts..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full rounded-lg bg-white dark:bg-[#171025] pl-8 pr-2 py-1 text-xs text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#40285e] focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                    />
                </div>

                {/* Row 1: 4-Tier Category Filter Segmented Capsule */}
                <div className="flex items-center justify-between gap-1">
                    <div className="w-full inline-flex items-center rounded-lg border border-slate-200 dark:border-[#40285e] bg-white/80 dark:bg-[#171025]/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200 dark:divide-[#40285e]">
                        <button
                            type="button"
                            onClick={() => setCategoryFilter('all')}
                            className={cn(
                                "flex-1 py-1 text-[10px] font-medium rounded-l-md transition-colors text-center cursor-pointer",
                                categoryFilter === 'all'
                                    ? "bg-blue-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="All prompt types"
                        >
                            All
                        </button>
                        <button
                            type="button"
                            onClick={() => setCategoryFilter('user')}
                            className={cn(
                                "flex-1 py-1 text-[10px] font-medium transition-colors flex items-center justify-center gap-1 cursor-pointer",
                                categoryFilter === 'user'
                                    ? "bg-sky-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="User Prompts submitted by humans"
                        >
                            <User className="w-2.5 h-2.5 shrink-0" />
                            <span>User</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setCategoryFilter('subagent')}
                            className={cn(
                                "flex-1 py-1 text-[10px] font-medium transition-colors flex items-center justify-center gap-1 cursor-pointer",
                                categoryFilter === 'subagent'
                                    ? "bg-purple-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="Automated subagent and background task instructions"
                        >
                            <Bot className="w-2.5 h-2.5 shrink-0" />
                            <span>Subagent</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setCategoryFilter('system')}
                            className={cn(
                                "flex-1 py-1 text-[10px] font-medium rounded-r-md transition-colors flex items-center justify-center gap-1 cursor-pointer",
                                categoryFilter === 'system'
                                    ? "bg-slate-700 dark:bg-slate-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="System messages & tool outputs"
                        >
                            <Terminal className="w-2.5 h-2.5 shrink-0" />
                            <span>System</span>
                        </button>
                    </div>
                </div>

                {/* Row 2: Status Filter Segmented Capsule & Archive Scope */}
                <div className="flex items-center justify-between gap-1.5 pt-0.5 min-w-0">
                    <div className="inline-flex items-center rounded-lg border border-slate-200 dark:border-[#40285e] bg-white/80 dark:bg-[#171025]/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200 dark:divide-[#40285e] shrink-0">
                        <button
                            type="button"
                            onClick={() => setActiveFilter(activeFilter === 'running' ? 'all' : 'running')}
                            className={cn(
                                "px-2.5 py-0.5 text-[10px] font-medium rounded-l-md transition-colors whitespace-nowrap cursor-pointer flex items-center gap-1",
                                activeFilter === 'running'
                                    ? "bg-emerald-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="Active running prompts"
                        >
                            <span className={cn("w-1.5 h-1.5 rounded-full", activeFilter === 'running' ? "bg-white" : "bg-emerald-500 animate-pulse")} />
                            <span>Running</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setActiveFilter(activeFilter === 'queued' ? 'all' : 'queued')}
                            className={cn(
                                "px-2.5 py-0.5 text-[10px] font-medium rounded-none transition-colors whitespace-nowrap cursor-pointer flex items-center gap-1",
                                activeFilter === 'queued'
                                    ? "bg-amber-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="Queued prompts"
                        >
                            <span className={cn("w-1.5 h-1.5 rounded-full", activeFilter === 'queued' ? "bg-white" : "bg-amber-500")} />
                            <span>Queued</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setActiveFilter(activeFilter === 'pinned' ? 'all' : 'pinned')}
                            className={cn(
                                "px-2 py-0.5 text-[10px] font-medium transition-colors whitespace-nowrap cursor-pointer flex items-center gap-0.5",
                                activeFilter === 'pinned'
                                    ? "bg-blue-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="Pinned prompts"
                        >
                            <Pin className="w-2.5 h-2.5 shrink-0" />
                            <span>Pinned</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => imageFilterEmitter.set(!effectiveImageFilter)}
                            className={cn(
                                "px-2 py-0.5 text-[10px] font-medium rounded-r-md transition-colors whitespace-nowrap cursor-pointer flex items-center gap-1",
                                effectiveImageFilter
                                    ? "bg-sky-600 text-white shadow-2xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="Filter prompts with attached images"
                            title={effectiveImageFilter ? "Showing only conversations with images (Click to clear)" : "Filter projects and prompts with images"}
                        >
                            <span className="text-[10px]">🖼️</span>
                            <span>Images</span>
                        </button>
                    </div>

                    <div className="inline-flex items-center rounded-lg border border-slate-200 dark:border-[#40285e] bg-white/80 dark:bg-[#171025]/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200 dark:divide-[#40285e] shrink-0">
                        <button
                            type="button"
                            onClick={handleToggleAutomated}
                            className={cn(
                                "flex items-center gap-1 px-2 py-0.5 text-[10px] font-medium rounded-l-md transition-colors whitespace-nowrap cursor-pointer",
                                effectiveAutomatedHidden
                                    ? "bg-purple-500/15 text-purple-700 dark:text-purple-300 font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-[#1f1730]"
                            )}
                            aria-label="Toggle automated prompts"
                            title={effectiveAutomatedHidden ? "Automated prompts hidden (Click to show)" : "Automated prompts visible (Click to hide)"}
                        >
                            {effectiveAutomatedHidden ? (
                                <EyeOff className="w-3 h-3 text-purple-500" />
                            ) : (
                                <Eye className="w-3 h-3 text-slate-400" />
                            )}
                            <span>{effectiveAutomatedHidden ? 'Hide Auto' : 'Show Auto'}</span>
                        </button>

                        <div className="relative flex items-center px-1.5 py-0.5">
                            <select
                                value={activeFilter === 'archived' ? 'archived' : 'all'}
                                onChange={(e) => setActiveFilter(e.target.value as ActiveFilter)}
                                className="appearance-none bg-transparent border-0 focus:ring-0 focus:outline-none text-[10px] font-medium text-slate-700 dark:text-slate-300 cursor-pointer pr-3 py-0"
                            >
                                <option value="all" className="bg-white dark:bg-[#171025]">All ({treeData.length})</option>
                                <option value="archived" className="bg-white dark:bg-[#171025]">Archived ({archivedCount})</option>
                            </select>
                            <ChevronDown className="w-2.5 h-2.5 text-slate-400 pointer-events-none absolute right-1 top-1/2 -translate-y-1/2" />
                        </div>
                    </div>
                </div>

                {/* Row 3: Action Buttons (Reset Filters & Re-send queued prompts) */}
                {(isFilterActive || isQueuedView) && (
                    <div className="flex items-center justify-between gap-1 pt-0.5">
                        {isFilterActive ? (
                            <button
                                type="button"
                                onClick={handleResetFilters}
                                className="flex items-center gap-1.5 px-2.5 py-0.5 text-[10px] font-semibold text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 bg-blue-50/80 dark:bg-blue-950/40 hover:bg-blue-100/80 dark:hover:bg-blue-900/50 border border-blue-200 dark:border-blue-800/60 rounded-md transition-all cursor-pointer shadow-2xs"
                                title="Reset all search and tree filters back to default"
                            >
                                <RotateCcw className="w-2.5 h-2.5 shrink-0" />
                                <span>Reset Filters</span>
                            </button>
                        ) : <div />}

                        {isQueuedView && (
                            <button
                                type="button"
                                onClick={onResendQueued}
                                disabled={isResendingQueued}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1 text-[10px] font-semibold rounded-md transition-colors cursor-pointer",
                                    "bg-amber-600/10 text-amber-700 dark:text-amber-300 border border-amber-500/40",
                                    "hover:bg-amber-600/20 disabled:opacity-50 disabled:cursor-not-allowed"
                                )}
                                title="Re-send all queued prompts in FIFO order (durable backup first; already-sent prompts are skipped)"
                            >
                                <RotateCcw className={cn("w-2.5 h-2.5 shrink-0", isResendingQueued && "animate-spin")} />
                                <span>{isResendingQueued ? 'Re-sending…' : 'Re-send queued prompts'}</span>
                            </button>
                        )}
                    </div>
                )}
            </div>

            {/* Tree Items List */}
            <div className="flex-1 overflow-y-auto p-1 space-y-0.5">
                {activeFilter === 'archived' ? (
                    effectiveArchivedProjects.length === 0 ? (
                        <div className="py-12 text-center text-xs text-slate-400">
                            No archived projects found.
                        </div>
                    ) : (
                        effectiveArchivedProjects.map((project) => renderProject(project))
                    )
                ) : isEffectiveLoading && treeData.length === 0 ? (
                    /* Animated Skeleton Shimmer replacing jarring false-empty flash */
                    <div className="space-y-2 p-1 animate-pulse">
                        {[1, 2, 3].map((n) => (
                            <div key={n} className="rounded-lg border border-slate-200/60 dark:border-[#40285e]/60 p-2.5 space-y-2 bg-slate-50/50 dark:bg-[#171025]/40">
                                <div className="flex items-center gap-2">
                                    <div className="w-4 h-4 rounded bg-slate-200 dark:bg-slate-800" />
                                    <div className="h-3.5 w-32 rounded bg-slate-200 dark:bg-slate-800" />
                                    <div className="ml-auto h-3 w-10 rounded bg-slate-200 dark:bg-slate-800" />
                                </div>
                                <div className="pl-6 space-y-1.5">
                                    <div className="h-3 w-48 rounded bg-slate-200/70 dark:bg-slate-800/70" />
                                    <div className="h-3 w-36 rounded bg-slate-200/70 dark:bg-slate-800/70" />
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <>
                        {effectiveActiveProjects.length === 0 && effectiveArchivedProjects.length === 0 ? (
                            treeData.length === 0 ? (
                                <div className="py-16 text-center space-y-3 px-6">
                                    <div className="w-12 h-12 rounded-xl bg-slate-100 dark:bg-[#171025] flex items-center justify-center mx-auto text-slate-400">
                                        <Folder className="w-6 h-6 text-slate-400" />
                                    </div>
                                    <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                                        No Projects Found in this Profile
                                    </div>
                                    <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mx-auto">
                                        No workspace storage or conversation records were found for instance &quot;{instanceName || instanceId}&quot;. Launch this instance and open a workspace to start logging prompts.
                                    </p>
                                </div>
                            ) : searchQuery.trim() ? (
                                <div className="py-12 text-center text-xs text-slate-400 space-y-2">
                                    <div>No conversations match &quot;{searchQuery}&quot;</div>
                                    <button
                                        onClick={() => setSearchQuery('')}
                                        className="text-blue-500 hover:underline text-[11px] cursor-pointer"
                                    >
                                        Clear search query
                                    </button>
                                </div>
                            ) : isFilterActive ? (
                                <div className="py-12 text-center text-xs text-slate-400 space-y-2.5 px-4">
                                    <div>No projects or conversations match the active filters.</div>
                                    <button
                                        type="button"
                                        onClick={handleResetFilters}
                                        className="inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-[11px] font-semibold bg-purple-500/15 text-purple-700 dark:text-purple-300 border border-purple-500/30 hover:bg-purple-500/25 transition-colors cursor-pointer shadow-2xs"
                                    >
                                        <RotateCcw className="w-3 h-3" />
                                        <span>Reset Filters</span>
                                    </button>
                                </div>
                            ) : (
                                <div className="py-12 text-center text-xs text-slate-400">
                                    No projects or conversations found.
                                </div>
                            )
                        ) : (
                            <>
                                {effectiveActiveProjects.map((project) => renderProject(project))}

                                {/* Collapsed at bottom: Archived Projects */}
                                {effectiveArchivedProjects.length > 0 && (
                                    <div className="space-y-0.5 mt-2 pt-2 border-t border-slate-200 dark:border-[#40285e]">
                                        <div
                                            onClick={() => setIsArchivedProjectsExpanded(!isArchivedProjectsExpanded)}
                                            className="group flex items-center justify-between rounded-md px-2.5 py-1.5 text-xs font-semibold text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#1f1730] cursor-pointer transition-colors"
                                            title="Toggle Archived Projects"
                                        >
                                            <div className="flex items-center gap-1.5 min-w-0">
                                                {isArchivedProjectsExpanded ? (
                                                    <ChevronDown className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                ) : (
                                                    <ChevronRight className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                )}
                                                <Archive className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                <span className="truncate">Archived Projects ({effectiveArchivedProjects.length})</span>
                                            </div>
                                            <span className="text-[10px] font-mono text-slate-400">
                                                {effectiveArchivedProjects.length}
                                            </span>
                                        </div>

                                        {isArchivedProjectsExpanded && (
                                            <div className="space-y-0.5 pl-1">
                                                {archivedProjects.map((project) => renderProject(project))}
                                            </div>
                                        )}
                                    </div>
                                )}
                            </>
                        )}
                    </>
                )}
            </div>

            {/* Centralized Singleton Context Menu & Transparent Dismissal Backdrop */}
            {activeContextMenu && (
                <>
                    {typeof document !== 'undefined' ? (
                        createPortal(
                            <div
                                className="fixed inset-0 z-[99998] bg-transparent cursor-default"
                                onClick={handleCloseContextMenu}
                                onContextMenu={(e) => {
                                    e.preventDefault();
                                    handleCloseContextMenu();
                                }}
                            />,
                            document.body
                        )
                    ) : (
                        <div
                            className="fixed inset-0 z-[99998] bg-transparent cursor-default"
                            onClick={handleCloseContextMenu}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                handleCloseContextMenu();
                            }}
                        />
                    )}
                    <PromptTreeContextMenu
                        {...activeContextMenu}
                        onClose={handleCloseContextMenu}
                    />
                </>
            )}
        </div>
    );
}
