import { useState, useMemo } from 'react';
import type React from 'react';
import { Check, Copy, ChevronDown, ChevronRight, Brain, Code } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { TruncatedContextCallout } from './TruncatedContextCallout';

export interface PromptMarkdownPreviewProps {
    content: string;
    className?: string;
    showAllWords?: boolean;
    onToggleExpand?: () => void;
    isTruncated?: boolean;
    maxInlineBytes?: number;
}

// Token regex matching bold, italics, code, links, images, truncation markers, and thinking tags
const TOKEN_REGEX =
    /(!\[([^\]]*)\]\(([^)]*)\)|\[([^\]]+)\]\(([^)]+)\)|<truncated\s+(\d+)\s+(bytes|lines)>|`([^`]+)`|\*\*([^*]+)\*\*|__([^_]+)__|~~([^~]+)~~|\*([^*]+)\*|_([^_]+)_)/gi;

const MAX_INLINE_BLOCK_BYTES = 50 * 1024; // 50 KB

/**
 * Fenced Code Block with charming language tag and 1-click copy button.
 */
function CharmingCodeBlock({
    language,
    code,
}: {
    language?: string;
    code: string;
}) {
    const [isCopied, setIsCopied] = useState(false);

    const handleCopy = async () => {
        try {
            await copyToClipboard(code);
            setIsCopied(true);
            setTimeout(() => setIsCopied(false), 2000);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'CharmingCodeBlock.copy',
                triggerAction: 'copy_code',
            });
        }
    };

    const displayLang = language?.trim() || 'text';

    return (
        <div className="my-3 overflow-hidden rounded-xl border border-slate-200 dark:border-purple-500/30 bg-slate-900 shadow-sm text-slate-100">
            {/* Code Header Bar */}
            <div className="flex items-center justify-between px-3.5 py-1.5 bg-slate-950/80 border-b border-slate-800 text-[10px] font-mono">
                <div className="flex items-center gap-1.5 text-purple-400">
                    <Code className="w-3 h-3" />
                    <span className="uppercase tracking-wider font-semibold">{displayLang}</span>
                </div>
                <button
                    type="button"
                    onClick={handleCopy}
                    className="flex items-center gap-1 px-2 py-0.5 rounded text-slate-400 hover:text-white hover:bg-slate-800 transition-colors cursor-pointer"
                    title="Copy code"
                >
                    {isCopied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                    <span>{isCopied ? 'Copied' : 'Copy'}</span>
                </button>
            </div>
            {/* Code Body */}
            <pre className="p-3.5 overflow-x-auto font-mono text-[11px] leading-relaxed select-text text-slate-200">
                <code>{code}</code>
            </pre>
        </div>
    );
}

/**
 * Collapsible Reasoning Card for <thinking> blocks in translucent lavender glass.
 */
function LavenderThinkingCard({ content }: { content: string }) {
    const [isExpanded, setIsExpanded] = useState(false);
    const wordCount = useMemo(() => content.trim().split(/\s+/).filter(Boolean).length, [content]);

    return (
        <div className="my-3 overflow-hidden rounded-xl border border-purple-500/30 bg-purple-950/20 backdrop-blur-md shadow-xs">
            <button
                type="button"
                onClick={() => setIsExpanded((prev) => !prev)}
                className="w-full flex items-center justify-between px-3.5 py-2 text-left bg-purple-500/5 hover:bg-purple-500/10 transition-colors cursor-pointer"
            >
                <div className="flex items-center gap-2 text-xs font-semibold text-purple-700 dark:text-purple-300">
                    <Brain className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                    <span>Agent Reasoning Trace</span>
                    <span className="text-[10px] font-mono font-normal opacity-75">
                        ({wordCount} words)
                    </span>
                </div>
                <div className="flex items-center gap-1 text-[11px] text-purple-500">
                    <span>{isExpanded ? 'Collapse' : 'Expand'}</span>
                    {isExpanded ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
                </div>
            </button>

            {isExpanded && (
                <div className="p-3.5 border-t border-purple-500/20 text-xs text-purple-900 dark:text-purple-200 leading-relaxed font-sans whitespace-pre-wrap select-text max-h-96 overflow-y-auto">
                    {content}
                </div>
            )}
        </div>
    );
}

/**
 * Parses inline markdown tokens (links, images, bold, italics, code, omission banners).
 */
function parseInlineMarkdown(text: string, onToggleExpand?: () => void): React.ReactNode[] {
    if (text.length > MAX_INLINE_BLOCK_BYTES) {
        return [
            <span key="oversized-text" className="text-xs text-slate-700 dark:text-slate-300">
                {text.substring(0, 2000)}
            </span>,
            <span
                key="oversized-trunc-notice"
                onClick={onToggleExpand}
                className="inline-flex items-center gap-1.5 px-2.5 py-0.5 my-1 rounded-full text-[10px] font-mono font-medium bg-amber-500/15 text-amber-800 dark:text-amber-200 border border-amber-500/30 shadow-2xs cursor-pointer hover:bg-amber-500/25 transition-all"
                title={`Omitted ${text.length - 2000} bytes of context - Click to inspect/expand`}
            >
                <span className="text-amber-500 font-bold">⚡</span>
                <span>Omitted {((text.length - 2000) / 1024).toFixed(1)} KB transcript context · Click to expand</span>
            </span>,
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

        const [
            full,
            ,
            imgAlt,
            imgSrc,
            linkText,
            linkUrl,
            truncCount,
            truncUnit,
            inlineCode,
            boldStar,
            boldUnder,
            delText,
            italicStar,
            italicUnder,
        ] = match;

        if (truncCount !== undefined) {
            const count = parseInt(truncCount, 10);
            const unit = truncUnit.toLowerCase();
            const formattedSize =
                unit === 'bytes'
                    ? count >= 1024 * 1024
                        ? `${(count / (1024 * 1024)).toFixed(1)} MB`
                        : `${(count / 1024).toFixed(1)} KB`
                    : `${count} lines`;
            rawNodes.push(
                <span
                    key={`trunc-${match.index}`}
                    onClick={onToggleExpand}
                    className="inline-flex items-center gap-1.5 px-2.5 py-0.5 my-0.5 rounded-full text-[10px] font-mono font-medium bg-amber-500/15 text-amber-800 dark:text-amber-200 border border-amber-500/30 shadow-2xs cursor-pointer hover:bg-amber-500/25 transition-all"
                    title={`Omitted ${truncCount} ${truncUnit} - Click to inspect/expand`}
                >
                    <span className="text-amber-500 font-bold">⚡</span>
                    <span>Omitted {formattedSize} context · Click to expand</span>
                </span>
            );
        } else if (imgSrc !== undefined) {
            rawNodes.push(
                <img
                    key={`img-${match.index}`}
                    src={imgSrc}
                    alt={imgAlt || ''}
                    className="inline-block max-h-48 max-w-full my-1 rounded-lg border border-slate-200 dark:border-[#40285e]"
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
                    className="text-purple-600 dark:text-purple-400 font-medium hover:underline"
                >
                    {linkText || linkUrl}
                </a>
            );
        } else if (inlineCode !== undefined) {
            rawNodes.push(
                <code
                    key={`code-${match.index}`}
                    className="px-1.5 py-0.5 rounded-[4px] bg-slate-100 dark:bg-purple-950/40 text-purple-700 dark:text-purple-300 font-mono text-[11px] border border-slate-200 dark:border-purple-500/30"
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

    // Convert newlines in text nodes
    const nodes: React.ReactNode[] = [];
    rawNodes.forEach((node, nodeIdx) => {
        if (typeof node === 'string') {
            if (node.includes('\n')) {
                const parts = node.split('\n');
                parts.forEach((part, pIdx) => {
                    if (part) nodes.push(part);
                    if (pIdx < parts.length - 1) {
                        nodes.push(<br key={`br-${nodeIdx}-${pIdx}`} className="my-1 block select-none" />);
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

/**
 * Charming Markdown Preview with enhanced typography, code copy blocks,
 * blockquotes, tables, and collapsible reasoning cards.
 */
export function PromptMarkdownPreview({
    content,
    className,
    showAllWords = true,
    onToggleExpand,
    isTruncated = false,
}: PromptMarkdownPreviewProps) {
    const renderedNodes = useMemo(() => {
        if (!content) {
            return <span className="text-slate-400 italic">No content</span>;
        }

        const lines = content.split('\n');
        const elements: React.ReactNode[] = [];
        let inCodeBlock = false;
        let codeLang = '';
        let codeLines: string[] = [];
        let inThinkingBlock = false;
        let thinkingLines: string[] = [];
        let inTable = false;
        let tableRows: string[][] = [];

        const flushTable = (keyIdx: number) => {
            if (!inTable || tableRows.length === 0) return;
            const header = tableRows[0];
            const body = tableRows.slice(1).filter((r) => !r.every((cell) => cell.match(/^:?-+:?$/)));

            elements.push(
                <div key={`table-${keyIdx}`} className="my-3 overflow-x-auto rounded-xl border border-slate-200 dark:border-[#40285e] shadow-2xs">
                    <table className="w-full text-left text-xs border-collapse">
                        <thead>
                            <tr className="bg-slate-100/90 dark:bg-purple-950/40 border-b border-slate-200 dark:border-[#40285e]">
                                {header.map((col, cIdx) => (
                                    <th key={cIdx} className="px-3 py-2 font-bold text-slate-800 dark:text-slate-200">
                                        {col.trim()}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-200/60 dark:divide-[#40285e]/60">
                            {body.map((row, rIdx) => (
                                <tr key={rIdx} className={cn('hover:bg-slate-50 dark:hover:bg-purple-950/20', rIdx % 2 === 1 && 'bg-slate-50/40 dark:bg-[#120a1f]/40')}>
                                    {row.map((cell, cIdx) => (
                                        <td key={cIdx} className="px-3 py-1.5 text-slate-700 dark:text-slate-300">
                                            {cell.trim()}
                                        </td>
                                    ))}
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            );
            tableRows = [];
            inTable = false;
        };

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];

            // Thinking tag check
            if (line.trim().startsWith('<thinking>') || line.trim().startsWith('<thought>')) {
                inThinkingBlock = true;
                thinkingLines = [line.replace(/^<(thinking|thought)>/, '')];
                continue;
            }
            if (inThinkingBlock) {
                if (line.includes('</thinking>') || line.includes('</thought>')) {
                    thinkingLines.push(line.replace(/<\/(thinking|thought)>.*$/, ''));
                    elements.push(
                        <LavenderThinkingCard key={`think-${i}`} content={thinkingLines.join('\n').trim()} />
                    );
                    inThinkingBlock = false;
                    thinkingLines = [];
                } else {
                    thinkingLines.push(line);
                }
                continue;
            }

            // Fenced code blocks
            if (line.trim().startsWith('```')) {
                flushTable(i);
                if (inCodeBlock) {
                    elements.push(
                        <CharmingCodeBlock key={`code-${i}`} language={codeLang} code={codeLines.join('\n')} />
                    );
                    inCodeBlock = false;
                    codeLang = '';
                    codeLines = [];
                } else {
                    inCodeBlock = true;
                    codeLang = line.trim().slice(3).trim();
                    codeLines = [];
                }
                continue;
            }

            if (inCodeBlock) {
                codeLines.push(line);
                continue;
            }

            // Table rows (starts and ends with |)
            if (line.trim().startsWith('|') && line.trim().endsWith('|')) {
                const cells = line
                    .trim()
                    .slice(1, -1)
                    .split('|');
                tableRows.push(cells);
                inTable = true;
                continue;
            } else if (inTable) {
                flushTable(i);
            }

            // Truncation callout
            const truncMatch = line.match(/^<truncated\s+(\d+)\s+(bytes|lines)>$/i);
            if (truncMatch) {
                elements.push(
                    <TruncatedContextCallout
                        key={`trunc-callout-${i}`}
                        omittedCount={parseInt(truncMatch[1], 10)}
                        omittedUnit={truncMatch[2].toLowerCase() as 'bytes' | 'lines'}
                        onInspect={onToggleExpand}
                    />
                );
                continue;
            }

            // Headings
            if (line.startsWith('# ')) {
                elements.push(
                    <h1 key={`h1-${i}`} className="text-base font-bold text-slate-900 dark:text-white mt-4 mb-2 pb-1 border-b border-slate-200 dark:border-purple-500/30">
                        {parseInlineMarkdown(line.slice(2), onToggleExpand)}
                    </h1>
                );
                continue;
            }
            if (line.startsWith('## ')) {
                elements.push(
                    <h2 key={`h2-${i}`} className="text-sm font-bold text-slate-900 dark:text-white mt-3.5 mb-1.5 pb-0.5 border-b border-slate-200/70 dark:border-purple-500/20">
                        {parseInlineMarkdown(line.slice(3), onToggleExpand)}
                    </h2>
                );
                continue;
            }
            if (line.startsWith('### ')) {
                elements.push(
                    <h3 key={`h3-${i}`} className="text-xs font-bold text-purple-700 dark:text-purple-300 mt-2.5 mb-1">
                        {parseInlineMarkdown(line.slice(4), onToggleExpand)}
                    </h3>
                );
                continue;
            }
            if (line.startsWith('#### ')) {
                elements.push(
                    <h4 key={`h4-${i}`} className="text-xs font-semibold text-slate-700 dark:text-slate-300 mt-2 mb-0.5">
                        {parseInlineMarkdown(line.slice(5), onToggleExpand)}
                    </h4>
                );
                continue;
            }

            // Blockquotes
            if (line.startsWith('> ')) {
                elements.push(
                    <blockquote
                        key={`quote-${i}`}
                        className="my-1.5 border-l-2 border-purple-500/60 bg-purple-500/5 px-3 py-1.5 rounded-r text-xs text-slate-700 dark:text-slate-300 italic"
                    >
                        {parseInlineMarkdown(line.slice(2), onToggleExpand)}
                    </blockquote>
                );
                continue;
            }

            // Unordered list items
            if (line.trim().startsWith('- ') || line.trim().startsWith('* ')) {
                elements.push(
                    <div key={`li-${i}`} className="flex items-start gap-2 my-0.5 text-xs text-slate-800 dark:text-slate-200">
                        <span className="text-purple-500 font-bold shrink-0 leading-relaxed">•</span>
                        <div className="flex-1 leading-relaxed">
                            {parseInlineMarkdown(line.trim().slice(2), onToggleExpand)}
                        </div>
                    </div>
                );
                continue;
            }

            // Blank line
            if (!line.trim()) {
                elements.push(<div key={`blank-${i}`} className="h-2" />);
                continue;
            }

            // Standard paragraph
            elements.push(
                <p key={`p-${i}`} className="text-xs text-slate-800 dark:text-slate-200 leading-relaxed my-1">
                    {parseInlineMarkdown(line, onToggleExpand)}
                </p>
            );
        }

        // Clean up unclosed blocks
        if (inCodeBlock) {
            elements.push(
                <CharmingCodeBlock key="code-final" language={codeLang} code={codeLines.join('\n')} />
            );
        }
        if (inThinkingBlock) {
            elements.push(
                <LavenderThinkingCard key="think-final" content={thinkingLines.join('\n').trim()} />
            );
        }
        flushTable(lines.length);

        return elements;
    }, [content, onToggleExpand]);

    return (
        <div className={cn('prose prose-xs dark:prose-invert max-w-none font-sans', className)}>
            {renderedNodes}
            {isTruncated && !showAllWords && onToggleExpand && (
                <div className="mt-2 pt-2 border-t border-slate-200 dark:border-purple-500/30">
                    <button
                        type="button"
                        onClick={onToggleExpand}
                        className="text-xs font-semibold text-purple-600 dark:text-purple-400 hover:underline cursor-pointer"
                    >
                        ⚡ Expand complete transcript content...
                    </button>
                </div>
            )}
        </div>
    );
}
