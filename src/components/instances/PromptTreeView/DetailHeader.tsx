import { useState } from 'react';
import { User, Bot, Terminal, Wrench, Clock, Copy, Check, Image as ImageIcon, ChevronDown, ExternalLink, RotateCw, ListPlus, FileText, Code, Maximize2 } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode } from './types';
import { formatSeqBadge, stripImagesFromPrompt } from './formatters';
import { classifyPromptTier } from './tierClassification';
import { isGhostConversation } from './conversationGuards';

export interface DetailHeaderProps {
    selectedConversation: AgmConversationNode;
    activePromptText: string;
    confirmationSuffix: string;
    setConfirmationSuffix: (v: string) => void;
    conversationTargetMode?: 'current' | 'new';
    setConversationTargetMode?: (v: 'current' | 'new') => void;
    isCopiedText: boolean;
    setIsCopiedText: (v: boolean) => void;
    isCopiedRaw: boolean;
    isCopied: boolean;
    setIsCopied: (v: boolean) => void;
    isEnqueueing: boolean;
    isResending: boolean;
    isFocusing: boolean;
    isExportMenuOpen: boolean;
    setIsExportMenuOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
    hasImages: boolean;
    instancePid?: number;
    instanceSeqNum: number;
    instanceExeName: string;
    instanceNameDisplay: string;
    formatDuration: (secs: number) => string;
    elapsedSeconds: number;
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
        isCopiedText, setIsCopiedText,
        isEnqueueing, isResending, isFocusing,
        isExportMenuOpen, setIsExportMenuOpen,
        hasImages, instancePid, instanceSeqNum, instanceExeName, instanceNameDisplay,
        formatDuration, elapsedSeconds,
        onCopyWithImages, onSaveImages, onExport, onFocusIde, onResend, onEnqueue, onOpenInspector,
    } = props;

    const [localCopiedText, setLocalCopiedText] = useState(false);
    const isCopiedTextActive = isCopiedText || localCopiedText;

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

                {/* Right: The 2 Canonical Segmented Dark-Glass Action Capsules */}
                <div className="flex items-center gap-1.5 overflow-x-auto no-scrollbar shrink-0">
                    {/* Capsule 1: Consolidated Content & Export Split Button */}
                    <div className="relative flex items-center rounded-full bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs shrink-0">
                        {/* Primary Click: Copy Clean Text */}
                        <button
                            type="button"
                            onClick={() => {
                                const cleanText = stripImagesFromPrompt(activePromptText);
                                navigator.clipboard.writeText(cleanText);
                                setLocalCopiedText(true);
                                setIsCopiedText(true);
                                setTimeout(() => {
                                    setLocalCopiedText(false);
                                    setIsCopiedText(false);
                                }, 2000);
                            }}
                            className="flex items-center gap-1 px-2.5 py-0.5 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="Copy clean prompt text (excluding embedded images)"
                        >
                            {isCopiedTextActive ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3 text-slate-500 dark:text-slate-400" />}
                            <span>{isCopiedTextActive ? 'Copied!' : 'Copy'}</span>
                        </button>

                        {/* Dropdown Chevron for Rich Copy & Export Options */}
                        <button
                            type="button"
                            onClick={() => setIsExportMenuOpen((prev) => !prev)}
                            className="flex items-center px-1.5 py-0.5 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="More copy and export options"
                        >
                            <ChevronDown className="w-3 h-3 text-slate-400" />
                        </button>

                        {/* Flyout Menu */}
                        {isExportMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-52 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl py-1 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md"
                                onMouseLeave={() => setIsExportMenuOpen(false)}
                            >
                                <button
                                    type="button"
                                    onClick={() => {
                                        onCopyWithImages();
                                        setIsExportMenuOpen(false);
                                    }}
                                    className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <Copy className="w-3.5 h-3.5 text-indigo-500" />
                                    <span>Copy with Rich Images</span>
                                </button>

                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            onSaveImages();
                                            setIsExportMenuOpen(false);
                                        }}
                                        className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                    >
                                        <ImageIcon className="w-3.5 h-3.5 text-blue-500" />
                                        <span>Save Images to Disk</span>
                                    </button>
                                )}

                                <div className="border-t border-slate-100 dark:border-slate-800 my-1" />

                                <button
                                    type="button"
                                    onClick={() => {
                                        onExport('md');
                                        setIsExportMenuOpen(false);
                                    }}
                                    className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <FileText className="w-3.5 h-3.5 text-cyan-500" />
                                    <span>Export as Markdown (.md)</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => {
                                        onExport('json');
                                        setIsExportMenuOpen(false);
                                    }}
                                    className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <Code className="w-3.5 h-3.5 text-amber-500" />
                                    <span>Export as JSON (.json)</span>
                                </button>
                            </div>
                        )}
                    </div>

                    {/* Capsule 2: Execution & Workflow */}
                    <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#40285e]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#40285e]/80 shadow-2xs shrink-0">
                        {/* Confirmation Suffix Dropdown */}
                        <div className="flex items-center px-1.5 py-0.5 rounded-l-full">
                            <select
                                value={confirmationSuffix}
                                onChange={(e) => setConfirmationSuffix(e.target.value)}
                                className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[85px] truncate pr-1 py-0"
                                title="Confirmation suffix appended on Resend"
                            >
                                <option value="None (Send as is)" className="bg-white dark:bg-[#171025]">Suffix: None</option>
                                <option value="Is it done?" className="bg-white dark:bg-[#171025]">Done?</option>
                                <option value="Is it released?" className="bg-white dark:bg-[#171025]">Released?</option>
                                <option value="Are you sure about it?" className="bg-white dark:bg-[#171025]">Sure?</option>
                                <option value="Double check all edge cases" className="bg-white dark:bg-[#171025]">Edge cases</option>
                                <option value="Verify build and tests" className="bg-white dark:bg-[#171025]">Verify tests</option>
                            </select>
                        </div>

                        {/* Destination Conversation Mode Dropdown */}
                        <div className="flex items-center px-1.5 py-0.5">
                            <select
                                value={conversationTargetMode}
                                onChange={(e) => setConversationTargetMode?.(e.target.value as 'current' | 'new')}
                                className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[90px] truncate pr-1 py-0"
                                title="Destination conversation mode for Send and Queue"
                            >
                                <option value="current" className="bg-white dark:bg-[#171025]">Current</option>
                                <option value="new" className="bg-white dark:bg-[#171025]">New Conv</option>
                            </select>
                        </div>

                        {/* Focus IDE Action (Icon-Only) */}
                        <button
                            type="button"
                            onClick={onFocusIde}
                            disabled={isFocusing}
                            className="flex items-center justify-center px-2 py-0.5 text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 rounded-none"
                            title="Focus Antigravity IDE Window or Launch Instance"
                        >
                            <ExternalLink className={cn('w-3 h-3 text-sky-500', isFocusing && 'animate-spin')} />
                        </button>

                        {/* Send Now Action (Primary Dispatch with Hotkey: N) */}
                        <button
                            type="button"
                            onClick={onResend}
                            disabled={isResending}
                            className="flex items-center gap-1 px-2 py-0.5 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 rounded-none"
                            title="Immediately inject prompt to running instance (Hotkey: N)"
                        >
                            <RotateCw className={cn('w-3 h-3 text-emerald-500', isResending && 'animate-spin')} />
                            <span>Send</span>
                            <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>
                        </button>

                        {/* Enqueue Action (Icon-Only) */}
                        <button
                            type="button"
                            onClick={onEnqueue}
                            disabled={isEnqueueing}
                            className="flex items-center justify-center px-2 py-0.5 text-purple-700 dark:text-purple-300 hover:bg-purple-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 rounded-none"
                            title="Enqueue Prompt into FIFO Scheduler Queue"
                        >
                            <ListPlus className="w-3.5 h-3.5 text-purple-500" />
                        </button>

                        {/* Full Inspector Action (Icon-Only) */}
                        <button
                            type="button"
                            onClick={() => onOpenInspector(selectedConversation)}
                            className="flex items-center justify-center px-2 py-0.5 text-blue-700 dark:text-cyan-300 hover:bg-blue-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer rounded-r-full"
                            title="Open Full-Screen Conversation Inspector"
                        >
                            <Maximize2 className="h-3 w-3 text-blue-500 dark:text-cyan-400" />
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
