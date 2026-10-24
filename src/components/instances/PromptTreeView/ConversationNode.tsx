import { useState, useEffect, useRef } from 'react';
import { Bot, User, Terminal, Wrench, Clock, ChevronRight, ChevronDown } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../../utils/cn';
import { PromptOriginTier } from './types';
import type { AgmConversationNode, AgmProjectTreeNode, TierClassificationResult } from './types';
import { isConversationOrTurnQueued, getConversationStatus } from './conversationGuards';
import { classifyPromptTier } from './tierClassification';
import { formatSeqBadge, formatRelativeRecency } from './formatters';
import { invalidatePromptTreeCache, renameConversationOptimistic } from './usePromptTreeData';
import { useErrorStore } from '../../../stores/error-store';

export { getConversationStatus };

export interface ConversationNodeProps {
    conv: AgmConversationNode;
    project: AgmProjectTreeNode;
    isSelected: boolean;
    isLastConv?: boolean;
    onSelect: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    onOpenInspector: (conv: AgmConversationNode, repoPath: string) => void;
    onRenameConversation?: (conversationId: string, newTitle: string, targetInst?: string) => Promise<void> | void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
    onOpenContextMenu?: (props: any) => void;
}

export function getTierIcon(tier: TierClassificationResult['tier']) {
    return tier === PromptOriginTier.SubagentInstruction
        ? Bot
        : tier === PromptOriginTier.SystemMessage
          ? Terminal
          : tier === PromptOriginTier.ToolOutput
            ? Wrench
            : User;
}

