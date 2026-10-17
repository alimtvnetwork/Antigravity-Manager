import { Eye, Code, Edit3, ChevronUp, ChevronDown, FileText } from 'lucide-react';
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
        activeWordCount, activeByteCount, formatByteSize,
        isTruncated, activePromptText, showAllWords, setShowAllWords,
        viewMode, setViewMode, setIsDetailsModalOpen,
    } = props;

    return (
        <div className="flex items-center justify-between gap-3 flex-wrap py-0.5 min-h-[32px]">
            {/* Left: Word count & Byte size stats + Expand toggle */}
            <div className="flex items-center gap-2">
                {previewTab === 'instruction' && (
                    <>
                        <span className="text-xs font-semibold text-slate-500 dark:text-slate-400 font-mono">
                            {activeWordCount} words · {formatByteSize(activeByteCount)}
                        </span>
                        {(isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
                            <button
                                type="button"
                                onClick={() => setShowAllWords(!showAllWords)}
                                className="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-semibold bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border border-cyan-500/20 hover:bg-cyan-500/20 transition-all cursor-pointer"
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
                    </>
                )}
            </div>

            {/* Right Controls: View Mode Capsule (Preview/Raw/Edit) + Details */}
            <div className="flex items-center gap-2 flex-wrap ml-auto">
                {previewTab === 'instruction' && (
                    <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#40285e]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#40285e]/80 shadow-2xs">
                        <button
                            type="button"
                            onClick={() => setViewMode('preview')}
                            className={cn(
                                'flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-l-full transition-colors cursor-pointer',
                                viewMode === 'preview'
                                    ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                                    : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60'
                            )}
                            title="Rich Markdown Preview"
                        >
                            <Eye className="w-3.5 h-3.5" />
                            <span>Preview</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setViewMode('raw')}
                            className={cn(
                                'flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-none transition-colors cursor-pointer',
                                viewMode === 'raw'
                                    ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                                    : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60'
                            )}
                            title="Plain Monospace Raw View"
                        >
                            <Code className="w-3.5 h-3.5" />
                            <span>Raw</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setViewMode('edit')}
                            className={cn(
                                'flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-r-full transition-colors cursor-pointer',
                                viewMode === 'edit'
                                    ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                                    : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60'
                            )}
                            title="Editable Textarea Mode"
                        >
                            <Edit3 className="w-3.5 h-3.5" />
                            <span>Edit</span>
                        </button>
                    </div>
                )}

                <button
                    type="button"
                    onClick={() => setIsDetailsModalOpen(true)}
                    className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-slate-100 dark:bg-[#090611] text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 border border-slate-200 dark:border-[#40285e] hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                    title="View full conversation & project details"
                >
                    <FileText className="w-3 h-3 text-slate-400" />
                    <span>Details</span>
                </button>
            </div>
        </div>
    );
}
