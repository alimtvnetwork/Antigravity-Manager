import { copyToClipboard } from '../../../utils/clipboard';
import { formatPromptWithAttachments } from '../../../utils/promptAttachment';
import { copyImageAsMarkdownText, writePureImageToClipboard, copyPromptWithRichImages, extractAssets, stripImagesFromPrompt, copyAllImagesToClipboard, copyPromptWithLocalFileLinks, copyPromptWithBase64Images } from './clipboard';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatRelativeRecency } from './formatters';
import { Clock, Copy, Check, Image as ImageIcon, ChevronDown, ExternalLink, RotateCw, ListPlus, FileText, Code, Maximize2, Repeat, Send, Shield } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { CopyFormat, type AgmConversationNode, type AgmPromptTurnNode } from './types';
import { copyPromptAiPayload, resolvePromptBody, dispatchRerunMessages, dispatchConversationId, loadPersistedRerunCount, loadPersistedRerunMessageOption, savePersistedRerunCount, savePersistedRerunMessageOption, extractUsedNudgesFromConversation, RERUN_RANDOM_MESSAGE_SET, RERUN_MIN_COUNT, RERUN_MAX_COUNT, type PromptAiCopyKind, type RerunMessageOption } from './usePromptTreePromptActions';
import { patchPromptTreeSwrCache, queuedPromptCount } from './usePromptTreeData';
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
            className="pointer-events-none absolute right-0 top-full z-30 mt-1 hidden max-w-full rounded-md border border-slate-200 bg-white px-2 py-1 text-left text-[10px] font-medium leading-snug text-slate-800 shadow-md group-hover/field:block dark:border-slate-800 dark:bg-slate-900 dark:text-slate-100"
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

