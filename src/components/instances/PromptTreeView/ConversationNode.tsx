import { useState } from 'react';
import { Bot, User, Terminal, Wrench, Clock, ChevronRight, ChevronDown } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode, AgmProjectTreeNode, TierClassificationResult } from './types';
import { isGhostConversation } from './conversationGuards';
import { classifyPromptTier } from './tierClassification';
import { formatSeqBadge } from './formatters';
import { ConversationContextMenu } from './ConversationContextMenu';
import { invalidatePromptTreeCache, renameConversationOptimistic } from './usePromptTreeData';
import { useErrorStore } from '../../../stores/error-store';

export interface ConversationNodeProps {
    conv: AgmConversationNode;
    project: AgmProjectTreeNode;
    isSelected: boolean;
    onSelect: (conv: AgmConversationNode, project: AgmProjectTreeNode) => void;
    onOpenInspector: (conv: AgmConversationNode, repoPath: string) => void;
    onRenameConversation?: (conversationId: string, newTitle: string) => Promise<void> | void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
}

export function getConversationStatus(conv: AgmConversationNode): { isRunning: boolean; isQueued: boolean } {
    const isRunning =
        Boolean(conv.is_running) &&
        !isGhostConversation(conv) &&
        !(conv.prompt_word_count === 0 && (!conv.prompt_preview_200w || !conv.prompt_preview_200w.trim()));
    const isQueued = Boolean(conv.is_queued) || conv.status.toLowerCase().includes('queue');
    return { isRunning, isQueued };
}

export function getTierIcon(tier: TierClassificationResult['tier']) {
    return tier === 'SUBAGENT_INSTRUCTION'
        ? Bot
        : tier === 'SYSTEM_MESSAGE'
          ? Terminal
          : tier === 'TOOL_OUTPUT'
            ? Wrench
            : User;
}

