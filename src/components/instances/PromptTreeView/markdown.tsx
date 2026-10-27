import { copyToClipboard } from '../../../utils/clipboard';
import { useState, useMemo, useEffect } from 'react';
import type React from 'react';
import { Check, Copy, ExternalLink } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { TruncatedContextCallout } from './TruncatedContextCallout';
import { formatPromptForMarkdown } from './formatters';
import { resolveImageSrc, extractLocalPathFromUri, isLikelyImageUri } from './clipboard';
import { useErrorStore } from '../../../stores/error-store';

/**
 * Opens a local file path or file:// URI directly in the user's IDE or OS default application
 * using the Tauri opener API.
 */
export async function openFileInIde(pathOrUri: string): Promise<void> {
    const raw = pathOrUri.trim().replace(/^<|>$/g, '');
    let clean = raw;
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
        // Keep raw clean
    }

    try {
        const { openPath } = await import('@tauri-apps/plugin-opener');
        await openPath(clean);
    } catch (err: unknown) {
        useErrorStore.getState().trackWarning(err, {
            source: 'markdown.openFileInIde',
            triggerAction: 'open_path',
        });
        throw err;
    }
}

// Hoisted static regular expression with explicit character exclusions to eliminate catastrophic backtracking
const TOKEN_REGEX =
    /(!\[([^\]]*)\]\(([^)]*)\)|\[([^\]]+)\]\(([^)]+)\)|<truncated\s+(\d+)\s+(bytes|lines)>|`([^`]+)`|\*\*([^*]+)\*\*|__([^_]+)__|~~([^~]+)~~|\*([^*]+)\*|_([^_]+)_)/gi;

// Maximum bytes cap for inline regex parsing to protect UI thread and prevent virtual DOM bloat
const MAX_INLINE_BLOCK_BYTES = 50 * 1024; // 50 KB

interface MarkdownImageProps {
    src: string;
    alt?: string;
    className?: string;
    onClick?: (src: string, alt?: string) => void;
    onContextMenu?: (e: React.MouseEvent, src: string, alt?: string) => void;
}

export function MarkdownImage({
    src,
    alt,
    className,
    onClick,
    onContextMenu,
}: MarkdownImageProps) {
    const [currentSrc, setCurrentSrc] = useState<string>(() => resolveImageSrc(src));
    const [hasError, setHasError] = useState(false);

    useEffect(() => {
        let isCancelled = false;
        const raw = src.trim();
        if (raw.startsWith('data:') || raw.startsWith('http://') || raw.startsWith('https://')) {
            setCurrentSrc(raw);
            setHasError(false);
            return;
        }

        const localPath = extractLocalPathFromUri(raw) || raw;
        const isAbsolute = /^[a-zA-Z]:[\\/]/.test(localPath) || localPath.startsWith('/') || localPath.startsWith('\\\\');
        if (isAbsolute && isLikelyImageUri(localPath)) {
            invoke<string>('read_image_as_base64', { path: localPath })
                .then((b64) => {
                    if (!isCancelled && b64 && b64.startsWith('data:image/')) {
                        setCurrentSrc(b64);
                        setHasError(false);
                    }
                })
                .catch((err) => {
                    const msg = err instanceof Error ? err.message : String(err);
                    if (
                        !msg.includes('unsupported_image') &&
                        !msg.includes('not_an_image') &&
                        !msg.includes('file_not_found') &&
                        !msg.includes('path_not_found') &&
                        !msg.includes('invalid_path')
                    ) {
                        useErrorStore.getState().trackWarning(err, {
                            source: 'MarkdownImage.read_image_as_base64',
                            triggerAction: 'read_inline_markdown_image',
                        });
                    }
                });
        }
        return () => {
            isCancelled = true;
        };
    }, [src]);

    const handleImgError = async () => {
        const raw = src.trim();
        const localPath = extractLocalPathFromUri(raw) || raw;
        if (localPath && isLikelyImageUri(localPath) && !currentSrc.startsWith('data:image/')) {
            try {
                const b64 = await invoke<string>('read_image_as_base64', { path: localPath });
                if (b64 && b64.startsWith('data:image/')) {
                    setCurrentSrc(b64);
                    setHasError(false);
                    return;
                }
            } catch (err) {
                const msg = err instanceof Error ? err.message : String(err);
                if (
                    !msg.includes('unsupported_image') &&
                    !msg.includes('not_an_image') &&
                    !msg.includes('file_not_found') &&
                    !msg.includes('path_not_found') &&
                    !msg.includes('invalid_path')
                ) {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'MarkdownImage.handleImgError',
                        triggerAction: 'read_image_as_base64_fallback',
                    });
                }
            }
        }
        setHasError(true);
    };

    if (hasError) {
        return (
            <span
                onClick={() => {
                    const localPath = extractLocalPathFromUri(src) || src;
                    const clean = localPath.replace(/^[/\\]+([a-zA-Z]:)/, '$1');
                    if (/^[a-zA-Z]:[\\/]/.test(clean) || clean.startsWith('/') || clean.startsWith('\\\\')) {
                        void import('@tauri-apps/plugin-opener')
                            .then(({ openPath }) => openPath(clean))
                            .catch((err) => {
                                useErrorStore.getState().trackWarning(err, {
                                    source: 'MarkdownImage.hasError.openPath',
                                    triggerAction: 'open_image_fallback',
                                });
                            });
                    }
                    onClick?.(src, alt);
                }}
                onContextMenu={(e) => onContextMenu?.(e, currentSrc, alt)}
                className="inline-flex items-center gap-1.5 px-2.5 py-1 my-1 rounded-md text-[11px] font-mono bg-slate-100/90 dark:bg-[#120a1f]/90 text-amber-600 dark:text-amber-400 border border-amber-500/30 backdrop-blur-md shadow-2xs cursor-pointer hover:bg-amber-500/10 transition-colors"
                title={`Image: ${src} (Click to open in IDE or inspect)`}
            >
                <span>📷</span>
                <span className="font-semibold">Image (Preview Unavailable)</span>
                {alt && <span className="text-slate-400 dark:text-slate-500 font-normal">· {alt}</span>}
            </span>
        );
    }

    return (
        <span className="relative inline-block group my-1">
            <img
                src={currentSrc}
                alt={alt || ''}
                onClick={() => onClick?.(src, alt)}
                onContextMenu={(e) => {
                    if (onContextMenu) {
                        e.preventDefault();
                        e.stopPropagation();
                        onContextMenu(e, currentSrc, alt);
                    }
                }}
                className={className}
                onError={handleImgError}
            />
            {/* Quick Open in IDE / OS Viewer floating button */}
            <button
                type="button"
                onClick={(e) => {
                    e.stopPropagation();
                    const localPath = extractLocalPathFromUri(src) || src;
                    const clean = localPath.replace(/^[/\\]+([a-zA-Z]:)/, '$1');
                    if (/^[a-zA-Z]:[\\/]/.test(clean) || clean.startsWith('/') || clean.startsWith('\\\\')) {
                        void import('@tauri-apps/plugin-opener')
                            .then(({ openPath }) => openPath(clean))
                            .catch((err) => {
                                useErrorStore.getState().trackWarning(err, {
                                    source: 'MarkdownImage.openInIde',
                                    triggerAction: 'open_image_file',
                                });
                            });
                    } else if (onClick) {
                        onClick(src, alt);
                    }
                }}
                className="absolute top-1 right-1 hidden group-hover:inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-sans font-medium bg-black/80 text-white backdrop-blur-md border border-white/20 shadow-xs cursor-pointer hover:bg-black/95 transition-all select-none"
                title={`Open in IDE / OS Viewer: ${src}`}
            >
                <ExternalLink className="w-2.5 h-2.5" />
                <span>Open</span>
            </button>
        </span>
    );
}

function parseInlineMarkdown(
    text: string,
    onToggleExpand?: () => void,
    onImageClick?: (src: string, alt?: string) => void,
    onImageContextMenu?: (e: React.MouseEvent, src: string, alt?: string) => void
): React.ReactNode[] {
    // Length guard: for large conversation excerpts, avoid regex parsing explosion
    if (text.length > MAX_INLINE_BLOCK_BYTES) {
        return [
            <span key="oversized-text" className="text-xs text-slate-700 dark:text-slate-300">
                {text.substring(0, 2000)}
            </span>,
            <span
                key="oversized-trunc-notice"
                onClick={onToggleExpand}
                className="inline-flex items-center gap-1.5 px-2.5 py-0.5 my-1 rounded-md text-[10px] font-mono font-medium bg-amber-500/15 text-amber-800 dark:text-amber-200 border border-amber-500/30 shadow-2xs cursor-pointer hover:bg-amber-500/25 transition-all"
                title={`Omitted ${text.length - 2000} bytes of context - Click to inspect/expand`}
            >
                <span className="text-amber-500 font-bold">⚡</span>
                <span>Omitted {((text.length - 2000) / 1024).toFixed(1)} KB transcript context · Click to expand</span>
            </span>
        ];
    }

    const rawNodes: React.ReactNode[] = [];
    TOKEN_REGEX.lastIndex = 0;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = TOKEN_REGEX.exec(text)) !== null) {
        if (match.index > lastIndex) {
            rawNodes.push(text.substring(lastIndex, match.index));
        }

        const [full, , imgAlt, imgSrc, linkText, linkUrl, truncCount, truncUnit, inlineCode, boldStar, boldUnder, delText, italicStar, italicUnder] = match;

        if (truncCount !== undefined) {
            const count = parseInt(truncCount, 10);
            const unit = truncUnit.toLowerCase();
            const formattedSize = unit === 'bytes'
                ? (count >= 1024 * 1024 ? `${(count / (1024 * 1024)).toFixed(1)} MB` : `${(count / 1024).toFixed(1)} KB`)
                : `${count} lines`;
            rawNodes.push(
                <span
                    key={`trunc-${match.index}`}
                    onClick={onToggleExpand}
                    className="inline-flex items-center gap-1.5 px-2.5 py-0.5 my-0.5 rounded-md text-[10px] font-mono font-medium bg-amber-500/15 text-amber-800 dark:text-amber-200 border border-amber-500/30 shadow-2xs cursor-pointer hover:bg-amber-500/25 transition-all"
                    title={`Omitted ${truncCount} ${truncUnit} from prompt transcript context - Click to inspect/expand`}
                >
                    <span className="text-amber-500 font-bold">⚡</span>
                    <span>Omitted {formattedSize} transcript context · Click to expand</span>
                </span>
            );
        } else if (imgSrc !== undefined) {
            rawNodes.push(
                <MarkdownImage
                    key={`img-${match.index}`}
                    src={imgSrc}
                    alt={imgAlt || ''}
                    onClick={onImageClick}
                    onContextMenu={onImageContextMenu}
                    className={`inline-block max-h-48 max-w-full my-1 rounded-md border border-slate-200 dark:border-[#40285e] ${onImageClick ? 'cursor-pointer hover:opacity-90 transition-opacity' : ''}`}
                />
            );
        } else if (linkUrl !== undefined) {
            const isFileLink = linkUrl.startsWith('file:') || /^[a-zA-Z]:[\\/]/.test(linkUrl) || (linkUrl.startsWith('/') && !linkUrl.startsWith('//'));
            if (isFileLink) {
                const isImage = Boolean(linkUrl.match(/\.(png|jpe?g|webp|gif|svg)(?:\?.*)?$/i));
                rawNodes.push(
                    <button
                        key={`link-${match.index}`}
                        type="button"
                        onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            const clean = (extractLocalPathFromUri(linkUrl) || linkUrl)
                                .replace(/^file:\/\/\/?/i, '')
                                .replace(/^[/\\]+([a-zA-Z]:)/, '$1');
                            if (isImage && onImageClick) {
                                onImageClick(clean, linkText || 'Image');
                            } else {
                                void import('@tauri-apps/plugin-opener')
                                    .then(({ openPath }) => openPath(clean))
                                    .catch((err) => {
                                        useErrorStore.getState().trackWarning(err, {
                                            source: 'markdown.handleFileLinkClick',
                                            triggerAction: 'open_file_in_ide',
                                        });
                                    });
                            }
                        }}
                        className="inline-flex items-center gap-1 my-0.5 px-2 py-0.5 rounded text-[11px] font-mono text-cyan-600 dark:text-cyan-400 bg-cyan-50/70 dark:bg-cyan-950/40 border border-cyan-200/60 dark:border-cyan-800/40 hover:bg-cyan-100 dark:hover:bg-cyan-900/60 transition-colors cursor-pointer select-text"
                        title={`Click to open in IDE or OS Viewer: ${linkUrl}`}
                    >
                        <span>{isImage ? '🖼️' : '📄'}</span>
                        <span className="underline decoration-cyan-500/40">{linkText || linkUrl}</span>
                    </button>
                );
            } else {
                rawNodes.push(
                    <a
                        key={`link-${match.index}`}
                        href={linkUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-500 dark:text-cyan-400 hover:underline"
                    >
                        {linkText || linkUrl}
                    </a>
                );
            }
        } else if (inlineCode !== undefined) {
            rawNodes.push(
                <code
                    key={`code-${match.index}`}
                    className="px-1.5 py-0.5 rounded-md bg-slate-100 dark:bg-[#090611] text-cyan-700 dark:text-cyan-300 font-mono text-[11px] border border-slate-200 dark:border-cyan-900/40"
                >
                    {inlineCode}
                </code>
            );
        } else if (boldStar !== undefined || boldUnder !== undefined) {
            rawNodes.push(
                <strong key={`bold-${match.index}`} className="font-bold text-slate-900 dark:text-white">
                    {boldStar || boldUnder}
                </strong>
            );
        } else if (delText !== undefined) {
            rawNodes.push(
                <del key={`del-${match.index}`} className="line-through opacity-70">
                    {delText}
                </del>
            );
        } else if (italicStar !== undefined || italicUnder !== undefined) {
            rawNodes.push(
                <em key={`italic-${match.index}`} className="italic">
                    {italicStar || italicUnder}
                </em>
            );
        } else {
            rawNodes.push(full);
        }

        lastIndex = TOKEN_REGEX.lastIndex;
    }

    if (lastIndex < text.length) {
        rawNodes.push(text.substring(lastIndex));
    }

    // Convert \n in text nodes to explicit <br className="my-1.5 block select-none" /> elements
    const nodes: React.ReactNode[] = [];
    rawNodes.forEach((node, nodeIdx) => {
        if (typeof node === 'string') {
            if (node.includes('\n')) {
                const parts = node.split('\n');
                parts.forEach((part, pIdx) => {
                    if (part) nodes.push(part);
                    if (pIdx < parts.length - 1) {
                        nodes.push(<br key={`br-${nodeIdx}-${pIdx}`} className="my-1.5 block select-none" />);
                    }
                });
            } else {
                nodes.push(node);
            }
        } else {
            nodes.push(node);
        }
    });

    return nodes.length > 0 ? nodes : [text];
}

