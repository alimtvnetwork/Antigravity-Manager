import { copyToClipboard } from '../../../utils/clipboard';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { User, Bot, Terminal, Wrench, Clock, Copy, Check, Image as ImageIcon, ChevronDown, ExternalLink, RotateCw, ListPlus, FileText, Code, Maximize2, Repeat } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { PromptOriginTier, CopyFormat, type AgmConversationNode } from './types';
import { formatSeqBadge, stripImagesFromPrompt } from './formatters';
import { copyPromptAiPayload, resolvePromptBody, dispatchRerunMessages, RERUN_MESSAGE_OPTIONS, RERUN_RANDOM_MESSAGE_SET, RERUN_DEFAULT_COUNT, RERUN_MIN_COUNT, RERUN_MAX_COUNT, type PromptAiCopyKind, type RerunMessageOption } from './usePromptTreePromptActions';
import { classifyPromptTier } from './tierClassification';
import { isGhostConversation } from './conversationGuards';
import { persistClientCrash } from '../../../lib/error-listener';
import { useErrorStore } from '../../../stores/error-store';
import { VERIFICATION_OPTIONS, parseVerificationTypeFromSuffix, getCustomSuffixTemplates, type SuffixTemplate } from './verificationTemplates';
import { SuffixTemplateModal } from './SuffixTemplateModal';

function DispatchFieldHint({ id, text }: { id: string; text: string }) {
    return (
        <span
            id={id}
            role="tooltip"
            className="pointer-events-none absolute right-0 top-full z-30 mt-1 hidden max-w-full rounded-md border border-slate-200 bg-white px-2 py-1 text-left text-[10px] font-medium leading-snug text-slate-800 shadow-md group-hover/field:block dark:border-[#40285e] dark:bg-[#0e0818] dark:text-slate-100"
        >
            {text}
        </span>
    );
}

function reportPromptActionError(err: unknown, source: string): void {
    persistClientCrash(err, source);
    const errorObj = err instanceof Error ? err : new Error(String(err));
    useErrorStore.getState().captureException(errorObj, {
        source,
        triggerAction: source,
    });
}

export interface DetailHeaderProps {
    previewTab?: 'instruction' | 'results';
    setPreviewTab?: (v: 'instruction' | 'results') => void;
    showAllWords?: boolean;
    setShowAllWords?: (v: boolean | ((prev: boolean) => boolean)) => void;
    isTruncated?: boolean;
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
    /** Optional repo path used as the rerun dispatch target (falls back to ''). */
    rerunRepoPath?: string;
    onOpenInspector: (conv: AgmConversationNode) => void;
    selectedTurnNumber?: number | null;
    setIsDetailsModalOpen?: (v: boolean) => void;
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
        onCopyWithImages, onSaveImages, onExport, onFocusIde, onResend, onEnqueue, rerunRepoPath, onOpenInspector,
        selectedTurnNumber,
        setIsDetailsModalOpen,
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

    // Double-click inline title editing state
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [editedTitle, setEditedTitle] = useState(selectedConversation.title || selectedConversation.short_id || '');

    useEffect(() => {
        setEditedTitle(selectedConversation.title || selectedConversation.short_id || '');
        setIsEditingTitle(false);
    }, [selectedConversation.conversation_id, selectedConversation.title, selectedConversation.short_id]);

    const handleCommitTitle = async () => {
        const trimmed = editedTitle.trim();
        setIsEditingTitle(false);
        if (!trimmed || trimmed === (selectedConversation.title || selectedConversation.short_id)) {
            setEditedTitle(selectedConversation.title || selectedConversation.short_id || '');
            return;
        }
        try {
            await invoke('rename_conversation', {
                instanceId: selectedConversation.instance_id || 'default',
                conversationId: selectedConversation.conversation_id,
                newTitle: trimmed,
            });
            selectedConversation.title = trimmed;
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.renameConversation');
        }
    };

    const [isTitleCopied, setIsTitleCopied] = useState(false);

    const handleCopyConversationTitle = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const titleToCopy = selectedConversation.title || selectedConversation.short_id || '';
        if (!titleToCopy) return;
        try {
            await copyToClipboard(titleToCopy);
            setIsTitleCopied(true);
            setTimeout(() => setIsTitleCopied(false), 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.copyConversationTitle');
        }
    };

