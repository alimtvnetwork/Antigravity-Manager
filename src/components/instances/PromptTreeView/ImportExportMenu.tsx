import { useEffect, useRef, useState } from 'react';
import {
    Check,
    ChevronDown,
    Copy,
    Database,
    FileDown,
    FileJson,
    FileUp,
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
    /** Copy ALL prompts across every project in the tree to the clipboard. */
    onCopyAllTree: () => Promise<void> | void;
    isLoading: boolean;
}

/**
 * Unified Import / Export section: one entry point hosting all tree-level
 * data operations — Export SQLite, Export JSON, Import SQLite, Import JSON —
 * plus Copy All Tree Prompts. Previously Backup/Restore lived as loose
 * capsule buttons in ModalHeader while per-prompt export lived in
 * DetailHeader's copy menu; this single flyout consolidates the tree-level
 * operations so the user finds them in one place.
 */
export function ImportExportMenu(props: ImportExportMenuProps) {
    const {
        onExportSqlite,
        onExportJson,
        onImportSqlite,
        onImportJson,
        onCopyAllTree,
        isLoading,
    } = props;

    const [isOpen, setIsOpen] = useState(false);
    const [copiedTree, setCopiedTree] = useState(false);
    const rootRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!isOpen) return;
        const onPointerDown = (e: PointerEvent) => {
            if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
                setIsOpen(false);
            }
        };
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') setIsOpen(false);
        };
        document.addEventListener('pointerdown', onPointerDown);
        document.addEventListener('keydown', onKeyDown);
        return () => {
            document.removeEventListener('pointerdown', onPointerDown);
            document.removeEventListener('keydown', onKeyDown);
        };
    }, [isOpen ]);

    const close = () => setIsOpen(false);

    const menuItemClass =
        'w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed';

    return (
        <div className="relative z-50 flex items-center" ref={rootRef}>
            <button
                type="button"
                onClick={() => setIsOpen((prev) => !prev)}
                disabled={isLoading}
                className="flex items-center gap-1.5 h-7 px-2.5 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
            >
                <Database className="w-3.5 h-3.5 text-violet-500 shrink-0" />
                <span>Import / Export</span>
                <ChevronDown
                    className={cn(
                        'w-3 h-3 text-slate-400 transition-transform duration-150 shrink-0',
                        isOpen && 'rotate-180'
                    )}
                />
            </button>

            {isOpen && (
                <div className="absolute right-0 top-full mt-1.5 z-50 w-64 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1">
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
                </div>
            )}
        </div>
    );
}
