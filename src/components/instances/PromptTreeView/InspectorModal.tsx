import { X, Copy, Check, Image as ImageIcon, FileText } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { RichMarkdownRenderer } from './markdown';
import type { InspectorPrompt } from './usePromptTreeState';

export interface InspectorModalProps {
    inspectorPrompt: InspectorPrompt;
    showAllWords: boolean;
    isCopied: boolean;
    setIsCopied: (v: boolean) => void;
    isCopiedRaw: boolean;
    onClose: () => void;
    onCopyWithImages: () => void;
}

export function InspectorModal(props: InspectorModalProps) {
    const {
        inspectorPrompt, showAllWords,
        isCopied, setIsCopied, isCopiedRaw,
        onClose, onCopyWithImages,
    } = props;
    return (
    <div className="fixed inset-0 z-[320] flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200">
        <div className="flex h-[92vh] w-full max-w-5xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl">
            {/* Inspector Header */}
            <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#15334d] px-6 py-4 bg-slate-50 dark:bg-[#071a27] shrink-0">
                <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-[5px] bg-purple-500/10 text-purple-600 dark:text-purple-400 border border-purple-500/20">
                        <FileText className="h-5 w-5" />
                    </div>
                    <div>
                        <h3 className="text-base font-bold text-slate-900 dark:text-white">
                            {inspectorPrompt.title}
                        </h3>
                        <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 font-mono mt-0.5">
                            <span>Conv: {inspectorPrompt.convId}</span>
                            <span>·</span>
                            <span className="truncate max-w-md">{inspectorPrompt.projectPath}</span>
                        </div>
                    </div>
                </div>

                {/* Inspector Actions Capsule */}
                <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs shrink-0">
                    <button
                        type="button"
                        onClick={() => {
                            const cleanText = stripImagesFromPrompt(inspectorPrompt.text);
                            navigator.clipboard.writeText(cleanText);
                            setIsCopied(true);
                            setTimeout(() => setIsCopied(false), 2000);
                        }}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Copy clean prompt text"
                    >
                        {isCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400" />}
                        <span>{isCopied ? 'Copied!' : 'Copy Text'}</span>
                    </button>
                    <button
                        type="button"
                        onClick={onCopyWithImages}
                        className="flex items-center gap-1.5 px-3 py-1.5 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Copy rich text with images"
                    >
                        {isCopiedRaw ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-indigo-500" />}
                        <span>{isCopiedRaw ? 'Copied + Imgs!' : '+ Imgs'}</span>
                    </button>
                    <button
                        type="button"
                        onClick={onClose}
                        className="flex items-center px-2.5 py-1.5 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 dark:hover:bg-rose-500/20 rounded-r-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Close Inspector"
                    >
                        <X className="h-4 w-4" />
                    </button>
                </div>
            </div>

            {/* Inspector Body */}
            <div className="flex-1 overflow-y-auto p-6 space-y-6">
                {/* Images Gallery */}
                {inspectorPrompt.images.length > 0 && (
                    <div className="space-y-2">
                        <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                            <ImageIcon className="h-4 w-4 text-blue-500" />
                            <span>Embedded Images ({inspectorPrompt.images.length})</span>
                        </div>
                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                            {inspectorPrompt.images.map((img, i) => (
                                <div
                                    key={i}
                                    className="overflow-hidden rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-slate-100 dark:bg-[#071a27] p-2"
                                >
                                    <img
                                        src={img}
                                        alt={`Prompt Image ${i + 1}`}
                                        className="w-full h-44 object-contain rounded-[4px]"
                                        onError={(e) => {
                                            (e.target as HTMLElement).style.display = 'none';
                                        }}
                                    />
                                    <div className="mt-1.5 truncate text-[10px] font-mono text-slate-500 dark:text-slate-400">
                                        {img.substring(0, 60)}...
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Referenced Files */}
                {inspectorPrompt.files.length > 0 && (
                    <div className="space-y-2">
                        <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                            <Folder className="h-4 w-4 text-amber-500" />
                            <span>Referenced Files ({inspectorPrompt.files.length})</span>
                        </div>
                        <div className="flex flex-wrap gap-2">
                            {inspectorPrompt.files.map((file, i) => (
                                <span
                                    key={i}
                                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-[5px] bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d] text-xs font-mono text-slate-700 dark:text-slate-300"
                                >
                                    <FileText className="w-3 h-3 text-slate-400" />
                                    <span className="truncate max-w-xs">{file}</span>
                                </span>
                            ))}
                        </div>
                    </div>
                )}

                {/* Full Prompt Text (Rendered Markdown) */}
                <div className="space-y-2">
                    <label className="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                        Full Prompt Instruction
                    </label>
                    <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 text-xs text-slate-900 dark:text-slate-100 leading-relaxed max-h-[600px] overflow-y-auto select-text shadow-inner">
                        <RichMarkdownRenderer
                            content={inspectorPrompt.text}
                            showAllWords={showAllWords}
                            onToggleExpand={() => setShowAllWords(!showAllWords)}
                            isTruncated={inspectorPrompt.text.trim().endsWith('...') || inspectorPrompt.text.trim().endsWith('…')}
                        />
                        {(inspectorPrompt.text.trim().endsWith('...') || inspectorPrompt.text.trim().endsWith('…') || showAllWords) && (
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
                </div>
            </div>
        </div>
    </div>

    );
}
