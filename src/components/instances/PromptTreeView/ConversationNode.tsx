import { useState } from 'react';
import { Bot, User, Terminal, Wrench, Clock } from 'lucide-react';
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
}: ConversationNodeProps) {
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [editingTitle, setEditingTitle] = useState(conv.title || '');
    const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);

    const { isRunning, isQueued } = getConversationStatus(conv);
    const promptCategory = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);
    const IconComponent = getTierIcon(promptCategory.tier);

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
                onClick={() => onSelect(conv, project)}
                onDoubleClick={() => onOpenInspector(conv, project.repo_path)}
                onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    setContextMenuPos({ x: e.clientX, y: e.clientY });
                }}
                className={cn(
                    'group flex items-center justify-between rounded-[5px] px-2 py-1.5 text-xs cursor-pointer transition-colors relative',
                    isSelected
                        ? 'bg-blue-600 text-white font-medium shadow-2xs'
                        : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#171025]'
                )}
                title={`${conv.title || 'Conversation'} · ${conv.step_count || 1} steps · Click to view`}
            >
                <div className="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
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
                        title={conv.short_id ? `GitMap SHA: ${conv.short_id}` : (conv.gitmap_seq_code ? `GitMap: ${conv.gitmap_seq_code}` : undefined)}
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

            {contextMenuPos && (
                <ConversationContextMenu
                    x={contextMenuPos.x}
                    y={contextMenuPos.y}
                    conversationId={conv.conversation_id}
                    currentTitle={conv.title || conv.short_id || ''}
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
                />
            )}
        </>
    );
}

