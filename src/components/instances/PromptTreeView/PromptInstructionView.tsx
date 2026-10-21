import { useState, useMemo, useEffect } from 'react';
import { Check, Copy, RotateCcw, ChevronUp, ChevronDown, Clock, Image as LucideImage } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../../utils/cn';
import { PromptViewMode, type AgmConversationNode, type AgmProjectTreeNode, type ViewMode } from './types';
import { RichMarkdownRenderer } from './markdown';
import { countWords, formatPromptForMarkdown } from './formatters';
import { PromptImagePreviewModal } from './PromptImagePreviewModal';
import { PromptImageContextMenu } from './PromptImageContextMenu';
import { resolveImageSrc, extractLocalPathFromUri, writePureImageToClipboard, copyAllImagesToClipboard } from './clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { useConfigStore } from '../../../stores/useConfigStore';
import { formatDateTime, formatDateOnly } from '../../../utils/date';

export interface ExtractedImage {
    src: string;
    alt: string;
}

/**
 * Normalizes an image URI into a uniform lowercase path string for deduplication keys.
 * Strips file:/// or file:// protocols, removes invalid leading slashes before Windows drive letters,
 * decodes percent-encoded characters, and normalizes backslashes to forward slashes.
 */
export function normalizeMediaUri(uri: string): string {
    if (!uri || !uri.trim()) return '';
    let clean = uri.trim();
    if (clean.startsWith('file:///')) {
        if (/^file:\/\/\/[a-zA-Z]:[\\/]/.test(clean)) {
            clean = clean.slice(8);
        } else {
            clean = clean.slice(7);
            if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
                clean = clean.slice(1);
            }
        }
    } else if (clean.startsWith('file://')) {
        clean = clean.slice(7);
        if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
            clean = clean.slice(1);
        }
    }

    try {
        clean = decodeURIComponent(clean);
    } catch {
        // Keep raw clean if decoding fails
    }

    clean = clean.replace(/\\/g, '/');
    if (/^\/[a-zA-Z]:\//.test(clean)) {
        clean = clean.slice(1);
    }

    return clean.toLowerCase();
}

