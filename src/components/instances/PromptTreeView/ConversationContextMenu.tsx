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
} from './usePromptTreePromptActions';

export interface ConversationContextMenuProps {
    x: number;
    y: number;
    conversationId: string;
    currentTitle: string;
    onClose: () => void;
    onRename: () => void;
    onInspect: () => void;
    onResendPrompts?: (count: number | 'all') => void;
    onVerifyLastPrompts?: (count: number) => void;
    onCloneAndCompact?: () => void;
    onExportJson?: () => void;
    onExportSqlite?: () => void;
    onExportYaml?: () => void;
    conversationNode?: AgmConversationNode;
    repoPath?: string;
    instanceId?: string;
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
    onVerifyLastPrompts,
    onCloneAndCompact,
    onExportJson,
    onExportSqlite,
    onExportYaml,
    conversationNode,
    repoPath,
    instanceId,
}: ConversationContextMenuProps) {
    const menuRef = useRef<HTMLDivElement>(null);
    const [copied, setCopied] = useState(false);
    const [isResendSubmenuOpen, setIsResendSubmenuOpen] = useState(false);
    const [isCustomNDialogOpen, setIsCustomNDialogOpen] = useState(false);
    const [customNCount, setCustomNCount] = useState<number>(5);

    const left = Math.max(8, Math.min(x, window.innerWidth - 260));
    const top = Math.max(8, Math.min(y, window.innerHeight - 380));

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        const handleClickOutside = (e: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        const handleScroll = () => {
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
        setCopied(true);
        setTimeout(() => {
            onClose();
        }, 400);
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
            className="fixed z-[10000] min-w-[230px] rounded-xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] py-1.5 shadow-2xl text-xs text-slate-800 dark:text-slate-200 select-none font-poppins animate-in fade-in zoom-in-95 duration-100"
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

            {/* Rename Conversation (Triple-trigger trigger 3, with F2 hint) */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-cyan-500/10 hover:text-cyan-600 dark:hover:text-cyan-400 cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onRename();
                    onClose();
                }}
            >
                <div className="flex items-center gap-2.5">
                    <Pencil className="w-3.5 h-3.5 text-cyan-500 shrink-0" />
                    <span>Rename Conversation</span>
                </div>
                <kbd className="text-[9px] font-mono font-semibold px-1 py-0.2 rounded bg-slate-100 dark:bg-purple-950/40 text-slate-400 border border-slate-200 dark:border-purple-500/20">F2</kbd>
            </button>

            {/* Resend Prompts Submenu */}
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
                        <span>Resend Prompts</span>
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

            {/* Section: Export Triggers */}
            <div className="px-3 pt-0.5 text-[9px] font-mono uppercase text-slate-400 dark:text-slate-500 tracking-wider">
                Export
            </div>

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

            {/* Inspect Transcript */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onInspect();
                    onClose();
                }}
            >
                <FileSearch className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400 shrink-0" />
                <span>Inspect Transcript</span>
            </button>

            {/* Copy Conversation ID */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={handleCopyId}
            >
                <div className="flex items-center gap-2.5">
                    {copied ? (
                        <Check className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                    ) : (
                        <Copy className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400 shrink-0" />
                    )}
                    <span>{copied ? 'Copied ID!' : 'Copy Conversation ID'}</span>
                </div>
            </button>
        </div>,
        document.body
    );
}
