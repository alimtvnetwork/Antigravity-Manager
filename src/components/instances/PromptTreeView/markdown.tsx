import { useState } from 'react';
import type React from 'react';
import { Check, Copy } from 'lucide-react';
import { TruncatedContextCallout } from './TruncatedContextCallout';
import { formatPromptForMarkdown } from './formatters';

function parseInlineMarkdown(text: string, onToggleExpand?: () => void): React.ReactNode[] {
    const rawNodes: React.ReactNode[] = [];
    const tokenRegex = /(!\[(.*?)\]\((.*?)\)|\[(.*?)\]\((.*?)\)|<truncated\s+(\d+)\s+(bytes|lines)>|`([^`]+)`|\*\*([^*]+)\*\*|__([^_]+)__|~~([^~]+)~~|\*([^*]+)\*|_([^_]+)_)/gi;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = tokenRegex.exec(text)) !== null) {
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
                    className="inline-flex items-center gap-1.5 px-2.5 py-0.5 my-0.5 rounded-full text-[10px] font-mono font-medium bg-amber-500/15 text-amber-800 dark:text-amber-200 border border-amber-500/30 shadow-2xs cursor-pointer hover:bg-amber-500/25 transition-all"
                    title={`Omitted ${truncCount} ${truncUnit} from prompt transcript context - Click to inspect/expand`}
                >
                    <span className="text-amber-500 font-bold">⚡</span>
                    <span>Omitted {formattedSize} transcript context · Click to expand</span>
                </span>
            );
        } else if (imgSrc !== undefined) {
            rawNodes.push(
                <img
                    key={`img-${match.index}`}
                    src={imgSrc}
                    alt={imgAlt || ''}
                    className="inline-block max-h-48 max-w-full my-1 rounded-[5px] border border-slate-200 dark:border-[#15334d]"
                    onError={(e) => {
                        (e.target as HTMLElement).style.display = 'none';
                    }}
                />
            );
        } else if (linkUrl !== undefined) {
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
        } else if (inlineCode !== undefined) {
            rawNodes.push(
                <code
                    key={`code-${match.index}`}
                    className="px-1.5 py-0.5 rounded-[4px] bg-slate-100 dark:bg-[#071a27] text-cyan-700 dark:text-cyan-300 font-mono text-[11px] border border-slate-200 dark:border-cyan-900/40"
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

        lastIndex = tokenRegex.lastIndex;
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


interface RichMarkdownRendererProps {
    content: string;
    showAllWords?: boolean;
    onToggleExpand?: () => void;
    isTruncated?: boolean;
}

// Rich Markdown renderer component
export function RichMarkdownRenderer({ content, showAllWords, onToggleExpand, isTruncated }: RichMarkdownRendererProps) {
    const [copiedBlockIndex, setCopiedBlockIndex] = useState<number | null>(null);

    const handleCopyCode = (code: string, index: number) => {
        navigator.clipboard.writeText(code);
        setCopiedBlockIndex(index);
        setTimeout(() => setCopiedBlockIndex(null), 2000);
    };

    if (!content.trim()) {
        return <div className="text-xs text-slate-400 italic">No content to preview.</div>;
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
                        <li key={idx}>{parseInlineMarkdown(item, onToggleExpand)}</li>
                    ))}
                </ul>
            );
        } else {
            elements.push(
                <ol key={`ol-${keyPrefix}`} className="list-decimal ml-5 my-2 space-y-1 text-xs text-slate-700 dark:text-slate-300">
                    {currentList.items.map((item, idx) => (
                        <li key={idx}>{parseInlineMarkdown(item, onToggleExpand)}</li>
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
                className="border-l-3 border-blue-500/80 bg-blue-50/30 dark:bg-blue-950/20 px-3 py-1.5 my-2 rounded-r-[5px] text-xs italic text-slate-700 dark:text-slate-300"
            >
                {parseInlineMarkdown(text, onToggleExpand)}
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
                        className="my-3 overflow-hidden rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-slate-900 text-slate-100"
                    >
                        <div className="flex items-center justify-between border-b border-slate-800 bg-slate-950/70 px-3 py-1.5 text-[11px] font-mono text-slate-400">
                            <span>{lang || 'code'}</span>
                            <button
                                type="button"
                                onClick={() => handleCopyCode(fullCode, blockIdx)}
                                className="flex items-center gap-1 rounded-[5px] px-2 py-0.5 hover:bg-slate-800 text-slate-300 transition-colors cursor-pointer"
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
            elements.push(<hr key={`hr-${i}`} className="my-3 border-slate-200 dark:border-[#15334d]" />);
            continue;
        }

        // Headings
        if (trimmed.startsWith('# ')) {
            elements.push(
                <h1 key={`h1-${i}`} className="text-base font-bold mt-3 mb-1.5 text-slate-900 dark:text-white pb-1 border-b border-slate-200 dark:border-[#15334d]">
                    {parseInlineMarkdown(trimmed.slice(2), onToggleExpand)}
                </h1>
            );
            continue;
        }
        if (trimmed.startsWith('## ')) {
            elements.push(
                <h2 key={`h2-${i}`} className="text-sm font-bold mt-2.5 mb-1 text-slate-900 dark:text-white pb-0.5 border-b border-slate-200/60 dark:border-[#15334d]/60">
                    {parseInlineMarkdown(trimmed.slice(3), onToggleExpand)}
                </h2>
            );
            continue;
        }
        if (trimmed.startsWith('### ')) {
            elements.push(
                <h3 key={`h3-${i}`} className="text-xs font-bold mt-2 mb-1 text-slate-900 dark:text-white">
                    {parseInlineMarkdown(trimmed.slice(4), onToggleExpand)}
                </h3>
            );
            continue;
        }
        if (trimmed.startsWith('#### ')) {
            elements.push(
                <h4 key={`h4-${i}`} className="text-xs font-semibold mt-1.5 mb-0.5 text-slate-800 dark:text-slate-200">
                    {parseInlineMarkdown(trimmed.slice(5), onToggleExpand)}
                </h4>
            );
            continue;
        }

        // Standalone image line: ![alt](url)
        const singleImgMatch = trimmed.match(/^!\[(.*?)\]\((.*?)\)$/);
        if (singleImgMatch) {
            elements.push(
                <div key={`imgline-${i}`} className="my-2">
                    <img
                        src={singleImgMatch[2]}
                        alt={singleImgMatch[1] || ''}
                        className="max-h-60 max-w-full rounded-[5px] border border-slate-200 dark:border-[#15334d] object-contain bg-slate-100 dark:bg-[#071a27]"
                        onError={(e) => {
                            (e.target as HTMLElement).style.display = 'none';
                        }}
                    />
                    {singleImgMatch[1] && <div className="text-[10px] text-slate-400 mt-0.5">{singleImgMatch[1]}</div>}
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
                        {parseInlineMarkdown(cleanLine, onToggleExpand)}
                    </p>
                    <span
                        onClick={(e) => {
                            e.stopPropagation();
                            onToggleExpand();
                        }}
                        title={showAllWords ? 'Click to collapse preview to 120 words' : 'Click to expand full prompt text'}
                        className="cursor-pointer font-bold text-cyan-600 dark:text-cyan-400 hover:underline px-1.5 py-0.5 rounded bg-cyan-500/10 hover:bg-cyan-500/20 transition-colors ml-1.5 inline-block select-none"
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
                        {parseInlineMarkdown(line, onToggleExpand)}
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
            <pre key="unclosed-code" className="my-2 p-3 font-mono text-xs rounded-[5px] bg-slate-900 text-slate-100 overflow-x-auto leading-relaxed select-text">
                {codeBuffer.join('\n')}
            </pre>
        );
    }

    return <div className="space-y-1">{elements}</div>;
}