export function ConversationNode({
    conv,
    project,
    isSelected,
    onSelect,
    onOpenInspector,
    onRenameConversation,
    onResendPrompts,
    onEnqueue,
    onRerunConversation,
}: ConversationNodeProps) {
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [editingTitle, setEditingTitle] = useState(conv.title || '');
    const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);
    const [turnMenuPos, setTurnMenuPos] = useState<{
        x: number;
        y: number;
        turnConv: AgmConversationNode;
    } | null>(null);
    const [selectedTurnNumber, setSelectedTurnNumber] = useState<number | null>(null);
    const [isExpanded, setIsExpanded] = useState<boolean>(Boolean(conv.is_running));

    const { isRunning, isQueued } = getConversationStatus(conv);
    const promptCategory = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);
    const IconComponent = getTierIcon(promptCategory.tier);
    const hasPrompts = Boolean(conv.prompts && conv.prompts.length > 0);

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
        renameConversationOptimistic(conv.conversation_id, trimmed, targetInst);

        try {
            if (onRenameConversation) {
                await onRenameConversation(conv.conversation_id, trimmed);
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
            renameConversationOptimistic(conv.conversation_id, prevTitle || '', targetInst);
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
                    setSelectedTurnNumber(null);
                    onSelect(conv, project);
                }}
                onDoubleClick={() => onOpenInspector(conv, project.repo_path)}
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
                    setContextMenuPos({ x: e.clientX, y: e.clientY });
                }}
                className={cn(
                    'group flex items-center justify-between rounded-[5px] px-2 py-1.5 text-xs cursor-pointer transition-colors relative outline-none focus-visible:ring-1 focus-visible:ring-cyan-500/50',
                    isSelected && selectedTurnNumber === null
                        ? 'bg-blue-600 text-white font-medium shadow-2xs'
                        : isSelected
                          ? 'bg-sky-500/10 border-l-2 border-sky-500 text-sky-400 font-medium'
                          : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#171025]'
                )}
                title={`${conv.title || 'Conversation'} · ${conv.step_count || 1} steps · Click to view (F2 to rename)`}
            >
                <div className="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
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
                    <span title={promptCategory.roleBadge || promptCategory.tier} className="shrink-0 flex items-center">
                        <IconComponent
                            className={cn('h-3.5 w-3.5 shrink-0', isSelected ? 'text-white' : (promptCategory.tier === 'SUBAGENT_INSTRUCTION' ? 'text-purple-500' : promptCategory.tier === 'SYSTEM_MESSAGE' ? 'text-zinc-500' : promptCategory.tier === 'TOOL_OUTPUT' ? 'text-amber-500' : 'text-sky-500'))}
                        />
                    </span>
                    <span
                        className={cn(
                            'text-[9px] font-mono px-1 py-0.2 rounded-[3px] shrink-0 font-medium whitespace-nowrap',
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
                                "px-1.5 py-0.2 rounded-full text-[8.5px] font-mono font-bold shrink-0 border",
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
                                setContextMenuPos({ x: e.clientX, y: e.clientY });
                            }}
                            className="truncate text-[11px] cursor-text hover:underline hover:decoration-dotted"
                            title={`${conv.title || conv.short_id || conv.conversation_id.slice(0, 8)} (Double-click to rename)`}
                        >
                            {conv.title || conv.short_id || conv.conversation_id.slice(0, 8)}
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                    {isRunning && (
                        <span className={cn(
                            "flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono border animate-pulse",
                            isSelected
                                ? "bg-emerald-400 text-emerald-950 border-emerald-300"
                                : "bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border-emerald-500/40"
                        )}>
                            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                            RUNNING
                        </span>
                    )}
                    {isQueued && !isRunning && (
                        <span className={cn(
                            "flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono border",
                            isSelected
                                ? "bg-amber-300 text-amber-950 border-amber-200"
                                : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/40"
                        )}>
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>QUEUED</span>
                        </span>
                    )}
                </div>
            </div>

            {/* Tier 3: Indented Prompt Subtree */}
            {hasPrompts && isExpanded && conv.prompts && (
                <div className="border-l border-slate-200 dark:border-purple-900/40 ml-4 pl-2 space-y-1 mt-1">
                    {conv.prompts.map((turn, tIdx) => {
                        const isLastTurn = tIdx === conv.prompts!.length - 1;
                        const hasImages = turn.media_paths && turn.media_paths.length > 0;
                        const isTurnActive = isSelected && selectedTurnNumber === turn.turn_number;
                        const createTurnConv = (): AgmConversationNode => ({
                            ...conv,
                            prompt_preview_200w: turn.preview_200w || turn.prompt_text.slice(0, 200),
                            full_prompt_text: turn.prompt_text,
                            latest_step_summary: `Turn ${turn.turn_number} · ${turn.status}`,
                            is_running: turn.is_running,
                            is_queued: turn.is_queued,
                            status: turn.status,
                        });

                        return (
                            <div
                                key={`turn-${turn.step_index}-${turn.turn_number}`}
                                tabIndex={0}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setSelectedTurnNumber(turn.turn_number);
                                    onSelect(createTurnConv(), project);
                                }}
                                onDoubleClick={(e) => {
                                    e.stopPropagation();
                                    onOpenInspector(createTurnConv(), project.repo_path);
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
                                    setTurnMenuPos({ x: e.clientX, y: e.clientY, turnConv: createTurnConv() });
                                }}
                                className={cn(
                                    "group/turn flex items-center justify-between rounded-[4px] px-1.5 py-1 text-xs cursor-pointer transition-colors outline-none focus-visible:ring-1 focus-visible:ring-sky-500/50",
                                    isTurnActive
                                        ? "bg-sky-500/10 border-l-2 border-sky-500 text-sky-400 font-medium"
                                        : "hover:bg-slate-100 dark:hover:bg-[#1a1228]"
                                )}
                                title={`Turn ${turn.turn_number} (${turn.status}) · Click to inspect prompt`}
                            >
                                <div className="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
                                    {/* Branch connector glyph */}
                                    <span className="text-slate-300 dark:text-purple-400 font-mono text-[10px] select-none shrink-0">
                                        {isLastTurn ? '└──' : '↳'}
                                    </span>
                                    {/* Turn ID badge */}
                                    <span className="font-mono text-[9px] px-1 py-0.2 rounded bg-slate-200 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 font-bold shrink-0">
                                        T{turn.turn_number}
                                    </span>
                                    {/* Media chips if images present */}
                                    {hasImages && (
                                        <span
                                            className="inline-flex items-center gap-0.5 px-1 py-0.2 rounded text-[9px] bg-sky-500/10 text-sky-600 dark:text-sky-300 border border-sky-500/30 shrink-0 font-medium"
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
                                    {turn.is_running ? (
                                        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 shrink-0 font-bold">
                                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
                                            RUNNING
                                        </span>
                                    ) : turn.is_queued ? (
                                        <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30 shrink-0 font-bold">
                                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                                            QUEUED
                                        </span>
                                    ) : (
                                        <span className="text-slate-400 dark:text-zinc-500 text-[9px] font-medium shrink-0">
                                            COMPLETED
                                        </span>
                                    )}
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}

            {turnMenuPos && (
                <ConversationContextMenu
                    x={turnMenuPos.x}
                    y={turnMenuPos.y}
                    conversationId={turnMenuPos.turnConv.conversation_id}
                    currentTitle={turnMenuPos.turnConv.latest_step_summary || `Turn ${turnMenuPos.turnConv.seq_code}`}
                    conversationNode={turnMenuPos.turnConv}
                    repoPath={project.repo_path}
                    instanceId={turnMenuPos.turnConv.instance_id || project.instance_id}
                    onClose={() => setTurnMenuPos(null)}
                    onRename={() => {
                        setTurnMenuPos(null);
                    }}
                    onInspect={() => {
                        setTurnMenuPos(null);
                        onOpenInspector(turnMenuPos.turnConv, project.repo_path);
                    }}
                    onResendPrompts={onResendPrompts}
                    onEnqueue={onEnqueue}
                    onRerunConversation={onRerunConversation}
                />
            )}

            {contextMenuPos && (
                <ConversationContextMenu
                    x={contextMenuPos.x}
                    y={contextMenuPos.y}
                    conversationId={conv.conversation_id}
                    currentTitle={conv.title || conv.short_id || ''}
                    conversationNode={conv}
                    repoPath={project.repo_path}
                    instanceId={conv.instance_id || project.instance_id}
                    onClose={() => setContextMenuPos(null)}
                    onRename={() => {
                        setContextMenuPos(null);
                        setIsEditingTitle(true);
                        setEditingTitle(conv.title || '');
                    }}
                    onInspect={() => {
                        setContextMenuPos(null);
                        onOpenInspector(conv, project.repo_path);
                    }}
                    onResendPrompts={onResendPrompts}
                    onEnqueue={onEnqueue}
                    onRerunConversation={onRerunConversation}
                />
            )}
        </>
    );
}

