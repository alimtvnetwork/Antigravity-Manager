import { copyToClipboard } from '../../../utils/clipboard';
import { useState } from 'react';
import { User, Bot, Terminal, Wrench, Clock, Copy, Check, Image as ImageIcon, ChevronDown, ExternalLink, RotateCw, ListPlus, FileText, Code, Maximize2 } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode } from './types';
import { formatSeqBadge, stripImagesFromPrompt } from './formatters';
import { copyPromptAiPayload, resolvePromptBody, type PromptAiCopyKind } from './usePromptTreePromptActions';
import { classifyPromptTier } from './tierClassification';
import { isGhostConversation } from './conversationGuards';
import { persistClientCrash } from '../../../lib/error-listener';
import { useErrorStore } from '../../../stores/error-store';

function reportPromptActionError(err: unknown, source: string): void {
    persistClientCrash(err, source);
    const errorObj = err instanceof Error ? err : new Error(String(err));
    useErrorStore.getState().captureException(errorObj, {
        source,
        triggerAction: source,
    });
}

export interface DetailHeaderProps {
    selectedConversation: AgmConversationNode;
    activePromptText: string;
    confirmationSuffix: string;
    setConfirmationSuffix: (v: string) => void;
    conversationTargetMode?: 'current' | 'new';
    setConversationTargetMode?: (v: 'current' | 'new') => void;
    newConversationTitle?: string;
    setNewConversationTitle?: (title: string) => void;
    isCopiedText: boolean;
    setIsCopiedText: (v: boolean) => void;
    isCopiedRaw: boolean;
    isCopied: boolean;
    setIsCopied: (v: boolean) => void;
    isCopiedAll?: boolean;
    setIsCopiedAll?: (v: boolean) => void;
    isEnqueueing: boolean;
    isResending: boolean;
    isFocusing: boolean;
    isCopyMenuOpen?: boolean;
    setIsCopyMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isSendMenuOpen?: boolean;
    setIsSendMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isEnqueueMenuOpen?: boolean;
    setIsEnqueueMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isExportMenuOpen?: boolean;
    setIsExportMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    hasImages: boolean;
    instancePid?: number;
    instanceSeqNum: number;
    instanceExeName: string;
    instanceNameDisplay: string;
    formatDuration: (secs: number) => string;
    elapsedSeconds: number;
    onCopyAllWithRichImages?: () => void;
    onCopyCleanText?: () => void;
    onCopyWithImages: () => void;
    onSaveImages: () => void;
    onExport: (format: 'md' | 'json') => void;
    onFocusIde: () => void;
    onResend: () => void;
    onEnqueue: () => void;
    onOpenInspector: (conv: AgmConversationNode) => void;
}