export type DispatchActionMode = 'send' | 'enqueue' | 'rerun';

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
    isDispatchMenuOpen?: boolean;
    setIsDispatchMenuOpen?: (v: boolean | ((prev: boolean) => boolean)) => void;
    dispatchMode?: DispatchActionMode;
    setDispatchMode?: (mode: DispatchActionMode) => void;
    hasImages: boolean;
    instanceId?: string;
    instancePid?: number;
    instanceSeqNum: number;
    instanceExeName: string;
    instanceNameDisplay: string;
    formatDuration: (secs: number) => string;
    elapsedSeconds: number;
    onCopyAllWithRichImages?: () => void | Promise<void | boolean>;
    onCopyCleanText?: () => void;
    onCopyImageAsText?: () => void;
    onCopyImageForOneNoteOrIde?: () => void;
    onCopySingleImagePng?: () => void;
    onCopyWithImages: () => void;
    onSaveImages: () => void;
    onExport: (format: 'md' | 'json') => void;
    onFocusIde: () => void;
    onResend: () => void;
    onEnqueue: () => void;
    loadTree?: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>;
    /** Optional repo path used as the rerun dispatch target (falls back to ''). */
    rerunRepoPath?: string;
    rerunHistory?: string[];
    onRerunHistoryChange?: (history: string[]) => void;
    onOpenInspector: (conv: AgmConversationNode) => void;
    onRenameConversation?: (conversationId: string, newTitle: string, instId?: string) => Promise<void>;
    selectedTurnNumber?: number | null;
    setIsDetailsModalOpen?: (v: boolean) => void;
    setActionMsg?: (msg: string | null) => void;
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
        isEnqueueMenuOpen: _isEnqueueMenuOpen, setIsEnqueueMenuOpen: _setIsEnqueueMenuOpen,
        isExportMenuOpen, setIsExportMenuOpen,
        isDispatchMenuOpen: propIsDispatchMenuOpen, setIsDispatchMenuOpen: propSetIsDispatchMenuOpen,
        dispatchMode: propDispatchMode, setDispatchMode: propSetDispatchMode,
        hasImages, instanceId: propInstanceId, instancePid,
        formatDuration, elapsedSeconds,
        onCopyAllWithRichImages, onCopyImageAsText,
        onCopyImageForOneNoteOrIde, onCopySingleImagePng,
        onSaveImages, onExport, onFocusIde, onResend, onEnqueue, rerunRepoPath,
        loadTree: _loadTree,
        rerunHistory: propRerunHistory, onRerunHistoryChange,
        onOpenInspector,
        onRenameConversation,
        selectedTurnNumber: _selectedTurnNumber,
        setActionMsg,
    } = props;

    const [primaryDispatchAction, setPrimaryDispatchAction] = useState<'send' | 'enqueue' | 'rerun'>(propDispatchMode || 'send');
    const [isDispatchMenuOpen, setIsDispatchMenuOpen] = useState(false);

    const [localNewConvTitle, setLocalNewConvTitle] = useState('');
    const newConversationTitle = propNewConversationTitle ?? localNewConvTitle;
    const setNewConversationTitle = propSetNewConversationTitle ?? setLocalNewConvTitle;

    const isMenuOpen = isDispatchMenuOpen || Boolean(propIsDispatchMenuOpen || propIsSendMenuOpen);

    const handleToggleDispatchMenu = () => {
        setIsDispatchMenuOpen((prev) => {
            const next = !prev;
            propSetIsDispatchMenuOpen?.(next);
            propSetIsSendMenuOpen?.(next);
            return next;
        });
    };

    const handleCloseDispatchMenu = () => {
        setIsDispatchMenuOpen(false);
        propSetIsDispatchMenuOpen?.(false);
        propSetIsSendMenuOpen?.(false);
    };

    const handleSwitchDispatchMode = (mode: 'send' | 'enqueue' | 'rerun') => {
        setPrimaryDispatchAction(mode);
        propSetDispatchMode?.(mode);
        if (mode === 'rerun') {
            setRerunMessageOption((prev) => prev || 'randomized');
        }
    };

    const [localCopiedText, setLocalCopiedText] = useState(false);
    const isCopiedTextActive = isCopiedText || localCopiedText;
    const [localCopiedAll, setLocalCopiedAll] = useState(false);
    const isCopiedAllActive = isCopiedAll || localCopiedAll;

    const [optimisticQueuedTurns, setOptimisticQueuedTurns] = useState<AgmPromptTurnNode[]>([]);
    useEffect(() => {
        setOptimisticQueuedTurns([]);
    }, [selectedConversation.conversation_id]);

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


    // Double-click inline title editing state
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [editedTitle, setEditedTitle] = useState(selectedConversation.title || selectedConversation.short_id || '');
    const [titleSuccessFeedback, setTitleSuccessFeedback] = useState<string | null>(null);

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
            const targetInst = selectedConversation.instance_id || 'default';
            if (onRenameConversation) {
                await onRenameConversation(selectedConversation.conversation_id, trimmed, targetInst);
            } else {
                await invoke('rename_conversation', {
                    instanceId: targetInst,
                    conversationId: selectedConversation.conversation_id,
                    newTitle: trimmed,
                });
                selectedConversation.title = trimmed;
            }
            setTitleSuccessFeedback('Conversation title updated & synced with IDE!');
            setActionMsg?.('Conversation title updated & synced with IDE!');
            setTimeout(() => setTitleSuccessFeedback(null), 3000);
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
    }, [isMenuOpen, confirmationSuffix]);

    // RERUN SECTION (Task-04): state + dispatch handler for the rerun capsule.
    // Option + count initialize from the persisted choice (default: randomized
    // nudge) so the auto-picked selection survives remounts instead of
    // resetting on every mount.
    const [rerunCount, setRerunCount] = useState<number>(() => loadPersistedRerunCount());
    const [rerunMessageOption, setRerunMessageOption] = useState<RerunMessageOption>(() => loadPersistedRerunMessageOption());
    const [showVariations, setShowVariations] = useState(false);
    const [localRerunHistory, setLocalRerunHistory] = useState<string[]>([]);
    const rerunHistory = propRerunHistory ?? localRerunHistory;
    const setRerunHistory = onRerunHistoryChange ?? setLocalRerunHistory;

    // Persist rerun picker choices; the conversation context menu reads the
    // same persisted values for its rerun action.
    useEffect(() => {
        savePersistedRerunMessageOption(rerunMessageOption);
        savePersistedRerunCount(rerunCount);
    }, [rerunMessageOption, rerunCount]);

    const [isRerunning, setIsRerunning] = useState(false);
    const [rerunFlash, setRerunFlash] = useState<string | null>(null);

    // RERUN WITH IMAGE (spec 154 §2.1): attached image staged in the flyout.
    const [attachedImage, setAttachedImage] = useState<{
        path: string;
        dataUri: string;
        fileName: string;
        byteSize: number;
    } | null>(null);
    const [isAttachingImage, setIsAttachingImage] = useState(false);

    // Drop a stale attachment and clear rerun history when the user switches conversations.
    useEffect(() => {
        setAttachedImage(null);
        setLocalRerunHistory([]);
        onRerunHistoryChange?.([]);
    }, [selectedConversation.conversation_id, onRerunHistoryChange]);

    const handleAttachRerunImage = async (): Promise<void> => {
        if (isAttachingImage) return;
        setIsAttachingImage(true);
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({
                multiple: false,
                filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg'] }],
            });
            // Cancelled: flyout stays open, no error, no state change.
            if (!selected) return;
            const filePath = typeof selected === 'string' ? selected : selected[0];
            if (!filePath) return;
            // read_image_as_base64 enforces the 100MB cap backend-side
            // ("file_too_large"); over-cap rejects here -> error-store toast,
            // flyout stays open (spec §2.1).
            const dataUri = await invoke<string>('read_image_as_base64', { path: filePath });
            if (!dataUri || !dataUri.startsWith('data:image/')) {
                throw new Error('read_image_as_base64 returned an unexpected payload');
            }
            const commaIdx = dataUri.indexOf(',');
            const b64 = commaIdx >= 0 ? dataUri.slice(commaIdx + 1) : '';
            const byteSize = Math.floor((b64.length * 3) / 4);
            const fileName = filePath.split(/[/\\]/).pop() || 'image';
            setAttachedImage({ path: filePath, dataUri, fileName, byteSize });
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'DetailHeader.rerunAttachImage',
                triggerAction: 'read_image_as_base64',
            });
        } finally {
            setIsAttachingImage(false);
        }
    };

    const handleRerun = async (): Promise<void> => {
        if (isRerunning) return;
        handleCloseDispatchMenu();
        setIsRerunning(true);
        setRerunFlash(null);
        try {
            const targetInstId = selectedConversation.instance_id?.trim() || propInstanceId?.trim() || 'default';
            const targetConvId = dispatchConversationId(
                conversationTargetMode,
                selectedConversation.conversation_id,
                Boolean(selectedConversation.is_running)
            ) || (conversationTargetMode === 'current' ? 'current' : undefined);

            // Resolve originalPrompt from active turns / prompt history rather than leaving it undefined
            let originalPrompt = activePromptText?.trim() || '';
            if (!originalPrompt && selectedConversation) {
                if (selectedConversation.selected_turn_number != null && selectedConversation.prompts && selectedConversation.prompts.length > 0) {
                    const matchedTurn = selectedConversation.prompts.find((p) => p.turn_number === selectedConversation.selected_turn_number);
                    if (matchedTurn?.prompt_text?.trim()) {
                        originalPrompt = matchedTurn.prompt_text.trim();
                    }
                }
                if (!originalPrompt && selectedConversation.prompts && selectedConversation.prompts.length > 0) {
                    const runningTurn = selectedConversation.prompts.find((p) => Boolean(p.is_running) && Boolean(p.prompt_text?.trim()));
                    if (runningTurn?.prompt_text?.trim()) {
                        originalPrompt = runningTurn.prompt_text.trim();
                    } else {
                        const lastTurnWithText = [...selectedConversation.prompts].reverse().find((p) => Boolean(p.prompt_text?.trim()));
                        if (lastTurnWithText?.prompt_text?.trim()) {
                            originalPrompt = lastTurnWithText.prompt_text.trim();
                        }
                    }
                }
                if (!originalPrompt) {
                    originalPrompt = (selectedConversation.full_prompt_text || selectedConversation.prompt_preview_200w || '').trim();
                }
            }

            // 3-tier fallback for repoPath: rerunRepoPath -> conversation repo_path -> default workspace
            const effectiveRepoPath = rerunRepoPath || (selectedConversation as { repo_path?: string })?.repo_path || '.';

            // Consolidate conversation media paths and staged rerun attachment into canonical format
            const existingMedia: string[] = selectedConversation?.media_paths || [];
            const allMedia: string[] = Array.from(new Set([
                ...existingMedia,
                ...(attachedImage ? [attachedImage.dataUri, attachedImage.path] : []),
            ])).filter((p): p is string => Boolean(p && typeof p === 'string' && p.trim().length > 0));

            const promptWithImage = formatPromptWithAttachments(
                originalPrompt,
                allMedia,
                { repoPath: effectiveRepoPath }
            );

            const effectiveOption: RerunMessageOption = rerunMessageOption || 'randomized';
            const combinedHistory = extractUsedNudgesFromConversation(selectedConversation, rerunHistory);
            const result = await dispatchRerunMessages({
                targetInstId,
                repoPath: effectiveRepoPath,
                convId: targetConvId,
                count: rerunCount,
                messageOption: effectiveOption,
                originalPrompt: promptWithImage,
                source: 'DetailHeader.rerunSection',
                mediaPaths: allMedia,
                history: combinedHistory,
            });
            if (result.failed > 0) {
                // Each failed rerun already landed in the error module via
                // dispatchRerunMessages; surface a visible summary as well.
                reportPromptActionError(
                    new Error(`Rerun: ${result.failed} of ${result.requested} dispatch(es) failed`),
                    'DetailHeader.rerunSection'
                );
            } else if (result.dispatched > 0) {
                const previewMsg = result.messages?.[0] ? ` ("${result.messages[0]}")` : '';
                setRerunFlash(`Enqueued ×${result.dispatched}`);
                setActionMsg?.(`Rerun ×${result.dispatched} enqueued in prompt queue!${previewMsg}`);
                setTimeout(() => setRerunFlash(null), 2500);
                setAttachedImage(null);
                if (result.messages && result.messages.length > 0) {
                    const nextHistory = [...rerunHistory, ...result.messages].slice(-10);
                    setRerunHistory(nextHistory);
                }

                if (selectedConversation && result.messages && result.messages.length > 0) {
                    const existingTurns = [
                        ...(selectedConversation.prompts || []),
                        ...optimisticQueuedTurns,
                    ];
                    const baseTurnNumber = existingTurns.length > 0
                        ? Math.max(...existingTurns.map((t) => t.turn_number)) + 1
                        : 1;
                    const newTurns: AgmPromptTurnNode[] = result.messages.map((msg, idx) => ({
                        turn_number: baseTurnNumber + idx,
                        step_index: baseTurnNumber + idx - 1,
                        prompt_text: msg,
                        preview_200w: msg.slice(0, 200),
                        status: 'QUEUED',
                        is_running: false,
                        is_queued: true,
                        timestamp: new Date().toISOString(),
                        created_at: Date.now() + idx,
                        media_paths: allMedia,
                        media_thumbnails: allMedia,
                    }));
                    const updatedTurns = [...existingTurns, ...newTurns];
                    setOptimisticQueuedTurns((prev) => [...prev, ...newTurns]);
                    patchPromptTreeSwrCache(targetInstId, (tree) =>
                        tree.map((projNode) => ({
                            ...projNode,
                            conversations: projNode.conversations.map((c) =>
                                c.conversation_id === selectedConversation.conversation_id
                                    ? { ...c, is_queued: true, prompts: updatedTurns }
                                    : c
                            ),
                        }))
                    );
                }
            }
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.rerunSection');
        } finally {
            setIsRerunning(false);
        }
    };
    // END RERUN SECTION (Task-04)

    const markCopiedAll = () => {
        setLocalCopiedAll(true);
        setIsCopiedAll?.(true);
        setTimeout(() => {
            setLocalCopiedAll(false);
            setIsCopiedAll?.(false);
        }, 2000);
    };

    const handleCopyAll = async () => {
        try {
            const source = resolvePromptBody(selectedConversation, activePromptText);
            if (!source) {
                setActionMsg?.('No prompt content to copy');
                setTimeout(() => setActionMsg?.(null), 2000);
                return;
            }
            const ok = await copyToClipboard(source);
            if (ok) {
                markCopiedAll();
                return;
            }
            reportPromptActionError(new Error('Failed to copy prompt to clipboard'), 'DetailHeader.copyAll');
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.copyAll');
        }
    };

    const handleCopyRich = async () => {
        try {
            if (onCopyAllWithRichImages) {
                const result = await Promise.resolve(onCopyAllWithRichImages());
                if (result !== false) {
                    markCopiedAll();
                }
                return;
            }
            const source = resolvePromptBody(selectedConversation, activePromptText);
            if (!source) {
                setActionMsg?.('No prompt content to copy');
                setTimeout(() => setActionMsg?.(null), 2000);
                return;
            }
            const ok = await copyPromptWithRichImages(source);
            if (ok) {
                markCopiedAll();
                return;
            }
            reportPromptActionError(new Error('Failed to copy prompt to clipboard'), 'DetailHeader.copyRich');
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.copyRich');
        }
    };

    const handleCopyCleanText = async () => {
        try {
            const source = resolvePromptBody(selectedConversation, activePromptText);
            const cleanText = stripImagesFromPrompt(source);
            const ok = await copyToClipboard(cleanText);
            if (ok) {
                setLocalCopiedText(true);
                setIsCopiedText(true);
                setTimeout(() => {
                    setLocalCopiedText(false);
                    setIsCopiedText(false);
                }, 2000);
                return;
            }
            reportPromptActionError(new Error('Failed to copy prompt text'), 'DetailHeader.copyCleanText');
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.copyCleanText');
        }
    };

    const [localCopiedImageText, setLocalCopiedImageText] = useState(false);
    const [localCopiedOneNote, setLocalCopiedOneNote] = useState(false);
    const [localCopiedSinglePng, setLocalCopiedSinglePng] = useState(false);
    const [localCopiedAllImages, setLocalCopiedAllImages] = useState(false);
    const [localCopiedLocalLinks, setLocalCopiedLocalLinks] = useState(false);
    const [localCopiedBase64, setLocalCopiedBase64] = useState(false);

    const handleCopyAllImages = async () => {
        try {
            const promptText = activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
            const { images } = extractAssets(promptText);
            const allImages = Array.from(new Set([...images, ...(selectedConversation?.media_paths || [])]));
            if (allImages.length > 0) {
                await copyAllImagesToClipboard(allImages);
                setLocalCopiedAllImages(true);
                setTimeout(() => setLocalCopiedAllImages(false), 2000);
            }
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopyAllImages');
        }
    };

    const handleCopyPromptLocalLinks = async () => {
        try {
            const promptText = activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
            await copyPromptWithLocalFileLinks(promptText, selectedConversation?.media_paths);
            setLocalCopiedLocalLinks(true);
            setTimeout(() => setLocalCopiedLocalLinks(false), 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopyPromptLocalLinks');
        }
    };

    const handleCopyPromptBase64 = async () => {
        try {
            const promptText = activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
            await copyPromptWithBase64Images(promptText, selectedConversation?.media_paths);
            setLocalCopiedBase64(true);
            setTimeout(() => setLocalCopiedBase64(false), 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopyPromptBase64');
        }
    };

    const handleCopyImageAsText = async () => {
        try {
            if (onCopyImageAsText) {
                onCopyImageAsText();
            } else {
                const promptText = activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
                await copyImageAsMarkdownText(promptText);
            }
            setLocalCopiedImageText(true);
            setTimeout(() => {
                setLocalCopiedImageText(false);
            }, 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopyImageAsText');
        }
    };

    const handleCopyImageForOneNote = async () => {
        try {
            if (onCopyImageForOneNoteOrIde) {
                onCopyImageForOneNoteOrIde();
            } else {
                const promptText = activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
                const { images } = extractAssets(promptText);
                const allImages = [...images, ...(selectedConversation?.media_paths || [])];
                if (allImages.length > 0) {
                    await writePureImageToClipboard(allImages[0]);
                } else {
                    await copyPromptWithRichImages(promptText, undefined, undefined, selectedConversation?.media_paths);
                }
            }
            setLocalCopiedOneNote(true);
            setTimeout(() => {
                setLocalCopiedOneNote(false);
            }, 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopyImageForOneNote');
        }
    };

    const handleCopySingleImagePng = async () => {
        try {
            if (onCopySingleImagePng) {
                onCopySingleImagePng();
            } else {
                const promptText = activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
                const { images } = extractAssets(promptText);
                const allImages = [...images, ...(selectedConversation?.media_paths || [])];
                if (allImages.length > 0) {
                    await writePureImageToClipboard(allImages[0]);
                }
            }
            setLocalCopiedSinglePng(true);
            setTimeout(() => {
                setLocalCopiedSinglePng(false);
            }, 2000);
        } catch (err) {
            reportPromptActionError(err, 'DetailHeader.onCopySingleImagePng');
        }
    };

    const handleCopyByFormat = (format: CopyFormat) => {
        switch (format) {
            case CopyFormat.RichMedia:
                void handleCopyRich();
                break;
            case CopyFormat.CleanText:
                void handleCopyCleanText();
                break;
            case CopyFormat.ImageAsText:
                void handleCopyImageAsText();
                break;
            case CopyFormat.Json:
                onExport('json');
                break;
        }
    };

    const visiblePrompts = [
        ...(selectedConversation.prompts || []),
        ...optimisticQueuedTurns.filter((extra) =>
            !(selectedConversation.prompts || []).some((p) =>
                p.turn_number === extra.turn_number && p.prompt_text === extra.prompt_text
            )
        ),
    ];
    const enqueuedCount = queuedPromptCount({
        ...selectedConversation,
        is_queued: Boolean(selectedConversation.is_queued) || optimisticQueuedTurns.length > 0,
        prompts: visiblePrompts,
    });

    return (
        <div className="pb-2.5 border-b border-slate-200 dark:border-slate-800 shrink-0 relative z-30">
            {/* Header Row: Identity & Badges & Breadcrumb Context (Left) + Action Capsule (Right) */}
            <div className="flex items-center justify-between gap-2.5 flex-wrap min-w-0">
                {/* Left: Title + Copy Title, followed by Status Capsule (nothing prefixes title) */}
                <div className="flex items-center gap-2.5 flex-wrap min-w-0 flex-1">
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
                            className="px-2.5 py-1 text-xl md:text-2xl font-bold font-ubuntu rounded-md bg-white dark:bg-slate-900 border border-purple-500/50 text-slate-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-purple-500 max-w-md md:max-w-xl lg:max-w-3xl shadow-inner"
                        />
                    ) : (
                        <div className="flex items-center gap-2 min-w-0">
                            <h3
                                onDoubleClick={(e) => {
                                    e.stopPropagation();
                                    setIsEditingTitle(true);
                                }}
                                className="text-xl md:text-2xl font-bold font-ubuntu text-slate-900 dark:text-white truncate max-w-md md:max-w-xl lg:max-w-3xl cursor-pointer hover:text-purple-600 dark:hover:text-purple-300 transition-colors tracking-tight"
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
                                    <Check className="w-4 h-4 text-emerald-500 animate-in zoom-in-75 duration-150" />
                                ) : (
                                    <Copy className="w-4 h-4" />
                                )}
                            </button>
                            {titleSuccessFeedback && (
                                <span className="text-[11px] font-medium text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20 animate-in fade-in duration-150 shrink-0">
                                    {titleSuccessFeedback}
                                </span>
                            )}
                        </div>
                    )}

                    {/* Indicator 2: Unified Status Capsule (appears AFTER title) */}
                    {Boolean(selectedConversation.is_running) && !isGhostConversation(selectedConversation) ? (
                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[9.5px] font-bold font-mono bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 shadow-2xs animate-pulse shrink-0">
                            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                            <span>RUNNING</span>
                            {instancePid ? <span className="opacity-80">PID: {instancePid}</span> : null}
                            <span className="border-l border-emerald-400/40 pl-1">{formatDuration(elapsedSeconds)}</span>
                        </span>
                    ) : (Boolean(selectedConversation.is_queued) || enqueuedCount > 0) ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold font-mono bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs shrink-0">
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>QUEUED</span>
                            {enqueuedCount > 0 ? <span className="opacity-90">{enqueuedCount}</span> : null}
                        </span>
                    ) : (
                        (() => {
                            const recency = formatRelativeRecency(selectedConversation.last_modified);
                            return (
                                <span
                                    className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[9px] font-mono text-zinc-400 bg-zinc-900/60 border border-zinc-700/50 shadow-2xs cursor-default shrink-0"
                                    title={`Last active: ${recency.formattedTime}`}
                                >
                                    <span className={cn("w-1.5 h-1.5 rounded-full shrink-0", recency.dotClass)} />
                                    <span>IDLE</span>
                                    <span className="text-zinc-500 border-l border-zinc-700/60 pl-1">{recency.label}</span>
                                </span>
                            );
                        })()
                    )}
                </div>

                {/* Right: The Unified Segmented Dark-Glass Action Capsule */}
                <div className="flex items-center shrink-0 ml-auto gap-2">
                    {/* Capsule A: Copy Split Button [ Copy | v ] */}
                    <div className="relative flex items-center rounded-lg bg-slate-100/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
                        {/* Primary Button: Copy full prompt as plain text */}
                        <button
                            type="button"
                            onClick={() => { void handleCopyAll(); }}
                            data-ui-name="prompt-copy-all-btn"
                            data-ui-id={`prompt-copy-all-${selectedConversation.conversation_id}`}
                            className="flex items-center gap-1.5 px-3 py-1 rounded-l-md text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-slate-800/80 text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            aria-label="Copy full prompt text"
                        >
                            {isCopiedAllActive ? (
                                <Check className="w-3 h-3 text-emerald-500 shrink-0" />
                            ) : (
                                <Copy className="w-3 h-3 text-purple-500 shrink-0" />
                            )}
                            <span>{isCopiedAllActive ? 'Copied!' : 'Copy'}</span>
                        </button>

                        {/* Dropdown Chevron for Clean Text, Image Saving, and Exports */}
                        <button
                            type="button"
                            onClick={() => setIsCopyMenuOpen((prev) => !prev)}
                            data-ui-name="prompt-copy-options-btn"
                            data-ui-id={`prompt-copy-options-${selectedConversation.conversation_id}`}
                            className="flex items-center justify-center px-1.5 py-1 rounded-r-md text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-slate-800/80 text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            aria-label="More copy and export options"
                        >
                            <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isCopyMenuOpen && "rotate-180")} />
                        </button>

                        {/* Flyout 1: Copy Options Menu */}
                        {isCopyMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-80 rounded-xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-xl p-1.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1"
                                onMouseLeave={() => setIsCopyMenuOpen(false)}
                            >
                                {/* Option 1: Copy All / Rich Media */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        handleCopyByFormat(CopyFormat.RichMedia);
                                        setIsCopyMenuOpen(false);
                                    }}
                                    data-ui-name="prompt-menu-copy-full-text-btn"
                                    data-ui-id={`prompt-menu-copy-full-text-${selectedConversation.conversation_id}`}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {isCopiedAllActive ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-purple-500" />}
                                        <span>{isCopiedAllActive ? 'Copied!' : 'Copy'}</span>
                                    </div>
                                    <span className="text-[10px] text-purple-600 dark:text-purple-300 font-mono px-1.5 py-0.2 rounded-md bg-purple-500/10 border border-purple-500/20">Rich Format</span>
                                </button>

                                {/* Option 1b: Copy Prompt (Local File Links) */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        void handleCopyPromptLocalLinks();
                                        setIsCopyMenuOpen(false);
                                    }}
                                    data-ui-name="prompt-menu-copy-local-links-btn"
                                    data-ui-id={`prompt-menu-copy-local-links-${selectedConversation.conversation_id}`}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {localCopiedLocalLinks ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-sky-400" />}
                                        <span>{localCopiedLocalLinks ? 'Copied with local links!' : 'Copy Prompt (Local File Links)'}</span>
                                    </div>
                                    <span className="text-[10px] text-sky-600 dark:text-sky-300 font-mono px-1.5 py-0.2 rounded-md bg-sky-500/10 border border-sky-500/20">file:///</span>
                                </button>

                                {/* Option 1c: Copy Prompt (Base64) */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        void handleCopyPromptBase64();
                                        setIsCopyMenuOpen(false);
                                    }}
                                    data-ui-name="prompt-menu-copy-base64-btn"
                                    data-ui-id={`prompt-menu-copy-base64-${selectedConversation.conversation_id}`}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {localCopiedBase64 ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-violet-400" />}
                                        <span>{localCopiedBase64 ? 'Copied with Base64!' : 'Copy Prompt (Base64)'}</span>
                                    </div>
                                    <span className="text-[10px] text-violet-600 dark:text-violet-300 font-mono px-1.5 py-0.2 rounded-md bg-violet-500/10 border border-violet-500/20">data:uri</span>
                                </button>

                                {/* Option 2: Copy Text Only */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        handleCopyByFormat(CopyFormat.CleanText);
                                        setIsCopyMenuOpen(false);
                                    }}
                                    data-ui-name="prompt-menu-copy-clean-text-btn"
                                    data-ui-id={`prompt-menu-copy-clean-${selectedConversation.conversation_id}`}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        {isCopiedTextActive ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-slate-400" />}
                                        <span>{isCopiedTextActive ? 'Copied text!' : 'Copy Text Only'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-500 dark:text-slate-400 font-mono px-1.5 py-0.2 rounded-md bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700">text only</span>
                                </button>

                                {/* Option 2b: Copy All Images (composite PNG) */}
                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            void handleCopyAllImages();
                                            setIsCopyMenuOpen(false);
                                        }}
                                        data-ui-name="prompt-menu-copy-all-images-btn"
                                        data-ui-id={`prompt-menu-copy-all-images-${selectedConversation.conversation_id}`}
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            {localCopiedAllImages ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-purple-400" />}
                                            <span>{localCopiedAllImages ? 'Copied all images!' : 'Copy All Images'}</span>
                                        </div>
                                        <span className="text-[10px] text-purple-600 dark:text-purple-300 font-mono px-1.5 py-0.2 rounded-md bg-purple-500/10 border border-purple-500/20">composite PNG</span>
                                    </button>
                                )}

                                {/* Option 3: Copy Image as Text (Markdown) */}
                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            handleCopyByFormat(CopyFormat.ImageAsText);
                                            setIsCopyMenuOpen(false);
                                        }}
                                        data-ui-name="prompt-menu-copy-image-text-btn"
                                        data-ui-id={`prompt-menu-copy-image-text-${selectedConversation.conversation_id}`}
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            {localCopiedImageText ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-sky-400" />}
                                            <span>{localCopiedImageText ? 'Copied image as text!' : 'Copy Image as Text (Markdown)'}</span>
                                        </div>
                                        <span className="text-[10px] text-sky-600 dark:text-sky-300 font-mono px-1.5 py-0.2 rounded-md bg-sky-500/10 border border-sky-500/20">Markdown</span>
                                    </button>
                                )}

                                {/* Option 4: Copy Image for OneNote/IDE */}
                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            void handleCopyImageForOneNote();
                                            setIsCopyMenuOpen(false);
                                        }}
                                        data-ui-name="prompt-menu-copy-onenote-btn"
                                        data-ui-id={`prompt-menu-copy-onenote-${selectedConversation.conversation_id}`}
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            {localCopiedOneNote ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-indigo-400" />}
                                            <span>{localCopiedOneNote ? 'Copied for OneNote/IDE!' : 'Copy Image for OneNote/IDE'}</span>
                                        </div>
                                        <span className="text-[10px] text-indigo-600 dark:text-indigo-300 font-mono px-1.5 py-0.2 rounded-md bg-indigo-500/10 border border-indigo-500/20">Rich/IDE</span>
                                    </button>
                                )}

                                {/* Option 5: Copy Single Image (PNG) */}
                                {hasImages && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            void handleCopySingleImagePng();
                                            setIsCopyMenuOpen(false);
                                        }}
                                        data-ui-name="prompt-menu-copy-single-png-btn"
                                        data-ui-id={`prompt-menu-copy-single-png-${selectedConversation.conversation_id}`}
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            {localCopiedSinglePng ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-purple-400" />}
                                            <span>{localCopiedSinglePng ? 'Copied Single Image!' : 'Copy Single Image (PNG)'}</span>
                                        </div>
                                        <span className="text-[10px] text-purple-600 dark:text-purple-300 font-mono px-1.5 py-0.2 rounded-md bg-purple-500/10 border border-purple-500/20">PNG</span>
                                    </button>
                                )}

                                {/* Sub-item 4: Save Images to Disk */}
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
                                        className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                    >
                                        <div className="flex items-center gap-2">
                                            <ImageIcon className="w-3.5 h-3.5 text-blue-500" />
                                            <span>Save Images to Disk</span>
                                        </div>
                                        <span className="text-[10px] text-slate-400 font-mono">Assets</span>
                                    </button>
                                )}

                                <div className="border-t border-slate-100 dark:border-slate-800 my-1" />

                                <button
                                    type="button"
                                    onClick={() => copyPromptAndAi('json')}
                                    className="group/field relative w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
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
                                    className="group/field relative w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
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
                                    className="group/field relative w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                    aria-describedby="copy-ai-md-hint"
                                >
                                    <DispatchFieldHint id="copy-ai-md-hint" text="Copy the prompt body and AI text as markdown" />
                                    <div className="flex items-center gap-2">
                                        {copiedAiKind === 'md' ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <FileText className="w-3.5 h-3.5 text-cyan-500" />}
                                        <span>{copiedAiKind === 'md' ? 'Copied markdown!' : 'Copy prompt and AI results as markdown'}</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">.md</span>
                                </button>

                                <div className="border-t border-slate-100 dark:border-slate-800 my-1" />

                                {/* Sub-item 4: Export as Markdown */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        onExport('md');
                                        setIsCopyMenuOpen(false);
                                    }}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
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
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <Code className="w-3.5 h-3.5 text-amber-500" />
                                        <span>Export as JSON</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">.json</span>
                                </button>
                            </div>
                        )}
                    </div>

                    {/* Capsule B: Combined [Send, Enqueue, Rerun] segmented pill capsule and dropdown */}
                    <div className="relative inline-flex items-center rounded-full bg-slate-100/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 shadow-2xs divide-x divide-slate-200/70 dark:divide-purple-500/20 shrink-0">
                        <button
                            type="button"
                            onClick={() => {
                                if (primaryDispatchAction === 'enqueue') {
                                    void Promise.resolve(onEnqueue()).catch((err) => {
                                        reportPromptActionError(err, 'DetailHeader.onEnqueue');
                                    });
                                    return;
                                }
                                if (primaryDispatchAction === 'rerun') {
                                    void handleRerun().catch((err) => {
                                        reportPromptActionError(err, 'DetailHeader.rerunSection');
                                    });
                                    return;
                                }
                                void Promise.resolve(onResend()).catch((err) => {
                                    reportPromptActionError(err, 'DetailHeader.onResend');
                                });
                            }}
                            disabled={isResending || isEnqueueing || isRerunning}
                            data-ui-name="prompt-dispatch-primary-btn"
                            data-ui-id={`prompt-dispatch-primary-${selectedConversation.conversation_id}`}
                            className={cn(
                                "flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold transition-all duration-150 ease-out active:translate-y-px cursor-pointer disabled:opacity-50 rounded-l-full",
                                primaryDispatchAction === 'enqueue'
                                    ? "text-amber-700 dark:text-amber-400 hover:bg-amber-500/10"
                                    : primaryDispatchAction === 'rerun'
                                      ? "text-sky-700 dark:text-sky-400 hover:bg-sky-500/10"
                                      : "text-emerald-700 dark:text-emerald-400 hover:bg-emerald-500/10"
                            )}
                            title={
                                primaryDispatchAction === 'enqueue'
                                    ? "Enqueue (FIFO): Stages prompt into FIFO queue (Hotkey: Q)"
                                    : primaryDispatchAction === 'rerun'
                                      ? `Rerun Turn (×${rerunCount}): Enqueues previous turn into FIFO queue (Hotkey: R)`
                                      : "Send (Preempt): Cancels active task and executes immediately (Hotkey: N)"
                            }
                            aria-label="Run selected dispatch mode"
                        >
                            {primaryDispatchAction === 'enqueue' ? (
                                <ListPlus className={cn('w-3 h-3 shrink-0', isEnqueueing && 'animate-pulse')} />
                            ) : primaryDispatchAction === 'rerun' ? (
                                <Repeat className={cn('w-3 h-3 shrink-0', isRerunning && 'animate-spin')} />
                            ) : (
                                <Send className={cn('w-3 h-3 shrink-0', isResending && 'animate-pulse')} />
                            )}
                            <span>
                                {primaryDispatchAction === 'enqueue'
                                    ? (isEnqueueing ? 'Queueing…' : 'Enqueue')
                                    : primaryDispatchAction === 'rerun'
                                      ? (isRerunning ? 'Rerunning…' : (rerunFlash ?? `Rerun ×${rerunCount}`))
                                      : (isResending ? 'Sending…' : 'Send')}
                            </span>
                            {primaryDispatchAction === 'enqueue' && (
                                <span className="min-w-4 h-4 px-1 rounded-full bg-amber-500/25 text-amber-800 dark:text-amber-200 text-[9px] font-bold font-mono leading-4 text-center">
                                    {enqueuedCount}
                                </span>
                            )}
                            <kbd className={cn(
                                "ml-0.5 px-1 py-0.2 border rounded text-[8.5px] font-mono font-bold",
                                primaryDispatchAction === 'enqueue'
                                    ? "bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30"
                                    : primaryDispatchAction === 'rerun'
                                      ? "bg-sky-500/15 text-sky-700 dark:text-sky-300 border-sky-500/30"
                                      : "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/30"
                            )}>
                                {primaryDispatchAction === 'enqueue' ? 'Q' : primaryDispatchAction === 'rerun' ? 'R' : 'N'}
                            </kbd>
                        </button>

                        {/* Dropdown Chevron for Unified Dispatch Menu (Send, Enqueue, Rerun, Target, Templates) */}
                        <button
                            type="button"
                            onClick={handleToggleDispatchMenu}
                            data-ui-name="prompt-dispatch-options-btn"
                            data-ui-id={`prompt-dispatch-options-${selectedConversation.conversation_id}`}
                            className="flex items-center justify-center px-2 py-1 rounded-r-full text-slate-700 dark:text-slate-300 hover:bg-slate-200/80 dark:hover:bg-slate-800/80 text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            aria-label="Unified dispatch configuration popover"
                            title="Dispatch options: Choose Send, Enqueue, or Rerun and configure target"
                        >
                            <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isMenuOpen && "rotate-180")} />
                        </button>

                        {/* Flyout 2: Unified Dispatch Configuration Popover */}
                        {isMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-[28rem] sm:w-[30rem] max-w-[95vw] rounded-xl bg-white/95 dark:bg-slate-900/95 border border-slate-200 dark:border-slate-800 shadow-2xl p-2.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-2"
                                onMouseLeave={handleCloseDispatchMenu}
                            >
                                {/* Menu Header */}
                                <div className="px-2 pt-0.5 pb-1 text-[10px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 select-none border-b border-slate-100 dark:border-slate-800 flex items-center justify-between">
                                    <span>Dispatch Configuration</span>
                                    <span className="text-[9px] text-slate-400">AGM:P001</span>
                                </div>

                                {/* Mode Switcher Segmented Pill: [ Send | Enqueue | Rerun ] */}
                                <div className="flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Mode:
                                    </span>
                                    <div className="inline-flex rounded-lg bg-slate-100 dark:bg-slate-950/80 p-0.5 border border-slate-200/80 dark:border-purple-500/20 text-[11px]">
                                        <button
                                             type="button"
                                             onClick={() => handleSwitchDispatchMode('send')}
                                             title="Send (Preempt): Cancels active task to execute immediately (Hotkey: N)"
                                             className={cn(
                                                 "flex items-center gap-1 px-2.5 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                 primaryDispatchAction === 'send'
                                                     ? "bg-white dark:bg-slate-800 text-emerald-600 dark:text-emerald-400 shadow-2xs font-semibold"
                                                     : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                             )}
                                         >
                                             <Send className="w-2.5 h-2.5" />
                                             <span>Send</span>
                                         </button>
                                         <button
                                             type="button"
                                             onClick={() => handleSwitchDispatchMode('enqueue')}
                                             title="Enqueue (FIFO): Queues prompt behind active task in IDE (Hotkey: Q)"
                                             className={cn(
                                                 "flex items-center gap-1 px-2.5 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                 primaryDispatchAction === 'enqueue'
                                                     ? "bg-white dark:bg-slate-800 text-amber-600 dark:text-amber-400 shadow-2xs font-semibold"
                                                     : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                             )}
                                         >
                                             <ListPlus className="w-2.5 h-2.5" />
                                             <span>Enqueue</span>
                                         </button>
                                         <button
                                             type="button"
                                             onClick={() => handleSwitchDispatchMode('rerun')}
                                             title="Rerun Turn: Enqueues turn N times into FIFO queue (Hotkey: R)"
                                             className={cn(
                                                 "flex items-center gap-1 px-2.5 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                 primaryDispatchAction === 'rerun'
                                                     ? "bg-white dark:bg-slate-800 text-sky-600 dark:text-sky-400 shadow-2xs font-semibold"
                                                     : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                             )}
                                         >
                                             <Repeat className="w-2.5 h-2.5" />
                                             <span>Rerun</span>
                                         </button>
                                     </div>
                                 </div>

                                {/* CONTEXTUAL OPTIONS: SEND or ENQUEUE MODE */}
                                {(primaryDispatchAction === 'send' || primaryDispatchAction === 'enqueue') && (
                                    <>
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
                                                <option value="current" className="bg-white dark:bg-slate-900">Current Conv</option>
                                                <option value="new" className="bg-white dark:bg-slate-900">New Conv</option>
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
                                                    className="w-full px-2 py-1 text-xs bg-white dark:bg-slate-900 border border-slate-200 dark:border-purple-500/30 rounded-md text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-purple-500/50"
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
                                                            className="flex items-center gap-1.5 px-2 py-0.5 rounded-lg bg-slate-100 dark:bg-purple-950/40 border border-slate-200/80 dark:border-purple-500/20 text-slate-800 dark:text-slate-200 text-xs font-semibold hover:bg-slate-200/80 dark:hover:bg-slate-800/80 transition-colors cursor-pointer max-w-[140px]"
                                                            title="Select confirmation suffix or verification template"
                                                        >
                                                            <span className="truncate">{currentSuffixLabel}</span>
                                                            <ChevronDown className={cn("w-3 h-3 text-slate-400 shrink-0 transition-transform duration-150", isSuffixPopoverOpen && "rotate-180")} />
                                                        </button>

                                                        {isSuffixPopoverOpen && (
                                                            <div
                                                                className="absolute right-0 top-full mt-1.5 z-50 w-64 rounded-xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-2xl p-1.5 space-y-1 backdrop-blur-md max-h-72 overflow-y-auto animate-in fade-in zoom-in-95 duration-100"
                                                                onMouseLeave={() => setIsSuffixPopoverOpen(false)}
                                                            >
                                                                <div className="px-2 py-0.5 text-[9.5px] font-mono uppercase tracking-wider text-slate-400 border-b border-slate-100 dark:border-slate-800">
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
                                                                                    : "text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800/80"
                                                                            )}
                                                                        >
                                                                            <span className="truncate">{opt.dropdownLabel || opt.label}</span>
                                                                            {isSelected && <Check className="w-3.5 h-3.5 text-purple-500 shrink-0" />}
                                                                        </button>
                                                                    );
                                                                })}

                                                                {customTemplates.length > 0 && (
                                                                    <>
                                                                        <div className="px-2 pt-1.5 pb-0.5 text-[9.5px] font-mono uppercase tracking-wider text-slate-400 border-b border-slate-100 dark:border-slate-800 mt-1">
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
                                                                                            : "text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800/80"
                                                                                    )}
                                                                                >
                                                                                    <span className="truncate">{ct.name}</span>
                                                                                    {isSelected && <Check className="w-3.5 h-3.5 text-purple-500 shrink-0" />}
                                                                                </button>
                                                                            );
                                                                        })}
                                                                    </>
                                                                )}

                                                                <div className="pt-1 border-t border-slate-100 dark:border-slate-800">
                                                                    <button
                                                                        type="button"
                                                                        onClick={() => {
                                                                            setIsSuffixPopoverOpen(false);
                                                                            setIsSuffixModalOpen(true);
                                                                        }}
                                                                        className="w-full flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-semibold text-purple-600 dark:text-purple-400 hover:bg-purple-50 dark:hover:bg-slate-800/80 transition-colors cursor-pointer"
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
                                    </>
                                )}

                                {/* CONTEXTUAL OPTIONS: RERUN MODE */}
                                {primaryDispatchAction === 'rerun' && (
                                    <>
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
                                                                ? "bg-white dark:bg-slate-800 text-sky-600 dark:text-sky-400 shadow-2xs font-semibold"
                                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                                        )}
                                                        title={`Dispatch the nudge ${n} time(s)`}
                                                    >
                                                        {n}
                                                    </button>
                                                ))}
                                            </div>
                                        </div>

                                        {/* Message selector: compact dropdown */}
                                        <div className="flex items-center justify-between px-2 py-1 text-xs gap-2">
                                            <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                                Message:
                                            </span>
                                            <select
                                                value={rerunMessageOption}
                                                onChange={(e) => setRerunMessageOption(e.target.value as RerunMessageOption)}
                                                className="flex-1 min-w-0 px-2.5 py-1.5 rounded-lg text-xs bg-slate-100 dark:bg-purple-950/40 text-slate-700 dark:text-slate-200 border border-slate-200/80 dark:border-purple-500/20 focus:outline-none focus:ring-1 focus:ring-sky-500/40 cursor-pointer truncate"
                                            >
                                                <optgroup label="Automated Variations" className="bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200">
                                                    <option value="randomized">🎲 Random from set (skips used)</option>
                                                </optgroup>
                                                <optgroup label="Strict Quality & E2E Verification" className="bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200">
                                                    <option value="tested-e2e">🛡️ Have you tested these below instructions with end-to-end testing?</option>
                                                    <option value="tested-e2e-and-released">🚀 Have you tested the below instructions with end-to-end testing and released it?</option>
                                                </optgroup>
                                                <optgroup label="Status Check Nudges" className="bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200">
                                                    <option value="is-it-completed">Is it completed?</option>
                                                    <option value="is-it-finished">Is it finished?</option>
                                                    <option value="is-it-released">Is it released?</option>
                                                    <option value="is-it-done">Is it done?</option>
                                                    <option value="is-it-published">Is it published?</option>
                                                    <option value="verify-and-finish">Verify and finish remaining items</option>
                                                    <option value="check-pending-tasks">Check pending tasks and continue</option>
                                                    <option value="run-verification">Run comprehensive verification</option>
                                                    <option value="confirm-pushed">Confirm committed and pushed</option>
                                                    <option value="fully-implemented-verified">Fully implemented and verified?</option>
                                                    <option value="all-tests-passed">Did all tests pass and is everything green?</option>
                                                    <option value="double-check-completion">Double check completion and finalize</option>
                                                    <option value="pending-work-left">Is there any pending work left?</option>
                                                    <option value="confirm-status-next-steps">Confirm completion status and next steps</option>
                                                    <option value="check-pending-proceed">Check pending tasks and proceed with next</option>
                                                    <option value="verify-build-cleanliness">Verify build cleanliness and finalize</option>
                                                </optgroup>
                                            </select>
                                        </div>

                                        {/* Quality Protocol Mode Banner when strict E2E templates are selected */}
                                        {(rerunMessageOption === 'tested-e2e' || rerunMessageOption === 'tested-e2e-and-released') && (
                                            <div className="mx-2 px-2.5 py-1.5 rounded-lg bg-sky-500/10 dark:bg-sky-950/30 border border-sky-500/25 flex items-center gap-2 text-[11px] text-sky-700 dark:text-sky-300">
                                                <Shield className="w-3.5 h-3.5 text-sky-500 shrink-0" />
                                                <span className="font-medium">
                                                    Strict Quality Protocol Mode: AI agent will be instructed to perform rigorous end-to-end testing verification.
                                                </span>
                                            </div>
                                        )}

                                        {/* Compact status pill for randomized mode */}
                                        {rerunMessageOption === 'randomized' && (() => {
                                            const usedNudges = extractUsedNudgesFromConversation(selectedConversation, rerunHistory);
                                            const unusedCount = Math.max(0, RERUN_RANDOM_MESSAGE_SET.length - usedNudges.length);
                                            return (
                                                <div className="mx-2 px-2 py-1 rounded-md bg-amber-500/10 border border-amber-500/25 flex items-center justify-between text-[10px] text-amber-700 dark:text-amber-300">
                                                    <span className="flex items-center gap-1.5 truncate">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-amber-500 shrink-0" />
                                                        <span>Auto-picks random unused ({unusedCount} of {RERUN_RANDOM_MESSAGE_SET.length} left)</span>
                                                    </span>
                                                    <button
                                                        type="button"
                                                        onClick={() => setShowVariations((prev) => !prev)}
                                                        className="text-[9.5px] font-mono underline hover:text-amber-900 dark:hover:text-amber-100 shrink-0 ml-1 cursor-pointer"
                                                    >
                                                        {showVariations ? 'Hide' : 'List'}
                                                    </button>
                                                </div>
                                            );
                                        })()}

                                        {/* Collapsible variations preview (only rendered if user clicks 'List') */}
                                        {rerunMessageOption === 'randomized' && showVariations && (
                                            <div className="mx-2 px-2 py-1.5 rounded-lg bg-slate-50/80 dark:bg-purple-950/20 border border-slate-200/60 dark:border-purple-500/20 max-h-28 overflow-y-auto">
                                                <div className="text-[9px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 mb-0.5 sticky top-0 bg-slate-50 dark:bg-[#1a0e2e]">
                                                    {RERUN_RANDOM_MESSAGE_SET.length} variations
                                                </div>
                                                <ul className="space-y-0.5 text-[10px] text-slate-500 dark:text-slate-400 list-disc list-inside">
                                                    {RERUN_RANDOM_MESSAGE_SET.map((m) => (
                                                        <li key={m}>{m}</li>
                                                    ))}
                                                </ul>
                                            </div>
                                        )}

                                        {/* Attach image staged before dispatch */}
                                        <div className="mx-2 px-2 py-1.5 rounded-lg bg-slate-50/80 dark:bg-purple-950/20 border border-slate-200/60 dark:border-purple-500/20">
                                            <div className="flex items-center justify-between">
                                                <span className="text-[11px] font-medium text-slate-500 dark:text-slate-400">
                                                    Image attachment:
                                                </span>
                                                {attachedImage ? (
                                                    <button
                                                        type="button"
                                                        onClick={() => setAttachedImage(null)}
                                                        className="text-[10px] font-semibold text-rose-600 dark:text-rose-400 hover:underline cursor-pointer"
                                                    >
                                                        Remove
                                                    </button>
                                                ) : (
                                                    <button
                                                        type="button"
                                                        onClick={() => void handleAttachRerunImage().catch((err) => {
                                                            reportPromptActionError(err, 'DetailHeader.rerunAttachImage');
                                                        })}
                                                        disabled={isAttachingImage}
                                                        className="flex items-center gap-1.5 px-2 py-1 rounded-lg text-[11px] font-semibold text-sky-700 dark:text-sky-300 bg-sky-500/10 hover:bg-sky-500/20 dark:hover:bg-slate-800/80 border border-sky-500/30 transition-colors cursor-pointer disabled:opacity-50"
                                                    >
                                                        <ImageIcon className={cn('w-3.5 h-3.5', isAttachingImage && 'animate-pulse')} />
                                                        <span>{isAttachingImage ? 'Reading…' : 'Attach image'}</span>
                                                    </button>
                                                )}
                                            </div>
                                            {attachedImage && (
                                                <div className="mt-1.5 flex items-center gap-2">
                                                    <img
                                                        src={attachedImage.dataUri}
                                                        alt={attachedImage.fileName}
                                                        className="w-10 h-10 rounded-lg object-cover border border-slate-200 dark:border-purple-500/30 shrink-0"
                                                    />
                                                    <div className="min-w-0">
                                                        <div className="text-[10.5px] font-medium text-slate-700 dark:text-slate-200 truncate">
                                                            {attachedImage.fileName}
                                                        </div>
                                                        <div className="text-[9.5px] font-mono text-slate-400">
                                                            {(attachedImage.byteSize / 1024).toFixed(1)} KB · re-injected on rerun
                                                        </div>
                                                    </div>
                                                </div>
                                            )}
                                        </div>

                                        {/* View Release Page & Notes button */}
                                        <div className="mx-2 pt-0.5">
                                            <button
                                                type="button"
                                                onClick={async () => {
                                                    const url = 'https://github.com/aukgit/antigravity-manager/releases';
                                                    try {
                                                        const { openUrl } = await import('@tauri-apps/plugin-opener');
                                                        await openUrl(url);
                                                    } catch {
                                                        window.open(url, '_blank', 'noopener,noreferrer');
                                                    }
                                                }}
                                                className="w-full flex items-center justify-center gap-1.5 px-2.5 py-1.5 text-xs text-purple-700 dark:text-purple-300 hover:bg-purple-100/50 dark:hover:bg-purple-950/40 rounded-lg border border-purple-500/20 font-medium transition-colors cursor-pointer"
                                                title="View official release notes, tags, and changelogs on GitHub"
                                            >
                                                <ExternalLink className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                                                <span>View Release Page &amp; Notes</span>
                                            </button>
                                        </div>
                                    </>
                                )}

                                <div className="border-t border-slate-100 dark:border-slate-800 my-1" />

                                {/* Focus IDE Window option */}
                                <button
                                    type="button"
                                    onClick={() => {
                                        onFocusIde();
                                        handleCloseDispatchMenu();
                                    }}
                                    disabled={isFocusing}
                                    className="w-full flex items-center justify-between px-2 py-1.5 text-xs text-sky-700 dark:text-sky-300 hover:bg-sky-50 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors disabled:opacity-50"
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
                                        handleCloseDispatchMenu();
                                    }}
                                    className="w-full flex items-center justify-between px-2 py-1.5 text-xs text-blue-700 dark:text-cyan-300 hover:bg-blue-50 dark:hover:bg-slate-800/80 rounded-lg text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <Maximize2 className="w-3.5 h-3.5 text-blue-500 dark:text-cyan-400 shrink-0" />
                                        <span className="font-medium">Full Inspector</span>
                                    </div>
                                    <span className="text-[10px] text-slate-400 font-mono">Full-Screen</span>
                                </button>

                                {/* Dispatch Action Button inside dropdown */}
                                <div className="pt-1 border-t border-slate-100 dark:border-slate-800">
                                    {primaryDispatchAction === 'enqueue' ? (
                                        <button
                                            type="button"
                                            onClick={() => {
                                                void Promise.resolve(onEnqueue()).catch((err) => {
                                                    reportPromptActionError(err, 'DetailHeader.onEnqueue');
                                                });
                                                handleCloseDispatchMenu();
                                            }}
                                            disabled={isEnqueueing}
                                            title="Enqueue (FIFO): Queues prompt behind active task in IDE"
                                            className="w-full flex items-center justify-center gap-2 py-1.5 px-3 bg-amber-500/15 hover:bg-amber-500/25 text-amber-700 dark:text-amber-300 border border-amber-500/30 rounded-lg text-xs font-semibold cursor-pointer transition-colors disabled:opacity-50"
                                        >
                                            <ListPlus className={cn('w-3.5 h-3.5 text-amber-500', isEnqueueing && 'animate-pulse')} />
                                            <span>Enqueue (FIFO)</span>
                                        </button>
                                    ) : primaryDispatchAction === 'rerun' ? (
                                        <button
                                            type="button"
                                            onClick={() => {
                                                void handleRerun().catch((err) => {
                                                    reportPromptActionError(err, 'DetailHeader.rerunSection');
                                                });
                                            }}
                                            disabled={isRerunning}
                                            aria-label={`Rerun ×${rerunCount} Now: Enqueues ${rerunMessageOption} completion or release nudge into FIFO queue`}
                                            className="w-full flex items-center justify-center gap-2 py-1.5 px-3 bg-sky-500/15 hover:bg-sky-500/25 text-sky-700 dark:text-sky-300 border border-sky-500/30 rounded-lg text-xs font-semibold cursor-pointer transition-colors disabled:opacity-50"
                                        >
                                            <Repeat className={cn('w-3.5 h-3.5 text-sky-500', isRerunning && 'animate-spin')} />
                                            <span>{isRerunning ? 'Rerunning…' : (rerunFlash ?? `Rerun ×${rerunCount} Now`)}</span>
                                        </button>
                                    ) : (
                                        <button
                                            type="button"
                                            onClick={() => {
                                                void Promise.resolve(onResend()).catch((err) => {
                                                    reportPromptActionError(err, 'DetailHeader.onResend');
                                                });
                                                handleCloseDispatchMenu();
                                            }}
                                            disabled={isResending}
                                            title="Cancel & Send / Send (Preempt): Cancels any running task to execute immediately"
                                            className="w-full flex items-center justify-center gap-2 py-1.5 px-3 bg-emerald-500/15 hover:bg-emerald-500/25 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30 rounded-lg text-xs font-semibold cursor-pointer transition-colors disabled:opacity-50"
                                        >
                                            <RotateCw className={cn('w-3.5 h-3.5 text-emerald-500', isResending && 'animate-spin')} />
                                            <span>Cancel &amp; Send</span>
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