    // Suffix Template Library & Popover state
    const [isSuffixPopoverOpen, setIsSuffixPopoverOpen] = useState(false);
    const [isSuffixModalOpen, setIsSuffixModalOpen] = useState(false);
    const [customTemplates, setCustomTemplates] = useState<SuffixTemplate[]>([]);

    useEffect(() => {
        setCustomTemplates(getCustomSuffixTemplates());
    }, [isSendMenuOpen, confirmationSuffix]);

    // RERUN SECTION (Task-04): state + dispatch handler for the rerun capsule.
    const [isRerunMenuOpen, setIsRerunMenuOpen] = useState(false);
    const [rerunCount, setRerunCount] = useState<number>(RERUN_DEFAULT_COUNT);
    const [rerunMessageOption, setRerunMessageOption] = useState<RerunMessageOption>('is-it-done');
    const [isRerunning, setIsRerunning] = useState(false);
    const [rerunFlash, setRerunFlash] = useState<string | null>(null);

    const handleRerun = async (): Promise<void> => {
        if (isRerunning) return;
        setIsRerunMenuOpen(false);
        setIsRerunning(true);
        setRerunFlash(null);
        try {
            const targetInstId = selectedConversation.instance_id || 'default';
            const targetConvId = conversationTargetMode === 'new'
                ? (newConversationTitle?.trim() || undefined)
                : selectedConversation.conversation_id;
            const originalPrompt = activePromptText || selectedConversation?.prompt_preview_200w || '';
            const result = await dispatchRerunMessages({
                targetInstId,
                repoPath: rerunRepoPath ?? '',
                convId: targetConvId,
                count: rerunCount,
                messageOption: rerunMessageOption,
                originalPrompt,
                source: 'DetailHeader.rerunSection',
            } as any);
            if (result.failed > 0) {
                // Each failed rerun already landed in the error module via
                // dispatchRerunMessages; surface a visible summary as well.
                reportPromptActionError(
                    new Error(`Rerun: ${result.failed} of ${result.requested} dispatch(es) failed`),
                    'DetailHeader.rerunSection'
                );
            } else if (result.dispatched > 0) {
                setRerunFlash(`Sent ×${result.dispatched}`);
                setTimeout(() => setRerunFlash(null), 2500);
            }
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.rerunSection');
        } finally {
            setIsRerunning(false);
        }
    };
    // END RERUN SECTION (Task-04)

    const handleCopyAll = () => {
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
    };

