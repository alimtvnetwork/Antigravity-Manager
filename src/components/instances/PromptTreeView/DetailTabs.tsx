import { Eye, Code, Edit3, ChevronUp, ChevronDown, FileText, MessageSquare, Terminal } from 'lucide-react';
import { cn } from '../../../utils/cn';

export interface DetailTabsProps {
    previewTab: 'instruction' | 'results';
    setPreviewTab?: (v: 'instruction' | 'results') => void;
    selectedConversation?: import('./types').AgmConversationNode;
    activeWordCount: number;
    activeByteCount: number;
    formatByteSize: (bytes: number) => string;
    isTruncated: boolean;
    activePromptText: string;
    showAllWords: boolean;
    setShowAllWords: (v: boolean | ((prev: boolean) => boolean)) => void;
    viewMode: import('./types').ViewMode;
    setViewMode: (v: import('./types').ViewMode) => void;
    setIsDetailsModalOpen: (v: boolean) => void;
}

export function DetailTabs(props: DetailTabsProps) {
    const {
        previewTab,
        setPreviewTab,
        selectedConversation,
        activeWordCount,
        activeByteCount,
        formatByteSize,
        isTruncated,
        activePromptText,
        showAllWords,
        setShowAllWords,
        viewMode,
        setViewMode,
        setIsDetailsModalOpen,
    } = props;

    const hasExpandToggle = previewTab === 'instruction' && Boolean(
        isTruncated ||
        activePromptText.trim().endsWith('...') ||
        activePromptText.trim().endsWith('…') ||
        showAllWords
    );

    return (
        <div className="flex items-center justify-between gap-2.5 flex-wrap py-0.5 min-h-[32px]">
            {/* Left: View Mode Controls Capsule (Preview, Raw, Edit, Details) */}
            <div className="flex items-center gap-2.5 shrink-0">
                <div className="inline-flex items-center rounded-lg bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
                    <button
                        type="button"
                        onClick={() => setViewMode('preview')}
                        className={cn(
                            'p-1.5 rounded-l-md transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            viewMode === 'preview'
                                ? 'bg-blue-600 text-white shadow-2xs'
                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60'
                        )}
                        title="Preview"
                        aria-label="Preview"
                    >
                        <Eye className="w-3.5 h-3.5" />
                    </button>
                    <button
                        type="button"
                        onClick={() => setViewMode('raw')}
                        className={cn(
                            'p-1.5 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            viewMode === 'raw'
                                ? 'bg-blue-600 text-white shadow-2xs'
                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60'
                        )}
                        title="Raw"
                        aria-label="Raw"
                    >
                        <Code className="w-3.5 h-3.5" />
                    </button>
                    <button
                        type="button"
                        onClick={() => setViewMode('edit')}
                        className={cn(
                            'p-1.5 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            viewMode === 'edit'
                                ? 'bg-blue-600 text-white shadow-2xs'
                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60'
                        )}
                        title="Edit"
                        aria-label="Edit"
                    >
                        <Edit3 className="w-3.5 h-3.5" />
                    </button>
                    <button
                        type="button"
                        onClick={() => setIsDetailsModalOpen(true)}
                        className="p-1.5 rounded-r-md text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Details"
                        aria-label="Details"
                    >
                        <FileText className="w-3.5 h-3.5 text-blue-500" />
                    </button>
                </div>
            </div>

            {/* Right: Stats + Single Segmented Capsule ([Input] [Output] [Expand Full/Collapse]) */}
            <div className="flex items-center gap-2 ml-auto shrink-0">
                {previewTab === 'instruction' && (
                    <span className="text-xs font-semibold text-slate-500 dark:text-slate-400 font-mono">
                        {activeWordCount} words · {formatByteSize(activeByteCount)}
                    </span>
                )}

                <div className="inline-flex items-center rounded-lg bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
                    {setPreviewTab && (
                        <>
                            <button
                                type="button"
                                onClick={() => setPreviewTab('instruction')}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1 rounded-l-md text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                    previewTab === 'instruction'
                                        ? "bg-blue-600 text-white shadow-2xs"
                                        : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                                )}
                                title="View Input Instruction"
                            >
                                <MessageSquare className="w-3.5 h-3.5" />
                                <span>Input</span>
                            </button>

                            <button
                                type="button"
                                onClick={() => setPreviewTab('results')}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                    hasExpandToggle ? "rounded-none" : "rounded-r-md",
                                    previewTab === 'results'
                                        ? "bg-blue-600 text-white shadow-2xs"
                                        : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                                )}
                                title="View Output Results"
                            >
                                {selectedConversation?.is_running ? (
                                    <span className="relative flex h-2 w-2">
                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                        <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d]"></span>
                                    </span>
                                ) : (
                                    <Terminal className="w-3.5 h-3.5 text-cyan-500" />
                                )}
                                <span>Output</span>
                                {(selectedConversation?.execution_results || selectedConversation?.latest_response) && (
                                    <span className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
                                )}
                            </button>
                        </>
                    )}

                    {hasExpandToggle && (
                        <button
                            type="button"
                            onClick={() => setShowAllWords((prev) => !prev)}
                            className={cn(
                                "flex items-center gap-1 px-2.5 py-1 text-[10px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                setPreviewTab ? "rounded-r-md" : "rounded-md",
                                showAllWords
                                    ? "bg-cyan-500/20 text-cyan-700 dark:text-cyan-300"
                                    : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60"
                            )}
                            title={showAllWords ? 'Collapse full prompt text' : 'Expand full prompt text'}
                        >
                            {showAllWords ? (
                                <>
                                    <ChevronUp className="w-3 h-3" />
                                    <span>Collapse</span>
                                </>
                            ) : (
                                <>
                                    <ChevronDown className="w-3 h-3" />
                                    <span>Expand Full</span>
                                </>
                            )}
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
}