export function ConversationNode({
    conv,
    project,
    isSelected,
    isLastConv = false,
    onSelect,
    onOpenInspector,
    onRenameConversation,
    onOpenContextMenu,
}: ConversationNodeProps) {
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [editingTitle, setEditingTitle] = useState(conv.title || '');
    const [selectedTurnNumber, setSelectedTurnNumber] = useState<number | null>(() => conv.selected_turn_number ?? null);
    const [isExpanded, setIsExpanded] = useState<boolean>(() => Boolean(conv.is_running) || isConversationOrTurnQueued(conv));
    const activeTurnRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (isSelected && activeTurnRef.current) {
            activeTurnRef.current.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        }
    }, [isSelected, selectedTurnNumber, isExpanded]);

    useEffect(() => {
        if (isSelected) {
            setSelectedTurnNumber(typeof conv.selected_turn_number === 'number' ? conv.selected_turn_number : null);
        } else {
            setSelectedTurnNumber(null);
        }
    }, [isSelected, conv.selected_turn_number]);

    useEffect(() => {
        if (Boolean(conv.is_running) || isConversationOrTurnQueued(conv)) {
            setIsExpanded(true);
        }
    }, [conv.is_running, conv.is_queued, conv.status, conv.prompts]);

    const { isRunning, isQueued } = getConversationStatus(conv);
    const promptCategory = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);
    const IconComponent = getTierIcon(promptCategory.tier);
    const hasPrompts = Boolean(conv.prompts && conv.prompts.length > 0);
    const enqueuedTurnsCount = conv.prompts
        ? conv.prompts.filter((p) => Boolean(p.is_queued) || (p.status && p.status.toLowerCase().includes('queue'))).length
        : 0;

    const handleSaveTitle = async (draftTitle: string) => {
        setIsEditingTitle(false);
        const trimmed = draftTitle.trim();
        if (!trimmed || trimmed === (conv.title || '')) {
            setEditingTitle(conv.title || '');
            return;
        }

        const prevTitle = conv.title;
        const targetInst = conv.instance_id || project.instance_id;

        // Optimistic local update
        conv.title = trimmed;
        setEditingTitle(trimmed);
        renameConversationOptimistic(conv.conversation_id, trimmed, targetInst, (updated) => {
            onSelect(updated, project);
        });

        try {
            if (onRenameConversation) {
                await onRenameConversation(conv.conversation_id, trimmed, targetInst);
            } else {
                await invoke('rename_conversation', {
                    instanceId: targetInst || null,
                    conversationId: conv.conversation_id,
                    newTitle: trimmed,
                });
                invalidatePromptTreeCache(targetInst);
            }
        } catch (err: unknown) {
            // Rollback on failure
            conv.title = prevTitle;
            setEditingTitle(prevTitle || '');
            renameConversationOptimistic(conv.conversation_id, prevTitle || '', targetInst, (updated) => {
                onSelect(updated, project);
            });
            useErrorStore.getState().captureException(err instanceof Error ? err : String(err), {
                source: 'ConversationNode.handleSaveTitle',
                triggerAction: 'rename_conversation',
            });
        }
    };

    return (
        <>
            <div
                key={conv.conversation_id}
                tabIndex={0}
                onClick={() => {
                    const runningTurn = conv.prompts?.find((p) => p.is_running) || conv.prompts?.[conv.prompts.length - 1];
                    const defaultTurnNum = runningTurn ? runningTurn.turn_number : null;
                    setSelectedTurnNumber(defaultTurnNum);
                    onSelect({ ...conv, selected_turn_number: defaultTurnNum }, project);
                }}
                onDoubleClick={() => {
                    const runningTurn = conv.prompts?.find((p) => p.is_running) || conv.prompts?.[conv.prompts.length - 1];
                    const defaultTurnNum = runningTurn ? runningTurn.turn_number : null;
                    setSelectedTurnNumber(defaultTurnNum);
                    const effectiveConv = { ...conv, selected_turn_number: defaultTurnNum };
                    onSelect(effectiveConv, project);
                    onOpenInspector(effectiveConv, project.repo_path);
                }}
                onKeyDown={(e) => {
                    if (isSelected && e.key === 'F2') {
                        e.preventDefault();
                        e.stopPropagation();
                        setIsEditingTitle(true);
                        setEditingTitle(conv.title || '');
                    }
                }}
                onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    onOpenContextMenu?.({
                        type: 'conversation',
                        x: e.clientX,
                        y: e.clientY,
                        conversationId: conv.conversation_id,
                        currentTitle: conv.title || conv.short_id || '',
                        conversationNode: conv,
                        repoPath: project.repo_path,
                        instanceId: conv.instance_id || project.instance_id,
                    });
                }}
                className={cn(
                    'group flex items-center justify-between rounded-md px-1.5 py-0.5 text-xs cursor-pointer transition-colors duration-150 relative outline-none focus-visible:ring-1 focus-visible:ring-purple-500/50 border border-transparent',
                    isSelected && selectedTurnNumber === null
                        ? 'bg-gradient-to-r from-violet-600/30 to-purple-600/20 text-white font-medium'
                        : isSelected
                          ? 'bg-purple-500/15 text-purple-300 font-medium'
                          : 'text-slate-600 dark:text-zinc-300 hover:bg-slate-100/80 dark:hover:bg-[#181126]/70'
                )}
                title={`${conv.title || 'Conversation'} · ${conv.step_count || 1} steps · Click to view (F2 to rename)`}
            >
                <div className="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
                    {/* Branch connector glyph to project */}
                    <span className="text-slate-400 dark:text-purple-400/80 font-mono text-[10px] select-none shrink-0" aria-hidden="true">
                        {isLastConv ? '└──' : '├──'}
                    </span>
                    {hasPrompts && (
                        <button
                            type="button"
                            onClick={(e) => {
                                e.stopPropagation();
                                setIsExpanded((prev) => !prev);
                            }}
                            className="p-0.5 rounded hover:bg-slate-200 dark:hover:bg-[#281b3d] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer shrink-0 transition-colors"
                            title={isExpanded ? 'Collapse prompt turns' : 'Expand prompt turns'}
                        >
                            {isExpanded ? (
                                <ChevronDown className="w-3 h-3" />
                            ) : (
                                <ChevronRight className="w-3 h-3" />
                            )}
                        </button>
                    )}
                    {promptCategory.tier === PromptOriginTier.SubagentInstruction && (
                        <span className="text-purple-400 dark:text-purple-400 font-mono text-[10px] select-none shrink-0" title="AI Subagent Instruction">
                            └── ↳
                        </span>
                    )}
                    <span title={promptCategory.roleBadge || promptCategory.tier} className="shrink-0 flex items-center">
                        <IconComponent
                            className={cn('h-3.5 w-3.5 shrink-0', isSelected ? 'text-white' : (promptCategory.tier === PromptOriginTier.SubagentInstruction ? 'text-purple-500' : promptCategory.tier === PromptOriginTier.SystemMessage ? 'text-zinc-500' : promptCategory.tier === PromptOriginTier.ToolOutput ? 'text-amber-500' : 'text-sky-500'))}
                        />
                    </span>
                    <span
                        className={cn(
                            'text-[9px] font-mono px-1 py-0.2 rounded-xs shrink-0 font-medium whitespace-nowrap',
                            isSelected
                                ? 'bg-blue-700/80 text-white'
                                : 'bg-slate-200/90 dark:bg-[#40285e] text-slate-600 dark:text-cyan-300'
                        )}
                        title={`Conversation #${conv.seq_id} (Sequential AGM Index)${conv.short_id ? ` · GitMap SHA: ${conv.short_id}` : (conv.gitmap_seq_code ? ` · GitMap: ${conv.gitmap_seq_code}` : '')}`}
                    >
                        {formatSeqBadge(conv.seq_code, 'C001')}
                    </span>
                    {(conv.repeat_badge || (conv.repeat_count && conv.repeat_count > 1)) && (
                        <span
                            className={cn(
                                "px-1.5 py-0.2 rounded-md text-[8.5px] font-mono font-bold shrink-0 border",
                                isSelected
                                    ? "bg-amber-300 text-amber-950 border-amber-200"
                                    : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30"
                            )}
                            title={`${conv.repeat_count || 2} duplicate/repeated prompt runs`}
                        >
                            {conv.repeat_badge || `x${conv.repeat_count}`}
                        </span>
                    )}
                    {isEditingTitle ? (
                        <input
                            type="text"
                            autoFocus
                            value={editingTitle}
                            onClick={(e) => e.stopPropagation()}
                            onDoubleClick={(e) => e.stopPropagation()}
                            onChange={(e) => setEditingTitle(e.target.value)}
                            onKeyDown={(e) => {
                                e.stopPropagation();
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleSaveTitle(editingTitle);
                                } else if (e.key === 'Escape') {
                                    e.preventDefault();
                                    setIsEditingTitle(false);
                                    setEditingTitle(conv.title || '');
                                }
                            }}
                            onBlur={() => handleSaveTitle(editingTitle)}
                            className="w-full px-1.5 py-0.5 rounded bg-white dark:bg-[#120a1f] border border-cyan-500 text-xs text-slate-900 dark:text-white outline-none ring-1 ring-cyan-500/50"
                        />
                    ) : (
                        <span
                            onDoubleClick={(e) => {
                                e.stopPropagation();
                                e.preventDefault();
                                setIsEditingTitle(true);
                                setEditingTitle(conv.title || '');
                            }}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                onOpenContextMenu?.({
                                    type: 'conversation',
                                    x: e.clientX,
                                    y: e.clientY,
                                    conversationId: conv.conversation_id,
                                    currentTitle: conv.title || conv.short_id || '',
                                    conversationNode: conv,
                                    repoPath: project.repo_path,
                                    instanceId: conv.instance_id || project.instance_id,
                                });
                            }}
                            className="truncate text-[11px] cursor-text hover:underline hover:decoration-dotted"
                            title={`${conv.title || conv.short_id || conv.conversation_id.slice(0, 8)} (Double-click to rename)`}
                        >
                            {conv.title || conv.short_id || conv.conversation_id.slice(0, 8)}
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                    {(() => {
                        const recency = formatRelativeRecency(conv.last_modified, isRunning, isQueued);
                        if (isRunning) {
                            return (
                                <span className={cn(
                                    "flex items-center gap-1 px-1.5 py-0.2 rounded-md text-[8.5px] font-bold font-mono border animate-pulse",
                                    isSelected && selectedTurnNumber === null
                                        ? "bg-emerald-400 text-emerald-950 border-emerald-300"
                                        : "bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border-emerald-500/40"
                                )} title="Running now">
                                    <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                                    RUNNING
                                </span>
                            );
                        }
                        if (isQueued) {
                            return (
                                <span className={cn(
                                    "flex items-center gap-1 px-1.5 py-0.2 rounded-md text-[8.5px] font-bold font-mono border",
                                    isSelected && selectedTurnNumber === null
                                        ? "bg-amber-300 text-amber-950 border-amber-200"
                                        : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/40"
                                )} title="Queued in FIFO queue">
                                    <Clock className="w-2.5 h-2.5 text-amber-500" />
                                    <span>QUEUED</span>
                                </span>
                            );
                        }
                        return (
                            <span
                                className={cn(
                                    "flex items-center gap-1 px-1.5 py-0.2 rounded-md text-[8.5px] font-mono border transition-colors",
                                    isSelected && selectedTurnNumber === null
                                        ? "bg-violet-950/60 border-violet-400/40 text-violet-200"
                                        : "bg-zinc-900/40 dark:bg-[#120d20]/60 border-zinc-700/40 dark:border-purple-900/40 text-zinc-400"
                                )}
                                title={`Last active: ${recency.formattedTime}`}
                            >
                                <span className={cn("w-1.5 h-1.5 rounded-full shrink-0", recency.dotClass)} />
                                <span className={cn("font-medium", recency.textClass)}>{recency.label}</span>
                            </span>
                        );
                    })()}
                    {enqueuedTurnsCount > 0 && (isRunning || !isQueued || enqueuedTurnsCount > 1) && (
                        <span
                            className={cn(
                                "flex items-center gap-1 px-1.5 py-0.2 rounded-md text-[8.5px] font-bold font-mono border",
                                isSelected && selectedTurnNumber === null
                                    ? "bg-amber-300 text-amber-950 border-amber-200"
                                    : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/40"
                            )}
                            title={`${enqueuedTurnsCount} prompt(s) enqueued in FIFO queue`}
                        >
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>+{enqueuedTurnsCount} queued</span>
                        </span>
                    )}
                </div>
            </div>

            {/* Tier 3: Indented Prompt Subtree */}
            {hasPrompts && isExpanded && conv.prompts && (
                <div className="border-l-2 border-slate-300/60 dark:border-purple-900/50 ml-5 pl-3 space-y-1 mt-1">
                    {conv.prompts.map((turn, tIdx) => {
                        const isLastTurn = tIdx === conv.prompts!.length - 1;
                        const hasImages = turn.media_paths && turn.media_paths.length > 0;
                        const isTurnActive = isSelected && selectedTurnNumber === turn.turn_number;
                        const isTurnRunning = Boolean(turn.is_running);
                        const shouldAttachRef = isTurnActive || (selectedTurnNumber === null && isTurnRunning);
                        const createTurnConv = (): AgmConversationNode => ({
                            ...conv,
                            prompt_preview_200w: turn.preview_200w || turn.prompt_text.slice(0, 200),
                            full_prompt_text: turn.prompt_text,
                            latest_step_summary: `Turn ${turn.turn_number} · ${turn.status}`,
                            is_running: turn.is_running,
                            is_queued: turn.is_queued,
                            status: turn.status,
                            selected_turn_number: turn.turn_number,
                            media_paths: turn.media_paths || [],
                            media_thumbnails: turn.media_thumbnails || [],
                        });

                        return (
                            <div
                                key={`turn-${turn.step_index}-${turn.turn_number}`}
                                ref={shouldAttachRef ? activeTurnRef : undefined}
                                tabIndex={0}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setSelectedTurnNumber(turn.turn_number);
                                    onSelect(createTurnConv(), project);
                                }}
                                onDoubleClick={(e) => {
                                    e.stopPropagation();
                                    const turnConv = createTurnConv();
                                    setSelectedTurnNumber(turn.turn_number);
                                    onSelect(turnConv, project);
                                    onOpenInspector(turnConv, project.repo_path);
                                }}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter' || e.key === ' ') {
                                        e.preventDefault();
                                        e.stopPropagation();
                                        setSelectedTurnNumber(turn.turn_number);
                                        onSelect(createTurnConv(), project);
                                    }
                                }}
                                onContextMenu={(e) => {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    const turnConv = createTurnConv();
                                    onOpenContextMenu?.({
                                        type: 'turn',
                                        x: e.clientX,
                                        y: e.clientY,
                                        conversationId: turnConv.conversation_id,
                                        currentTitle: turnConv.latest_step_summary || `Turn ${turn.turn_number}`,
                                        conversationNode: turnConv,
                                        repoPath: project.repo_path,
                                        instanceId: turnConv.instance_id || project.instance_id,
                                    });
                                }}
                                className={cn(
                                    "group/turn flex items-center justify-between rounded-md px-1.5 py-1 text-xs cursor-pointer transition-colors outline-none focus-visible:ring-1 focus-visible:ring-sky-500/50",
                                    isTurnActive
                                        ? "bg-sky-500/10 border-l-2 border-sky-500 text-sky-400 font-medium"
                                        : "hover:bg-slate-100 dark:hover:bg-[#1a1228]"
                                )}
                                title={`Turn ${turn.turn_number} (${turn.status}) · Click to inspect prompt`}
                            >
                                <div className="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
                                    {/* Branch connector glyph */}
                                    <span className="text-slate-300 dark:text-purple-400 font-mono text-[10px] select-none shrink-0">
                                        {isLastTurn ? '└──' : '├──'}
                                    </span>
                                    {/* Turn ID badge */}
                                    <span className="font-mono text-[9px] px-1 py-0.2 rounded-xs bg-slate-200 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 font-bold shrink-0">
                                        T{turn.turn_number}
                                    </span>
                                    {/* Media chips if images present */}
                                    {hasImages && (
                                        <span
                                            className="inline-flex items-center gap-0.5 px-1 py-0.2 rounded-xs text-[9px] bg-sky-500/10 text-sky-600 dark:text-sky-300 border border-sky-500/30 shrink-0 font-medium"
                                            title={`Attached images (${turn.media_paths.length}):\n${turn.media_paths.join('\n')}`}
                                        >
                                            🖼️ {turn.media_paths.length}
                                        </span>
                                    )}
                                    {/* Prompt preview snippet in Poppins font */}
                                    <span className="font-[Poppins,sans-serif] font-poppins text-[11px] truncate text-slate-700 dark:text-slate-300 flex-1">
                                        {turn.preview_200w || turn.prompt_text || `Prompt Turn #${turn.turn_number}`}
                                    </span>
                                </div>
                                {/* Status Badges */}
                                <div className="flex items-center gap-1 shrink-0">
                                    {turn.is_running || (turn.status && (turn.status.toLowerCase().includes('run') || turn.status.toLowerCase().includes('exec'))) ? (
                                        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[10px] font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 shrink-0 font-bold">
                                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
                                            RUNNING
                                        </span>
                                    ) : (Boolean(turn.is_queued) || (turn.status && turn.status.toLowerCase().includes('queue'))) ? (
                                        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[10px] font-medium bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30 shrink-0 font-bold">
                                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                                            QUEUED
                                        </span>
                                    ) : (
                                        (() => {
                                            const turnRecency = formatRelativeRecency(turn.created_at || turn.timestamp);
                                            return (
                                                <span
                                                    className="inline-flex items-center gap-1 px-1.5 py-0.2 rounded text-[9px] font-mono text-zinc-400 bg-zinc-900/40 dark:bg-[#120d20]/60 border border-zinc-700/40 dark:border-purple-900/40"
                                                    title={`Turn executed: ${turnRecency.formattedTime}`}
                                                >
                                                    <span className={cn("w-1 h-1 rounded-full shrink-0", turnRecency.dotClass)} />
                                                    <span className={cn("font-medium", turnRecency.textClass)}>{turnRecency.label}</span>
                                                </span>
                                            );
                                        })()
                                    )}
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </>
    );
}

