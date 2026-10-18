import { useState } from 'react';
import { Terminal, Copy, Check, ExternalLink, Sparkles, Wrench, ChevronDown, FileText, Code } from 'lucide-react';
import type { AgmConversationNode } from './types';
import { RichMarkdownRenderer } from './markdown';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';
import { copyPromptAiPayload, resolvePromptBody, type PromptAiCopyKind } from './usePromptTreePromptActions';

export interface ResultsViewProps {
    selectedConversation: AgmConversationNode;
    isCopiedResults: boolean;
    setIsCopiedResults: (v: boolean) => void;
    isFocusing: boolean;
    instancePid?: number;
    formatDuration: (secs: number) => string;
    onFocusIde: () => void;
    elapsedSeconds: number;
    showAllWords?: boolean;
}

export function ResultsView(props: ResultsViewProps) {
    const {
        selectedConversation,
        isCopiedResults, setIsCopiedResults,
        isFocusing, instancePid, formatDuration, onFocusIde,
        elapsedSeconds, showAllWords,
    } = props;

    const [isAiCopyMenuOpen, setIsAiCopyMenuOpen] = useState(false);
    const [copiedAiKind, setCopiedAiKind] = useState<PromptAiCopyKind | null>(null);

    const copyPromptAndAi = (kind: PromptAiCopyKind) => {
        const promptBody = resolvePromptBody(selectedConversation);
        void copyPromptAiPayload(kind, selectedConversation, promptBody, `ResultsView.copyPromptAi.${kind}`)
            .then((ok) => {
                if (!ok) return;
                setCopiedAiKind(kind);
                setTimeout(() => setCopiedAiKind(null), 2000);
            })
            .catch((err: unknown) => {
                useErrorStore.getState().captureException(
                    err instanceof Error ? err : new Error(String(err)),
                    {
                        source: `ResultsView.copyPromptAi.${kind}`,
                        triggerAction: `copy_prompt_ai_${kind}`,
                    }
                );
            });
        setIsAiCopyMenuOpen(false);
    };

    const handleCopyResults = async () => {
        try {
            const parts: string[] = [];
            if (selectedConversation.latest_response) {
                parts.push(selectedConversation.latest_response);
            }
            if (selectedConversation.execution_results) {
                parts.push(`\n\n--- Execution Details ---\n${selectedConversation.execution_results}`);
            }
            const resText = parts.join('\n') || '';
            await copyToClipboard(resText);
            setIsCopiedResults(true);
            setTimeout(() => setIsCopiedResults(false), 2000);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ResultsView.handleCopyResults',
                triggerAction: 'copy_results',
            });
        }
    };

    return (
    <div className="space-y-3">
        {/* In-Flight Live Telemetry Status Banner if running */}
        {selectedConversation.is_running && (
            <div className="rounded-xl border border-emerald-500/40 bg-emerald-500/10 p-3.5 text-emerald-800 dark:text-emerald-200 shadow-xs space-y-2">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <span className="relative flex h-2.5 w-2.5">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-[#1af18d]"></span>
                        </span>
                        <span className="font-bold text-xs uppercase tracking-wider font-mono">
                            Live Execution in Progress
                        </span>
                    </div>
                    <div className="flex items-center gap-2 text-[10px] font-mono">
                        {instancePid && (
                            <span className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/10">
                                PID: {instancePid}
                            </span>
                        )}
                        <span className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/10">
                            Elapsed: {formatDuration(elapsedSeconds)}
                        </span>
                        <span className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/10">
                            {selectedConversation.step_count || 1} steps
                        </span>
                    </div>
                </div>
                {selectedConversation.latest_step_summary && (
                    <div className="flex items-center gap-2 text-xs bg-black/5 dark:bg-black/20 p-2 rounded-lg font-mono">
                        <Terminal className="w-3.5 h-3.5 text-cyan-400 shrink-0" />
                        <span className="truncate">{selectedConversation.latest_step_summary}</span>
                    </div>
                )}
                {selectedConversation.tool_calls_summary && (
                    <div className="flex items-center gap-2 text-xs text-amber-700 dark:text-amber-300">
                        <Wrench className="w-3.5 h-3.5 shrink-0" />
                        <span className="truncate">{selectedConversation.tool_calls_summary}</span>
                    </div>
                )}
            </div>
        )}

        {/* Display captured AI Response or Execution Results */}
        {(selectedConversation.execution_results || selectedConversation.latest_response) ? (
            <div className="space-y-3">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider">
                        Latest AI Response & Execution Output
                    </span>
                    <div className="relative flex items-center rounded-full bg-slate-100 dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] divide-x divide-slate-200 dark:divide-[#40285e] p-0.5 shadow-2xs">
                        <button
                            type="button"
                            onClick={handleCopyResults}
                            className="flex items-center gap-1.5 px-3 py-1 rounded-l-full text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                            aria-label="Copy AI execution results to clipboard"
                        >
                            {isCopiedResults ? (
                                <>
                                    <Check className="w-3.5 h-3.5 text-emerald-500" />
                                    <span>Copied AI Results!</span>
                                </>
                            ) : (
                                <>
                                    <Copy className="w-3.5 h-3.5" />
                                    <span>Copy AI Results</span>
                                </>
                            )}
                        </button>
                        <button
                            type="button"
                            onClick={() => setIsAiCopyMenuOpen((prev) => !prev)}
                            className="flex items-center justify-center px-2 py-1 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                            aria-label="More copy options"
                        >
                            <ChevronDown className={cn('w-3.5 h-3.5 text-slate-400 transition-transform', isAiCopyMenuOpen && 'rotate-180')} />
                        </button>
                        {isAiCopyMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-80 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 space-y-1"
                                onMouseLeave={() => setIsAiCopyMenuOpen(false)}
                            >
                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('json')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer"
                                >
                                    <span className="flex items-center gap-2">
                                        {copiedAiKind === 'json' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Code className="w-3.5 h-3.5 text-amber-500" />}
                                        {copiedAiKind === 'json' ? 'Copied AI JSON!' : 'Copy AI results as JSON'}
                                    </span>
                                    <span className="text-[10px] text-slate-400 font-mono">JSON</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('text')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer"
                                >
                                    <span className="flex items-center gap-2">
                                        {copiedAiKind === 'text' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-slate-400" />}
                                        {copiedAiKind === 'text' ? 'Copied text!' : 'Copy prompt and AI results as text'}
                                    </span>
                                    <span className="text-[10px] text-slate-400 font-mono">Text</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('md')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer"
                                >
                                    <span className="flex items-center gap-2">
                                        {copiedAiKind === 'md' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-cyan-500" />}
                                        {copiedAiKind === 'md' ? 'Copied markdown!' : 'Copy prompt and AI results as markdown'}
                                    </span>
                                    <span className="text-[10px] text-slate-400 font-mono">.md</span>
                                </button>
                            </div>
                        )}
                    </div>
                </div>

                {/* 1. Assistant Text Response */}
                {selectedConversation.latest_response && (
                    <div className={cn(
                        "rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#090611] p-5 text-xs text-slate-800 dark:text-slate-200 leading-relaxed overflow-y-auto shadow-inner",
                        showAllWords ? "max-h-none" : "max-h-[460px]"
                    )}>
                        <div className="flex items-center gap-1.5 mb-2 pb-1.5 border-b border-slate-200/60 dark:border-[#40285e]/60 text-emerald-600 dark:text-emerald-400 font-bold text-xs">
                            <Sparkles className="w-3.5 h-3.5" />
                            <span>AI Assistant Response</span>
                        </div>
                        <RichMarkdownRenderer
                            content={selectedConversation.latest_response}
                            showAllWords={true}
                            onToggleExpand={() => {}}
                            isTruncated={false}
                        />
                    </div>
                )}

                {/* 2. Tool & Terminal Execution Details */}
                {selectedConversation.execution_results && (
                    <div className={cn(
                        "rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-900 text-slate-100 p-4 font-mono text-[11px] leading-relaxed overflow-y-auto select-text shadow-inner",
                        showAllWords ? "max-h-none" : "max-h-[300px]"
                    )}>
                        <div className="flex items-center gap-1.5 mb-2 pb-1 border-b border-slate-800 text-cyan-400 font-semibold text-[10px]">
                            <Terminal className="w-3 h-3" />
                            <span>Execution & Tool Details</span>
                        </div>
                        <pre className="whitespace-pre-wrap break-words">{selectedConversation.execution_results}</pre>
                    </div>
                )}
            </div>
        ) : selectedConversation.is_running ? (
            <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-8 flex flex-col items-center justify-center text-center space-y-3">
                <div className="relative flex h-10 w-10 items-center justify-center">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <div className="relative flex h-8 w-8 items-center justify-center rounded-full bg-emerald-500 text-white shadow-md">
                        <Sparkles className="w-4 h-4 animate-spin" />
                    </div>
                </div>
                <div className="space-y-1">
                    <div className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider font-mono">
                        In-Flight Antigravity Execution Active
                    </div>
                    <p className="text-[11px] text-slate-500 dark:text-slate-400 max-w-md">
                        {selectedConversation.latest_step_summary || 'Agent is actively thinking, evaluating code, or executing tools in the main Antigravity window.'}
                    </p>
                </div>
                <div className="flex items-center gap-2 pt-1 font-mono text-[10px] text-emerald-700 dark:text-[#1af18d] bg-emerald-500/10 px-3 py-1 rounded-md border border-emerald-500/20">
                    <span>Elapsed: {formatDuration(elapsedSeconds)}</span>
                    <span>·</span>
                    <span>Step {selectedConversation.step_count || 1}</span>
                </div>
                <button
                    type="button"
                    onClick={onFocusIde}
                    disabled={isFocusing}
                    className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-md text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white transition-all duration-150 cursor-pointer shadow-xs mt-2"
                    title="Open and focus the Antigravity main window"
                >
                    <ExternalLink className="w-3.5 h-3.5" />
                    <span>Open in Antigravity Window</span>
                </button>
            </div>
        ) : (
            <div className="rounded-xl border border-dashed border-slate-200 dark:border-[#40285e] p-8 flex flex-col items-center justify-center text-center space-y-2 text-slate-400">
                <Terminal className="w-8 h-8 text-slate-400/80 mb-1" />
                <span className="font-semibold text-slate-600 dark:text-slate-300 text-xs">
                    No execution results recorded yet
                </span>
                <p className="text-[11px] max-w-sm text-slate-500">
                    This prompt has not produced output in the transcript log, or is currently waiting in queue.
                </p>
                <div className="flex items-center gap-2 pt-2">
                    <button
                        type="button"
                        onClick={onFocusIde}
                        disabled={isFocusing}
                        className="flex items-center gap-1 px-3 py-1.5 rounded-md text-xs font-semibold bg-blue-600 text-white hover:bg-blue-700 transition-colors cursor-pointer"
                    >
                        <ExternalLink className="w-3.5 h-3.5" />
                        <span>Open IDE Window</span>
                    </button>
                </div>
            </div>
        )}
    </div>
    );
}