function extractImagesFromPrompt(
    text: string,
    conv: AgmConversationNode
): ExtractedImage[] {
    const list: ExtractedImage[] = [];
    const seen = new Set<string>();

    // 1. From prompt turns
    if (conv.prompts && conv.prompts.length > 0) {
        const turns = conv.selected_turn_number != null
            ? conv.prompts.filter((t) => t.turn_number === conv.selected_turn_number)
            : [conv.prompts.find((t) => t.is_running) || conv.prompts[conv.prompts.length - 1]];
        for (const t of turns) {
            if (t.media_paths) {
                for (const p of t.media_paths) {
                    if (p) {
                        const key = normalizeMediaUri(p) || p;
                        if (!seen.has(key)) {
                            seen.add(key);
                            list.push({ src: p, alt: `Turn ${t.turn_number} Image` });
                        }
                    }
                }
            }
        }
    }

    // 1.5 From conversation-level media_paths
    if (conv.media_paths && conv.media_paths.length > 0) {
        for (const p of conv.media_paths) {
            if (p) {
                const key = normalizeMediaUri(p) || p;
                if (!seen.has(key)) {
                    seen.add(key);
                    list.push({ src: p, alt: 'Attached Image' });
                }
            }
        }
    }

    // 2. From markdown text ![alt](url)
    const imgRegex = /!\[(.*?)\]\((.*?)\)/g;
    let match: RegExpExecArray | null;
    while ((match = imgRegex.exec(text)) !== null) {
        const alt = match[1] || 'Embedded Image';
        const src = match[2];
        if (src) {
            const key = normalizeMediaUri(src) || src;
            if (!seen.has(key)) {
                seen.add(key);
                list.push({ src, alt });
            }
        }
    }

    // 3. From HTML <img> tags
    const htmlImgRegex = /<img\s+[^>]*?src=["'](.*?)["'][^>]*?>/gi;
    while ((match = htmlImgRegex.exec(text)) !== null) {
        const src = match[1];
        if (src) {
            const key = normalizeMediaUri(src) || src;
            if (!seen.has(key)) {
                seen.add(key);
                list.push({ src, alt: 'HTML Image' });
            }
        }
    }

    // 4. From standalone raw data:image/... URIs
    const rawDataUriRegex = /(data:image\/[a-zA-Z0-9.+_-]+;base64,[A-Za-z0-9+/=]+)/g;
    while ((match = rawDataUriRegex.exec(text)) !== null) {
        const src = match[1];
        if (src) {
            const key = normalizeMediaUri(src) || src;
            if (!seen.has(key)) {
                seen.add(key);
                list.push({ src, alt: 'Base64 Image' });
            }
        }
    }

    // 5. From sub_runs / sub-prompts (sub-image tracking)
    if (conv.sub_runs && conv.sub_runs.length > 0) {
        for (const sub of conv.sub_runs) {
            if (sub.media_paths) {
                for (const p of sub.media_paths) {
                    if (p) {
                        const key = normalizeMediaUri(p) || p;
                        if (!seen.has(key)) {
                            seen.add(key);
                            list.push({ src: p, alt: `Sub-agent (${sub.title || sub.short_id || 'child'}) Image` });
                        }
                    }
                }
            }
        }
    }

    return list;
}

interface PromptThumbnailCardProps {
    img: ExtractedImage;
    repoPath?: string;
    onPreview: (src: string, alt?: string) => void;
    onContextMenu: (e: React.MouseEvent, src: string, alt?: string, rawSrc?: string) => void;
}

function PromptThumbnailCard({ img, repoPath, onPreview, onContextMenu }: PromptThumbnailCardProps) {
    const [resolvedSrc, setResolvedSrc] = useState<string>(() => resolveImageSrc(img.src, repoPath));
    const [hasFailed, setHasFailed] = useState(false);
    const [isThumbnailCopied, setIsThumbnailCopied] = useState(false);

    useEffect(() => {
        let isCancelled = false;
        const load = async () => {
            const raw = img.src.trim();
            if (raw.startsWith('data:') || raw.startsWith('http://') || raw.startsWith('https://')) {
                setResolvedSrc(raw);
                setHasFailed(false);
                return;
            }
            let localPath = extractLocalPathFromUri(raw) || raw;
            const isAbsolute = /^[a-zA-Z]:[\\/]/.test(localPath) || localPath.startsWith('/') || localPath.startsWith('\\\\');
            if (!isAbsolute && repoPath && repoPath.trim().length > 0) {
                const sep = repoPath.includes('/') ? '/' : '\\';
                localPath = `${repoPath.replace(/[/\\]+$/, '')}${sep}${localPath.replace(/^(\.[/\\]|[/\\])+/, '')}`;
            }

            try {
                const base64Data = await invoke<string>('read_image_as_base64', { path: localPath });
                if (!isCancelled && base64Data && base64Data.startsWith('data:image/')) {
                    setResolvedSrc(base64Data);
                    setHasFailed(false);
                    return;
                }
            } catch (err: unknown) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'PromptThumbnailCard.load_base64',
                    triggerAction: 'read_image_as_base64_proactive',
                });
            }
            if (!isCancelled) {
                setResolvedSrc(resolveImageSrc(img.src, repoPath));
            }
        };
        load();
        return () => {
            isCancelled = true;
        };
    }, [img.src, repoPath]);

    const handleImgError = async () => {
        const raw = img.src.trim();
        let localPath = extractLocalPathFromUri(raw);
        if (!localPath && !raw.startsWith('data:') && !raw.startsWith('http://') && !raw.startsWith('https://')) {
            localPath = raw;
        }

        if (localPath) {
            const isAbsolute = /^[a-zA-Z]:[\\/]/.test(localPath) || localPath.startsWith('/') || localPath.startsWith('\\\\');
            if (!isAbsolute && repoPath && repoPath.trim().length > 0) {
                const sep = repoPath.includes('/') ? '/' : '\\';
                localPath = `${repoPath.replace(/[/\\]+$/, '')}${sep}${localPath.replace(/^(\.[/\\]|[/\\])+/, '')}`;
            }

            try {
                const base64Data = await invoke<string>('read_image_as_base64', { path: localPath });
                if (base64Data && base64Data.startsWith('data:image/')) {
                    setResolvedSrc(base64Data);
                    setHasFailed(false);
                    return;
                }
            } catch (err: unknown) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'PromptThumbnailCard.handleImgError',
                    triggerAction: 'read_image_as_base64_fallback',
                });
            }
        }
        setHasFailed(true);
    };

    return (
        <div
            className="relative group shrink-0"
            onClick={() => onPreview(resolvedSrc, img.alt)}
            onContextMenu={(e) => onContextMenu(e, resolvedSrc, img.alt, img.src)}
        >
            {!hasFailed ? (
                <img
                    src={resolvedSrc}
                    alt={img.alt}
                    onError={handleImgError}
                    className="h-16 w-24 object-cover rounded-md border border-slate-200 dark:border-[#40285e] hover:border-blue-500 dark:hover:border-blue-400 cursor-pointer shadow-2xs transition-all hover:scale-102 bg-slate-100 dark:bg-[#090611]"
                />
            ) : (
                <div
                    className="h-16 w-24 flex flex-col items-center justify-center rounded-md border border-dashed border-amber-500/40 bg-amber-500/5 text-amber-600 dark:text-amber-400 p-1 text-center cursor-pointer shadow-2xs hover:border-amber-500 transition-all hover:scale-102"
                    title={`Image: ${img.src} (Click to preview · Right-click for options)`}
                >
                    <LucideImage className="w-5 h-5 mb-0.5 opacity-70" />
                    <span className="text-[9px] font-mono truncate w-full px-0.5">{img.alt || 'Attached Image'}</span>
                </div>
            )}
            {/* 1-Click Pure PNG Clipboard Button */}
            <button
                type="button"
                onClick={async (e) => {
                    e.stopPropagation();
                    e.preventDefault();
                    const target = resolvedSrc || img.src;
                    const ok = await writePureImageToClipboard(target);
                    if (ok) {
                        setIsThumbnailCopied(true);
                        setTimeout(() => setIsThumbnailCopied(false), 2000);
                    }
                }}
                className="absolute top-1 right-1 p-1 rounded-sm bg-black/75 hover:bg-purple-600 text-white transition-all opacity-0 group-hover:opacity-100 cursor-pointer shadow-xs z-10"
                title="Copy pure PNG to clipboard for OneNote / chat"
                aria-label="Copy pure PNG image"
            >
                {isThumbnailCopied ? <Check className="w-2.5 h-2.5 text-emerald-400" /> : <Copy className="w-2.5 h-2.5 text-white" />}
            </button>
            <div className="absolute bottom-0 inset-x-0 bg-black/60 text-[9px] text-white truncate px-1 py-0.5 rounded-b-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                {img.alt}
            </div>
        </div>
    );
}

export interface PromptInstructionViewProps {
    selectedConversation: AgmConversationNode;
    selectedTurnNumber?: number | null;
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
    isCopied?: boolean;
    setIsCopied?: (v: boolean) => void;
    isCopiedAll?: boolean;
    onCopyAllWithRichImages?: () => void;
    openInspector: (conv: AgmConversationNode, repoPath: string) => void;
    selectedProject: AgmProjectTreeNode | null;
    activeByteCount: number;
    setActivePromptText: (v: string) => void;
    setActionMsg: (v: string | null) => void;
    promptPreviewWords?: number;
}