    const handleCopyCleanText = () => {
        try {
            if (onCopyCleanText) {
                onCopyCleanText();
            } else {
                const cleanText = stripImagesFromPrompt(activePromptText);
                void copyToClipboard(cleanText).catch((err) => {
                    reportPromptActionError(err, 'DetailHeader.copyCleanText');
                });
            }
            setLocalCopiedText(true);
            setIsCopiedText(true);
            setTimeout(() => {
                setLocalCopiedText(false);
                setIsCopiedText(false);
            }, 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopyCleanText');
        }
    };

    const handleCopyByFormat = (format: CopyFormat) => {
        switch (format) {
            case CopyFormat.RichMedia:
                handleCopyAll();
                break;
            case CopyFormat.CleanText:
                handleCopyCleanText();
                break;
            case CopyFormat.Json:
                onExport('json');
                break;
        }
    };

    const activeTurnNumber = selectedTurnNumber ?? selectedConversation.selected_turn_number ?? null;
    const enqueuedCount = selectedConversation.prompts?.filter((p) => p.is_queued || p.status === 'QUEUED').length || (selectedConversation.is_queued ? 1 : 0);

    return (
        <div className="pb-2.5 border-b border-slate-200 dark:border-[#40285e] shrink-0 relative z-30">
            {/* Header Row: Identity & Badges & Breadcrumb Context (Left) + Action Capsule (Right) */}
            <div className="flex items-center justify-between gap-2.5 flex-wrap min-w-0">
                {/* Left: Indicator 1 (Sequence + Tier), Indicator 2 (Status Capsule), Title, Instance Context */}
                <div className="flex items-center gap-2 flex-wrap min-w-0">
                    {/* Indicator 1: Clean Sequence Code with compact Origin Tier Icon & Scope Differentiation */}
                    {(() => {
                        const rawSeq = selectedConversation.seq_code || 'C001';
                        const seqDisplay = formatSeqBadge(rawSeq, 'C001');
                        const tierInfo = classifyPromptTier(activePromptText, selectedConversation.title);
                        const isTurnSelected = activeTurnNumber !== null;

                        return (
                            <span
                                className={cn(
                                    "inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-mono font-bold border cursor-help transition-colors",
                                    isTurnSelected
                                        ? "bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-500/30 shadow-2xs"
                                        : "bg-blue-500/10 text-blue-600 dark:text-cyan-400 border-blue-500/20 hover:border-blue-500/50"
                                )}
                                title={
                                    isTurnSelected
                                        ? `Turn T${activeTurnNumber} of ${selectedConversation.prompts?.length || selectedConversation.step_count || 1} · ${seqDisplay} · ${tierInfo.tier}`
                                        : `Conversation Session #${rawSeq.replace(/^C/i, '') || '001'} · ${tierInfo.tier} (Right-click to copy JSON parameters)`
                                }
                                onContextMenu={(e) => {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    const paramPayload = {
                                        sequence: seqDisplay,
                                        raw_sequence: rawSeq,
                                        tier: tierInfo.tier,
                                        selected_turn: activeTurnNumber,
                                        conversation_id: selectedConversation.conversation_id,
                                        instance_id: selectedConversation.instance_id || 'default',
                                        instance_name: instanceNameDisplay,
                                        title: selectedConversation.title || selectedConversation.short_id || '',
                                    };
                                    void copyToClipboard(JSON.stringify(paramPayload, null, 2)).catch((err) => {
                                        reportPromptActionError(err, 'DetailHeader.copyBadgeParams');
                                    });
                                }}
                            >
                                {tierInfo.tier === PromptOriginTier.UserPrompt && <User className="w-2.5 h-2.5 text-sky-500 shrink-0" />}
                                {tierInfo.tier === PromptOriginTier.SubagentInstruction && <Bot className="w-2.5 h-2.5 text-purple-500 shrink-0" />}
                                {tierInfo.tier === PromptOriginTier.SystemMessage && <Terminal className="w-2.5 h-2.5 text-slate-500 shrink-0" />}
                                {tierInfo.tier === PromptOriginTier.ToolOutput && <Wrench className="w-2.5 h-2.5 text-amber-500 shrink-0" />}
                                <span>{isTurnSelected ? `Turn T${activeTurnNumber} · ${seqDisplay}` : `Session ${seqDisplay}`}</span>
                            </span>
                        );
                    })()}

                    {/* Scope Breadcrumb Badge */}
                    {activeTurnNumber !== null ? (
                        <span className="text-[10px] font-mono text-purple-600 dark:text-purple-400 bg-purple-500/10 px-1.5 py-0.5 rounded-md border border-purple-500/20 font-semibold shrink-0">
                            ↳ Turn {activeTurnNumber} of {selectedConversation.prompts?.length || selectedConversation.step_count || 1}
                        </span>
                    ) : (
                        <span className="text-[10px] font-mono text-cyan-700 dark:text-cyan-300 bg-cyan-500/10 px-1.5 py-0.5 rounded-md border border-cyan-500/20 font-semibold shrink-0">
                            Full Session ({selectedConversation.prompts?.length || selectedConversation.step_count || 1} turns)
                        </span>
                    )}

                    {/* Indicator 2: Unified Status Capsule (normalized rounded-md) */}
                    {Boolean(selectedConversation.is_running) && !isGhostConversation(selectedConversation) ? (
                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[9.5px] font-bold font-mono bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 shadow-2xs animate-pulse">
                            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                            <span>RUNNING</span>
                            {instancePid ? <span className="opacity-80">PID: {instancePid}</span> : null}
                            <span className="border-l border-emerald-400/40 pl-1">{formatDuration(elapsedSeconds)}</span>
                        </span>
                    ) : Boolean(selectedConversation.is_queued) ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold font-mono bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs">
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>QUEUED</span>
                        </span>
                    ) : (
                        <span className="inline-flex items-center px-1.5 py-0.5 rounded-md text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800/60 border border-slate-200 dark:border-slate-700">
                            IDLE
                        </span>
                    )}

                    {/* Title with Double-Click Inline Rename and 1-Click Header Copy Button */}
                    {isEditingTitle ? (
                        <input
                            type="text"
                            autoFocus
                            value={editedTitle}
                            onChange={(e) => setEditedTitle(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    void handleCommitTitle();
                                } else if (e.key === 'Escape') {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    setEditedTitle(selectedConversation.title || selectedConversation.short_id || '');
                                    setIsEditingTitle(false);
                                }
                            }}
                            onBlur={() => void handleCommitTitle()}
                            onClick={(e) => e.stopPropagation()}
                            onDoubleClick={(e) => e.stopPropagation()}
                            className="px-1.5 py-0.5 text-xs font-bold font-ubuntu rounded bg-white dark:bg-[#120a1f] border border-purple-500/50 text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-purple-500 max-w-[200px] md:max-w-xs shadow-inner"
                        />
                    ) : (
                        <div className="flex items-center gap-1.5 min-w-0">
                            <h3
                                onDoubleClick={(e) => {
                                    e.stopPropagation();
                                    setIsEditingTitle(true);
                                }}
                                className="text-xs font-bold font-ubuntu text-slate-900 dark:text-white truncate max-w-[200px] md:max-w-xs cursor-pointer hover:text-purple-600 dark:hover:text-purple-300 transition-colors"
                                title={`${selectedConversation.title || selectedConversation.short_id} (Double-click to rename)`}
                            >
                                {selectedConversation.title || selectedConversation.short_id}
                            </h3>
                            <button
                                type="button"
                                onClick={handleCopyConversationTitle}
                                data-ui-name="header-copy-title-btn"
                                className="p-1 rounded-md text-slate-400 hover:text-slate-100 hover:bg-slate-800/60 active:scale-95 transition-all cursor-pointer shrink-0"
                                title="Copy conversation title"
                            >
                                {isTitleCopied ? (
                                    <Check className="w-3.5 h-3.5 text-emerald-500 animate-in zoom-in-75 duration-150" />
                                ) : (
                                    <Copy className="w-3.5 h-3.5" />
                                )}
                            </button>
                        </div>
                    )}

