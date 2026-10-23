import { copyToClipboard } from '../../../utils/clipboard';
import { copyImageAsMarkdownText, writePureImageToClipboard, copyPromptWithRichImages, extractAssets, stripImagesFromPrompt, copyAllImagesToClipboard, copyPromptWithLocalFileLinks, copyPromptWithBase64Images } from './clipboard';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatRelativeRecency } from './formatters';
import { Clock, Copy, Check, Image as ImageIcon, ChevronDown, ExternalLink, RotateCw, ListPlus, FileText, Code, Maximize2, Repeat } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { CopyFormat, type AgmConversationNode } from './types';
import { copyPromptAiPayload, resolvePromptBody, dispatchRerunMessages, loadPersistedRerunCount, loadPersistedRerunMessageOption, savePersistedRerunCount, savePersistedRerunMessageOption, RERUN_MESSAGE_OPTIONS, RERUN_RANDOM_MESSAGE_SET, RERUN_MIN_COUNT, RERUN_MAX_COUNT, type PromptAiCopyKind, type RerunMessageOption } from './usePromptTreePromptActions';
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
    onCopyAllWithRichImages?: () => void;
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
    /** Optional repo path used as the rerun dispatch target (falls back to ''). */
    rerunRepoPath?: string;
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
        onCopyAllWithRichImages, onCopyCleanText, onCopyImageAsText,
        onCopyImageForOneNoteOrIde, onCopySingleImagePng,
        onCopyWithImages, onSaveImages, onExport, onFocusIde, onResend, onEnqueue, rerunRepoPath, onOpenInspector,
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

    // Drop a stale attachment when the user switches conversations.
    useEffect(() => {
        setAttachedImage(null);
    }, [selectedConversation.conversation_id]);

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
            const targetConvId = conversationTargetMode === 'new'
                ? (newConversationTitle?.trim() || undefined)
                : selectedConversation.conversation_id;

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

            // Preserve both existing conversation media paths and any newly attached rerun image
            let promptWithImage = originalPrompt;
            const existingMedia: string[] = selectedConversation?.media_paths || [];
            if (existingMedia.length > 0) {
                const missing = existingMedia.filter((p: string) => p && !promptWithImage.includes(p));
                if (missing.length > 0) {
                    promptWithImage += '\n\n' + missing.map((p: string, idx: number) => `![Turn Image ${idx + 1}](${p})`).join('\n\n');
                }
            }
            if (attachedImage && !promptWithImage.includes(attachedImage.path)) {
                promptWithImage = `${promptWithImage.trim()}\n\n![attached-image](${attachedImage.path})`;
            }

            const rerunMediaList: string[] = Array.from(new Set([
                ...existingMedia,
                ...(attachedImage ? [attachedImage.dataUri, attachedImage.path] : []),
            ])).filter((p): p is string => Boolean(p && typeof p === 'string' && p.trim().length > 0));

            const effectiveOption: RerunMessageOption = rerunMessageOption || 'randomized';
            const result = await dispatchRerunMessages({
                targetInstId,
                repoPath: effectiveRepoPath,
                convId: targetConvId,
                count: rerunCount,
                messageOption: effectiveOption,
                originalPrompt: promptWithImage,
                source: 'DetailHeader.rerunSection',
                mediaPaths: rerunMediaList,
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
                setRerunFlash(`Sent ×${result.dispatched}`);
                setActionMsg?.(`Rerun ×${result.dispatched} dispatched successfully!${previewMsg}`);
                setTimeout(() => setRerunFlash(null), 2500);
                setAttachedImage(null);
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
                handleCopyAll();
                break;
            case CopyFormat.CleanText:
                handleCopyCleanText();
                break;
            case CopyFormat.ImageAsText:
                void handleCopyImageAsText();
                break;
            case CopyFormat.Json:
                onExport('json');
                break;
        }
    };

    const enqueuedCount = selectedConversation.prompts?.filter((p) => p.is_queued || p.status === 'QUEUED').length || (selectedConversation.is_queued ? 1 : 0);

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
                    ) : Boolean(selectedConversation.is_queued) ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold font-mono bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs shrink-0">
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>QUEUED</span>
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
                        {/* Primary Button: Copy with images (Rich HTML + binary PNG) */}
                        <button
                            type="button"
                            onClick={handleCopyAll}
                            data-ui-name="prompt-copy-all-btn"
                            data-ui-id={`prompt-copy-all-${selectedConversation.conversation_id}`}
                            className="flex items-center gap-1.5 px-3 py-1 rounded-l-md text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-slate-800/80 text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            aria-label="Copy prompt text and all embedded images (rich HTML + binary PNG)"
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

                    {/* Capsule B: Unified Dispatch Capsule [ Send (Preempt) | Enqueue (FIFO) | v (Options) ] */}
                    <div className="relative flex items-center rounded-lg bg-slate-100/90 dark:bg-[var(--ui-canvas,#090611)]/90 backdrop-blur-md border border-slate-200/80 dark:border-[var(--ui-border-subtle,rgba(255,255,255,0.12))] p-0.5 divide-x divide-slate-200/70 dark:divide-[var(--ui-border-subtle,rgba(255,255,255,0.08))] shadow-2xs shrink-0">
                        {/* 1. Send (Preempt): Cancels any running task and executes immediately */}
                        <button
                            type="button"
                            onClick={() => {
                                void Promise.resolve(onResend()).catch((err) => {
                                    reportPromptActionError(err, 'DetailHeader.onResend');
                                });
                            }}
                            disabled={isResending}
                            data-ui-name="prompt-send-preempt-btn"
                            data-ui-id={`prompt-send-preempt-${selectedConversation.conversation_id}`}
                            className="flex items-center gap-1.5 px-3 py-1 rounded-l-md text-[11px] font-semibold text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-emerald-950/40 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                            title="Send (Preempt): Cancels any running task to execute immediately (Hotkey: N)"
                            aria-label="Send (Preempt): Cancels any running task to execute immediately (Hotkey: N)"
                        >
                            <RotateCw className={cn('w-3 h-3 text-emerald-500 shrink-0', isResending && 'animate-spin')} />
                            <span>{isResending ? 'Sending…' : 'Send'}</span>
                            <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>
                        </button>

                        {/* 2. Enqueue (FIFO): Stages prompt into background FIFO queue without cancelling active task */}
                        <button
                            type="button"
                            onClick={() => {
                                void Promise.resolve(onEnqueue()).catch((err) => {
                                    reportPromptActionError(err, 'DetailHeader.onEnqueue');
                                });
                            }}
                            disabled={isEnqueueing}
                            data-ui-name="prompt-enqueue-fifo-btn"
                            data-ui-id={`prompt-enqueue-fifo-${selectedConversation.conversation_id}`}
                            className="flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold text-amber-800 dark:text-amber-300 hover:bg-amber-100/80 dark:hover:bg-amber-950/40 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                            title="Enqueue (FIFO): Queues prompt behind active task in IDE without interrupting"
                            aria-label="Enqueue (FIFO): Queues prompt behind active task in IDE without interrupting"
                        >
                            <ListPlus className={cn('w-3 h-3 text-amber-500 dark:text-amber-400 shrink-0', isEnqueueing && 'animate-pulse')} />
                            <span>{isEnqueueing ? 'Queueing…' : 'Enqueue'}</span>
                        </button>

                        {/* 3. Rerun (Randomized): Directly dispatches auto-picked randomized completion/release check */}
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
                            className="flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-sky-950/40 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                            title={`Rerun (${rerunMessageOption}): Dispatches auto-picked completion/release nudge (Hotkey: R)`}
                            aria-label="Rerun: Dispatches auto-picked completion/release nudge"
                        >
                            <Repeat className={cn('w-3 h-3 text-sky-500 shrink-0', isRerunning && 'animate-spin')} />
                            <span>{isRerunning ? 'Rerunning…' : (rerunFlash ?? `Rerun ×${rerunCount}`)}</span>
                        </button>

                        {/* 4. Dropdown Chevron for Unified Dispatch Menu (Rerun, Target, Templates) */}
                        <button
                            type="button"
                            onClick={handleToggleDispatchMenu}
                            data-ui-name="prompt-dispatch-options-btn"
                            data-ui-id={`prompt-dispatch-options-${selectedConversation.conversation_id}`}
                            className="flex items-center justify-center px-1.5 py-1 rounded-r-md text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-slate-800/80 text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            aria-label="Unified dispatch configuration popover"
                            title="Dispatch options & Rerun settings"
                        >
                            <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-150", isMenuOpen && "rotate-180")} />
                        </button>

                        {/* Flyout 2: Unified Dispatch Configuration Popover */}
                        {isMenuOpen && (
                            <div
                                className="absolute right-0 top-full mt-1.5 z-50 w-80 rounded-xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-xl p-2.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-2"
                                onMouseLeave={handleCloseDispatchMenu}
                            >
                                {/* Menu Header */}
                                <div className="px-2 pt-0.5 pb-1 text-[10px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 select-none border-b border-slate-100 dark:border-slate-800 flex items-center justify-between">
                                    <span>Dispatch Configuration</span>
                                    <span className="text-[9px] text-slate-400">AGM:P001</span>
                                </div>

                                {/* Mode Switcher Segmented Pill: [ Send Now | Enqueue FIFO | Rerun Turn ] */}
                                <div className="flex items-center justify-between px-2 py-1 text-xs">
                                    <span className="text-slate-500 dark:text-slate-400 text-[11px] font-medium shrink-0">
                                        Mode:
                                    </span>
                                    <div className="inline-flex rounded-lg bg-slate-100 dark:bg-[var(--ui-canvas,rgba(0,0,0,0.3))] p-0.5 border border-slate-200/80 dark:border-[var(--ui-border-subtle,rgba(255,255,255,0.1))] text-[11px]">
                                        <button
                                             type="button"
                                             onClick={() => handleSwitchDispatchMode('send')}
                                             title="Cancel & Send / Send (Preempt): Cancels any running task to execute immediately"
                                             className={cn(
                                                 "px-2 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                 primaryDispatchAction === 'send'
                                                     ? "bg-white dark:bg-[var(--ui-surface-hover,rgba(255,255,255,0.1))] text-emerald-600 dark:text-emerald-400 shadow-2xs font-semibold"
                                                     : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                             )}
                                         >
                                             Send (Preempt)
                                         </button>
                                         <button
                                             type="button"
                                             onClick={() => handleSwitchDispatchMode('enqueue')}
                                             title="Enqueue (FIFO): Queues prompt behind active task in IDE"
                                             className={cn(
                                                 "px-2 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                 primaryDispatchAction === 'enqueue'
                                                     ? "bg-white dark:bg-[var(--ui-surface-hover,rgba(255,255,255,0.1))] text-amber-600 dark:text-amber-400 shadow-2xs font-semibold"
                                                     : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                             )}
                                         >
                                             Enqueue (FIFO)
                                         </button>
                                         <button
                                             type="button"
                                             onClick={() => handleSwitchDispatchMode('rerun')}
                                             className={cn(
                                                 "px-2 py-0.5 rounded-md font-medium transition-colors cursor-pointer",
                                                 primaryDispatchAction === 'rerun'
                                                     ? "bg-white dark:bg-[var(--ui-surface-hover,rgba(255,255,255,0.1))] text-sky-600 dark:text-sky-400 shadow-2xs font-semibold"
                                                     : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 dark:hover:bg-slate-800/80"
                                             )}
                                         >
                                             Rerun Turn
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

                                        {/* Message options */}
                                        <div className="px-2 pt-1 text-[11px] font-medium text-slate-500 dark:text-slate-400">
                                            Message:
                                        </div>
                                        <div className="space-y-1 max-h-52 overflow-y-auto pr-0.5">
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
                                                                : "border-transparent text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/80"
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
                                            <div className="mx-2 px-2 py-1.5 rounded-lg bg-slate-50/80 dark:bg-purple-950/20 border border-slate-200/60 dark:border-purple-500/20 max-h-36 overflow-y-auto">
                                                <div className="text-[9.5px] font-mono uppercase tracking-wider text-slate-400 dark:text-slate-500 mb-1 sticky top-0 bg-slate-50 dark:bg-[#1a0e2e]">
                                                    Built-in set ({RERUN_RANDOM_MESSAGE_SET.length} variations — picked per dispatch)
                                                </div>
                                                <ul className="space-y-0.5 text-[10.5px] text-slate-500 dark:text-slate-400 list-disc list-inside">
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
