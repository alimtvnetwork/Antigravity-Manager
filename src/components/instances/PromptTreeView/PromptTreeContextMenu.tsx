import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import {
    Pencil,
    FileSearch,
    Copy,
    Check,
    RotateCw,
    Repeat,
    ListPlus,
    Pin,
    Trash2,
    Archive,
    ArchiveRestore,
    FileJson,
    Database,
    FileText,
} from 'lucide-react';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';
import {
    exportConversationAsJson,
    exportConversationAsYaml,
    exportConversationAsSqlite,
    loadPersistedRerunMessageOption,
    loadPersistedRerunCount,
} from './usePromptTreePromptActions';
import { copyPromptWithRichImages } from './clipboard';
import { stripImagesFromPrompt } from './formatters';

export type TreeContextMenuType = 'project' | 'conversation' | 'turn';

export interface PromptTreeContextMenuProps {
    type?: TreeContextMenuType;
    x: number;
    y: number;
    onClose: () => void;

    // Project props
    project?: AgmProjectTreeNode;
    projectId?: string;
    projectName?: string;
    isProjectPinned?: boolean;
    isProjectArchived?: boolean;
    onRefreshProject?: () => void;
    onTogglePinProject?: () => void;
    onToggleArchiveProject?: () => void;
    onExportProjectJson?: () => void;
    onExportProjectSqlite?: () => void;
    onExportProjectYaml?: () => void;

    // Conversation & Turn props
    conversationId?: string;
    currentTitle?: string;
    conversationNode?: AgmConversationNode;
    repoPath?: string;
    instanceId?: string;
    isPinned?: boolean;
    onRename?: () => void;
    onInspect?: () => void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
    onTogglePin?: () => void;
    onDelete?: () => void;
    onExportJson?: () => void;
    onExportSqlite?: () => void;
    onExportYaml?: () => void;
}