                    {/* Subtle Instance Context (Breadcrumb style) */}
                    <span
                        className="text-[10px] font-mono text-slate-400 dark:text-slate-500 truncate cursor-help hover:text-slate-200 transition-colors"
                        title={`Instance #${instanceSeqNum} · ${instanceExeName} (${instanceNameDisplay}) (Right-click to copy context JSON)`}
                        onContextMenu={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            const contextSeq = formatSeqBadge(selectedConversation.seq_code, `C${String(instanceSeqNum).padStart(3, '0')}`).replace(/^#/, '');
                            const contextPayload = {
                                context: `#${contextSeq} · ${instanceNameDisplay}`,
                                instance_seq: instanceSeqNum,
                                instance_name: instanceNameDisplay,
                                instance_exe: instanceExeName,
                                instance_pid: instancePid,
                                conversation_id: selectedConversation.conversation_id,
                            };
                            void copyToClipboard(JSON.stringify(contextPayload, null, 2)).catch((err) => {
                                reportPromptActionError(err, 'DetailHeader.copyContextParams');
                            });
                        }}
                    >
                        #{formatSeqBadge(selectedConversation.seq_code, `C${String(instanceSeqNum).padStart(3, '0')}`).replace(/^#/, '')} · {instanceNameDisplay}
                    </span>
                </div>

                {/* Right: The Unified Segmented Dark-Glass Action Capsule */}
                <div className="flex items-center shrink-0 ml-auto">
                    <div className="relative flex items-center rounded-lg bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
                        {/* Segment 1: Copy Split Button [ Copy All | v ] */}
                        <div className="flex items-center divide-x divide-slate-200/70 dark:divide-purple-500/20">
                            {/* Primary Button: Copy with images (Rich HTML + binary PNG) */}
                            <button
                                type="button"
                                onClick={handleCopyAll}
                                data-ui-name="prompt-copy-all-btn"
                                data-ui-id={`prompt-copy-all-${selectedConversation.conversation_id}`}
                                className="flex items-center gap-1.5 px-3 py-1 rounded-l-md text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Copy prompt text and all embedded images (rich HTML + binary PNG)"
                            >
                                {isCopiedAllActive ? (
                                    <Check className="w-3 h-3 text-emerald-500 shrink-0" />
                                ) : (
                                    <Copy className="w-3 h-3 text-purple-500 shrink-0" />
                                )}
                                <span>{isCopiedAllActive ? 'Copied All!' : 'Copy All'}</span>
                            </button>

