import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import {
    Folder,
    RotateCw,
    Pin,
    Archive,
    ArchiveRestore,
    FileJson,
    Database,
    FolderInput,
    Repeat,
    FileText,
    Trash2,
    Sparkles,
} from 'lucide-react';
import { improvementsService } from '../../../services/improvementsService';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';

export interface ProjectContextMenuProps {
    x: number;
    y: number;
    projectId: string;
    projectName: string;
    repoPath?: string;
    isPinned: boolean;
    isArchived: boolean;
    onClose: () => void;
    onRefresh: () => void;
    onTogglePin: () => void;
    onToggleArchive: () => void;
    onImportProject?: (repo: string) => void;
    onOpenImport?: () => void;
    onRerunProject?: (repo: string) => void;
    onRerunQueued?: () => void;
    onRerunAll?: () => void;
    onExportJson: () => void;
    onExportSqlite: () => void;
    onExportYaml: () => void;
    onDeleteProject?: () => void;
    onDelete?: () => void;
}

export function ProjectContextMenu({
    x,
    y,
    projectId,
    projectName,
    repoPath: _repoPath,
    isPinned,
    isArchived,
    onClose,
    onRefresh,
    onTogglePin,
    onToggleArchive,
    onImportProject,
    onOpenImport,
    onRerunProject,
    onRerunQueued,
    onRerunAll,
    onExportJson,
    onExportSqlite,
    onExportYaml,
    onDeleteProject,
    onDelete,
}: ProjectContextMenuProps) {
    const menuRef = useRef<HTMLDivElement>(null);

    const MENU_WIDTH = 230;
    const MENU_HEIGHT = 380;
    const left = Math.max(8, Math.min(x, window.innerWidth - MENU_WIDTH));
    const top = Math.max(8, Math.min(y, window.innerHeight - MENU_HEIGHT));

    useEffect(() => {
        const mountTime = Date.now();

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        const handleClickOutside = (e: MouseEvent) => {
            if (Date.now() - mountTime < 100) return;
            if (e.button === 2) return;
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

    return createPortal(
        <div
            ref={menuRef}
            className="fixed z-[100050] min-w-[210px] rounded-xl border border-slate-200 dark:border-[#40285e] bg-white/95 dark:bg-[#171025]/95 backdrop-blur-md py-1.5 shadow-2xl text-xs text-slate-800 dark:text-slate-200 select-none animate-in fade-in zoom-in-95 duration-100"
            style={{ top, left }}
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => {
                e.preventDefault();
                e.stopPropagation();
            }}
        >
            <div className="flex items-center gap-1.5 px-3 py-1 mb-1 border-b border-slate-100 dark:border-[#2b1b42] text-[11px] font-semibold text-slate-500 dark:text-slate-400 truncate max-w-[240px]">
                <Folder className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                <span className="truncate">{projectName || projectId}</span>
            </div>

            {/* Refresh Project */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onRefresh();
                    onClose();
                }}
            >
                <RotateCw className="w-3.5 h-3.5 text-blue-500" />
                <span>Refresh Project</span>
            </button>

            {/* Pin / Unpin */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onTogglePin();
                    onClose();
                }}
            >
                <Pin className="w-3.5 h-3.5 text-amber-500" />
                <span>{isPinned ? 'Unpin Project' : 'Pin Project to Top'}</span>
            </button>

            {/* Archive / Unarchive */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onToggleArchive();
                    onClose();
                }}
            >
                {isArchived ? (
                    <ArchiveRestore className="w-3.5 h-3.5 text-rose-500" />
                ) : (
                    <Archive className="w-3.5 h-3.5 text-slate-400" />
                )}
                <span>{isArchived ? 'Unarchive Project' : 'Archive Project'}</span>
            </button>

            <div className="border-t border-slate-100 dark:border-[#2b1b42] my-1" />

            {/* Import Prompts into Project */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    if (onImportProject) {
                        onImportProject(projectName || projectId);
                    } else if (onOpenImport) {
                        onOpenImport();
                    }
                    onClose();
                }}
            >
                <FolderInput className="w-3.5 h-3.5 text-sky-500" />
                <span>Import Prompts into Project...</span>
            </button>

            {/* Rerun Project Prompts */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    if (onRerunProject) {
                        onRerunProject(projectName || projectId);
                    } else if (onRerunQueued) {
                        onRerunQueued();
                    }
                    onClose();
                }}
            >
                <div className="flex items-center gap-2.5">
                    <RotateCw className="w-3.5 h-3.5 text-emerald-500" />
                    <span>Rerun Queued Prompts</span>
                </div>
                <span className="text-[10px] font-mono text-slate-400">FIFO</span>
            </button>

            {onRerunAll && (
                <button
                    type="button"
                    className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a] cursor-pointer"
                    onClick={(e) => {
                        e.stopPropagation();
                        onRerunAll();
                        onClose();
                    }}
                >
                    <div className="flex items-center gap-2.5">
                        <Repeat className="w-3.5 h-3.5 text-blue-500" />
                        <span>Rerun All Prompts</span>
                    </div>
                    <span className="text-[10px] font-mono text-slate-400">All</span>
                </button>
            )}

            {/* Plan 100 Improvements */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-amber-500/10 hover:text-amber-600 dark:hover:text-amber-400 cursor-pointer"
                onClick={async (e) => {
                    e.stopPropagation();
                    onClose();
                    try {
                        await improvementsService.plan('system', false, projectId);
                        showToast(`100 Improvements plan dispatched for ${projectName || projectId}`, 'success', 2500);
                    } catch (err) {
                        useErrorStore.getState().captureException(
                            err instanceof Error ? err : new Error(String(err)),
                            {
                                source: 'ProjectContextMenu.planImprovements',
                                triggerAction: 'improvements_plan',
                            }
                        );
                    }
                }}
            >
                <div className="flex items-center gap-2.5">
                    <Sparkles className="w-3.5 h-3.5 text-amber-500" />
                    <span>Plan 100 Improvements...</span>
                </div>
                <span className="text-[10px] font-mono text-amber-500 font-bold">100</span>
            </button>

            <div className="border-t border-slate-100 dark:border-[#2b1b42] my-1" />
            <div className="px-3 pt-0.5 pb-1 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
                Export Section
            </div>

            {/* Export as JSON */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-amber-500/10 hover:text-amber-600 dark:hover:text-amber-400 cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onExportJson();
                    onClose();
                }}
            >
                <div className="flex items-center gap-2.5">
                    <FileJson className="w-3.5 h-3.5 text-amber-500" />
                    <span>Export Project as JSON...</span>
                </div>
                <span className="text-[10px] font-mono text-slate-400">.json</span>
            </button>

            {/* Export as SQLite */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-indigo-500/10 hover:text-indigo-600 dark:hover:text-indigo-400 cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onExportSqlite();
                    onClose();
                }}
            >
                <div className="flex items-center gap-2.5">
                    <Database className="w-3.5 h-3.5 text-indigo-500" />
                    <span>Export Project as SQLite...</span>
                </div>
                <span className="text-[10px] font-mono text-slate-400">.db</span>
            </button>

            {/* Export as YAML */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-emerald-500/10 hover:text-emerald-600 dark:hover:text-emerald-400 cursor-pointer"
                onClick={(e) => {
                    e.stopPropagation();
                    onExportYaml();
                    onClose();
                }}
            >
                <div className="flex items-center gap-2.5">
                    <FileText className="w-3.5 h-3.5 text-emerald-500" />
                    <span>Export Project as YAML...</span>
                </div>
                <span className="text-[10px] font-mono text-slate-400">.yaml</span>
            </button>

            <div className="border-t border-slate-100 dark:border-[#2b1b42] my-1" />

            {/* Delete / Remove Project */}
            <button
                type="button"
                className="flex w-full items-center justify-between px-3 py-1.5 text-left transition-colors hover:bg-rose-500/10 hover:text-rose-600 dark:hover:text-rose-400 cursor-pointer text-rose-600 dark:text-rose-400"
                onClick={(e) => {
                    e.stopPropagation();
                    if (onDeleteProject) {
                        onDeleteProject();
                    } else if (onDelete) {
                        onDelete();
                    } else if (onToggleArchive && !isArchived) {
                        onToggleArchive();
                    }
                    onClose();
                }}
                title="Remove this project from the current view"
            >
                <div className="flex items-center gap-2.5">
                    <Trash2 className="w-3.5 h-3.5 text-rose-500 shrink-0" />
                    <span>Remove Project from View</span>
                </div>
                <span className="text-[10px] font-mono text-rose-400 font-bold">Remove</span>
            </button>
        </div>,
        document.body
    );
}