export function DetailHeader(props: DetailHeaderProps) {
    const {
        selectedConversation, activePromptText,
        confirmationSuffix, setConfirmationSuffix,
        conversationTargetMode = 'current', setConversationTargetMode,
        newConversationTitle: propNewConversationTitle,
        setNewConversationTitle: propSetNewConversationTitle,
        isCopiedText, setIsCopiedText,
        isCopiedAll = false, setIsCopiedAll,
        isEnqueueing, isResending, isFocusing,
        isCopyMenuOpen: propIsCopyMenuOpen, setIsCopyMenuOpen: propSetIsCopyMenuOpen,
        isSendMenuOpen: propIsSendMenuOpen, setIsSendMenuOpen: propSetIsSendMenuOpen,
        isEnqueueMenuOpen: propIsEnqueueMenuOpen, setIsEnqueueMenuOpen: propSetIsEnqueueMenuOpen,
        isExportMenuOpen, setIsExportMenuOpen,
        hasImages, instancePid, instanceSeqNum, instanceExeName, instanceNameDisplay,
        formatDuration, elapsedSeconds,
        onCopyAllWithRichImages, onCopyCleanText,
        onCopyWithImages, onSaveImages, onExport, onFocusIde, onResend, onEnqueue, onOpenInspector,
    } = props;

    const [localNewConvTitle, setLocalNewConvTitle] = useState('');
    const newConversationTitle = propNewConversationTitle ?? localNewConvTitle;
    const setNewConversationTitle = propSetNewConversationTitle ?? setLocalNewConvTitle;
    const [dispatchMode, setDispatchMode] = useState<'send' | 'enqueue'>('send');

    const [localCopiedText, setLocalCopiedText] = useState(false);
    const isCopiedTextActive = isCopiedText || localCopiedText;
    const [localCopiedAll, setLocalCopiedAll] = useState(false);
    const isCopiedAllActive = isCopiedAll || localCopiedAll;

    const [copiedAiKind, setCopiedAiKind] = useState<PromptAiCopyKind | null>(null);
    const [localIsCopyMenuOpen, setLocalIsCopyMenuOpen] = useState(false);
    const isCopyMenuOpen = propIsCopyMenuOpen ?? isExportMenuOpen ?? localIsCopyMenuOpen;
    const setIsCopyMenuOpen = propSetIsCopyMenuOpen ?? setIsExportMenuOpen ?? setLocalIsCopyMenuOpen;

    const copyPromptAndAi = (kind: PromptAiCopyKind) => {
        const promptBody = resolvePromptBody(selectedConversation, activePromptText);
        void copyPromptAiPayload(kind, selectedConversation, promptBody, `DetailHeader.copyPromptAi.${kind}`)
            .then((ok) => {
                if (!ok) return;
                setCopiedAiKind(kind);
                setTimeout(() => setCopiedAiKind(null), 2000);
            })
            .catch((err) => {
                reportPromptActionError(err, `DetailHeader.copyPromptAi.${kind}`);
            });
        setIsCopyMenuOpen(false);
    };

    const [localIsSendMenuOpen, setLocalIsSendMenuOpen] = useState(false);
    const isSendMenuOpen = propIsSendMenuOpen ?? propIsEnqueueMenuOpen ?? localIsSendMenuOpen;
    const setIsSendMenuOpen = propSetIsSendMenuOpen ?? propSetIsEnqueueMenuOpen ?? setLocalIsSendMenuOpen;

    return (
        <div className="space-y-2 pb-2.5 border-b border-slate-200 dark:border-[#40285e] shrink-0">
            {/* Row 1: Identity, Badges & Actions */}
            <div className="flex items-center justify-between gap-2.5 flex-wrap">
                {/* Left: Indicator 1 (Sequence + Tier), Indicator 2 (Status Capsule), Title, Instance Context */}
                <div className="flex items-center gap-2 flex-wrap min-w-0">
                    {/* Indicator 1: Clean Sequence Code with compact Origin Tier Icon */}
                    {(() => {
                        const rawSeq = selectedConversation.seq_code || 'C001';
                        const seqDisplay = formatSeqBadge(rawSeq, 'C001');
                        const tierInfo = classifyPromptTier(activePromptText, selectedConversation.title);
                        return (
                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20" title={`${tierInfo.tier}`}>
                                {tierInfo.tier === 'USER_PROMPT' && <User className="w-2.5 h-2.5 text-sky-500 shrink-0" />}
                                {tierInfo.tier === 'SUBAGENT_INSTRUCTION' && <Bot className="w-2.5 h-2.5 text-purple-500 shrink-0" />}
                                {tierInfo.tier === 'SYSTEM_MESSAGE' && <Terminal className="w-2.5 h-2.5 text-slate-500 shrink-0" />}
                                {tierInfo.tier === 'TOOL_OUTPUT' && <Wrench className="w-2.5 h-2.5 text-amber-500 shrink-0" />}
                                <span>{seqDisplay}</span>
                            </span>
                        );
                    })()}

                    {/* Indicator 2: Unified Status Capsule */}
                    {Boolean(selectedConversation.is_running) && !isGhostConversation(selectedConversation) ? (
                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[9.5px] font-bold font-mono bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 shadow-2xs animate-pulse">
                            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                            <span>RUNNING</span>
                            {instancePid ? <span className="opacity-80">PID: {instancePid}</span> : null}
                            <span className="border-l border-emerald-400/40 pl-1">{formatDuration(elapsedSeconds)}</span>
                        </span>
                    ) : Boolean(selectedConversation.is_queued) ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9.5px] font-bold font-mono bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs">
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>QUEUED</span>
                        </span>
                    ) : (
                        <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800/60 border border-slate-200 dark:border-slate-700">
                            IDLE
                        </span>
                    )}

                    {/* Title */}
                    <h3 className="text-xs font-bold text-slate-900 dark:text-white truncate max-w-[200px] md:max-w-xs" title={selectedConversation.title || selectedConversation.short_id}>
                        {selectedConversation.title || selectedConversation.short_id}
                    </h3>

                    {/* Subtle Instance Context (Breadcrumb style) */}
                    <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 truncate" title={`Instance #${instanceSeqNum} · ${instanceExeName} (${instanceNameDisplay})`}>
                        #{formatSeqBadge(selectedConversation.seq_code, `C${String(instanceSeqNum).padStart(3, '0')}`).replace(/^#/, '')} · {instanceNameDisplay}
                    </span>
                </div>

                {/* Right: The Unified Segmented Dark-Glass Action Capsule */}
                <div className="flex items-center shrink-0">
                    <div className="relative flex items-center rounded-full bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
                        {/* Segment 1: Content & Media Copy Split Button [ Copy All | v ] */}
                        <div className="flex items-center divide-x divide-slate-200/70 dark:divide-purple-500/20">
                            {/* Primary Click: One-Shot Copy All (Content + Images) */}
                            <button
                                type="button"
                                onClick={() => {
                                    try {
                                        if (onCopyAllWithRichImages) {
                                            onCopyAllWithRichImages();
                                        } else {
                                            onCopyWithImages();
                                        }
                                        setLocalCopiedAll(true);
                                        setIsCopiedAll?.(true);
                                        setTimeout(() => {
                                            setLocalCopiedAll(false);
                                            setIsCopiedAll?.(false);
                                        }, 2000);
                                    } catch (err) {
                                        reportPromptActionError(err, 'DetailHeader.copyAllWithRichImages');
                                    }
                                }}
                                className="flex items-center gap-1.5 px-3 py-1 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Copy full prompt text and all embedded images in one shot"
                            >
                                {isCopiedAllActive ? (
                                    <Check className="w-3 h-3 text-emerald-500 shrink-0" />
                                ) : (
                                    <Copy className="w-3 h-3 text-slate-500 dark:text-slate-400 shrink-0" />
                                )}
                                <span>{isCopiedAllActive ? 'Copied All!' : 'Copy All'}</span>
                            </button>

                            {/* Dropdown Chevron for Clean Text, Image Saving, and Exports */}
                            <button
                                type="button"
                                onClick={() => setIsCopyMenuOpen((prev) => !prev)}
                                className="flex items-center justify-center px-1.5 py-1 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="More copy and export options"
                            >
                                <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isCopyMenuOpen && "rotate-180")} />
                            </button>
                        </div>

                        {/* Segment 2: Unified Dispatch Split Button [ Send N | v ] */}
                        <div className="flex items-center divide-x divide-slate-200/70 dark:divide-purple-500/20">
                            {/* Main Button: Quick Send (with N hotkey badge) */}
                            {dispatchMode === 'enqueue' ? (
                                <button
                                    type="button"
                                    onClick={() => {
                                        void Promise.resolve(onEnqueue()).catch((err) => {
                                            reportPromptActionError(err, 'DetailHeader.onEnqueue');
                                        });
                                    }}
                                    disabled={isEnqueueing}
                                    className="flex items-center gap-1.5 px-3 py-1 text-amber-800 dark:text-amber-300 hover:bg-amber-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                                    title="Enqueue prompt into FIFO background scheduler queue"
                                >
                                    <ListPlus className={cn('w-3 h-3 text-amber-500 dark:text-amber-400', isEnqueueing && 'animate-pulse')} />
                                    <span>Enqueue</span>
                                </button>
                            ) : (
                                <button
                                    type="button"
                                    onClick={() => {
                                        void Promise.resolve(onResend()).catch((err) => {
                                            reportPromptActionError(err, 'DetailHeader.onResend');
                                        });
                                    }}
                                    disabled={isResending}
                                    className="flex items-center gap-1.5 px-3 py-1 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                                    title="Immediately inject prompt to running instance (Hotkey: N)"
                                >
                                    <RotateCw className={cn('w-3 h-3 text-emerald-500', isResending && 'animate-spin')} />
                                    <span>Send</span>
                                    <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>
                                </button>
                            )}

                            {/* Split ChevronDown: Unified Dispatch Configuration Popover/Menu */}
                            <button
                                type="button"
                                onClick={() => setIsSendMenuOpen((prev) => !prev)}
                                className="flex items-center justify-center px-1.5 py-1 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Unified dispatch configuration popover"
                            >
                                <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isSendMenuOpen && "rotate-180")} />
                            </button>
                        </div>

                        {/* Flyout 1: Copy Options Menu */}
                        {isCopyMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-80 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1"
                                onMouseLeave={() => setIsCopyMenuOpen(false)}
                            >
                                {/* Sub-item 1: Copy Clean Text Only */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        try {
                                            if (onCopyCleanText) {
                                                onCopyCleanText();
                                            } else {
                                                const cleanText = stripImagesFromPrompt(activePromptText);
                                                void copyToClipboard(cleanText).catch((err) => {
                                                    reportPromptActionError(err, 'DetailHeader.copyCleanText');
                                                });
                                                setLocalCopiedText(true);
                                                setIsCopiedText(true);
                                                setTimeout(() => {
                                                    setLocalCopiedText(false);
                                                    setIsCopiedText(false);
                                                }, 2000);
                                            }
                                        } catch (err) {
                                            reportPromptActionError(err, 'DetailHeader.onCopyCleanText');
                                        }
                                        setIsCopyMenuOpen(false);
                                    }}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {isCopiedTextActive ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-slate-400" />}
                                        <span>{isCopiedTextActive ? 'Copied Clean Text!' : 'Copy Clean Text'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">No Images</span>
                                </button>

                                {/* Sub-item 2: Copy with Images */}
                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            try {
                                                if (onCopyAllWithRichImages) {
                                                    onCopyAllWithRichImages();
                                                } else {
                                                    onCopyWithImages();
                                                }
                                                setLocalCopiedAll(true);
                                                setIsCopiedAll?.(true);
                                                setTimeout(() => {
                                                    setLocalCopiedAll(false);
                                                    setIsCopiedAll?.(false);
                                                }, 2000);
                                            } catch (err) {
                                                reportPromptActionError(err, 'DetailHeader.onCopyWithImages');
                                            }
                                            setIsCopyMenuOpen(false);
                                        }}
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            {isCopiedAllActive ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-purple-500" />}
                                            <span>{isCopiedAllActive ? 'Copied with Images!' : 'Copy with Images'}</span>
                                        </div>
                                        <span className="text-[10px] text-slate-400 font-mono">Rich HTML</span>
                                    </button>
                                )}

                                {/* Sub-item 3: Save Images to Disk */}
                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            try {
                                                onSaveImages();
                                            } catch (err) {
                                                reportPromptActionError(err, 'DetailHeader.onSaveImages');
                                            }
                                            setIsCopyMenuOpen(false);
                                        }}
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            <ImageIcon className="w-3.5 h-3.5 text-blue-500" />
                                            <span>Save Images to Disk</span>
                                        </div>
                                        <span className="text-[10px] text-slate-400 font-mono">Assets</span>
                                    </button>
                                )}

                                <div className="border-t border-slate-100 dark:border-[#40285e]/60 my-1" />

                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('json')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    title="Copy latest_response and execution_results as JSON"
                                >
                                    <div className="flex items-center gap-2">
                                        {copiedAiKind === 'json' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Code className="w-3.5 h-3.5 text-amber-500" />}
                                        <span>{copiedAiKind === 'json' ? 'Copied AI JSON!' : 'Copy AI results as JSON'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">JSON</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('text')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    title="Copy the prompt body and AI text as plain text"
                                >
                                    <div className="flex items-center gap-2">
                                        {copiedAiKind === 'text' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-slate-400" />}
                                        <span>{copiedAiKind === 'text' ? 'Copied text!' : 'Copy prompt and AI results as text'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">Text</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('md')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    title="Copy the prompt body and AI text as markdown"
                                >
                                    <div className="flex items-center gap-2">
                                        {copiedAiKind === 'md' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-cyan-500" />}
                                        <span>{copiedAiKind === 'md' ? 'Copied markdown!' : 'Copy prompt and AI results as markdown'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">.md</span>
                                </button>

                                <div className="border-t border-slate-100 dark:border-[#40285e]/60 my-1" />

                                {/* Sub-item 4: Export as Markdown */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        onExport('md');
                                        setIsCopyMenuOpen(false);
                                    }}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <FileText className="w-3.5 h-3.5 text-cyan-500" />
                                        <span>Export as Markdown</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">.md</span>
                                </button>

                                {/* Sub-item 5: Export as JSON */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        onExport('json');
                                        setIsCopyMenuOpen(false);
                                    }}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <Code className="w-3.5 h-3.5 text-amber-500" />
                                        <span>Export as JSON</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">.json</span>
                                </button>
                            </div>
                        )}

                        {/* Flyout 2: Unified Dispatch Configuration Popover/Menu */}
                        {isSendMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-72 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-2 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1.5"
                                onMouseLeave={() => setIsSendMenuOpen(false)}
                            >
                                {/* Menu Header */}
                                <div className="px-2 pt-0.5 pb-1 text-[10px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 select-none border-b border-slate-100 dark:border-[#40285e]/60 flex items-center justify-between">
                                    <span>Dispatch Configuration</span>
                                    <span className="text-[9px] text-slate-400">AGM:P001</span>
                                </div>

                                {/* Mode Toggle: Send Immediately vs Enqueue to FIFO Queue */}
                                <div className="flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Mode:
                                    </span>
                                    <div className="inline-flex rounded-lg bg-slate-100 dark:bg-purple-950/40 p-0.5 border border-slate-200/80 dark:border-purple-500/20 text-[11px]">
                                        <button
                                            type="button"
                                            onClick={() => setDispatchMode('send')}
                                            className={cn(
                                                "px-2 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                dispatchMode === 'send'
                                                    ? "bg-white dark:bg-[#2c1a4d] text-emerald-600 dark:text-emerald-400 shadow-2xs font-semibold"
                                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                            )}
                                        >
                                            Send Now
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => setDispatchMode('enqueue')}
                                            className={cn(
                                                "px-2 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                dispatchMode === 'enqueue'
                                                    ? "bg-white dark:bg-[#2c1a4d] text-amber-600 dark:text-amber-400 shadow-2xs font-semibold"
                                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                            )}
                                        >
                                            Enqueue FIFO
                                        </button>
                                    </div>
                                </div>

                                {/* Destination: Current Conversation vs New Conversation */}
                                <div className="flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Destination:
                                    </span>
                                    <select
                                        value={conversationTargetMode}
                                        onChange={(e) => setConversationTargetMode?.(e.target.value as 'current' | 'new')}
                                        className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-xs font-semibold focus:ring-0 focus:outline-none cursor-pointer py-0.5 text-right"
                                        title="Destination conversation mode"
                                    >
                                        <option value="current" className="bg-white dark:bg-[#171025]">Current Conv</option>
                                        <option value="new" className="bg-white dark:bg-[#171025]">New Conv</option>
                                    </select>
                                </div>

                                {/* When "New Conversation" is selected: Inline title input */}
                                {conversationTargetMode === 'new' && (
                                    <div className="px-2 py-1.5 space-y-1 bg-slate-50/80 dark:bg-purple-950/20 rounded-lg border border-slate-200/60 dark:border-purple-500/20">
                                        <label className="text-[10px] font-mono text-slate-500 dark:text-slate-400 block">
                                            New Conversation Title:
                                        </label>
                                        <input
                                            type="text"
                                            value={newConversationTitle}
                                            onChange={(e) => setNewConversationTitle(e.target.value)}
                                            placeholder="e.g. Bugfix auth refresh, Feature draft..."
                                            className="w-full px-2 py-1 text-xs bg-white dark:bg-[#120a1f] border border-slate-200 dark:border-purple-500/30 rounded-md text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-purple-500/50"
                                        />
                                    </div>
                                )}

                                {/* Suffix presets */}
                                <div className="flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Suffix:
                                    </span>
                                    <select
                                        value={confirmationSuffix}
                                        onChange={(e) => setConfirmationSuffix(e.target.value)}
                                        className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-xs font-semibold focus:ring-0 focus:outline-none cursor-pointer py-0.5 text-right max-w-[140px] truncate"
                                        title="Confirmation suffix appended on dispatch"
                                    >
                                        <option value="None (Send as is)" className="bg-white dark:bg-[#171025]">None</option>
                                        <option value="Is it done?" className="bg-white dark:bg-[#171025]">Done?</option>
                                        <option value="Is it released?" className="bg-white dark:bg-[#171025]">Released?</option>
                                        <option value="Are you sure about it?" className="bg-white dark:bg-[#171025]">Sure?</option>
                                        <option value="Double check all edge cases" className="bg-white dark:bg-[#171025]">Edge cases</option>
                                        <option value="Verify build and tests" className="bg-white dark:bg-[#171025]">Verify tests</option>
                                    </select>
                                </div>

                                <div className="border-t border-slate-100 dark:border-[#40285e]/60 my-1" />

                                {/* Focus IDE Window option */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        onFocusIde();
                                        setIsSendMenuOpen(false);
                                    }}
                                    disabled={isFocusing}
                                    className="w-full flex items-center justify-between px-2 py-1.5 text-xs text-sky-700 dark:text-sky-300 hover:bg-sky-50 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors disabled:opacity-50"
                                >
                                    <div className="flex items-center gap-2">
                                        <ExternalLink className={cn('w-3.5 h-3.5 text-sky-500 shrink-0', isFocusing && 'animate-spin')} />
                                        <span className="font-medium">Focus IDE Window</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">Launch/Focus</span>
                                </button>

                                {/* Full Inspector option */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        onOpenInspector(selectedConversation);
                                        setIsSendMenuOpen(false);
                                    }}
                                    className="w-full flex items-center justify-between px-2 py-1.5 text-xs text-blue-700 dark:text-cyan-300 hover:bg-blue-50 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <Maximize2 className="w-3.5 h-3.5 text-blue-500 dark:text-cyan-400 shrink-0" />
                                        <span className="font-medium">Full Inspector</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">Full-Screen</span>
                                </button>

                                {/* Dispatch Action Button inside dropdown */}
                                <div className="pt-1 border-t border-slate-100 dark:border-[#40285e]/60">
                                    {dispatchMode === 'enqueue' ? (
                                        <button
                                            type="button"
                                            onClick={() => {
                                                void Promise.resolve(onEnqueue()).catch((err) => {
                                                    reportPromptActionError(err, 'DetailHeader.onEnqueue');
                                                });
                                                setIsSendMenuOpen(false);
                                            }}
                                            disabled={isEnqueueing}
                                            className="w-full flex items-center justify-center gap-2 py-1.5 px-3 bg-amber-500/15 hover:bg-amber-500/25 text-amber-700 dark:text-amber-300 border border-amber-500/30 rounded-lg text-xs font-semibold cursor-pointer transition-colors disabled:opacity-50"
                                        >
                                            <ListPlus className={cn('w-3.5 h-3.5 text-amber-500', isEnqueueing && 'animate-pulse')} />
                                            <span>Enqueue to FIFO Queue</span>
                                        </button>
                                    ) : (
                                        <button
                                            type="button"
                                            onClick={() => {
                                                void Promise.resolve(onResend()).catch((err) => {
                                                    reportPromptActionError(err, 'DetailHeader.onResend');
                                                });
                                                setIsSendMenuOpen(false);
                                            }}
                                            disabled={isResending}
                                            className="w-full flex items-center justify-center gap-2 py-1.5 px-3 bg-emerald-500/15 hover:bg-emerald-500/25 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30 rounded-lg text-xs font-semibold cursor-pointer transition-colors disabled:opacity-50"
                                        >
                                            <RotateCw className={cn('w-3.5 h-3.5 text-emerald-500', isResending && 'animate-spin')} />
                                            <span>Send Immediately</span>
                                        </button>
                                    )}
                                </div>

                                {/* FIFO Scheduler Notice */}
                                <div className="px-2 pt-0.5 text-[9.5px] text-slate-400 font-mono text-center">
                                    FIFO Queue &amp; Direct Dispatch Invariant
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