                            {/* Dropdown Chevron for Clean Text, Image Saving, and Exports */}
                            <button
                                type="button"
                                onClick={() => setIsCopyMenuOpen((prev) => !prev)}
                                data-ui-name="prompt-copy-options-btn"
                                data-ui-id={`prompt-copy-options-${selectedConversation.conversation_id}`}
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
                                    data-ui-name="prompt-enqueue-btn"
                                    data-ui-id={`prompt-enqueue-${selectedConversation.conversation_id}`}
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
                                    data-ui-name="prompt-send-btn"
                                    data-ui-id={`prompt-send-${selectedConversation.conversation_id}`}
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
                                data-ui-name="prompt-dispatch-options-btn"
                                data-ui-id={`prompt-dispatch-options-${selectedConversation.conversation_id}`}
                                className="flex items-center justify-center px-1.5 py-1 rounded-none text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Unified dispatch configuration popover"
                            >
                                <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isSendMenuOpen && "rotate-180")} />
                            </button>
                        </div>

                        {/* Segment 3: Rerun Split Button [ Rerun xN | v ] */}
                        <div className="flex items-center divide-x divide-slate-200/70 dark:divide-purple-500/20">
                            {/* Main Button: Quick Rerun (with N count badge) */}
                            <button
                                type="button"
                                onClick={() => {
                                    void handleRerun().catch((err) => {
                                        reportPromptActionError(err, 'DetailHeader.rerunSection');
                                    });
                                }}
                                disabled={isRerunning}
                                data-ui-name="prompt-rerun-btn"
                                data-ui-id={`prompt-rerun-${selectedConversation.conversation_id}`}
                                className="flex items-center gap-1.5 px-3 py-1 rounded-none text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                                title={`Dispatch the chosen rerun nudge ${rerunCount} time(s) to the running instance`}
                            >
                                {rerunFlash ? (
                                    <Check className="w-3 h-3 text-emerald-500 shrink-0" />
                                ) : (
                                    <Repeat className={cn('w-3 h-3 text-sky-500 shrink-0', isRerunning && 'animate-spin')} />
                                )}
                                <span>{isRerunning ? 'Rerunning…' : rerunFlash ?? `Rerun ×${rerunCount}`}</span>
                            </button>

                            {/* Split ChevronDown: Rerun Configuration Flyout */}
                            <button
                                type="button"
                                onClick={() => setIsRerunMenuOpen((prev) => !prev)}
                                data-ui-name="prompt-rerun-options-btn"
                                data-ui-id={`prompt-rerun-options-${selectedConversation.conversation_id}`}
                                className={cn(
                                    "flex items-center justify-center px-1.5 py-1 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                    !setIsDetailsModalOpen && "rounded-r-md"
                                )}
                                title="Rerun configuration: count and message"
                            >
                                <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isRerunMenuOpen && "rotate-180")} />
                            </button>
                        </div>

                        {/* Segment 4: Details Button */}
                        {setIsDetailsModalOpen && (
                            <button
                                type="button"
                                onClick={() => setIsDetailsModalOpen(true)}
                                data-ui-name="prompt-details-btn"
                                data-ui-id={`prompt-details-${selectedConversation.conversation_id}`}
                                className="flex items-center gap-1.5 px-3 py-1 rounded-r-md text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Open Conversation & Instance Details (Target IDE, PID, executable path, process status)"
                            >
                                <FileText className="w-3.5 h-3.5 text-blue-500 shrink-0" />
                                <span>Details</span>
                            </button>
                        )}

                        {/* Flyout 1: Copy Options Menu */}
                        {isCopyMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-80 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1"
                                onMouseLeave={() => setIsCopyMenuOpen(false)}
                            >
                                {/* Option 1: Copy Full Prompt (includes image) */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        handleCopyByFormat(CopyFormat.RichMedia);
                                        setIsCopyMenuOpen(false);
                                    }}
                                    data-ui-name="prompt-menu-copy-full-text-btn"
                                    data-ui-id={`prompt-menu-copy-full-text-${selectedConversation.conversation_id}`}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {isCopiedAllActive ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-purple-500" />}
                                        <span>{isCopiedAllActive ? 'Copied Full Prompt!' : 'Copy Full Prompt'}</span>
                                    </div>
                                    <span className="text-[10px] text-purple-600 dark:text-purple-300 font-mono px-1.5 py-0.2 rounded-md bg-purple-500/10 border border-purple-500/20">includes image</span>
                                </button>

                                {/* Option 2: Copy without image (copy as text) */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        handleCopyByFormat(CopyFormat.CleanText);
                                        setIsCopyMenuOpen(false);
                                    }}
                                    data-ui-name="prompt-menu-copy-clean-text-btn"
                                    data-ui-id={`prompt-menu-copy-clean-${selectedConversation.conversation_id}`}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {isCopiedTextActive ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-slate-400" />}
                                        <span>{isCopiedTextActive ? 'Copied as text!' : 'Copy without image'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-500 dark:text-slate-400 font-mono px-1.5 py-0.2 rounded-md bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700">copy as text</span>
                                </button>

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
                                    className="group/field relative w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    aria-describedby="copy-ai-json-hint"
                                >
                                    <DispatchFieldHint id="copy-ai-json-hint" text="Copy latest_response and execution_results as JSON" />
                                    <div className="flex items-center gap-2">
                                        {copiedAiKind === 'json' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Code className="w-3.5 h-3.5 text-amber-500" />}
                                        <span>{copiedAiKind === 'json' ? 'Copied AI JSON!' : 'Copy AI results as JSON'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">JSON</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('text')}
                                    className="group/field relative w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    aria-describedby="copy-ai-text-hint"
                                >
                                    <DispatchFieldHint id="copy-ai-text-hint" text="Copy the prompt body and AI text as plain text" />
                                    <div className="flex items-center gap-2">
                                        {copiedAiKind === 'text' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-slate-400" />}
                                        <span>{copiedAiKind === 'text' ? 'Copied text!' : 'Copy prompt and AI results as text'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">Text</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('md')}
                                    className="group/field relative w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg text-left cursor-pointer transition-colors"
                                    aria-describedby="copy-ai-md-hint"
                                >
                                    <DispatchFieldHint id="copy-ai-md-hint" text="Copy the prompt body and AI text as markdown" />
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
                                <div className="group/field relative flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Destination:
                                    </span>
                                    <select
                                        value={conversationTargetMode}
                                        onChange={(e) => setConversationTargetMode?.(e.target.value as 'current' | 'new')}
                                        className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-xs font-semibold focus:ring-0 focus:outline-none cursor-pointer py-0.5 text-right"
                                        aria-describedby="dispatch-destination-hint"
                                    >
                                        <option value="current" className="bg-white dark:bg-[#171025]">Current Conv</option>
                                        <option value="new" className="bg-white dark:bg-[#171025]">New Conv</option>
                                    </select>
                                    <DispatchFieldHint id="dispatch-destination-hint" text="Destination conversation mode" />
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

                                {/* Suffix presets with custom dark-glass popover */}
                                {(() => {
                                    const parsedType = parseVerificationTypeFromSuffix(confirmationSuffix);
                                    const matchedBuiltin = VERIFICATION_OPTIONS.find((o) => o.value === parsedType);
                                    const matchedCustom = customTemplates.find((c) => c.id === confirmationSuffix);
                                    const currentSuffixLabel = matchedCustom ? matchedCustom.name : (matchedBuiltin?.dropdownLabel || matchedBuiltin?.label || 'None');

                                    return (
                                        <div className="group/field relative flex items-center justify-between px-2 py-1 text-xs">
                                            <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                                Suffix:
                                            </span>
                                            <div className="relative">
                                                <button
                                                    type="button"
                                                    onClick={() => setIsSuffixPopoverOpen((prev) => !prev)}
                                                    className="flex items-center gap-1.5 px-2 py-0.5 rounded-lg bg-slate-100 dark:bg-purple-950/40 border border-slate-200/80 dark:border-purple-500/20 text-slate-800 dark:text-slate-200 text-xs font-semibold hover:bg-slate-200/80 dark:hover:bg-purple-900/50 transition-colors cursor-pointer max-w-[140px]"
                                                    title="Select confirmation suffix or verification template"
                                                >
                                                    <span className="truncate">{currentSuffixLabel}</span>
                                                    <ChevronDown className={cn("w-3 h-3 text-slate-400 shrink-0 transition-transform duration-150", isSuffixPopoverOpen && "rotate-180")} />
                                                </button>

                                                {isSuffixPopoverOpen && (
                                                    <div
                                                        className="absolute right-0 top-full mt-1.5 z-50 w-64 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-2xl p-1.5 space-y-1 backdrop-blur-md max-h-72 overflow-y-auto animate-in fade-in zoom-in-95 duration-100"
                                                        onMouseLeave={() => setIsSuffixPopoverOpen(false)}
                                                    >
                                                        <div className="px-2 py-0.5 text-[9.5px] font-mono uppercase tracking-wider text-slate-400 border-b border-slate-100 dark:border-[#40285e]/60">
                                                            Built-in Presets
                                                        </div>
                                                        {VERIFICATION_OPTIONS.map((opt) => {
                                                            const isSelected = parsedType === opt.value && !matchedCustom;
                                                            return (
                                                                <button
                                                                    key={opt.value}
                                                                    type="button"
                                                                    onClick={() => {
                                                                        setConfirmationSuffix(opt.value === 'none' ? 'None (Send as is)' : opt.value);
                                                                        setIsSuffixPopoverOpen(false);
                                                                    }}
                                                                    className={cn(
                                                                        "w-full flex items-center justify-between px-2 py-1 rounded-lg text-left text-xs transition-colors cursor-pointer",
                                                                        isSelected
                                                                            ? "bg-purple-500/15 text-purple-700 dark:text-purple-300 font-semibold"
                                                                            : "text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e]"
                                                                    )}
                                                                >
                                                                    <span className="truncate">{opt.dropdownLabel || opt.label}</span>
                                                                    {isSelected && <Check className="w-3.5 h-3.5 text-purple-500 shrink-0" />}
                                                                </button>
                                                            );
                                                        })}

                                                        {customTemplates.length > 0 && (
                                                            <>
                                                                <div className="px-2 pt-1.5 pb-0.5 text-[9.5px] font-mono uppercase tracking-wider text-slate-400 border-b border-slate-100 dark:border-[#40285e]/60 mt-1">
                                                                    Custom Templates
                                                                </div>
                                                                {customTemplates.map((ct) => {
                                                                    const isSelected = confirmationSuffix === ct.id;
                                                                    return (
                                                                        <button
                                                                            key={ct.id}
                                                                            type="button"
                                                                            onClick={() => {
                                                                                setConfirmationSuffix(ct.id);
                                                                                setIsSuffixPopoverOpen(false);
                                                                            }}
                                                                            className={cn(
                                                                                "w-full flex items-center justify-between px-2 py-1 rounded-lg text-left text-xs transition-colors cursor-pointer",
                                                                                isSelected
                                                                                    ? "bg-purple-500/15 text-purple-700 dark:text-purple-300 font-semibold"
                                                                                    : "text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e]"
                                                                            )}
                                                                        >
                                                                            <span className="truncate">{ct.name}</span>
                                                                            {isSelected && <Check className="w-3.5 h-3.5 text-purple-500 shrink-0" />}
                                                                        </button>
                                                                    );
                                                                })}
                                                            </>
                                                        )}

                                                        <div className="pt-1 border-t border-slate-100 dark:border-[#40285e]/60">
                                                            <button
                                                                type="button"
                                                                onClick={() => {
                                                                    setIsSuffixPopoverOpen(false);
                                                                    setIsSuffixModalOpen(true);
                                                                }}
                                                                className="w-full flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-semibold text-purple-600 dark:text-purple-400 hover:bg-purple-50 dark:hover:bg-purple-950/40 transition-colors cursor-pointer"
                                                            >
                                                                <span>+ Manage Suffix Templates...</span>
                                                            </button>
                                                        </div>
                                                    </div>
                                                )}
                                            </div>
                                            <DispatchFieldHint id="dispatch-suffix-hint" text="Confirmation suffix appended on dispatch" />
                                        </div>
                                    );
                                })()}

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

                        {/* Flyout 3: Rerun Configuration Flyout */}
                        {isRerunMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-72 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-2 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1.5"
                                onMouseLeave={() => setIsRerunMenuOpen(false)}
                            >
                                {/* Menu Header */}
                                <div className="px-2 pt-0.5 pb-1 text-[10px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 select-none border-b border-slate-100 dark:border-[#40285e]/60 flex items-center justify-between">
                                    <span>Rerun Configuration</span>
                                    <span className="text-[9px] text-slate-400">AGM:P001</span>
                                </div>

                                {/* Count selector: Times (N) */}
                                <div className="flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Times (N):
                                    </span>
                                    <div className="inline-flex rounded-lg bg-slate-100 dark:bg-purple-950/40 p-0.5 border border-slate-200/80 dark:border-purple-500/20 text-[11px]">
                                        {Array.from({ length: RERUN_MAX_COUNT - RERUN_MIN_COUNT + 1 }, (_, idx) => RERUN_MIN_COUNT + idx).map((n) => (
                                            <button
                                                key={n}
                                                type="button"
                                                onClick={() => setRerunCount(n)}
                                                className={cn(
                                                    "px-2 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                    rerunCount === n
                                                        ? "bg-white dark:bg-[#2c1a4d] text-sky-600 dark:text-sky-400 shadow-2xs font-semibold"
                                                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                                )}
                                                title={`Dispatch the nudge ${n} time(s)`}
                                            >
                                                {n}
                                            </button>
                                        ))}
                                    </div>
                                </div>

                                {/* Message options */}
                                <div className="px-2 py-1 text-[11px] font-medium text-slate-500 dark:text-slate-400">
                                    Message:
                                </div>
                                <div className="space-y-1">
                                    {RERUN_MESSAGE_OPTIONS.map((opt) => {
                                        const active = rerunMessageOption === opt.value;
                                        const isRandom = opt.value === 'randomized';
                                        return (
                                            <button
                                                key={opt.value}
                                                type="button"
                                                onClick={() => setRerunMessageOption(opt.value)}
                                                className={cn(
                                                    "w-full flex items-center justify-between px-2.5 py-1.5 text-xs rounded-lg text-left cursor-pointer transition-colors border",
                                                    active
                                                        ? "bg-sky-500/10 border-sky-500/40 text-slate-800 dark:text-slate-100"
                                                        : "border-transparent text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e]"
                                                )}
                                                title={opt.hint}
                                            >
                                                <span className="flex items-center gap-2">
                                                    <span className={cn("w-3 h-3 rounded-full border-2 flex items-center justify-center", active ? "border-sky-500" : "border-slate-300 dark:border-slate-600")}>
                                                        {active && <span className="w-1.5 h-1.5 rounded-full bg-sky-500" />}
                                                    </span>
                                                    <span className="font-medium">{opt.label}</span>
                                                </span>
                                                {isRandom && (
                                                    <span className="text-[9px] font-mono font-bold uppercase tracking-wider px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                                                        Randomized
                                                    </span>
                                                )}
                                            </button>
                                        );
                                    })}
                                </div>

                                {/* Built-in randomized set preview */}
                                {rerunMessageOption === 'randomized' && (
                                    <div className="mx-2 px-2 py-1.5 rounded-lg bg-slate-50/80 dark:bg-purple-950/20 border border-slate-200/60 dark:border-purple-500/20">
                                        <div className="text-[9.5px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 mb-1">
                                            Built-in set (one picked per dispatch)
                                        </div>
                                        <ul className="space-y-0.5 text-[10.5px] text-slate-500 dark:text-slate-400 list-disc list-inside">
                                            {RERUN_RANDOM_MESSAGE_SET.map((m) => (
                                                <li key={m}>{m}</li>
                                            ))}
                                        </ul>
                                    </div>
                                )}

                                {/* Dispatch Action Button inside flyout */}
                                <div className="pt-1 border-t border-slate-100 dark:border-[#40285e]/60">
                                    <button
                                        type="button"
                                        onClick={() => {
                                            void handleRerun().catch((err) => {
                                                reportPromptActionError(err, 'DetailHeader.rerunSection');
                                            });
                                        }}
                                        disabled={isRerunning}
                                        className="w-full flex items-center justify-center gap-2 py-1.5 px-3 bg-sky-500/15 hover:bg-sky-500/25 text-sky-700 dark:text-sky-300 border border-sky-500/30 rounded-lg text-xs font-semibold cursor-pointer transition-colors disabled:opacity-50"
                                    >
                                        <Repeat className={cn('w-3.5 h-3.5 text-sky-500', isRerunning && 'animate-spin')} />
                                        <span>{isRerunning ? 'Rerunning…' : `Rerun ×${rerunCount} Now`}</span>
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                    {/* END RERUN SECTION (Task-04) */}
                </div>
            </div>

            {/* FIFO Enqueued Prompts Status Banner */}
            {enqueuedCount > 0 && (
                <div className="flex items-center gap-2 px-2.5 py-1 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-200 text-xs font-medium mt-1.5">
                    <Clock className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                    <span>{enqueuedCount} prompt(s) enqueued in FIFO queue for this conversation.</span>
                </div>
            )}

            {/* Suffix Template Management Modal */}
            <SuffixTemplateModal
                isOpen={isSuffixModalOpen}
                onClose={() => setIsSuffixModalOpen(false)}
                onSelectTemplate={(tid) => setConfirmationSuffix(tid)}
                activeTemplateId={confirmationSuffix}
            />
        </div>
    );
}