export interface RichMarkdownRendererProps {
    content: string;
    showAllWords?: boolean;
    onToggleExpand?: () => void;
    isTruncated?: boolean;
    onImageClick?: (src: string, alt?: string) => void;
    onImageContextMenu?: (e: React.MouseEvent, src: string, alt?: string) => void;
}

// Rich Markdown renderer component
export function RichMarkdownRenderer({
    content,
    showAllWords,
    onToggleExpand,
    isTruncated,
    onImageClick,
    onImageContextMenu,
}: RichMarkdownRendererProps) {
    const [copiedBlockIndex, setCopiedBlockIndex] = useState<number | null>(null);

    const handleCopyCode = (code: string, index: number) => {
        copyToClipboard(code);
        setCopiedBlockIndex(index);
        setTimeout(() => setCopiedBlockIndex(null), 2000);
    };

    const renderedContent = useMemo(() => {
        if (!content.trim()) {
            return <div className="text-xs text-slate-400 italic">No content to preview.</div>;
        }

        // For massive content exceeding 50 KB without showAllWords, render preview callout
        if (content.length > MAX_INLINE_BLOCK_BYTES && !showAllWords) {
            return (
                <div className="space-y-2">
                    <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words">
                        {parseInlineMarkdown(content.slice(0, 2000), onToggleExpand, onImageClick, onImageContextMenu)}
                    </p>
                    <TruncatedContextCallout
                        omittedBytes={content.length - 2000}
                        onExpandFull={onToggleExpand}
                    />
                </div>
            );
        }

        const formattedContent = formatPromptForMarkdown(content);
        const lines = formattedContent.split('\n');
        const elements: React.ReactNode[] = [];
    let inCodeBlock = false;
    let codeLanguage = '';
    let codeBuffer: string[] = [];
    let listBuffer: { type: 'ul' | 'ol'; items: string[] } | null = null;
    let blockquoteBuffer: string[] = [];

    const flushList = (keyPrefix: number) => {
        if (!listBuffer) return;
        const currentList = listBuffer;
        listBuffer = null;
        if (currentList.type === 'ul') {
            elements.push(
                <ul key={`ul-${keyPrefix}`} className="list-disc ml-5 my-2 space-y-1 text-xs text-slate-700 dark:text-slate-300">
                    {currentList.items.map((item, idx) => (
                        <li key={idx}>{parseInlineMarkdown(item, onToggleExpand, onImageClick, onImageContextMenu)}</li>
                    ))}
                </ul>
            );
        } else {
            elements.push(
                <ol key={`ol-${keyPrefix}`} className="list-decimal ml-5 my-2 space-y-1 text-xs text-slate-700 dark:text-slate-300">
                    {currentList.items.map((item, idx) => (
                        <li key={idx}>{parseInlineMarkdown(item, onToggleExpand, onImageClick, onImageContextMenu)}</li>
                    ))}
                </ol>
            );
        }
    };

    const flushBlockquote = (keyPrefix: number) => {
        if (blockquoteBuffer.length === 0) return;
        const text = blockquoteBuffer.join('\n');
        blockquoteBuffer = [];
        elements.push(
            <blockquote
                key={`quote-${keyPrefix}`}
                className="border-l-3 border-blue-500/80 bg-blue-50/30 dark:bg-blue-950/20 px-3 py-1.5 my-2 rounded-r-md text-xs italic text-slate-700 dark:text-slate-300"
            >
                {parseInlineMarkdown(text, onToggleExpand, onImageClick, onImageContextMenu)}
            </blockquote>
        );
    };

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const trimmed = line.trim();

        // Check code blocks ```
        if (trimmed.startsWith('```')) {
            if (inCodeBlock) {
                // End code block
                const fullCode = codeBuffer.join('\n');
                const blockIdx = i;
                const lang = codeLanguage;
                elements.push(
                    <div
                        key={`codeblock-${blockIdx}`}
                        className="my-3 overflow-hidden rounded-md border border-slate-200 dark:border-[#40285e] bg-slate-900 text-slate-100"
                    >
                        <div className="flex items-center justify-between border-b border-slate-800 bg-slate-950/70 px-3 py-1.5 text-[11px] font-mono text-slate-400">
                            <span>{lang || 'code'}</span>
                            <button
                                type="button"
                                onClick={() => handleCopyCode(fullCode, blockIdx)}
                                className="flex items-center gap-1 rounded-md px-2 py-0.5 hover:bg-slate-800 text-slate-300 transition-colors cursor-pointer"
                                title="Copy code"
                            >
                                {copiedBlockIndex === blockIdx ? (
                                    <>
                                        <Check className="w-3 h-3 text-emerald-400" />
                                        <span className="text-[10px] text-emerald-400">Copied</span>
                                    </>
                                ) : (
                                    <>
                                        <Copy className="w-3 h-3" />
                                        <span className="text-[10px]">Copy</span>
                                    </>
                                )}
                            </button>
                        </div>
                        <pre className="p-3 font-mono text-xs overflow-x-auto leading-relaxed select-text">
                            {fullCode}
                        </pre>
                    </div>
                );
                codeBuffer = [];
                codeLanguage = '';
                inCodeBlock = false;
            } else {
                flushList(i);
                flushBlockquote(i);
                inCodeBlock = true;
                codeLanguage = trimmed.slice(3).trim();
                codeBuffer = [];
            }
            continue;
        }

        if (inCodeBlock) {
            codeBuffer.push(line);
            continue;
        }

        // Check blockquote >
        if (trimmed.startsWith('>')) {
            flushList(i);
            blockquoteBuffer.push(trimmed.slice(1).trim());
            continue;
        } else {
            flushBlockquote(i);
        }

        // Check bullet lists
        const ulMatch = line.match(/^(\s*)[-*+]\s+(.*)$/);
        if (ulMatch) {
            if (listBuffer && listBuffer.type !== 'ul') {
                flushList(i);
            }
            if (!listBuffer) {
                listBuffer = { type: 'ul', items: [] };
            }
            listBuffer.items.push(ulMatch[2]);
            continue;
        }

        // Check numbered lists
        const olMatch = line.match(/^(\s*)\d+\.\s+(.*)$/);
        if (olMatch) {
            if (listBuffer && listBuffer.type !== 'ol') {
                flushList(i);
            }
            if (!listBuffer) {
                listBuffer = { type: 'ol', items: [] };
            }
            listBuffer.items.push(olMatch[2]);
            continue;
        }

        flushList(i);

        if (!trimmed) {
            elements.push(<br key={`br-${i}`} className="my-1.5 block select-none" />);
            continue;
        }

        // Truncated transcript omission banner
        const truncLineMatch = trimmed.match(/^<truncated\s+(\d+)\s+(bytes|lines)>/i);
        if (truncLineMatch) {
            const count = parseInt(truncLineMatch[1], 10);
            const unit = truncLineMatch[2].toLowerCase();
            elements.push(
                <TruncatedContextCallout
                    key={`trunc-block-${i}`}
                    omittedBytes={unit === 'bytes' ? count : undefined}
                    omittedLines={unit === 'lines' ? count : undefined}
                    onExpandFull={onToggleExpand}
                />
            );
            continue;
        }

        // Horizontal rules
        if (/^(---|\*\*\*|___)$/.test(trimmed)) {
            elements.push(<hr key={`hr-${i}`} className="my-3 border-slate-200 dark:border-[#40285e]" />);
            continue;
        }

        // Headings
        if (trimmed.startsWith('# ')) {
            elements.push(
                <h1 key={`h1-${i}`} className="text-base font-bold mt-3 mb-1.5 text-slate-900 dark:text-white pb-1 border-b border-slate-200 dark:border-[#40285e]">
                    {parseInlineMarkdown(trimmed.slice(2), onToggleExpand, onImageClick, onImageContextMenu)}
                </h1>
            );
            continue;
        }
        if (trimmed.startsWith('## ')) {
            elements.push(
                <h2 key={`h2-${i}`} className="text-sm font-bold mt-2.5 mb-1 text-slate-900 dark:text-white pb-0.5 border-b border-slate-200/60 dark:border-[#40285e]/60">
                    {parseInlineMarkdown(trimmed.slice(3), onToggleExpand, onImageClick, onImageContextMenu)}
                </h2>
            );
            continue;
        }
        if (trimmed.startsWith('### ')) {
            elements.push(
                <h3 key={`h3-${i}`} className="text-xs font-bold mt-2 mb-1 text-slate-900 dark:text-white">
                    {parseInlineMarkdown(trimmed.slice(4), onToggleExpand, onImageClick, onImageContextMenu)}
                </h3>
            );
            continue;
        }
        if (trimmed.startsWith('#### ')) {
            elements.push(
                <h4 key={`h4-${i}`} className="text-xs font-semibold mt-1.5 mb-0.5 text-slate-800 dark:text-slate-200">
                    {parseInlineMarkdown(trimmed.slice(5), onToggleExpand, onImageClick, onImageContextMenu)}
                </h4>
            );
            continue;
        }

        // Standalone image line: ![alt](url)
        const singleImgMatch = trimmed.match(/^!\[(.*?)\]\((.*?)\)$/);
        if (singleImgMatch) {
            const imgSrc = singleImgMatch[2];
            const imgAlt = singleImgMatch[1] || '';
            const isLocal = imgSrc.startsWith('file:') || /^[a-zA-Z]:[\\/]/.test(imgSrc);
            elements.push(
                <div key={`imgline-${i}`} className="my-2 group relative inline-block max-w-full">
                    <MarkdownImage
                        src={imgSrc}
                        alt={imgAlt}
                        onClick={onImageClick}
                        onContextMenu={onImageContextMenu}
                        className={`max-h-60 max-w-full rounded-md border border-slate-200 dark:border-[#40285e] object-contain bg-slate-100 dark:bg-[#090611] ${onImageClick ? 'cursor-pointer hover:opacity-90 transition-opacity' : ''}`}
                    />
                    <div className="flex items-center justify-between gap-2 mt-0.5">
                        {imgAlt && <div className="text-[10px] text-slate-400">{imgAlt}</div>}
                        {isLocal && (
                            <button
                                type="button"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    void openFileInIde(imgSrc).catch((err: unknown) => {
                                        useErrorStore.getState().trackWarning(err, {
                                            source: 'RichMarkdownRenderer.openInIde',
                                            triggerAction: 'open_standalone_image',
                                        });
                                    });
                                }}
                                className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono text-cyan-600 dark:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-950/40 border border-cyan-300/40 dark:border-cyan-800/40 transition-colors cursor-pointer"
                                title="Open in IDE / OS Viewer"
                            >
                                <ExternalLink className="w-3 h-3" />
                                <span>Open in IDE</span>
                            </button>
                        )}
                    </div>
                </div>
            );
            continue;
        }

        // Standard paragraph with explicit <br /> line gap
        const isTrailingEllipsis = trimmed.endsWith('...') || trimmed.endsWith('…');
        const isEndOfExpandedText = Boolean(showAllWords && isTruncated && i === lines.length - 1);

        if ((isTrailingEllipsis || isEndOfExpandedText) && onToggleExpand) {
            const cleanLine = isTrailingEllipsis ? line.replace(/(\.{3}|…)\s*$/, '') : line;
            elements.push(
                <div key={`p-wrap-${i}`} className="my-1.5 leading-relaxed">
                    <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words inline">
                        {parseInlineMarkdown(cleanLine, onToggleExpand, onImageClick, onImageContextMenu)}
                    </p>
                    <span
                        onClick={(e) => {
                            e.stopPropagation();
                            onToggleExpand();
                        }}
                        title={showAllWords ? 'Click to collapse preview to 120 words' : 'Click to expand full prompt text'}
                        className="cursor-pointer font-bold text-cyan-600 dark:text-cyan-400 hover:underline px-1.5 py-0.5 rounded-md bg-cyan-500/10 hover:bg-cyan-500/20 transition-colors ml-1.5 inline-block select-none"
                    >
                        {showAllWords ? 'Collapse full text' : 'Expand full text'}
                    </span>
                    <br className="my-1.5 block select-none" />
                </div>
            );
        } else {
            elements.push(
                <div key={`p-wrap-${i}`} className="my-1.5 leading-relaxed">
                    <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words">
                        {parseInlineMarkdown(line, onToggleExpand, onImageClick, onImageContextMenu)}
                    </p>
                    <br className="my-1.5 block select-none" />
                </div>
            );
        }
    }

    flushList(lines.length);
    flushBlockquote(lines.length);

    if (inCodeBlock && codeBuffer.length > 0) {
        elements.push(
            <pre key="unclosed-code" className="my-2 p-3 font-mono text-xs rounded-md bg-slate-900 text-slate-100 overflow-x-auto leading-relaxed select-text">
                {codeBuffer.join('\n')}
            </pre>
        );
    }

        return <div className="space-y-1">{elements}</div>;
    }, [content, showAllWords, onToggleExpand, isTruncated, copiedBlockIndex]);

    return renderedContent;
}

