import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import {
    Pencil,
    FileSearch,
    Copy,
    Check,
    RotateCw,
    ShieldCheck,
    CopyCheck,
    FileJson,
    Database,
    FileText,
    ChevronRight,
    ChevronDown,
    FolderInput,
    Repeat,
    ListPlus,
    Pin,
    Trash2,
} from 'lucide-react';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import type { AgmConversationNode } from './types';
import {
    dispatchMultiPromptResend,
    handleVerifyLastPromptsAndEnqueue,
    handleCloneAndCompactConversation,
    exportConversationAsJson,
    exportConversationAsYaml,
    exportConversationAsSqlite,
    extractConversationPromptsList,
    dispatchRerunMessages,
} from './usePromptTreePromptActions';
import { enqueuePrompt } from '../../../services/instanceService';
import { cn } from '../../../utils/cn';

export interface ConversationContextMenuProps {
    x: number;
    y: number;
    conversationId: string;
    currentTitle: string;
    onClose: () => void;
    onRename?: () => void;
    onInspect: () => void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerun?: () => void;
    onCopyPrompt?: () => void;
    onVerifyLastPrompts?: (count: number) => void;
    onCloneAndCompact?: () => void;
    onImportConversation?: (convoId: string) => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
    onExportJson?: () => void;
    onExportSqlite?: () => void;
    onExportYaml?: () => void;
    conversationNode?: AgmConversationNode;
    repoPath?: string;
    instanceId?: string;
    isPinned?: boolean;
    onTogglePin?: () => void;
    onPinConversation?: (convoId: string) => void;
    onDelete?: () => void;
    onDeleteConversation?: (convoId: string) => void;
}

