import {
    Folder,
    ChevronRight,
    ChevronDown,
    Clock,
    Pin,
    Archive,
    ArchiveRestore,
    RotateCw,
    Bot,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';
import { isGhostConversation, isStaleOrEmptyConversation } from './conversationGuards';
import { classifyPromptTier, assembleConversationHierarchy } from './tierClassification';
import { formatSeqBadge } from './formatters';
import { ConversationNode, getConversationStatus } from './ConversationNode';

export interface ProjectNodeProps {
    project: AgmProjectTreeNode;
    isExpanded: boolean;
    isSelected: boolean;
    isPinned: boolean;
    isArchived: boolean;
    isRefreshing: boolean;
    isStaleGroupExpanded: boolean;
    activeFilter: 'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived';
    categoryFilter: 'all' | 'user' | 'subagent' | 'system';
    searchQuery: string;
    selectedConversationId: string | undefined;
    onSelectProject: (project: AgmProjectTreeNode) => void;
    onToggleExpand: (projectId: string) => void;
    onToggleStaleGroup: (projectId: string) => void;
    onSelectConversation: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    onOpenInspector: (conv: AgmConversationNode, repoPath: string) => void;
    onRefresh: (projectId: string, e: React.MouseEvent) => void;
    onTogglePin: (projectId: string, e: React.MouseEvent) => void;
    onToggleArchive: (projectId: string, e: React.MouseEvent) => void;
}

export function sortConversations(
    convs: AgmConversationNode[],
    activeFilter: ProjectNodeProps['activeFilter']
): AgmConversationNode[] {
    let sorted = [...convs];
    if (activeFilter === 'running') {
        sorted = sorted.filter((c) => Boolean(c.is_running));
    }
    return sorted.sort((a, b) => {
        const aRunning = Boolean(a.is_running);
        const bRunning = Boolean(b.is_running);
        if (aRunning !== bRunning) {
            return aRunning ? -1 : 1;
        }
        const aTime = new Date(a.last_modified).getTime() || 0;
        const bTime = new Date(b.last_modified).getTime() || 0;
        return bTime - aTime;
    });
}

interface ConversationListProps {
    convList: AgmConversationNode[];
    project: AgmProjectTreeNode;
    categoryFilter: ProjectNodeProps['categoryFilter'];
    selectedConversationId: string | undefined;
    onSelectConversation: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    onOpenInspector: (conv: AgmConversationNode, repoPath: string) => void;
}

function ConversationListWithGrouping({
    convList,
    project,
    categoryFilter,
    selectedConversationId,
    onSelectConversation,
    onOpenInspector,
}: ConversationListProps) {
    if (categoryFilter === 'subagent') {
        return (
            <>
                {convList.map((conv) => (
                    <ConversationNode
                        key={conv.conversation_id}
                        conv={conv}
                        project={project}
                        isSelected={selectedConversationId === conv.conversation_id}
                        onSelect={onSelectConversation}
                        onOpenInspector={onOpenInspector}
                    />
                ))}
            </>
        );
    }

    const hierarchy = assembleConversationHierarchy(convList);

    return (
        <>
            {hierarchy.map((root) => (
                <div key={`root-${root.primaryNode.conversation_id}`} className="space-y-0.5">
                    <ConversationNode
                        conv={root.primaryNode}
                        project={project}
                        isSelected={selectedConversationId === root.primaryNode.conversation_id}
                        onSelect={onSelectConversation}
                        onOpenInspector={onOpenInspector}
                    />

                    {/* Rendering Child Sub-Points (Indented with Branch Glyphs) */}
                    {root.subagents.length > 0 && (
                        <div className="pl-5 space-y-0.5 border-l-2 border-purple-300/40 dark:border-purple-800/40 ml-3.5 my-0.5">
                            {root.subagents.map((subNode, subIdx) => {
                                const isSubSelected = selectedConversationId === subNode.primaryNode.conversation_id;
                                const { isRunning: isSubRunning } = getConversationStatus(subNode.primaryNode);
                                const isLast = subIdx === root.subagents.length - 1;
                                return (
                                    <div
                                        key={subNode.primaryNode.conversation_id}
                                        onClick={() => onSelectConversation(subNode.primaryNode, project)}
                                        onDoubleClick={() => onOpenInspector(subNode.primaryNode, project.repo_path)}
                                        className={cn(
                                            "relative flex items-center justify-between rounded-[5px] px-2 py-1 text-xs cursor-pointer transition-colors group",
                                            isSubSelected
                                                ? "bg-purple-600 text-white font-medium shadow-2xs"
                                                : "text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438]"
                                        )}
                                        title={`AI Subagent Task · ${subNode.primaryNode.title || 'Subagent'} · ${subNode.primaryNode.step_count || 1} steps · Click to view`}
                                    >
                                        <div className="flex items-center gap-1.5 min-w-0">
                                            {/* Tree Branch Connector Glyph: ↳ or └── */}
                                            <span className="text-purple-400 dark:text-purple-400 font-mono text-[11px] select-none shrink-0">
                                                {isLast ? '└──' : '↳'}
                                            </span>
                                            <span title={subNode.classification.roleBadge || 'Subagent'} className="shrink-0 flex items-center">
                                                <Bot
                                                    className={cn("w-3.5 h-3.5 shrink-0", isSubSelected ? "text-white" : "text-purple-500")}
                                                />
                                            </span>
                                            <span
                                                className={cn(
                                                    'text-[9px] font-mono px-1 py-0.2 rounded-[3px] shrink-0 font-medium whitespace-nowrap',
                                                    isSubSelected
                                                        ? 'bg-purple-700 text-white'
                                                        : 'bg-slate-200/90 dark:bg-[#15334d] text-slate-600 dark:text-cyan-300'
                                                )}
                                                title={subNode.primaryNode.short_id ? `GitMap SHA: ${subNode.primaryNode.short_id}` : (subNode.primaryNode.gitmap_seq_code ? `GitMap: ${subNode.primaryNode.gitmap_seq_code}` : undefined)}
                                            >
                                                {formatSeqBadge(subNode.primaryNode.seq_code, 'C001')}
                                            </span>
                                            <span className="truncate text-[11px]">
                                                {subNode.primaryNode.title || 'Subagent Task'}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-1.5 shrink-0">
                                            {isSubRunning && (
                                                <span className={cn(
                                                    "flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono border animate-pulse",
                                                    isSubSelected
                                                        ? "bg-emerald-400 text-emerald-950 border-emerald-300"
                                                        : "bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border-emerald-500/40"
                                                )}>
                                                    <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                                                    RUNNING
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            ))}
        </>
    );
}

export function ProjectNode(props: ProjectNodeProps) {
    const {
        project,
        isExpanded,
        isSelected,
        isPinned,
        isArchived,
        isRefreshing,
        isStaleGroupExpanded,
        activeFilter,
        categoryFilter,
        searchQuery,
        selectedConversationId,
        onSelectProject,
        onToggleExpand,
        onToggleStaleGroup,
        onSelectConversation,
        onOpenInspector,
        onRefresh,
        onTogglePin,
        onToggleArchive,
    } = props;

    const filteredConvs = project.conversations.filter((c) => {
        if (activeFilter === 'archived') return true;
        return !isGhostConversation(c);
    });

    const totalProjectPrompts = filteredConvs.reduce(
        (sum, c) => sum + (c.step_count > 0 ? c.step_count : 1),
        0
    );

    const sortedConvs = sortConversations(project.conversations, activeFilter);
    const activeConversations: AgmConversationNode[] = [];
    const staleConversations: AgmConversationNode[] = [];

    sortedConvs.forEach((conv) => {
        if (isGhostConversation(conv) && activeFilter !== 'archived') {
            return;
        }
        const cat = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);
        if (categoryFilter === 'all') {
            if (cat.isNonPrompt && !searchQuery.trim()) return;
        } else if (categoryFilter === 'user') {
            if (cat.tier !== 'USER_PROMPT') return;
        } else if (categoryFilter === 'subagent') {
            if (cat.tier !== 'SUBAGENT_INSTRUCTION') return;
        } else if (categoryFilter === 'system') {
            if (cat.tier !== 'SYSTEM_MESSAGE' && cat.tier !== 'TOOL_OUTPUT') return;
        }
        if (activeFilter === 'running' && !conv.is_running) {
            return;
        }
        if (activeFilter === 'queued' && !conv.is_queued && !conv.status.toLowerCase().includes('queue')) {
            return;
        }
        if (isStaleOrEmptyConversation(conv)) {
            staleConversations.push(conv);
        } else {
            activeConversations.push(conv);
        }
    });

    const runningCount = project.conversations.filter(
        (c) => Boolean(c.is_running) &&
               !isGhostConversation(c) &&
               !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
    ).length;
    const queuedCount = project.conversations.filter(
        (c) => (Boolean(c.is_queued) || c.status.toLowerCase().includes('queue')) && !c.is_running
    ).length;

    const listProps = {
        project,
        categoryFilter,
        selectedConversationId,
        onSelectConversation,
        onOpenInspector,
    };

    return (
        <div key={project.project_id} className="space-y-0.5">
            {/* Layer 1: Project Row */}
            <div
                onClick={() => {
                    onSelectProject(project);
                    onToggleExpand(project.project_id);
                }}
                className={cn(
                    'group flex items-center justify-between rounded-[5px] px-2.5 py-1.5 text-xs font-semibold cursor-pointer transition-colors',
                    isSelected
                        ? 'bg-blue-100/70 text-blue-900 dark:bg-blue-950/60 dark:text-cyan-300'
                        : 'text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#0c2438]'
                )}
                title={`Project: ${project.repo_name} · ${totalProjectPrompts} prompts total`}
            >
                <div className="flex items-center gap-1.5 min-w-0">
                    {isExpanded ? (
                        <ChevronDown className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                    ) : (
                        <ChevronRight className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                    )}
                    <Folder className="h-4 w-4 text-amber-500 shrink-0" />
                    <span
                        className={cn(
                            'text-[9px] font-mono px-1 py-0.2 rounded-[3px] shrink-0 font-medium whitespace-nowrap',
                            isSelected
                                ? 'bg-blue-600/20 text-blue-700 dark:text-cyan-300 border border-blue-500/30'
                                : 'bg-slate-200 dark:bg-[#15334d] text-slate-600 dark:text-cyan-400'
                        )}
                        title={project.gitmap_seq_code || (project.seq_id ? `Project #${project.seq_id}` : undefined)}
                    >
                        {formatSeqBadge(project.seq_code, '#P001')}
                    </span>
                    <span className="truncate">{project.repo_name}</span>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                    {runningCount > 0 && (
                        <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/30">
                            <span className="relative flex h-2 w-2">
                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)] animate-pulse"></span>
                            </span>
                            <span className="text-[9px] font-bold font-mono text-emerald-700 dark:text-[#1af18d]">
                                {runningCount} RUNNING
                            </span>
                        </div>
                    )}
                    {queuedCount > 0 && (
                        <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/30">
                            <Clock className="w-2.5 h-2.5 text-amber-500 shrink-0" />
                            <span className="text-[9px] font-bold font-mono text-amber-700 dark:text-amber-300">
                                {queuedCount} QUEUED
                            </span>
                        </div>
                    )}
                    {/* Project Action Button Cluster - Hover Only */}
                    <div className={cn(
                        "flex items-center gap-0.5 transition-opacity duration-150 shrink-0",
                        isPinned || isArchived || isRefreshing
                            ? "opacity-100"
                            : "opacity-0 group-hover:opacity-100"
                    )}>
                        <button
                            type="button"
                            onClick={(e) => onRefresh(project.project_id, e)}
                            disabled={isRefreshing}
                            className="p-1 rounded-[5px] text-slate-400 hover:text-blue-500 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                            title="Refresh this project"
                        >
                            <RotateCw className={cn("w-3 h-3", isRefreshing && "animate-spin text-blue-500")} />
                        </button>
                        <button
                            type="button"
                            onClick={(e) => onTogglePin(project.project_id, e)}
                            className={cn(
                                "p-1 rounded-[5px] transition-colors cursor-pointer",
                                isPinned
                                    ? "text-amber-500 bg-amber-50 dark:bg-amber-950/40"
                                    : "text-slate-400 hover:text-amber-500 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                            )}
                            title={isPinned ? "Unpin project" : "Pin project to top"}
                        >
                            <Pin className={cn("w-3 h-3", isPinned && "fill-current")} />
                        </button>
                        <button
                            type="button"
                            onClick={(e) => onToggleArchive(project.project_id, e)}
                            className={cn(
                                "p-1 rounded-[5px] transition-colors cursor-pointer",
                                isArchived
                                    ? "text-rose-500 bg-rose-50 dark:bg-rose-950/40"
                                    : "text-slate-400 hover:text-rose-500 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                            )}
                            title={isArchived ? "Unarchive project" : "Archive / Less Favorite (Thumbs Down)"}
                        >
                            {isArchived ? (
                                <ArchiveRestore className="w-3 h-3 text-rose-500" />
                            ) : (
                                <Archive className="w-3 h-3" />
                            )}
                        </button>
                    </div>
                </div>
            </div>

            {/* Layer 2: Conversations List */}
            {isExpanded && (
                <div className="ml-4 pl-2 border-l border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                    {/* Active Conversations */}
                    <ConversationListWithGrouping convList={activeConversations} {...listProps} />

                    {/* Collapsible Stale / Empty Prompts Node */}
                    {staleConversations.length > 0 && (
                        <div className="space-y-0.5 mt-1 pt-1 border-t border-slate-200/50 dark:border-[#15334d]/40">
                            <div
                                onClick={() => onToggleStaleGroup(project.project_id)}
                                className="group flex items-center justify-between rounded-[5px] px-2 py-1 text-xs font-medium text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438] cursor-pointer transition-colors"
                                title="Toggle Archived / Stale Prompts"
                            >
                                <div className="flex items-center gap-1.5 min-w-0">
                                    {isStaleGroupExpanded ? (
                                        <ChevronDown className="h-3 w-3 text-slate-400 shrink-0" />
                                    ) : (
                                        <ChevronRight className="h-3 w-3 text-slate-400 shrink-0" />
                                    )}
                                    <Folder className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                    <span className="truncate">
                                        Archived / Stale Prompts ({staleConversations.length})
                                    </span>
                                </div>
                                <span className="text-[10px] font-mono text-slate-400">
                                    {staleConversations.length}
                                </span>
                            </div>

                            {isStaleGroupExpanded && (
                                <div className="ml-4 pl-2 border-l border-dashed border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                                    <ConversationListWithGrouping convList={staleConversations} {...listProps} />
                                </div>
                            )}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