export function PromptTreeContextMenu(props: PromptTreeContextMenuProps) {
    const {
        type = 'conversation',
        x,
        y,
        onClose,

        // Project props
        project,
        projectId,
        projectName,
        isProjectPinned = false,
        isProjectArchived = false,
        onRefreshProject,
        onTogglePinProject,
        onToggleArchiveProject,
        onExportProjectJson,
        onExportProjectSqlite,
        onExportProjectYaml,

        // Conversation & Turn props
        conversationId = '',
        currentTitle = '',
        conversationNode,
        repoPath = '',
        instanceId,
        isPinned: isPinnedProp = false,
        onRename,
        onInspect,
        onResendPrompts,
        onEnqueue,
        onRerunConversation,
        onTogglePin,
        onDelete,
        onExportJson,
        onExportSqlite,
        onExportYaml,
    } = props;

    const menuRef = useRef<HTMLDivElement>(null);
    const [isIdCopied, setIsIdCopied] = useState(false);
    const [isPromptCopied, setIsPromptCopied] = useState(false);
    const [isTextOnlyCopied, setIsTextOnlyCopied] = useState(false);

    // Context menu width/height clamping
    const MENU_WIDTH = 250;
    const MENU_HEIGHT = type === 'project' ? 320 : 420;
    const left = Math.max(8, Math.min(x, window.innerWidth - MENU_WIDTH));
    const top = Math.max(8, Math.min(y, window.innerHeight - MENU_HEIGHT));

    useEffect(() => {
        const handleClickOutside = (e: Event) => {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        const handleScroll = (e: Event) => {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        window.addEventListener('pointerdown', handleClickOutside, { capture: true });
        window.addEventListener('contextmenu', handleClickOutside, { capture: true });
        window.addEventListener('keydown', handleKeyDown, { capture: true });
        window.addEventListener('scroll', handleScroll, { capture: true, passive: true });
        window.addEventListener('blur', onClose);

        return () => {
            window.removeEventListener('pointerdown', handleClickOutside, { capture: true });
            window.removeEventListener('contextmenu', handleClickOutside, { capture: true });
            window.removeEventListener('keydown', handleKeyDown, { capture: true });
            window.removeEventListener('scroll', handleScroll, { capture: true } as EventListenerOptions);
            window.removeEventListener('blur', onClose);
        };
    }, [onClose]);

    const handleCopyId = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const targetId = type === 'project' ? (projectId || project?.project_id || '') : conversationId;
        if (!targetId) return;
        await copyToClipboard(targetId);
        setIsIdCopied(true);
        setTimeout(() => onClose(), 400);
    };

    const handleCopyFullPrompt = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const text = conversationNode?.full_prompt_text || conversationNode?.prompt_preview_200w || currentTitle;
        if (!text) return;
        const mediaPaths: string[] = [];
        if (conversationNode?.prompts) {
            for (const t of conversationNode.prompts) {
                if (t.media_paths) {
                    for (const p of t.media_paths) {
                        if (p && !mediaPaths.includes(p)) {
                            mediaPaths.push(p);
                        }
                    }
                }
            }
        }
        await copyPromptWithRichImages(text, undefined, undefined, mediaPaths.length > 0 ? mediaPaths : undefined);
        setIsPromptCopied(true);
        setTimeout(() => onClose(), 400);
    };

    const handleCopyTextOnly = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const text = conversationNode?.full_prompt_text || conversationNode?.prompt_preview_200w || currentTitle;
        if (!text) return;
        const cleanText = stripImagesFromPrompt(text);
        await copyToClipboard(cleanText);
        setIsTextOnlyCopied(true);
        setTimeout(() => onClose(), 400);
    };

    const handleDeleteConversation = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (onDelete) {
            onDelete();
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
                source: 'PromptTreeContextMenu.handleDelete',
                triggerAction: 'delete_conversation',
            });
        }
        onClose();
    };

    return createPortal(
        <div
            ref={menuRef}
            style={{ left, top }}
            className="fixed z-[99999] w-60 rounded-xl bg-white/95 dark:bg-[#171025]/95 backdrop-blur-md border border-slate-200 dark:border-[#40285e] shadow-2xl p-1.5 text-xs text-slate-700 dark:text-slate-200 animate-in fade-in zoom-in-95 duration-100 select-none"
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => e.preventDefault()}
        >
            {/* Header / Title */}
            <div className="px-2.5 py-1.5 border-b border-slate-200/80 dark:border-[#40285e]/80 mb-1">
                <div className="font-semibold truncate text-[11px] text-slate-900 dark:text-white">
                    {type === 'project'
                        ? (projectName || project?.repo_name || 'Project')
                        : type === 'turn'
                          ? `Turn · ${currentTitle || conversationId.slice(0, 8)}`
                          : (currentTitle || conversationId.slice(0, 8))}
                </div>
                <div className="text-[10px] font-mono text-slate-400 dark:text-slate-500 truncate">
                    {type === 'project' ? (projectId || project?.project_id) : conversationId}
                </div>
            </div>

            {/* Menu Items for PROJECT */}
            {type === 'project' && (
                <div className="space-y-0.5">
                    {onRefreshProject && (
                        <button
                            type="button"
                            onClick={() => {
                                onRefreshProject();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <RotateCw className="w-3.5 h-3.5 text-blue-500" />
                            <span>Refresh Project</span>
                        </button>
                    )}
                    {onTogglePinProject && (
                        <button
                            type="button"
                            onClick={() => {
                                onTogglePinProject();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <Pin className="w-3.5 h-3.5 text-amber-500" />
                            <span>{isProjectPinned ? 'Unpin Project' : 'Pin Project to Top'}</span>
                        </button>
                    )}
                    {onToggleArchiveProject && (
                        <button
                            type="button"
                            onClick={() => {
                                onToggleArchiveProject();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            {isProjectArchived ? (
                                <ArchiveRestore className="w-3.5 h-3.5 text-rose-500" />
                            ) : (
                                <Archive className="w-3.5 h-3.5 text-slate-400" />
                            )}
                            <span>{isProjectArchived ? 'Unarchive Project' : 'Archive Project'}</span>
                        </button>
                    )}

                    <div className="h-px bg-slate-200/80 dark:bg-[#40285e]/80 my-1" />

                    {onExportProjectJson && (
                        <button
                            type="button"
                            onClick={() => {
                                onExportProjectJson();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <FileJson className="w-3.5 h-3.5 text-emerald-500" />
                            <span>Export as JSON</span>
                        </button>
                    )}
                    {onExportProjectSqlite && (
                        <button
                            type="button"
                            onClick={() => {
                                onExportProjectSqlite();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <Database className="w-3.5 h-3.5 text-cyan-500" />
                            <span>Export as SQLite (.db)</span>
                        </button>
                    )}
                    {onExportProjectYaml && (
                        <button
                            type="button"
                            onClick={() => {
                                onExportProjectYaml();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <FileText className="w-3.5 h-3.5 text-purple-500" />
                            <span>Export as YAML</span>
                        </button>
                    )}
                </div>
            )}

            {/* Menu Items for CONVERSATION & TURN */}
            {type !== 'project' && (
                <div className="space-y-0.5">
                    {onInspect && (
                        <button
                            type="button"
                            onClick={() => {
                                onInspect();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <FileSearch className="w-3.5 h-3.5 text-blue-500" />
                            <span>Inspect Telemetry & Transcript</span>
                        </button>
                    )}

                    {type === 'conversation' && onRename && (
                        <button
                            type="button"
                            onClick={() => {
                                onRename();
                                onClose();
                            }}
                            className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <div className="flex items-center gap-2">
                                <Pencil className="w-3.5 h-3.5 text-amber-500" />
                                <span>Rename Conversation</span>
                            </div>
                            <kbd className="px-1 py-0.2 text-[9px] font-mono bg-slate-100 dark:bg-[#120a1f] border border-slate-300 dark:border-purple-500/40 rounded">F2</kbd>
                        </button>
                    )}

                    <div className="h-px bg-slate-200/80 dark:bg-[#40285e]/80 my-1" />

                    {/* Copy Full Prompt */}
                    <button
                        type="button"
                        onClick={handleCopyFullPrompt}
                        className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                    >
                        <div className="flex items-center gap-2">
                            {isPromptCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-purple-500" />}
                            <span>{isPromptCopied ? 'Copied Prompt!' : 'Copy Full Prompt'}</span>
                        </div>
                        <span className="text-[9px] font-mono text-purple-600 dark:text-purple-400">with images</span>
                    </button>

                    {/* Copy Clean Text */}
                    <button
                        type="button"
                        onClick={handleCopyTextOnly}
                        className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                    >
                        <div className="flex items-center gap-2">
                            {isTextOnlyCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-slate-400" />}
                            <span>{isTextOnlyCopied ? 'Copied Text!' : 'Copy Text Only'}</span>
                        </div>
                        <span className="text-[9px] font-mono text-slate-400">clean text</span>
                    </button>

                    {/* Copy ID */}
                    <button
                        type="button"
                        onClick={handleCopyId}
                        className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                    >
                        {isIdCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-slate-400" />}
                        <span>{isIdCopied ? 'Copied ID!' : 'Copy Conversation ID'}</span>
                    </button>

                    <div className="h-px bg-slate-200/80 dark:bg-[#40285e]/80 my-1" />

                    {/* Execution Actions */}
                    {onResendPrompts && (
                        <button
                            type="button"
                            onClick={() => {
                                onResendPrompts(1);
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <RotateCw className="w-3.5 h-3.5 text-emerald-500" />
                            <span>Resend Prompt (Immediate)</span>
                        </button>
                    )}

                    {onEnqueue && (
                        <button
                            type="button"
                            onClick={() => {
                                onEnqueue();
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <ListPlus className="w-3.5 h-3.5 text-amber-500" />
                            <span>Enqueue to FIFO Queue</span>
                        </button>
                    )}

                    {onRerunConversation && (
                        <button
                            type="button"
                            onClick={() => {
                                onRerunConversation(conversationId, loadPersistedRerunMessageOption());
                                onClose();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                        >
                            <Repeat className="w-3.5 h-3.5 text-sky-500" />
                            <span>Rerun Nudge (×{loadPersistedRerunCount()})</span>
                        </button>
                    )}

                    {type === 'conversation' && (
                        <>
                            <div className="h-px bg-slate-200/80 dark:bg-[#40285e]/80 my-1" />

                            {onTogglePin && (
                                <button
                                    type="button"
                                    onClick={() => {
                                        onTogglePin();
                                        onClose();
                                    }}
                                    className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                                >
                                    <Pin className="w-3.5 h-3.5 text-amber-500" />
                                    <span>{isPinnedProp ? 'Unpin Conversation' : 'Pin Conversation'}</span>
                                </button>
                            )}

                            {/* Export Options */}
                            <button
                                type="button"
                                onClick={() => {
                                    if (onExportJson) {
                                        onExportJson();
                                    } else if (conversationNode) {
                                        exportConversationAsJson(conversationNode, repoPath);
                                    }
                                    onClose();
                                }}
                                className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                            >
                                <FileJson className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Export as JSON</span>
                            </button>

                            <button
                                type="button"
                                onClick={() => {
                                    if (onExportSqlite) {
                                        onExportSqlite();
                                    } else if (conversationNode) {
                                        void exportConversationAsSqlite(conversationNode);
                                    }
                                    onClose();
                                }}
                                className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                            >
                                <Database className="w-3.5 h-3.5 text-cyan-500" />
                                <span>Export as SQLite (.db)</span>
                            </button>

                            <button
                                type="button"
                                onClick={() => {
                                    if (onExportYaml) {
                                        onExportYaml();
                                    } else if (conversationNode) {
                                        exportConversationAsYaml(conversationNode, repoPath);
                                    }
                                    onClose();
                                }}
                                className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer text-xs"
                            >
                                <FileText className="w-3.5 h-3.5 text-purple-500" />
                                <span>Export as YAML</span>
                            </button>

                            <div className="h-px bg-slate-200/80 dark:bg-[#40285e]/80 my-1" />

                            <button
                                type="button"
                                onClick={handleDeleteConversation}
                                className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-left text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer text-xs font-medium"
                            >
                                <Trash2 className="w-3.5 h-3.5" />
                                <span>Delete Conversation</span>
                            </button>
                        </>
                    )}
                </div>
            )}
        </div>,
        document.body
    );
}
