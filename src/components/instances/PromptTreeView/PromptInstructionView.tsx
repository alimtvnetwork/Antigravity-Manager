import { Copy, Check } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode, ViewMode } from './types';
import { RichMarkdownRenderer } from './markdown';
import { TruncatedContextCallout } from './TruncatedContextCallout';
import { countWords } from './formatters';

export interface PromptInstructionViewProps {
    selectedConversation: AgmConversationNode;
    previewTab: 'instruction' | 'results';
    viewMode: ViewMode;
    activePromptText: string;
    editedPromptText: string;
    setEditedPromptText: (v: string) => void;
    displayedMarkdown: string;
    showAllWords: boolean;
    setShowAllWords: (v: boolean | ((prev: boolean) => boolean)) => void;
    isTruncated: boolean;
    totalWords: number;
    activeWordCount: number;
    formatByteSize: (bytes: number) => string;
    isCopied: boolean;
    setIsCopied: (v: boolean) => void;
}

export function PromptInstructionView(props: PromptInstructionViewProps) {
    const {
        selectedConversation, previewTab, viewMode,
        activePromptText, editedPromptText, setEditedPromptText,
        displayedMarkdown, showAllWords, setShowAllWords,
        isTruncated, totalWords, activeWordCount, formatByteSize,
        isCopied, setIsCopied,
    } = props;
    return (
<div className="space-y-3">

        {/* Preview Mode (Rich Markdown) */}
        {viewMode === 'preview' && (
            <div className="space-y-3">

        <div
            onDoubleClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
            className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 text-xs text-slate-800 dark:text-slate-200 leading-relaxed max-h-[460px] overflow-y-auto cursor-pointer hover:border-blue-400 dark:hover:border-blue-500/50 transition-colors shadow-inner"
        >
            <RichMarkdownRenderer
                content={displayedMarkdown}
                showAllWords={showAllWords}
                onToggleExpand={() => setShowAllWords(!showAllWords)}
                isTruncated={isTruncated}
            />
            {(isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
                <div className="mt-3 pt-2 border-t border-slate-200/50 dark:border-[#15334d]/50 flex items-center">
                    <span
                        onClick={(e) => {
                            e.stopPropagation();
                            setShowAllWords(!showAllWords);
                        }}
                        title="Click to expand or collapse full prompt text"
                        className="cursor-pointer text-xs font-semibold text-cyan-600 dark:text-cyan-400 hover:underline flex items-center gap-1 select-none"
                    >
                        {showAllWords ? 'Collapse full text' : 'Expand full text'}
                    </span>
                </div>
            )}
        </div>

        {/* Truncation & Copy Controls */}
        <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
            <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                {isTruncated && !showAllWords
                    ? `${totalWords} words · ${formatByteSize(activeByteCount)}`
                    : `${activeWordCount} words · ${formatByteSize(activeByteCount)}`}
            </div>

            <div className="flex items-center gap-2">
                {(isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
                    <button
                        type="button"
                        onClick={() => setShowAllWords(!showAllWords)}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-blue-50 dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 border border-blue-200 dark:border-[#15334d] hover:bg-blue-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                        title={showAllWords ? 'Collapse preview to 120 words' : 'Expand full prompt'}
                    >
                        {showAllWords ? (
                            <>
                                <ChevronUp className="w-3.5 h-3.5" />
                                <span>Show Less</span>
                            </>
                        ) : (
                            <>
                                <ChevronDown className="w-3.5 h-3.5" />
                                <span>Show All ({totalWords || activeWordCount} words)</span>
                            </>
                        )}
                    </button>
                )}

                <button
                    type="button"
                    onClick={() => {
                        navigator.clipboard.writeText(activePromptText);
                        setIsCopied(true);
                        setTimeout(() => setIsCopied(false), 2000);
                    }}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                    title="Copy full prompt text to clipboard"
                >
                    {isCopied ? (
                        <>
                            <Check className="w-3.5 h-3.5 text-emerald-500" />
                            <span>Copied Full Prompt!</span>
                        </>
                    ) : (
                        <>
                            <Copy className="w-3.5 h-3.5" />
                            <span>Copy Full Prompt</span>
                        </>
                    )}
                </button>
            </div>
        </div>
    </div>
)}

{/* Tab 2: Raw Mode (Monospace Clean Text with Explicit <br /> Line Gaps) */}
{viewMode === 'raw' && (
    <div className="space-y-3">
        <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 font-mono text-xs text-slate-800 dark:text-slate-200 leading-relaxed whitespace-pre-wrap break-words max-h-[460px] overflow-y-auto select-text shadow-inner">
            {activePromptText ? (
                formatPromptForMarkdown(activePromptText).split('\n').map((line, idx, arr) => (
                    <span key={`raw-line-${idx}`} className="block">
                        {line || <br className="my-1.5 block select-none" />}
                        {idx < arr.length - 1 && <br className="my-1.5 block select-none" />}
                    </span>
                ))
            ) : (
                'No prompt content recorded.'
            )}
        </div>
        <div className="flex items-center justify-end pt-1">
            <button
                type="button"
                onClick={() => {
                    navigator.clipboard.writeText(activePromptText);
                    setIsCopied(true);
                    setTimeout(() => setIsCopied(false), 2000);
                }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                title="Copy raw text to clipboard"
            >
                {isCopied ? (
                    <>
                        <Check className="w-3.5 h-3.5 text-emerald-500" />
                        <span>Copied Raw!</span>
                    </>
                ) : (
                    <>
                        <Copy className="w-3.5 h-3.5" />
                        <span>Copy Raw</span>
                    </>
                )}
            </button>
        </div>
    </div>
)}

{/* Tab 3: Edit Mode (Editable Textarea) */}
{viewMode === 'edit' && (
    <div className="space-y-3">
        <div className="flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 font-mono">
            <span>Modify prompt instruction directly:</span>
            <span>
                {editedPromptText.length} chars · {countWords(editedPromptText)} words
            </span>
        </div>
        <textarea
            value={editedPromptText}
            onChange={(e) => setEditedPromptText(e.target.value)}
            className="w-full h-80 rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-4 font-mono text-xs text-slate-900 dark:text-slate-100 leading-relaxed focus:outline-none focus:ring-2 focus:ring-blue-500/40 resize-y"
            placeholder="Type or modify prompt instruction here..."
        />
        <div className="flex items-center justify-between pt-1">
            <button
                type="button"
                onClick={() => setEditedPromptText(activePromptText)}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer"
                title="Reset to original text"
            >
                <RotateCcw className="w-3.5 h-3.5 text-slate-400" />
                <span>Reset</span>
            </button>
            <button
                type="button"
                onClick={() => {
                    setActivePromptText(editedPromptText);
                    setActionMsg('Prompt changes saved to current session!');
                    setTimeout(() => setActionMsg(null), 2500);
                }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-emerald-50 dark:bg-[#0c2438] text-emerald-700 dark:text-emerald-300 border border-emerald-300/60 dark:border-[#15334d] hover:bg-emerald-100 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer"
                title="Save prompt changes"
            >
                <Check className="w-3.5 h-3.5 text-emerald-500" />
                <span>Save Changes</span>
            </button>
        </div>
    </div>
)}
</div>

    );
}
