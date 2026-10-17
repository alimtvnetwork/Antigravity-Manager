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
    FileText,
} from 'lucide-react';

export interface ProjectContextMenuProps {
    x: number;
    y: number;
    projectId: string;
    projectName: string;
    isPinned: boolean;
    isArchived: boolean;
    onClose: () => void;
    onRefresh: () => void;
    onTogglePin: () => void;
    onToggleArchive: () => void;
    onExportJson: () => void;
    onExportSqlite: () => void;
    onExportYaml: () => void;
}

export function ProjectContextMenu({
    x,
    y,
    projectId,
    projectName,
    isPinned,
    isArchived,
    onClose,
    onRefresh,
    onTogglePin,
    onToggleArchive,
    onExportJson,
    onExportSqlite,
    onExportYaml,
}: ProjectContextMenuProps) {
    const menuRef = useRef<HTMLDivElement>(null);

    const left = Math.max(8, Math.min(x, window.innerWidth - 230));
    const top = Math.max(8, Math.min(y, window.innerHeight - 280));

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

    return createPortal(
        <div
            ref={menuRef}
            className="fixed z-[10000] min-w-[210px] rounded-xl border border-slate-200 dark:border-[#40285e] bg-white/95 dark:bg-[#171025]/95 backdrop-blur-md py-1.5 shadow-2xl text-xs text-slate-800 dark:text-slate-200 select-none animate-in fade-in zoom-in-95 duration-100"
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
        </div>,
        document.body
    );
}
