import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import {
    Check,
    ChevronDown,
    Copy,
    Database,
    FileDown,
    FileJson,
    FileUp,
    RotateCcw,
    Upload,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { downloadPromptTreeStatusJson } from './usePromptTreeCoreActions';

export interface ImportExportMenuProps {
    /** Export the full prompt tree to a portable SQLite `.db` file. */
    onExportSqlite: () => void;
    /** Export the full prompt tree to a JSON backup (existing handler). */
    onExportJson: () => void;
    /** Import a portable SQLite `.db` tree export into the live database. */
    onImportSqlite: () => void;
    /** Import a JSON tree backup (existing handler). */
    onImportJson: () => void;
    /** Restore prompts from a JSON backup and dispatch to instance. */
    onRestore?: () => void;
    /** Export both running and queued prompts combined into a JSON file. */
    onExportRunningAndQueuedJson?: () => void;
    /** Copy ALL prompts across every project in the tree to the clipboard. */
    onCopyAllTree: () => Promise<void> | void;
    isLoading: boolean;
}

/**
 * Unified Import / Export section: one entry point hosting all tree-level
 * data operations — Export SQLite, Export JSON, Import SQLite, Import JSON —
 * plus Copy All Tree Prompts. Rendered via React Portal with fixed z-[99999]
 * and transparent backdrop to eliminate stacking context bleed-through.
 */
export function ImportExportMenu(props: ImportExportMenuProps) {
    const {
        onExportSqlite,
        onExportJson,
        onImportSqlite,
        onImportJson,
        onRestore,
        onExportRunningAndQueuedJson,
        onCopyAllTree,
        isLoading,
    } = props;

    const [isOpen, setIsOpen] = useState(false);
    const [copiedTree, setCopiedTree] = useState(false);
    const [menuPosition, setMenuPosition] = useState<{ top: number; right: number } | null>(null);
    const btnRef = useRef<HTMLButtonElement>(null);

    const updatePosition = () => {
        if (btnRef.current) {
            const rect = btnRef.current.getBoundingClientRect();
            setMenuPosition({
                top: rect.bottom + 6,
                right: window.innerWidth - rect.right,
            });
        }
    };

    const handleToggle = () => {
        if (!isOpen) {
            updatePosition();
        }
        setIsOpen((prev) => !prev);
    };

    const close = () => setIsOpen(false);

    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') setIsOpen(false);
        };
        const handleScrollOrResize = () => {
            updatePosition();
        };
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('resize', handleScrollOrResize);
        window.addEventListener('scroll', handleScrollOrResize, true);
        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('resize', handleScrollOrResize);
            window.removeEventListener('scroll', handleScrollOrResize, true);
        };
    }, [isOpen]);

    const menuItemClass =
        'w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed';

    return (
        <div className="relative flex items-center">
            <button
                ref={btnRef}
                type="button"
                onClick={handleToggle}
                disabled={isLoading}
                className="flex items-center justify-center w-7 h-7 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                title="Import / Export"
                aria-label="Import / Export"
            >
                <Database className="w-3.5 h-3.5 text-violet-500 shrink-0" />
                <ChevronDown
                    className={cn(
                        'w-3 h-3 text-slate-400 transition-transform duration-150 shrink-0',
                        isOpen && 'rotate-180'
                    )}
                />
            </button>

            {isOpen && menuPosition && typeof document !== 'undefined' && createPortal(
                <>
                    {/* Transparent outside-click capture backdrop layer */}
                    <div
                        className="fixed inset-0 z-[99998] bg-transparent"
                        onClick={close}
                    />
                    {/* Dropdown popup with fixed z-[99999] */}
                    <div
                        style={{
                            top: `${menuPosition.top}px`,
                            right: `${menuPosition.right}px`,
                        }}
                        className="fixed z-[99999] w-64 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1"
                    >
                        {/* Copy All Tree Prompts */}
                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                try {
                                    void Promise.resolve(onCopyAllTree()).finally(() => {
                                        setCopiedTree(true);
                                        setTimeout(() => setCopiedTree(false), 2000);
                                    });
                                } finally {
                                    close();
                                }
                            }}
                            className={menuItemClass}
                            title="Copy every prompt in the tree (all projects) to the clipboard"
                        >
                            <div className="flex items-center gap-2">
                                {copiedTree ? (
                                    <Check className="w-3.5 h-3.5 text-emerald-500" />
                                ) : (
                                    <Copy className="w-3.5 h-3.5 text-sky-500" />
                                )}
                                <span>{copiedTree ? 'Copied Tree!' : 'Copy All Tree Prompts'}</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">Clipboard</span>
                        </button>

                        <div className="border-t border-slate-100 dark:border-[#40285e]/60 my-1" />
                        <div className="px-2.5 pt-0.5 text-[10px] font-bold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Export
                        </div>

                        {/* Export SQLite */}
                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                onExportSqlite();
                                close();
                            }}
                            className={menuItemClass}
                            title="Export the full prompt tree to a portable SQLite .db file"
                        >
                            <div className="flex items-center gap-2">
                                <FileDown className="w-3.5 h-3.5 text-indigo-500" />
                                <span>Export Tree as SQLite</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.db</span>
                        </button>

                        {/* Export JSON */}
                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                onExportJson();
                                close();
                            }}
                            className={menuItemClass}
                            title="Export the full prompt tree to a JSON backup file"
                        >
                            <div className="flex items-center gap-2">
                                <FileJson className="w-3.5 h-3.5 text-amber-500" />
                                <span>Export Tree as JSON</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.json</span>
                        </button>

                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                close();
                                void downloadPromptTreeStatusJson('running');
                            }}
                            className={menuItemClass}
                            title="Download JSON for prompts whose status is running, from the tree already loaded"
                        >
                            <div className="flex items-center gap-2">
                                <FileJson className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Export Running Prompts</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.json</span>
                        </button>

                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                close();
                                void downloadPromptTreeStatusJson('queued');
                            }}
                            className={menuItemClass}
                            title="Download JSON for prompts whose status is queued, from the tree already loaded"
                        >
                            <div className="flex items-center gap-2">
                                <FileJson className="w-3.5 h-3.5 text-amber-500" />
                                <span>Export Queued Prompts</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.json</span>
                        </button>

                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                close();
                                if (onExportRunningAndQueuedJson) {
                                    onExportRunningAndQueuedJson();
                                } else {
                                    void downloadPromptTreeStatusJson('running_and_queued');
                                }
                            }}
                            className={menuItemClass}
                            title="Download JSON for both running and queued prompts, from the tree already loaded"
                        >
                            <div className="flex items-center gap-2">
                                <FileJson className="w-3.5 h-3.5 text-purple-500" />
                                <span>Export Running & Queued</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.json</span>
                        </button>

                        <div className="border-t border-slate-100 dark:border-[#40285e]/60 my-1" />
                        <div className="px-2.5 pt-0.5 text-[10px] font-bold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Import
                        </div>

                        {/* Import SQLite */}
                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                onImportSqlite();
                                close();
                            }}
                            className={menuItemClass}
                            title="Import a portable SQLite .db tree export into the live database"
                        >
                            <div className="flex items-center gap-2">
                                <FileUp className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Import Tree from SQLite</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.db</span>
                        </button>

                        {/* Import JSON */}
                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                onImportJson();
                                close();
                            }}
                            className={menuItemClass}
                            title="Restore prompts from a JSON backup file"
                        >
                            <div className="flex items-center gap-2">
                                <Upload className="w-3.5 h-3.5 text-teal-500" />
                                <span>Import Tree from JSON</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.json</span>
                        </button>

                        {/* Restore Prompts from Backup */}
                        <button
                            type="button"
                            disabled={isLoading}
                            onClick={() => {
                                close();
                                if (onRestore) {
                                    onRestore();
                                } else {
                                    onImportJson();
                                }
                            }}
                            className={menuItemClass}
                            title="Restore prompts from a backup file and resume dispatch to instance"
                        >
                            <div className="flex items-center gap-2">
                                <RotateCcw className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Restore Prompts from Backup</span>
                            </div>
                            <span className="text-[10px] text-slate-400 font-mono">.json</span>
                        </button>
                    </div>
                </>,
                document.body
            )}
        </div>
    );
}
