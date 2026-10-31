import { Eye, Code, Edit3, Activity, ChevronUp, ChevronDown, MessageSquare, Terminal } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { PromptViewMode, type AgmConversationNode, type ViewMode } from './types';

export interface DetailTabsProps {
    previewTab: 'instruction' | 'results';
    setPreviewTab?: (v: 'instruction' | 'results') => void;
    selectedConversation?: AgmConversationNode;
    activeWordCount: number;
    activeByteCount: number;
    formatByteSize: (bytes: number) => string;
    isTruncated: boolean;
    activePromptText: string;
    showAllWords: boolean;
    setShowAllWords: (v: boolean | ((prev: boolean) => boolean)) => void;
    viewMode: ViewMode;
    setViewMode: (v: ViewMode) => void;
    setIsDetailsModalOpen?: (v: boolean) => void;
}

export function DetailTabs(props: DetailTabsProps) {
    const {
        previewTab,
        setPreviewTab,
        selectedConversation,
        activeWordCount,
        activeByteCount,
        formatByteSize,
        showAllWords,
        setShowAllWords,
        viewMode,
        setViewMode,
        setIsDetailsModalOpen,
    } = props;

    return (
        <div className="flex items-center justify-between gap-2.5 flex-wrap py-0.5 min-h-[32px]">
            {/* Left: View Mode Controls Capsule (Preview, Raw, Edit) */}
            <div className="flex items-center gap-2.5 shrink-0">
                <div className="inline-flex items-center rounded-md bg-slate-100/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-200 dark:border-slate-800 p-0.5 divide-x divide-slate-200 dark:divide-slate-800 shadow-2xs shrink-0">
                    <button
                        type="button"
                        onClick={() => setViewMode(PromptViewMode.Preview)}
                        className={cn(
                            'h-7 px-2.5 flex items-center justify-center rounded-l-md text-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            viewMode === PromptViewMode.Preview
                                ? 'bg-blue-500/15 text-blue-600 dark:bg-blue-500/20 dark:text-cyan-300 font-semibold shadow-2xs'
                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80'
                        )}
                        aria-label="Preview"
                    >
                        <Eye className="w-3.5 h-3.5" />
                    </button>
                    <button
                        type="button"
                        onClick={() => setViewMode(PromptViewMode.Raw)}
                        className={cn(
                            'h-7 px-2.5 flex items-center justify-center rounded-none text-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            viewMode === PromptViewMode.Raw
                                ? 'bg-blue-500/15 text-blue-600 dark:bg-blue-500/20 dark:text-cyan-300 font-semibold shadow-2xs'
                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80'
                        )}
                        aria-label="Raw"
                    >
                        <Code className="w-3.5 h-3.5" />
                    </button>
                    <button
                        type="button"
                        onClick={() => setViewMode(PromptViewMode.Edit)}
                        className={cn(
                            'h-7 px-2.5 flex items-center justify-center text-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            setIsDetailsModalOpen ? 'rounded-none' : 'rounded-r-md',
                            viewMode === PromptViewMode.Edit
                                ? 'bg-blue-500/15 text-blue-600 dark:bg-blue-500/20 dark:text-cyan-300 font-semibold shadow-2xs'
                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80'
                        )}
                        aria-label="Edit"
                    >
                        <Edit3 className="w-3.5 h-3.5" />
                    </button>
                    {setIsDetailsModalOpen && (
                        <button
                            type="button"
                            onClick={() => setIsDetailsModalOpen(true)}
                            className="h-7 px-2.5 rounded-r-md flex items-center justify-center gap-1.5 text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80 text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="Open Conversation & Instance Details"
                        >
                            <Activity className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                            <span className="leading-none">Details</span>
                            {selectedConversation?.is_running && (
                                <span className="w-2 h-2 rounded-full bg-[#1af18d] shadow-[0_0_6px_#1af18d] animate-pulse shrink-0" />
                            )}
                        </button>
                    )}
                </div>
            </div>

            {/* Right: Stats + Single Segmented Capsule ([Input] [Output] [Expand Full/Collapse]) */}
            <div className="flex items-center gap-2.5 ml-auto shrink-0">
                {previewTab === 'instruction' && (
                    <span className="text-xs font-semibold text-slate-500 dark:text-slate-400 font-mono">
                        {activeWordCount} words · {formatByteSize(activeByteCount)}
                    </span>
                )}

                <div className="inline-flex items-center rounded-md bg-slate-100/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-200 dark:border-slate-800 p-0.5 divide-x divide-slate-200 dark:divide-slate-800 shadow-2xs shrink-0">
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
                                aria-label="Input Instruction"
                            >
                                <MessageSquare className="w-3.5 h-3.5" />
                                <span>Input</span>
                            </button>

                            <button
                                type="button"
                                onClick={() => setPreviewTab('results')}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1 rounded-none text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                    previewTab === 'results'
                                        ? "bg-blue-600 text-white shadow-2xs"
                                        : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                                )}
                                aria-label="Output Results"
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

                    {/* Expand Full / Collapse button: always retained in the segmented capsule across input/output tab switching */}
                    <button
                        type="button"
                        onClick={() => setShowAllWords((prev) => !prev)}
                        className={cn(
                            "flex items-center gap-1 px-2.5 py-1 text-[10px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                            setPreviewTab ? "rounded-r-md" : "rounded-md",
                            showAllWords
                                ? "bg-cyan-500/20 text-cyan-700 dark:text-cyan-300"
                                : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-slate-800/80"
                        )}
                        aria-label={showAllWords ? 'Collapse full prompt text' : 'Expand full prompt text'}
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
                </div>
            </div>
        </div>
    );
}
