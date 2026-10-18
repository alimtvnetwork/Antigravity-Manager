import {
    X,
    Layers,
    Maximize2,
    Minimize2,
    RefreshCw,
    PanelLeftClose,
    PanelLeftOpen,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { SyncInterval } from '../../../services/instanceService';
import { setPromptTreeSyncInterval } from '../../../services/instanceService';
import { ImportExportMenu } from './ImportExportMenu';

export interface ModalHeaderProps {
    instanceName: string;
    sequenceNumber?: number;
    executablePath?: string;
    isFullscreen: boolean;
    setIsFullscreen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isLoading: boolean;
    syncInterval: SyncInterval;
    setSyncInterval: (v: SyncInterval) => void;
    restoreFileInputRef: React.RefObject<HTMLInputElement | null>;
    onClose: () => void;
    onBackup: () => void;
    onRestore: () => void;
    onExportSqlite: () => void;
    onImportSqlite: () => void;
    onCopyAllTreePrompts: () => Promise<void> | void;
    onFileRestore: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onRefreshAll: () => void;
    onResetTree: () => void;
    isTreeCollapsed?: boolean;
    onToggleTree?: () => void;
}

const SYNC_INTERVALS: { value: SyncInterval; label: string }[] = [
    { value: 'off', label: 'Off' },
    { value: '15s', label: '15s' },
    { value: '30s', label: '30s' },
    { value: '1m', label: '1m' },
    { value: '2m', label: '2m' },
];

export function ModalHeader(props: ModalHeaderProps) {
    const {
        instanceName,
        sequenceNumber,
        executablePath,
        isFullscreen,
        setIsFullscreen,
        isLoading,
        syncInterval,
        setSyncInterval,
        restoreFileInputRef,
        onClose,
        onBackup,
        onRestore,
        onExportSqlite,
        onImportSqlite,
        onCopyAllTreePrompts,
        onFileRestore,
        onRefreshAll,
        onResetTree,
        isTreeCollapsed,
        onToggleTree,
    } = props;

    return (
        <div className="flex items-center justify-between border-b border-slate-200/80 dark:border-[#40285e] px-4 py-2.5 bg-slate-50/80 dark:bg-[#090611] shrink-0">
            <div className="flex items-center gap-2.5 min-w-0">
                <div className="flex h-8 w-8 items-center justify-center rounded-[5px] bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20 shrink-0">
                    <Layers className="h-4 w-4" />
                </div>
                <div className="min-w-0">
                    <div className="flex items-center gap-2 flex-wrap min-w-0">
                        <h2 className="text-sm font-bold text-slate-900 dark:text-white shrink-0">
                            Project & Prompt Tree View
                        </h2>
                        {sequenceNumber != null && sequenceNumber > 0 && (
                            <span className="shrink-0 rounded-[5px] bg-slate-100 dark:bg-[#171025] px-1.5 py-0.5 text-[10px] font-bold text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-[#40285e]">
                                #{sequenceNumber}
                            </span>
                        )}
                        <span className="font-semibold text-slate-900 dark:text-white text-xs truncate max-w-[140px]" title={instanceName}>
                            {instanceName}
                        </span>
                        {executablePath && (() => {
                            const trailing = executablePath.replace(/^.*[\\/]/, '');
                            return (
                                <span
                                    className="hidden sm:inline-block shrink-0 rounded-[5px] bg-slate-100 dark:bg-[#171025] px-1.5 py-0.5 text-[10px] font-mono text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-[#40285e] max-w-[150px] truncate"
                                    title={executablePath}
                                >
                                    ...\{trailing}
                                </span>
                            );
                        })()}
                    </div>
                </div>
            </div>

            {/* Top Right Header Controls: Two Independent Segmented Dark-Glass Capsules */}
            <div className="flex items-center gap-1.5 shrink-0">
                {/* Capsule 1: Data Operations Capsule */}
                <div className="inline-flex items-center rounded-md bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#40285e]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#40285e]/80 shadow-2xs">
                    {/* Unified Import / Export section: all tree-level data
                        operations (Export SQLite, Export JSON, Import SQLite,
                        Import JSON, Copy All Tree Prompts) in one entry point. */}
                    <ImportExportMenu
                        onExportSqlite={onExportSqlite}
                        onExportJson={onBackup}
                        onImportSqlite={onImportSqlite}
                        onImportJson={onRestore}
                        onCopyAllTree={onCopyAllTreePrompts}
                        isLoading={isLoading}
                    />

                    <button
                        type="button"
                        onClick={onResetTree}
                        disabled={isLoading}
                        className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                        title="Clear the prompt tree cache and reload. Running and queued prompts stay."
                    >
                        <span>Reset</span>
                    </button>

                    {/* Refresh Button */}
                    <button
                        type="button"
                        onClick={onRefreshAll}
                        disabled={isLoading}
                        className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                        title="Refresh all projects"
                    >
                        <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isLoading && "animate-spin")} />
                        <span>Refresh</span>
                    </button>

                    {/* Sync Interval Selector with Transparent Inline Styling */}
                    <div className="flex items-center gap-1 px-2 py-1">
                        <span className="text-[10px] font-medium text-slate-500 dark:text-slate-400">Sync:</span>
                        <select
                            value={syncInterval}
                            onChange={(e) => {
                                const val = e.target.value as SyncInterval;
                                setSyncInterval(val);
                                setPromptTreeSyncInterval(val);
                            }}
                            className="bg-transparent border-0 focus:ring-0 focus:outline-none text-[11px] font-semibold text-slate-700 dark:text-slate-200 cursor-pointer"
                            title="Auto-sync interval"
                        >
                            {SYNC_INTERVALS.map((opt) => (
                                <option key={opt.value} value={opt.value}>
                                    {opt.label}
                                </option>
                            ))}
                        </select>
                    </div>
                    <input
                        type="file"
                        ref={restoreFileInputRef}
                        accept=".json,.db,.sqlite"
                        onChange={onFileRestore}
                        style={{ display: 'none' }}
                    />
                </div>

                {/* Capsule 2: Window Controls Capsule */}
                <div className="inline-flex items-center rounded-md bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#40285e]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#40285e]/80 shadow-2xs">
                    {/* Tree View Collapse / Expand Toggle */}
                    {onToggleTree && (
                        <button
                            type="button"
                            onClick={onToggleTree}
                            className="flex items-center gap-1.5 px-2.5 py-1 rounded-l-md text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer"
                            title={isTreeCollapsed ? "Show Prompt Tree Panel" : "Hide Prompt Tree Panel"}
                        >
                            {isTreeCollapsed ? (
                                <PanelLeftOpen className="w-3.5 h-3.5 text-blue-500" />
                            ) : (
                                <PanelLeftClose className="w-3.5 h-3.5 text-slate-400" />
                            )}
                            <span className="hidden sm:inline">{isTreeCollapsed ? 'Show Tree' : 'Hide Tree'}</span>
                        </button>
                    )}

                    {/* Fullscreen Toggle */}
                    <button
                        type="button"
                        onClick={() => setIsFullscreen(!isFullscreen)}
                        className={cn("p-1.5 text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer", !onToggleTree && "rounded-l-md")}
                        title={isFullscreen ? "Exit fullscreen" : "Fullscreen"}
                    >
                        {isFullscreen ? (
                            <Minimize2 className="w-4 h-4" />
                        ) : (
                            <Maximize2 className="w-4 h-4" />
                        )}
                    </button>

                    {/* Close Button with Rose Dark-Glass Hover */}
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-r-md text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-100/80 dark:hover:bg-rose-950/40 transition-colors cursor-pointer"
                        title="Close"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
            </div>
        </div>
    );
}