export function PromptInstructionView(props: PromptInstructionViewProps) {
    const {
        selectedConversation, selectedTurnNumber: selectedTurnNumberProp, viewMode,
        activePromptText, editedPromptText, setEditedPromptText,
        displayedMarkdown, showAllWords, setShowAllWords,
        isTruncated, totalWords, activeWordCount, formatByteSize,
        openInspector, selectedProject, activeByteCount,
        setActivePromptText, setActionMsg,
        promptPreviewWords,
    } = props;

    const { config } = useConfigStore();
    const effectiveWordLimit = promptPreviewWords ?? config?.prompt_preview_words ?? 200;

    const selectedTurnNumber = selectedTurnNumberProp !== undefined
        ? selectedTurnNumberProp
        : (selectedConversation.selected_turn_number ?? null);

    const [activeTurnPreview, setActiveTurnPreview] = useState<number | null>(null);
    const [isOverviewExpanded, setIsOverviewExpanded] = useState<boolean>(() => {
        try {
            const saved = localStorage.getItem('agm_prompt_overview_expanded');
            return saved !== null ? JSON.parse(saved) === true : false;
        } catch {
            return false;
        }
    });

    const handleToggleOverview = () => {
        setIsOverviewExpanded((prev) => {
            const next = !prev;
            try {
                localStorage.setItem('agm_prompt_overview_expanded', JSON.stringify(next));
            } catch (err: unknown) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'PromptInstructionView.toggleOverview',
                    triggerAction: 'save_overview_state',
                });
            }
            return next;
        });
    };

    const [previewImage, setPreviewImage] = useState<{ src: string; alt?: string } | null>(null);
    const [contextMenu, setContextMenu] = useState<{
        isOpen: boolean;
        position: { x: number; y: number };
        imageSrc: string;
        rawPath?: string;
        altText?: string;
    }>({
        isOpen: false,
        position: { x: 0, y: 0 },
        imageSrc: '',
    });

    const handleOpenPreview = (src: string, alt?: string) => {
        setContextMenu((prev) => ({ ...prev, isOpen: false }));
        setPreviewImage({ src, alt });
    };

    const handleImageContextMenu = (e: React.MouseEvent, src: string, alt?: string, rawSrc?: string) => {
        e.preventDefault();
        e.stopPropagation();
        setContextMenu({
            isOpen: true,
            position: { x: e.clientX, y: e.clientY },
            imageSrc: src,
            rawPath: rawSrc || src,
            altText: alt,
        });
    };

    const attachedImages = useMemo(
        () => extractImagesFromPrompt(activePromptText, selectedConversation),
        [activePromptText, selectedConversation]
    );

    const [copiedPureImageIdx, setCopiedPureImageIdx] = useState<number | null>(null);
    const [isCopyingAllImages, setIsCopyingAllImages] = useState(false);
    const [isCopiedAllImages, setIsCopiedAllImages] = useState(false);

    const handleCopyFirstImage = async () => {
        if (!attachedImages || attachedImages.length === 0) return;
        const first = attachedImages[0];
        const ok = await writePureImageToClipboard(first.src);
        if (ok) {
            setCopiedPureImageIdx(0);
            setActionMsg?.('Image copied to clipboard as pure PNG for OneNote / chat!');
            setTimeout(() => {
                setCopiedPureImageIdx(null);
                setActionMsg?.(null);
            }, 2500);
        }
    };

    const handleCopyAllImages = async () => {
        if (!attachedImages || attachedImages.length === 0) return;
        setIsCopyingAllImages(true);
        try {
            const uris = attachedImages.map((img) => img.src);
            const ok = await copyAllImagesToClipboard(uris, selectedProject?.repo_path);
            if (ok) {
                setIsCopiedAllImages(true);
                setActionMsg?.(`Copied all ${attachedImages.length} images to clipboard!`);
                setTimeout(() => {
                    setIsCopiedAllImages(false);
                    setActionMsg?.(null);
                }, 2500);
            }
        } finally {
            setIsCopyingAllImages(false);
        }
    };

    const handleCopyImagesUnified = async () => {
        if (!attachedImages || attachedImages.length === 0) return;
        if (attachedImages.length === 1) {
            await handleCopyFirstImage();
        } else {
            await handleCopyAllImages();
        }
    };

    const turns = selectedConversation.prompts || [];
    const totalTurns = selectedConversation.prompts?.length || 1;

    const isRunning = Boolean(selectedConversation.is_running) || turns.some((t) => Boolean(t.is_running));
    const isQueued = !isRunning && (
        Boolean(selectedConversation.is_queued) ||
        (selectedConversation.status && selectedConversation.status.toLowerCase().includes('queue')) ||
        turns.some((t) => Boolean(t.is_queued))
    );
    const statusLabel = isRunning ? 'RUNNING' : isQueued ? 'QUEUED' : 'IDLE';

    const enqueuedTurns = useMemo(() => {
        return turns.filter((t) => Boolean(t.is_queued) || (t.status && t.status.toLowerCase().includes('queue')));
    }, [turns]);

    const { timestampRange, durationDisplay } = useMemo(() => {
        const formatTime24 = (d: Date): string => {
            const h = String(d.getHours()).padStart(2, '0');
            const m = String(d.getMinutes()).padStart(2, '0');
            return `${h}:${m}`;
        };

        if (turns.length === 0) {
            let formatted = 'N/A';
            if (selectedConversation.last_modified) {
                const d = new Date(selectedConversation.last_modified);
                if (!isNaN(d.getTime())) {
                    formatted = `${formatDateOnly(d)} ${formatTime24(d)}`;
                }
            }
            return { timestampRange: formatted, durationDisplay: 'Single prompt' };
        }

        const timestamps = turns
            .map((t) => {
                let ms = 0;
                if (typeof t.created_at === 'number' && t.created_at > 0) {
                    ms = t.created_at < 100_000_000_000 ? t.created_at * 1000 : t.created_at;
                } else if (t.timestamp) {
                    const parsed = new Date(t.timestamp).getTime();
                    if (!isNaN(parsed) && parsed > 0) ms = parsed;
                }
                return ms;
            })
            .filter((t) => t > 0)
            .sort((a, b) => a - b);

        if (timestamps.length === 0) {
            let formatted = 'N/A';
            if (selectedConversation.last_modified) {
                const d = new Date(selectedConversation.last_modified);
                if (!isNaN(d.getTime())) {
                    formatted = `${formatDateOnly(d)} ${formatTime24(d)}`;
                }
            }
            return { timestampRange: formatted, durationDisplay: 'Active session' };
        }

        const first = new Date(timestamps[0]);
        const last = new Date(timestamps[timestamps.length - 1]);

        const isSameDay = first.getFullYear() === last.getFullYear() &&
                          first.getMonth() === last.getMonth() &&
                          first.getDate() === last.getDate();

        const range = timestamps.length === 1
            ? `${formatDateOnly(first)} ${formatTime24(first)}`
            : isSameDay
                ? `${formatTime24(first)} → ${formatTime24(last)}`
                : `${formatDateOnly(first)} ${formatTime24(first)} → ${formatDateOnly(last)} ${formatTime24(last)}`;

        const wallDiffSeconds = Math.max(0, Math.floor((last.getTime() - first.getTime()) / 1000));
        const firstDaySec = first.getHours() * 3600 + first.getMinutes() * 60 + first.getSeconds();
        const lastDaySec = last.getHours() * 3600 + last.getMinutes() * 60 + last.getSeconds();
        const daySpanSec = lastDaySec >= firstDaySec ? lastDaySec - firstDaySec : wallDiffSeconds;
        const diffSeconds = isSameDay && daySpanSec > 0 ? daySpanSec : wallDiffSeconds;

        let durationStr = '';
        if (diffSeconds < 60) {
            durationStr = `${diffSeconds}s`;
        } else if (diffSeconds < 3600) {
            const mins = Math.floor(diffSeconds / 60);
            durationStr = `${mins}m`;
        } else {
            const hrs = Math.floor(diffSeconds / 3600);
            const mins = Math.floor((diffSeconds % 3600) / 60);
            durationStr = mins === 0 ? `${hrs}h` : `${hrs}h ${mins}m`;
        }
        return { timestampRange: range, durationDisplay: durationStr };
    }, [turns, selectedConversation.last_modified]);

    const inspectedTurnNumber = activeTurnPreview ?? selectedTurnNumber;
    const inspectedTurn = useMemo(() => {
        if (inspectedTurnNumber === null) return null;
        return turns.find((t) => t.turn_number === inspectedTurnNumber) || null;
    }, [turns, inspectedTurnNumber]);

    const inspectedTurnSent = useMemo(() => {
        if (!inspectedTurn) return null;
        const ts = inspectedTurn.created_at || (inspectedTurn.timestamp ? new Date(inspectedTurn.timestamp).getTime() : 0);
        if (!ts) return null;
        return formatDateTime(ts);
    }, [inspectedTurn]);

    const turnElapsed = useMemo(() => {
        if (!inspectedTurn) return null;
        if (inspectedTurn.is_running) {
            const ts = inspectedTurn.created_at || (inspectedTurn.timestamp ? new Date(inspectedTurn.timestamp).getTime() : 0);
            if (ts) {
                const diffSec = Math.max(0, Math.floor((Date.now() - ts) / 1000));
                if (diffSec < 60) return `${diffSec}s`;
                if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ${diffSec % 60}s`;
                return `${Math.floor(diffSec / 3600)}h ${Math.floor((diffSec % 3600) / 60)}m`;
            }
            return 'Running';
        }
        return inspectedTurn.status || 'COMPLETED';
    }, [inspectedTurn]);

    return (
        <div className="space-y-3">
            {/* Single Prompt Turn View Scope Banner */}
            {selectedTurnNumber !== null && (
                <div className="flex items-center justify-between p-2.5 rounded-lg border border-sky-300/60 dark:border-sky-800/40 bg-sky-50/70 dark:bg-sky-950/20 text-xs">
                    <div className="flex items-center gap-2 min-w-0">
                        <span className="font-mono text-[10px] px-2 py-0.5 rounded-md bg-sky-200/80 dark:bg-sky-900/60 text-sky-800 dark:text-sky-200 font-bold shrink-0">
                            TURN T{selectedTurnNumber}
                        </span>
                        <span className="text-sky-900 dark:text-sky-100 font-medium truncate">
                            Inspecting single prompt turn {selectedTurnNumber} of {totalTurns}
                        </span>
                    </div>
                    <button
                        type="button"
                        onClick={() => {
                            selectedConversation.selected_turn_number = null;
                            const fullText = selectedConversation.full_prompt_text || selectedConversation.prompts?.[0]?.prompt_text || '';
                            if (fullText) {
                                setActivePromptText(fullText);
                            }
                            setActiveTurnPreview(null);
                            setActionMsg?.('Switched to Full Conversation Session View');
                            setTimeout(() => setActionMsg?.(null), 2000);
                        }}
                        className="flex items-center gap-1 text-[11px] font-semibold text-blue-600 dark:text-cyan-400 hover:underline shrink-0 cursor-pointer ml-2"
                        title="Switch back to complete conversation session dashboard"
                    >
                        <span>View Full Session Summary</span>
                        <span>&rarr;</span>
                    </button>
                </div>
            )}

            {/* Conversation Session Summary Dashboard (Rendered when selectedTurnNumber === null) */}
            {selectedTurnNumber === null && (
                !isOverviewExpanded ? (
                    /* Compact Collapsed Single-Line Summary Bar */
                    <div className="flex items-center justify-between p-2.5 rounded-xl border border-slate-200/80 dark:border-[#40285e]/80 bg-gradient-to-r from-slate-50 via-white to-purple-50/20 dark:from-[#171025] dark:via-[#120a1f] dark:to-[#1a0e2e] shadow-xs text-xs font-poppins">
                        <div className="flex items-center gap-2 min-w-0">
                            <span className="font-mono text-[10px] px-1.5 py-0.5 rounded-md bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 font-bold border border-purple-200 dark:border-purple-800/40 shrink-0">
                                {selectedConversation.seq_code || 'CONV'}
                            </span>
                            {isRunning ? (
                                <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[9.5px] font-bold bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
                                    <span>RUNNING</span>
                                </span>
                            ) : isQueued ? (
                                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                                    <Clock className="w-2.5 h-2.5 text-amber-500" />
                                    <span>QUEUED</span>
                                </span>
                            ) : (
                                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[9.5px] font-bold bg-slate-200/80 dark:bg-[#251b3a] text-slate-700 dark:text-slate-300 border border-slate-300/60 dark:border-[#40285e]">
                                    <Check className="w-2.5 h-2.5 text-emerald-500" />
                                    <span>IDLE</span>
                                </span>
                            )}
                            <span className="font-mono text-[11px] font-bold text-slate-700 dark:text-slate-300">
                                {totalTurns} {totalTurns === 1 ? 'turn' : 'turns'}
                            </span>
                            <span className="font-mono text-[10px] text-slate-400 dark:text-slate-500 hidden sm:inline-block">
                                · {durationDisplay} ({timestampRange})
                            </span>
                            {enqueuedTurns.length > 0 && (
                                <span className="text-[9.5px] font-mono px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-950/60 text-amber-800 dark:text-amber-300 border border-amber-300/40">
                                    FIFO: {enqueuedTurns.length} pending
                                </span>
                            )}
                        </div>
                        <button
                            type="button"
                            onClick={handleToggleOverview}
                            className="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold rounded-lg text-purple-600 dark:text-purple-400 border border-purple-200 dark:border-purple-800/40 bg-purple-50/50 dark:bg-purple-950/20 hover:bg-purple-100 dark:hover:bg-purple-900/40 transition-colors cursor-pointer shrink-0 ml-2"
                            title="Expand full session overview and metrics"
                        >
                            <span>Session Overview</span>
                            <ChevronDown className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                        </button>
                    </div>
                ) : (
                    /* Full Expanded Overview Card */
                    <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-gradient-to-br from-slate-50 via-white to-purple-50/20 dark:from-[#171025] dark:via-[#120a1f] dark:to-[#1a0e2e] p-4 shadow-xs space-y-3 font-poppins">
                        {/* Streamlined Header: Deduplicated, no repeated title or conversation ID */}
                        <div className="flex items-center justify-between pb-2 border-b border-slate-200/60 dark:border-[#2b1b42]">
                            <div className="flex items-center gap-2">
                                <span className="text-xs font-bold text-slate-800 dark:text-slate-100">
                                    Session Overview & Metrics
                                </span>
                                {enqueuedTurns.length > 0 && (
                                    <span className="text-[9.5px] font-mono px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-950/60 text-amber-800 dark:text-amber-300 border border-amber-300/40">
                                        {enqueuedTurns.length} pending
                                    </span>
                                )}
                            </div>
                            <button
                                type="button"
                                onClick={handleToggleOverview}
                                className="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold rounded-lg text-purple-600 dark:text-purple-400 border border-purple-200 dark:border-purple-800/40 bg-purple-50/50 dark:bg-purple-950/20 hover:bg-purple-100 dark:hover:bg-purple-900/40 transition-colors cursor-pointer"
                                title="Collapse session overview"
                            >
                                <span>Collapse Overview</span>
                                <ChevronUp className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                            </button>
                        </div>

                    {/* Metrics Grid: Total Turns, Status, Timestamp Range, Duration (Shown when expanded) */}
                    <div className="space-y-2 pt-0.5 animate-in fade-in duration-150">
                        <p className="text-[10px] text-slate-400 dark:text-slate-500 truncate font-mono">
                            ID: {selectedConversation.conversation_id}
                        </p>
                        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5 text-xs">
                            <div className="p-2 rounded-lg bg-white/80 dark:bg-[#1a1228]/80 border border-slate-200/60 dark:border-[#382352]/60">
                                <span className="text-[10px] font-medium text-slate-400 dark:text-slate-500 block">Total Turns</span>
                                <span className="text-sm font-bold text-slate-800 dark:text-slate-100 font-mono">
                                    {totalTurns} {totalTurns === 1 ? 'turn' : 'turns'}
                                </span>
                            </div>
                            <div className="p-2 rounded-lg bg-white/80 dark:bg-[#1a1228]/80 border border-slate-200/60 dark:border-[#382352]/60">
                                <span className="text-[10px] font-medium text-slate-400 dark:text-slate-500 block">Session Status</span>
                                <span className={cn(
                                    "text-sm font-bold font-mono",
                                    isRunning ? "text-emerald-600 dark:text-emerald-400" : isQueued ? "text-amber-600 dark:text-amber-400" : "text-slate-700 dark:text-slate-300"
                                )}>
                                    {statusLabel}
                                </span>
                            </div>
                            <div className="p-2 rounded-lg bg-white/80 dark:bg-[#1a1228]/80 border border-slate-200/60 dark:border-[#382352]/60">
                                <span className="text-[10px] font-medium text-slate-400 dark:text-slate-500 block">
                                    {inspectedTurn ? `Turn T${inspectedTurn.turn_number} Sent` : 'Time Range'}
                                </span>
                                <span className="text-xs font-semibold text-slate-700 dark:text-slate-200 font-mono truncate block" title={inspectedTurnSent || timestampRange}>
                                    {inspectedTurnSent || timestampRange}
                                </span>
                            </div>
                            <div className="p-2 rounded-lg bg-white/80 dark:bg-[#1a1228]/80 border border-slate-200/60 dark:border-[#382352]/60">
                                <span className="text-[10px] font-medium text-slate-400 dark:text-slate-500 block">
                                    {inspectedTurn ? 'Turn Execution' : 'Session Timespan'}
                                </span>
                                <span className="text-sm font-bold text-slate-800 dark:text-slate-100 font-mono" title={inspectedTurn ? `Total session span: ${durationDisplay}` : undefined}>
                                    {inspectedTurn ? turnElapsed : durationDisplay}
                                </span>
                                {inspectedTurn && (
                                    <span className="text-[9px] text-slate-400 block truncate font-mono">
                                        Session: {durationDisplay}
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Enqueued Prompts Section (listing pending queued turns with badges) */}
                    {enqueuedTurns.length > 0 && (
                        <div className="p-2.5 rounded-lg bg-amber-50/70 dark:bg-amber-950/20 border border-amber-300/60 dark:border-amber-800/40 space-y-1.5">
                            <div className="flex items-center justify-between text-xs font-bold text-amber-800 dark:text-amber-300">
                                <div className="flex items-center gap-1.5">
                                    <Clock className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
                                    <span>Enqueued Prompts ({enqueuedTurns.length} pending in FIFO queue)</span>
                                </div>
                                <span className="text-[9px] font-mono px-1.5 py-0.2 rounded-md bg-amber-200/70 dark:bg-amber-900/60 text-amber-900 dark:text-amber-200 font-semibold">
                                    FIFO Priority
                                </span>
                            </div>
                            <div className="space-y-1 max-h-36 overflow-y-auto pr-1">
                                {enqueuedTurns.map((turn, qIdx) => (
                                    <div
                                        key={`enqueued-${turn.step_index || qIdx}`}
                                        onClick={() => {
                                            if (turn.prompt_text) {
                                                setActivePromptText(turn.prompt_text);
                                                setActiveTurnPreview(turn.turn_number);
                                                setActionMsg?.(`Viewing Queued Turn T${turn.turn_number}`);
                                                setTimeout(() => setActionMsg?.(null), 2000);
                                            }
                                        }}
                                        className="flex items-center justify-between p-1.5 rounded-md bg-white/90 dark:bg-[#1a1228]/90 border border-amber-200/60 dark:border-amber-800/30 text-xs hover:border-amber-400 dark:hover:border-amber-600 transition-colors cursor-pointer"
                                        aria-label={`View queued prompt turn T${turn.turn_number}`}
                                    >
                                        <div className="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
                                            <span className="font-mono text-[9px] px-1 py-0.2 rounded-xs bg-amber-100 dark:bg-amber-950/60 text-amber-800 dark:text-amber-300 font-bold shrink-0">
                                                T{turn.turn_number}
                                            </span>
                                            <span className="text-[11px] text-slate-700 dark:text-slate-300 truncate">
                                                {turn.preview_200w || turn.prompt_text || `Queued prompt #${turn.turn_number}`}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-1 shrink-0">
                                            {turn.media_paths && turn.media_paths.length > 0 && (
                                                <span className="text-[9px] font-mono px-1 py-0.2 rounded-xs bg-sky-100 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 font-medium">
                                                    🖼️ {turn.media_paths.length}
                                                </span>
                                            )}
                                            <span className="text-[9px] font-mono font-bold text-amber-600 dark:text-amber-400 px-1 py-0.2 rounded-xs bg-amber-50 dark:bg-amber-950/40 border border-amber-300/40">
                                                QUEUE #{qIdx + 1}
                                            </span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Chronological turn navigation chips (T1, T2, T3...) */}
                    {turns.length > 1 && (
                        <div className="space-y-1.5 pt-1 border-t border-slate-200/40 dark:border-[#2b1b42]/60">
                            <div className="flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400 font-medium">
                                <span>Chronological Prompt Turns:</span>
                                <span className="text-[10px] font-mono text-slate-400">
                                    Click any turn chip to preview prompt
                                </span>
                            </div>
                            <div className="flex items-center gap-x-2 gap-y-2 flex-wrap">
                                {turns.map((turn) => {
                                    const isTurnRunning = Boolean(turn.is_running);
                                    const isTurnQueued = Boolean(turn.is_queued) || (turn.status && turn.status.toLowerCase().includes('queue'));
                                    const isTurnSelected = activeTurnPreview === turn.turn_number;
                                    return (
                                        <button
                                            key={`chip-turn-${turn.turn_number}`}
                                            type="button"
                                            onClick={() => {
                                                if (turn.prompt_text) {
                                                    setActivePromptText(turn.prompt_text);
                                                    setActiveTurnPreview(turn.turn_number);
                                                    setActionMsg?.(`Loaded Turn T${turn.turn_number} into viewer`);
                                                    setTimeout(() => setActionMsg?.(null), 2000);
                                                }
                                            }}
                                            className={cn(
                                                "inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-mono font-semibold transition-all cursor-pointer shadow-2xs border",
                                                isTurnSelected
                                                    ? "bg-purple-600 text-white border-purple-600 shadow-xs"
                                                    : isTurnRunning
                                                    ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30 hover:bg-emerald-500/20"
                                                    : isTurnQueued
                                                    ? "bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/30 hover:bg-amber-500/20"
                                                    : "bg-white dark:bg-[#1a1228] text-slate-700 dark:text-slate-300 border-slate-200 dark:border-[#382352] hover:border-purple-400 dark:hover:border-purple-500 hover:text-purple-600 dark:hover:text-purple-300"
                                            )}
                                            aria-label={`Turn T${turn.turn_number} - ${turn.status || 'COMPLETED'}`}
                                        >
                                            <span className={cn(
                                                "w-1.5 h-1.5 rounded-full shrink-0",
                                                isTurnSelected ? "bg-white" : isTurnRunning ? "bg-emerald-500 animate-ping" : isTurnQueued ? "bg-amber-500" : "bg-slate-400 dark:bg-purple-400"
                                            )} />
                                            <span>T{turn.turn_number}</span>
                                            {turn.media_paths && turn.media_paths.length > 0 && (
                                                <span className="text-[9px]">🖼️</span>
                                            )}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                    </div>
                )
            )}
            {/* Image Thumbnail Gallery Strip */}
            {attachedImages.length > 0 && (
                <div className="p-2.5 rounded-lg border border-slate-200 dark:border-[#40285e] bg-white/70 dark:bg-[#120a1f]/80 backdrop-blur-xs space-y-2">
                    <div className="flex items-center justify-between text-xs font-semibold text-slate-700 dark:text-slate-300 flex-wrap gap-2">
                        <span className="flex items-center gap-1.5">
                            <span>🖼️ Attached Images ({attachedImages.length})</span>
                        </span>
                        <div className="flex items-center gap-1.5">
                            <button
                                type="button"
                                onClick={handleCopyImagesUnified}
                                disabled={isCopyingAllImages}
                                className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-medium bg-purple-50 hover:bg-purple-100 text-purple-700 dark:bg-purple-950/40 dark:hover:bg-purple-900/60 dark:text-purple-300 border border-purple-200 dark:border-purple-800/40 transition-colors shadow-2xs cursor-pointer disabled:opacity-50"
                                title={attachedImages.length > 1 ? `Copy all ${attachedImages.length} images to clipboard as composite PNG (ready for OneNote or chat)` : "Copy image to clipboard as pure PNG (ready to paste directly into OneNote or chat)"}
                            >
                                {(isCopiedAllImages || copiedPureImageIdx !== null) ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3 text-purple-600 dark:text-purple-400" />}
                                <span>{(isCopiedAllImages || copiedPureImageIdx !== null) ? 'Copied' : 'Copy'}</span>
                            </button>
                            <span className="text-[10px] text-slate-400 font-normal ml-1 hidden sm:inline">
                                Click to preview
                            </span>
                        </div>
                    </div>
                    <div className="flex items-center gap-2 overflow-x-auto pb-1 pt-0.5">
                        {attachedImages.map((img, idx) => (
                            <PromptThumbnailCard
                                key={`img-strip-${idx}`}
                                img={img}
                                repoPath={selectedProject?.repo_path}
                                onPreview={handleOpenPreview}
                                onContextMenu={handleImageContextMenu}
                            />
                        ))}
                    </div>
                </div>
            )}

            {/* Preview Mode (Rich Markdown) */}
            {viewMode === PromptViewMode.Preview && (
                <div className="space-y-3">
                    <div
                        onDoubleClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                        className="rounded-xl border border-slate-200 dark:border-purple-500/25 bg-slate-50 dark:bg-[#19191c] p-5 text-xs text-slate-800 dark:text-slate-100 leading-relaxed max-h-[460px] overflow-y-auto cursor-pointer hover:border-purple-400 dark:hover:border-purple-500/50 transition-colors shadow-inner"
                    >
                        <RichMarkdownRenderer
                            content={displayedMarkdown}
                            showAllWords={showAllWords}
                            onToggleExpand={() => setShowAllWords(!showAllWords)}
                            isTruncated={isTruncated}
                            onImageClick={handleOpenPreview}
                            onImageContextMenu={handleImageContextMenu}
                        />
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
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold bg-blue-50 dark:bg-[#171025] text-blue-600 dark:text-cyan-300 border border-blue-200 dark:border-[#40285e] hover:bg-blue-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                                    title={showAllWords ? `Collapse preview to ${effectiveWordLimit} words` : 'Expand full prompt'}
                                >
                                    {showAllWords ? (
                                        <>
                                            <ChevronUp className="w-3.5 h-3.5" />
                                            <span>Show Less ({effectiveWordLimit}w)</span>
                                        </>
                                    ) : (
                                        <>
                                            <ChevronDown className="w-3.5 h-3.5" />
                                            <span>Show All ({totalWords || activeWordCount} words)</span>
                                        </>
                                    )}
                                </button>
                            )}
                        </div>
                    </div>
                </div>
            )}

            {/* Tab 2: Raw Mode (Monospace Clean Text with Explicit <br /> Line Gaps) */}
            {viewMode === PromptViewMode.Raw && (
                <div className="space-y-3">
                    <div className="rounded-xl border border-slate-200 dark:border-purple-500/25 bg-slate-50 dark:bg-[#19191c] p-5 font-mono text-xs text-slate-800 dark:text-slate-100 leading-relaxed whitespace-pre-wrap break-words max-h-[460px] overflow-y-auto select-text shadow-inner">
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
                </div>
            )}

            {/* Tab 3: Edit Mode (Editable Textarea) */}
            {viewMode === PromptViewMode.Edit && (
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
                        className="w-full h-80 rounded-xl border border-slate-200 dark:border-purple-500/25 bg-slate-50 dark:bg-[#19191c] p-4 font-mono text-xs text-slate-900 dark:text-slate-100 leading-relaxed focus:outline-none focus:ring-2 focus:ring-purple-500/40 resize-y"
                        placeholder="Type or modify prompt instruction here..."
                    />
                    <div className="flex items-center justify-between pt-1">
                        <button
                            type="button"
                            onClick={() => setEditedPromptText(activePromptText)}
                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-slate-100 dark:bg-[#171025] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#40285e] hover:bg-slate-200 dark:hover:bg-[#40285e] text-xs font-semibold transition-colors cursor-pointer"
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
                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-emerald-50 dark:bg-[#171025] text-emerald-700 dark:text-emerald-300 border border-emerald-300/60 dark:border-[#40285e] hover:bg-emerald-100 dark:hover:bg-[#40285e] text-xs font-semibold transition-colors cursor-pointer"
                            title="Save prompt changes"
                        >
                            <Check className="w-3.5 h-3.5 text-emerald-500" />
                            <span>Save Changes</span>
                        </button>
                    </div>
                </div>
            )}

            {/* Enqueued Prompts Section (listing pending queued turns with badges directly beneath prompt views) */}
            {enqueuedTurns.length > 0 && (
                <div className="p-3 rounded-xl border border-amber-300/70 dark:border-amber-700/50 bg-gradient-to-r from-amber-50/80 via-white to-amber-50/40 dark:from-[#1f162b] dark:via-[#191124] dark:to-[#1a0e2e] shadow-xs space-y-2 font-poppins">
                    <div className="flex items-center justify-between text-xs font-bold text-amber-800 dark:text-amber-300">
                        <div className="flex items-center gap-2">
                            <Clock className="w-4 h-4 text-amber-600 dark:text-amber-400 animate-pulse" />
                            <span>Enqueued Prompts Queue ({enqueuedTurns.length} pending in FIFO sequence)</span>
                        </div>
                        <span className="text-[10px] font-mono px-2 py-0.5 rounded-md bg-amber-200/80 dark:bg-amber-900/60 text-amber-900 dark:text-amber-200 font-bold border border-amber-300/50 dark:border-amber-700/50">
                            FIFO DISPATCH
                        </span>
                    </div>
                    <div className="space-y-1.5 max-h-48 overflow-y-auto pr-1">
                        {enqueuedTurns.map((turn, qIdx) => (
                            <div
                                key={`enqueued-strip-${turn.step_index || qIdx}`}
                                onClick={() => {
                                    if (turn.prompt_text) {
                                        setActivePromptText(turn.prompt_text);
                                        setActiveTurnPreview(turn.turn_number);
                                        setActionMsg?.(`Loaded Queued Turn T${turn.turn_number} into viewer`);
                                        setTimeout(() => setActionMsg?.(null), 2000);
                                    }
                                }}
                                className="flex items-center justify-between p-2 rounded-lg bg-white/95 dark:bg-[#150d22]/90 border border-amber-200/80 dark:border-amber-800/40 text-xs hover:border-amber-400 dark:hover:border-amber-500 hover:shadow-xs transition-all cursor-pointer group"
                                aria-label={`View queued prompt turn T${turn.turn_number}`}
                            >
                                <div className="flex items-center gap-2 min-w-0 flex-1 mr-2">
                                    <span className="font-mono text-[9px] px-1.5 py-0.5 rounded-sm bg-amber-100 dark:bg-amber-950/80 text-amber-800 dark:text-amber-300 font-bold shrink-0 border border-amber-300/40">
                                        T{turn.turn_number}
                                    </span>
                                    <span className="text-[11px] text-slate-700 dark:text-slate-200 truncate group-hover:text-amber-700 dark:group-hover:text-amber-300 transition-colors">
                                        {turn.preview_200w || turn.prompt_text || `Queued prompt #${turn.turn_number}`}
                                    </span>
                                </div>
                                <div className="flex items-center gap-1.5 shrink-0">
                                    {turn.media_paths && turn.media_paths.length > 0 && (
                                        <span className="text-[9.5px] font-mono px-1.5 py-0.5 rounded-sm bg-sky-100 dark:bg-sky-950/60 text-sky-700 dark:text-sky-300 font-medium">
                                            🖼️ {turn.media_paths.length}
                                        </span>
                                    )}
                                    <span className="text-[9.5px] font-mono font-bold text-amber-700 dark:text-amber-300 px-1.5 py-0.5 rounded-sm bg-amber-100/80 dark:bg-amber-950/70 border border-amber-400/50">
                                        QUEUE #{qIdx + 1}
                                    </span>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Context Menu for Image Actions */}
            <PromptImageContextMenu
                isOpen={contextMenu.isOpen}
                position={contextMenu.position}
                imageSrc={contextMenu.imageSrc}
                altText={contextMenu.altText}
                rawPath={contextMenu.rawPath}
                allImages={attachedImages.map((img) => resolveImageSrc(img.src, selectedProject?.repo_path))}
                onClose={() => setContextMenu((prev) => ({ ...prev, isOpen: false }))}
                onOpenPreview={handleOpenPreview}
            />

            {/* Modal Lightbox for Image Preview with Zoom / Pan */}
            <PromptImagePreviewModal
                isOpen={previewImage !== null}
                imageSrc={previewImage?.src || null}
                altText={previewImage?.alt}
                onClose={() => setPreviewImage(null)}
            />
        </div>
    );
}
