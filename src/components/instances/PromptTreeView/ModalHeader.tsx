import { useState, useEffect } from 'react';
import {
    X,
    Layers,
    Maximize2,
    Minimize2,
    RefreshCw,
    RotateCcw,
    PanelLeftClose,
    PanelLeftOpen,
    ChevronDown,
    Check,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { SyncInterval } from '../../../services/instanceService';
import { setPromptTreeSyncInterval } from '../../../services/instanceService';
import { ImportExportMenu } from './ImportExportMenu';
import { imageFilterEmitter } from './usePromptTreeData';

export interface ModalHeaderProps {
    instanceName: string;
    appVersion?: string;
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
    isAutoSyncing?: boolean;
    isRefreshing?: boolean;
    isResetting?: boolean;
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
        appVersion,
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
        isAutoSyncing,
        isRefreshing,
        isResetting,
    } = props;

    const [isImageFilterActive, setIsImageFilterActive] = useState<boolean>(() => imageFilterEmitter.get());
    const [isSyncDropdownOpen, setIsSyncDropdownOpen] = useState<boolean>(false);

    useEffect(() => {
        const unsub = imageFilterEmitter.subscribe((active) => {
            setIsImageFilterActive(active);
        });
        return () => {
            unsub();
        };
    }, []);

    const handleToggleImageFilter = () => {
        imageFilterEmitter.toggle();
    };

    return (
        <div className="relative z-40 flex items-center justify-between border-b border-slate-200/80 dark:border-purple-900/40 px-4 py-2.5 bg-slate-50/80 dark:bg-[#0c0817]/95 backdrop-blur-xl shrink-0">
            <div className="flex items-center gap-2.5 min-w-0">
                <div className="flex h-8 w-8 items-center justify-center rounded-[5px] bg-purple-500/10 text-purple-600 dark:text-purple-400 border border-purple-500/20 shrink-0">
                    <Layers className="h-4 w-4" />
                </div>
                <div className="min-w-0">
                    <div className="flex items-center gap-2 flex-wrap min-w-0">
                        <h2 className="text-sm font-bold text-slate-900 dark:text-white shrink-0">
                            Project & Prompt Tree View
                        </h2>
                        {appVersion && (
                            <span className="shrink-0 rounded-[5px] bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/20 px-1.5 py-0.5 text-[10px] font-mono font-semibold" title={`Antigravity Manager v${appVersion}`}>
                                v{appVersion}
                            </span>
                        )}
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

            {/* Top Right Header Controls: Single Contiguous Segmented Dark-Glass Pill Capsule */}
            <div className="flex items-center shrink-0">
                <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#120d20]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-900/50 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-900/40 shadow-2xs">
                    {/* Segment 1: Unified Import / Export section */}
                    <ImportExportMenu
                        onExportSqlite={onExportSqlite}
                        onExportJson={onBackup}
                        onImportSqlite={onImportSqlite}
                        onImportJson={onRestore}
                        onRestore={onRestore}
                        onCopyAllTree={onCopyAllTreePrompts}
                        isLoading={isLoading}
                    />

                    {/* Segment 2: Reset Button */}
                    <button
                        type="button"
                        onClick={onResetTree}
                        disabled={isLoading || isResetting}
                        className="flex items-center gap-1.5 h-7 px-2.5 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                        title="Wipe database and regenerate prompt tree and sequences fresh from disk transcripts"
                    >
                        <RotateCcw className={cn("w-3.5 h-3.5 text-amber-500", isResetting && "animate-spin")} />
                        <span>{isResetting ? 'Resetting...' : 'Reset'}</span>
                    </button>

                    {/* Segment 3: Refresh Button */}
                    <button
                        type="button"
                        onClick={onRefreshAll}
                        disabled={isLoading}
                        className="flex items-center gap-1.5 h-7 px-2.5 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                        title="Refresh all projects"
                    >
                        <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", (isLoading || isAutoSyncing || isRefreshing) && "animate-spin")} />
                        <span>Refresh</span>
                    </button>

                    {/* Segment 3.5: Image Filter Toggle Button */}
                    <button
                        type="button"
                        onClick={handleToggleImageFilter}
                        className={cn(
                            "flex items-center gap-1.5 h-7 px-2.5 text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                            isImageFilterActive
                                ? "bg-sky-600 text-white shadow-2xs"
                                : "text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90"
                        )}
                        title={isImageFilterActive ? "Showing only conversations with images" : "Filter conversations with images"}
                    >
                        <span>🖼️</span>
                        <span>Images</span>
                    </button>

                    {/* Segment 4: Sync Interval Selector with Modernized Dark-Glass Popover */}
                    <div className="relative flex items-center h-7 px-2.5">
                        <button
                            type="button"
                            onClick={() => setIsSyncDropdownOpen((prev) => !prev)}
                            className="flex items-center gap-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:text-purple-400 transition-colors cursor-pointer"
                            title="Auto-sync interval (Click to change)"
                        >
                            <span className="text-[10px] font-medium text-slate-500 dark:text-slate-400 select-none">Sync:</span>
                            <span className="text-purple-600 dark:text-purple-300 font-mono font-bold">{syncInterval}</span>
                            <ChevronDown className={cn("w-3 h-3 text-slate-400 transition-transform duration-150", isSyncDropdownOpen && "rotate-180")} />
                        </button>

                        {isSyncDropdownOpen && (
                            <>
                                <div
                                    className="fixed inset-0 z-50"
                                    onClick={() => setIsSyncDropdownOpen(false)}
                                />
                                <div className="absolute right-0 top-full mt-1.5 w-28 py-1 rounded-xl bg-slate-900/95 dark:bg-[#171025]/95 backdrop-blur-xl border border-slate-700/60 dark:border-purple-900/60 shadow-2xl z-50">
                                    <div className="px-2.5 py-1 text-[9px] uppercase tracking-wider font-semibold text-slate-400 border-b border-slate-800/60 dark:border-purple-900/40">
                                        Sync Interval
                                    </div>
                                    {SYNC_INTERVALS.map((opt) => (
                                        <button
                                            key={opt.value}
                                            type="button"
                                            onClick={() => {
                                                setSyncInterval(opt.value);
                                                setPromptTreeSyncInterval(opt.value);
                                                setIsSyncDropdownOpen(false);
                                            }}
                                            className={cn(
                                                "w-full text-left px-2.5 py-1.5 text-xs flex items-center justify-between transition-colors cursor-pointer",
                                                syncInterval === opt.value
                                                    ? "bg-purple-600/20 text-purple-300 font-semibold"
                                                    : "text-slate-300 hover:bg-slate-800/60 dark:hover:bg-[#281b3d]"
                                            )}
                                        >
                                            <span className="font-mono text-[11px]">{opt.label}</span>
                                            {syncInterval === opt.value && (
                                                <Check className="w-3.5 h-3.5 text-purple-400" />
                                            )}
                                        </button>
                                    ))}
                                </div>
                            </>
                        )}
                    </div>

                    {/* Segment 5: Tree View Collapse / Expand Toggle */}
                    {onToggleTree && (
                        <button
                            type="button"
                            onClick={onToggleTree}
                            className="flex items-center gap-1.5 h-7 px-2.5 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer"
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

                    {/* Segment 6: Fullscreen Toggle */}
                    <button
                        type="button"
                        onClick={() => setIsFullscreen(!isFullscreen)}
                        className="flex items-center justify-center w-7 h-7 text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer"
                        title={isFullscreen ? "Exit fullscreen" : "Fullscreen"}
                    >
                        {isFullscreen ? (
                            <Minimize2 className="w-3.5 h-3.5" />
                        ) : (
                            <Maximize2 className="w-3.5 h-3.5" />
                        )}
                    </button>

                    {/* Segment 7: Close Button with Rose Dark-Glass Hover */}
                    <button
                        type="button"
                        onClick={onClose}
                        className="flex items-center justify-center w-7 h-7 rounded-r-full text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-100/80 dark:hover:bg-rose-950/40 transition-colors cursor-pointer"
                        title="Close"
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>

                <input
                    type="file"
                    ref={restoreFileInputRef}
                    accept=".json,.db,.sqlite"
                    onChange={onFileRestore}
                    style={{ display: 'none' }}
                />
            </div>
        </div>
    );
}