export function ConversationContextMenu({
    x,
    y,
    conversationId,
    currentTitle,
    onClose,
    onRename,
    onInspect,
    onResendPrompts,
    onEnqueue,
    onRerun,
    onCopyPrompt,
    onVerifyLastPrompts,
    onCloneAndCompact,
    onImportConversation,
    onRerunConversation,
    onExportJson,
    onExportSqlite,
    onExportYaml,
    conversationNode,
    repoPath,
    instanceId,
    isPinned: isPinnedProp,
    onTogglePin,
    onPinConversation,
    onDelete,
    onDeleteConversation,
}: ConversationContextMenuProps) {
    const menuRef = useRef<HTMLDivElement>(null);
    const [isIdCopied, setIsIdCopied] = useState(false);
    const [isPromptCopied, setIsPromptCopied] = useState(false);
    const [isEnqueueing, setIsEnqueueing] = useState(false);
    const [isRerunning, setIsRerunning] = useState(false);
    const [isResendSubmenuOpen, setIsResendSubmenuOpen] = useState(false);
    const [isCustomNDialogOpen, setIsCustomNDialogOpen] = useState(false);
    const [customNCount, setCustomNCount] = useState<number>(5);

    const [isPinned, setIsPinned] = useState<boolean>(() => {
        if (typeof isPinnedProp === 'boolean') return isPinnedProp;
        try {
            const raw = localStorage.getItem(`agm_pinned_conversations_${instanceId || 'default'}`);
            if (raw) {
                const list = JSON.parse(raw);
                return Array.isArray(list) && list.includes(conversationId);
            }
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ConversationContextMenu.initIsPinned',
                triggerAction: 'read_pinned_conversations',
            });
        }
        return false;
    });

    const handleTogglePin = (e: React.MouseEvent) => {
        e.stopPropagation();
        if (onTogglePin) {
            onTogglePin();
            onClose();
            return;
        }
        if (onPinConversation) {
            onPinConversation(conversationId);
            onClose();
            return;
        }
        try {
            const storageKey = `agm_pinned_conversations_${instanceId || 'default'}`;
            const raw = localStorage.getItem(storageKey);
            const list: string[] = raw ? JSON.parse(raw) : [];
            const nextList = list.includes(conversationId)
                ? list.filter((id) => id !== conversationId)
                : [...list, conversationId];
            localStorage.setItem(storageKey, JSON.stringify(nextList));
            setIsPinned(!isPinned);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ConversationContextMenu.handleTogglePin',
                triggerAction: 'persist_pinned_conversations',
            });
        }
        onClose();
    };

    const handleDelete = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (onDelete) {
            onDelete();
            onClose();
            return;
        }
        if (onDeleteConversation) {
            onDeleteConversation(conversationId);
            onClose();
            return;
        }
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('delete_conversation_record', {
                conversationId,
                instanceId: instanceId || undefined,
            });
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ConversationContextMenu.handleDelete',
                triggerAction: 'delete_conversation',
            });
        }
        onClose();
    };

    const left = Math.max(8, Math.min(x, window.innerWidth - 260));
    const top = Math.max(8, Math.min(y, window.innerHeight - 480));

    useEffect(() => {
        const mountTime = Date.now();

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        const handleClickOutside = (e: MouseEvent) => {
            if (Date.now() - mountTime < 100) return;
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        const handleScroll = (e: Event) => {
            if (Date.now() - mountTime < 150) return;
            if (menuRef.current && menuRef.current.contains(e.target as Node)) return;
            onClose();
        };

        window.addEventListener('keydown', handleKeyDown);
        document.addEventListener('mousedown', handleClickOutside);
        window.addEventListener('scroll', handleScroll, true);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            document.removeEventListener('mousedown', handleClickOutside);
            window.removeEventListener('scroll', handleScroll, true);
        };
    }, [onClose]);

    const handleCopyId = async (e: React.MouseEvent) => {
        e.stopPropagation();
        await copyToClipboard(conversationId);
        setIsIdCopied(true);
        setTimeout(() => {
            onClose();
        }, 400);
    };

    const handleCopyPromptText = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (onCopyPrompt) {
            onCopyPrompt();
            setIsPromptCopied(true);
            setTimeout(() => onClose(), 400);
            return;
        }
        const textToCopy = conversationNode
            ? (conversationNode.full_prompt_text || conversationNode.prompt_preview_200w || currentTitle)
            : currentTitle;
        if (!textToCopy) return;
        try {
            await copyToClipboard(textToCopy);
            setIsPromptCopied(true);
            setTimeout(() => {
                onClose();
            }, 400);
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'ConversationContextMenu.handleCopyPromptText',
                    triggerAction: 'copy_prompt_text',
                }
            );
        }
    };

    const handleDoEnqueue = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (isEnqueueing) return;
        if (onEnqueue) {
            onEnqueue();
            onClose();
            return;
        }
        setIsEnqueueing(true);
        try {
            const promptText = conversationNode
                ? (conversationNode.full_prompt_text || conversationNode.prompt_preview_200w || currentTitle)
                : currentTitle;
            if (promptText) {
                await enqueuePrompt(
                    instanceId || 'default',
                    repoPath || '',
                    promptText,
                    conversationId
                );
            }
            onClose();
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'ConversationContextMenu.handleDoEnqueue',
                    triggerAction: 'enqueue_prompt',
                }
            );
        } finally {
            setIsEnqueueing(false);
        }
    };

    const handleDoRerun = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (isRerunning) return;
        if (onRerun) {
            onRerun();
            onClose();
            return;
        }
        if (onRerunConversation) {
            onRerunConversation(conversationId, 'is-it-done');
            onClose();
            return;
        }
        setIsRerunning(true);
        try {
            const originalPrompt = conversationNode
                ? (conversationNode.full_prompt_text || conversationNode.prompt_preview_200w || currentTitle)
                : currentTitle;
            await dispatchRerunMessages({
                targetInstId: instanceId || 'default',
                repoPath: repoPath || '',
                convId: conversationId,
                count: 1,
                messageOption: 'is-it-done',
                originalPrompt,
                source: 'ConversationContextMenu.handleDoRerun',
            });
            onClose();
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'ConversationContextMenu.handleDoRerun',
                    triggerAction: 'rerun_conversation',
                }
            );
        } finally {
            setIsRerunning(false);
        }
    };

    const handleDoResend = async (count: number | 'all') => {
        onClose();
        if (onResendPrompts) {
            onResendPrompts(count);
            return;
        }
        try {
            const prompts = conversationNode ? extractConversationPromptsList(conversationNode) : [];
            await dispatchMultiPromptResend({
                conversationId,
                instanceId: instanceId || 'default',
                repoPath: repoPath || '',
                count,
                prompts,
            });
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'ConversationContextMenu.handleDoResend',
                    triggerAction: 'resend_prompts',
                }
            );
        }
    };

    const handleDoVerify = async () => {
        onClose();
        if (onVerifyLastPrompts) {
            onVerifyLastPrompts(10);
            return;
        }
        try {
            const prompts = conversationNode ? extractConversationPromptsList(conversationNode) : [];
            await handleVerifyLastPromptsAndEnqueue({
                instanceId: instanceId || 'default',
                repoPath: repoPath || '',
                conversationId,
                convTitle: currentTitle,
                count: 10,
                prompts,
            });
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'ConversationContextMenu.handleDoVerify',
                    triggerAction: 'verify_last_prompts',
                }
            );
        }
    };

    const handleDoCloneCompact = async () => {
        onClose();
        if (onCloneAndCompact) {
            onCloneAndCompact();
            return;
        }
        try {
            if (conversationNode) {
                await handleCloneAndCompactConversation({
                    instanceId: instanceId || 'default',
                    repoPath: repoPath || '',
                    sourceConv: conversationNode,
                });
            }
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'ConversationContextMenu.handleDoCloneCompact',
                    triggerAction: 'clone_compact',
                }
            );
        }
    };

    const handleDoExportJson = () => {
        onClose();
        if (onExportJson) {
            onExportJson();
            return;
        }
        if (conversationNode) {
            exportConversationAsJson(conversationNode, repoPath);
        } else {
            exportConversationAsJson(
                {
                    seq_id: 1,
                    seq_code: 'C001',
                    gitmap_seq_code: '',
                    conversation_id: conversationId,
                    short_id: conversationId.slice(0, 8),
                    title: currentTitle,
                    status: 'idle',
                    is_running: false,
                    step_count: 1,
                    instance_id: instanceId || 'default',
                    prompt_preview_200w: '',
                    prompt_word_count: 0,
                    last_modified: new Date().toISOString(),
                },
                repoPath
            );
        }
    };

    const handleDoExportYaml = () => {
        onClose();
        if (onExportYaml) {
            onExportYaml();
            return;
        }
        if (conversationNode) {
            exportConversationAsYaml(conversationNode, repoPath);
        } else {
            exportConversationAsYaml(
                {
                    seq_id: 1,
                    seq_code: 'C001',
                    gitmap_seq_code: '',
                    conversation_id: conversationId,
                    short_id: conversationId.slice(0, 8),
                    title: currentTitle,
                    status: 'idle',
                    is_running: false,
                    step_count: 1,
                    instance_id: instanceId || 'default',
                    prompt_preview_200w: '',
                    prompt_word_count: 0,
                    last_modified: new Date().toISOString(),
                },
                repoPath
            );
        }
    };

    const handleDoExportSqlite = async () => {
        onClose();
        if (onExportSqlite) {
            onExportSqlite();
            return;
        }
        if (conversationNode) {
            await exportConversationAsSqlite(conversationNode);
        }
    };

    return createPortal(
        <div
            ref={menuRef}
            className="fixed z-[100050] min-w-[240px] rounded-xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] py-1.5 shadow-2xl text-xs text-slate-800 dark:text-slate-200 select-none font-poppins animate-in fade-in zoom-in-95 duration-100"
            style={{ top, left }}
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => {
                e.preventDefault();
                e.stopPropagation();
            }}
        >
            {/* Header info */}
            <div className="px-3 py-1 mb-1 border-b border-slate-100 dark:border-[#2b1b42] text-[10px] text-slate-400 dark:text-slate-500 truncate max-w-[240px] font-mono">
                {currentTitle || conversationId.slice(0, 8)}
            </div>

            {/* Core Action 1: Inspect Transcript */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-cyan-500/10 hover:text-cyan-600 dark:hover:text-cyan-400 cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onInspect();
                    onClose();
                }}
                title="Inspect conversation transcript in full inspector"
            >
                <div className="flex items-center gap-2.5">
                    <FileSearch className="w-3.5 h-3.5 text-cyan-500 shrink-0" />
                    <span className="font-medium">Inspect Transcript</span>
                </div>
                <span className="text-[9px] font-mono text-cyan-500 font-bold">Inspect</span>
            </button>

            {/* Core Action 2: Resend Prompts Submenu */}
            <div className="relative">
                <button
                    type="button"
                    className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-emerald-500/10 hover:text-emerald-600 dark:hover:text-emerald-400 cursor-pointer"
                    onClick={(e) => {
                        e.stopPropagation();
                        setIsResendSubmenuOpen((prev) => !prev);
                    }}
                >
                    <div className="flex items-center gap-2.5">
                        <RotateCw className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                        <span className="font-medium">Resend Prompts</span>
                    </div>
                    {isResendSubmenuOpen ? (
                        <ChevronDown className="w-3.5 h-3.5 text-slate-400" />
                    ) : (
                        <ChevronRight className="w-3.5 h-3.5 text-slate-400" />
                    )}
                </button>

                {isResendSubmenuOpen && (
                    <div className="mx-2 my-1 pl-4 border-l-2 border-emerald-500/30 space-y-0.5 animate-in fade-in duration-100">
                        <button
                            type="button"
                            className="flex w-full items-center justify-between px-2 py-1 text-left rounded hover:bg-emerald-500/10 text-[11px] cursor-pointer"
                            onClick={() => void handleDoResend('all')}
                        >
                            <span>Resend All Prompts</span>
                            <span className="text-[9px] font-mono text-slate-400">All</span>
                        </button>
                        <button
                            type="button"
                            className="flex w-full items-center justify-between px-2 py-1 text-left rounded hover:bg-emerald-500/10 text-[11px] cursor-pointer"
                            onClick={() => void handleDoResend(5)}
                        >
                            <span>Resend Last 5 Prompts</span>
                            <span className="text-[9px] font-mono text-slate-400">×5</span>
                        </button>
                        <button
                            type="button"
                            className="flex w-full items-center justify-between px-2 py-1 text-left rounded hover:bg-emerald-500/10 text-[11px] cursor-pointer"
                            onClick={() => void handleDoResend(10)}
                        >
                            <span>Resend Last 10 Prompts</span>
                            <span className="text-[9px] font-mono text-slate-400">×10</span>
                        </button>

                        {/* Custom N Prompts picker */}
                        {!isCustomNDialogOpen ? (
                            <button
                                type="button"
                                className="flex w-full items-center justify-between px-2 py-1 text-left rounded hover:bg-emerald-500/10 text-[11px] cursor-pointer text-emerald-600 dark:text-emerald-400 font-medium"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setIsCustomNDialogOpen(true);
                                }}
                            >
                                <span>Resend Custom N Prompts...</span>
                            </button>
                        ) : (
                            <div className="p-1.5 rounded bg-slate-50 dark:bg-purple-950/30 border border-emerald-500/30 flex items-center gap-1.5 text-[11px]">
                                <span className="text-slate-400 shrink-0">Count:</span>
                                <input
                                    type="number"
                                    min={1}
                                    max={100}
                                    value={customNCount}
                                    onChange={(e) => setCustomNCount(Math.max(1, Math.min(100, Number(e.target.value) || 1)))}
                                    className="w-12 px-1 py-0.5 text-center text-xs font-mono font-bold rounded bg-white dark:bg-[#120a1f] border border-slate-300 dark:border-purple-500/40 text-slate-900 dark:text-white"
                                    onClick={(e) => e.stopPropagation()}
                                    onKeyDown={(e) => {
                                        e.stopPropagation();
                                        if (e.key === 'Enter') {
                                            void handleDoResend(customNCount);
                                        }
                                    }}
                                />
                                <button
                                    type="button"
                                    onClick={() => void handleDoResend(customNCount)}
                                    className="px-2 py-0.5 rounded bg-emerald-600 text-white font-semibold text-[10px] hover:bg-emerald-500 cursor-pointer"
                                >
                                    Send
                                </button>
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* Core Action 3: Enqueue to FIFO Queue */}
            <button
                type="button"
                disabled={isEnqueueing}
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-amber-500/10 hover:text-amber-600 dark:hover:text-amber-400 cursor-pointer disabled:opacity-50"
                onClick={handleDoEnqueue}
                title="Enqueue prompt into FIFO background scheduler queue"
            >
                <div className="flex items-center gap-2.5">
                    <ListPlus className={cn("w-3.5 h-3.5 text-amber-500 shrink-0", isEnqueueing && "animate-pulse")} />
                    <span className="font-medium">{isEnqueueing ? 'Enqueueing…' : 'Enqueue to FIFO Queue'}</span>
                </div>
                <span className="text-[9px] font-mono text-amber-500 font-bold">Queue</span>
            </button>

            {/* Core Action 4: Rerun Nudge Action */}
            <button
                type="button"
                disabled={isRerunning}
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-sky-500/10 hover:text-sky-600 dark:hover:text-sky-400 cursor-pointer disabled:opacity-50"
                onClick={handleDoRerun}
                title="Dispatch a fast rerun nudge (is-it-done) to this conversation"
            >
                <div className="flex items-center gap-2.5">
                    <Repeat className={cn("w-3.5 h-3.5 text-sky-500 shrink-0", isRerunning && "animate-spin")} />
                    <span className="font-medium">{isRerunning ? 'Rerunning…' : 'Rerun Nudge (Is It Done?)'}</span>
                </div>
                <span className="text-[9px] font-mono text-sky-400 font-bold">Rerun</span>
            </button>

            {/* Core Action 5: Copy Prompt Text */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={handleCopyPromptText}
                title="Copy prompt text of this conversation or turn"
            >
                <div className="flex items-center gap-2.5">
                    {isPromptCopied ? (
                        <Check className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                    ) : (
                        <Copy className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400 shrink-0" />
                    )}
                    <span>{isPromptCopied ? 'Copied Prompt!' : 'Copy Prompt Text'}</span>
                </div>
                <span className="text-[9px] font-mono text-slate-400">Copy</span>
            </button>

            {/* Core Action 5b: Copy Conversation ID */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={handleCopyId}
                title="Copy conversation UUID to clipboard"
            >
                <div className="flex items-center gap-2.5">
                    {isIdCopied ? (
                        <Check className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                    ) : (
                        <Copy className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400 shrink-0" />
                    )}
                    <span>{isIdCopied ? 'Copied ID!' : 'Copy Conversation ID'}</span>
                </div>
                <span className="text-[9px] font-mono text-slate-400">UUID</span>
            </button>

            <div className="my-1 border-t border-slate-100 dark:border-[#2b1b42]" />

            {/* Pin / Unpin Conversation */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-amber-500/10 hover:text-amber-600 dark:hover:text-amber-400 cursor-pointer"
                onClick={handleTogglePin}
                title={isPinned ? 'Unpin conversation from top' : 'Pin conversation to top'}
            >
                <div className="flex items-center gap-2.5">
                    <Pin className={cn("w-3.5 h-3.5 text-amber-500 shrink-0", isPinned && "fill-current")} />
                    <span className="font-medium">{isPinned ? 'Unpin Conversation' : 'Pin Conversation to Top'}</span>
                </div>
                <span className="text-[9px] font-mono text-amber-500 font-bold">{isPinned ? 'Pinned' : 'Pin'}</span>
            </button>

            {/* Rename Conversation (F2) */}
            {onRename && (
                <button
                    type="button"
                    className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-purple-500/10 hover:text-purple-600 dark:hover:text-purple-400 cursor-pointer"
                    onClick={(e) => {
                        e.stopPropagation();
                        onRename();
                        onClose();
                    }}
                >
                    <div className="flex items-center gap-2.5">
                        <Pencil className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                        <span>Rename Conversation</span>
                    </div>
                    <kbd className="text-[9px] font-mono font-semibold px-1 py-0.2 rounded bg-slate-100 dark:bg-purple-950/40 text-slate-400 border border-slate-200 dark:border-purple-500/20">F2</kbd>
                </button>
            )}

            {/* Verify Last 10 Prompts & Enqueue */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-purple-500/10 hover:text-purple-600 dark:hover:text-purple-400 cursor-pointer"
                onClick={() => void handleDoVerify()}
                title="Synthesize 5-Chapter verification audit of the last 10 prompts and enqueue into instance"
            >
                <div className="flex items-center gap-2.5">
                    <ShieldCheck className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                    <span>Verify Last 10 & Enqueue</span>
                </div>
                <span className="text-[9px] font-mono text-purple-400">Audit</span>
            </button>

            {/* Clone Conversation & Compact Memory */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-blue-500/10 hover:text-blue-600 dark:hover:text-blue-400 cursor-pointer"
                onClick={() => void handleDoCloneCompact()}
                title="Synthesize memory compaction context from conversation and initialize a fresh session"
            >
                <div className="flex items-center gap-2.5">
                    <CopyCheck className="w-3.5 h-3.5 text-blue-500 shrink-0" />
                    <span>Clone & Compact Memory</span>
                </div>
                <span className="text-[9px] font-mono text-blue-400">Handoff</span>
            </button>

            <div className="my-1 border-t border-slate-100 dark:border-[#2b1b42]" />

            {/* Section: Import & Export Triggers */}
            <div className="px-3 pt-0.5 text-[9px] font-mono uppercase text-slate-400 dark:text-slate-500 tracking-wider">
                Import & Export
            </div>

            {/* Import into Conversation */}
            {onImportConversation && (
                <button
                    type="button"
                    className="flex w-full items-center justify-between px-3 py-1 text-left transition-colors hover:bg-sky-500/10 hover:text-sky-600 dark:hover:text-sky-400 cursor-pointer"
                    onClick={(e) => {
                        e.stopPropagation();
                        onImportConversation(conversationId);
                        onClose();
                    }}
                >
                    <div className="flex items-center gap-2.5">
                        <FolderInput className="w-3.5 h-3.5 text-sky-500 shrink-0" />
                        <span>Import into Conversation...</span>
                    </div>
                    <span className="text-[9px] font-mono text-slate-400">Import</span>
                </button>
            )}

            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={handleDoExportJson}
            >
                <div className="flex items-center gap-2.5">
                    <FileJson className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                    <span>Export Conversation as JSON...</span>
                </div>
                <span className="text-[9px] font-mono text-slate-400">.json</span>
            </button>

            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={() => void handleDoExportSqlite()}
            >
                <div className="flex items-center gap-2.5">
                    <Database className="w-3.5 h-3.5 text-indigo-500 shrink-0" />
                    <span>Export Conversation as SQLite .db...</span>
                </div>
                <span className="text-[9px] font-mono text-slate-400">.db</span>
            </button>

            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={handleDoExportYaml}
            >
                <div className="flex items-center gap-2.5">
                    <FileText className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                    <span>Export Conversation as YAML...</span>
                </div>
                <span className="text-[9px] font-mono text-slate-400">.yaml</span>
            </button>

            <div className="my-1 border-t border-slate-100 dark:border-[#2b1b42]" />

            {/* Delete Conversation */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-rose-500/10 hover:text-rose-600 dark:hover:text-rose-400 cursor-pointer text-rose-600 dark:text-rose-400"
                onClick={handleDelete}
                title="Delete this conversation session"
            >
                <div className="flex items-center gap-2.5">
                    <Trash2 className="w-3.5 h-3.5 text-rose-500 shrink-0" />
                    <span className="font-medium">Delete Conversation</span>
                </div>
                <span className="text-[9px] font-mono text-rose-400 font-bold">Delete</span>
            </button>
        </div>,
        document.body
    );
}
