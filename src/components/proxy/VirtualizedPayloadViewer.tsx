import React, { useState, useRef, useEffect, useMemo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
    Search,
    ChevronUp,
    ChevronDown,
    Copy,
    CheckCircle,
    X,
    WrapText,
    CaseSensitive,
} from 'lucide-react';

export interface VirtualizedPayloadViewerProps {
    cardId: string;
    title: string;
    badge: string;
    badgeStyle: string;
    rawPayload?: string;
    concisePayload?: string;
    headersJson?: string;
    viewMode: 'concise' | 'full';
    emptyPlaceholder: string;
    onCopy: (content: string) => Promise<void>;
    isCopied: boolean;
    duration?: number;
    timingNode?: React.ReactNode;
}

interface SearchMatch {
    lineIndex: number;
    colStart: number;
    length: number;
    globalIndex: number;
}

interface LineToken {
    text: string;
    type: 'key' | 'string' | 'number' | 'boolean' | 'null' | 'punct' | 'plain';
    start: number;
    end: number;
}

const getTokenClass = (type: string) => {
    switch (type) {
        case 'key':
            return 'text-sky-600 dark:text-sky-400 font-medium';
        case 'string':
            return 'text-emerald-700 dark:text-emerald-300';
        case 'boolean':
            return 'text-purple-600 dark:text-purple-400 font-semibold';
        case 'null':
            return 'text-rose-500 dark:text-rose-400 font-semibold italic';
        case 'number':
            return 'text-amber-600 dark:text-amber-300 font-semibold';
        case 'punct':
            return 'text-gray-400 dark:text-gray-500';
        default:
            return 'text-gray-700 dark:text-gray-300';
    }
};

// Global token cache pool: avoids re-tokenizing rendered lines during scrolling, reducing CPU usage
const tokenCache = new Map<string, LineToken[]>();

// Lossless single-line JSON tokenizer with O(1) character dispatch
const tokenizeJsonLine = (line: string): LineToken[] => {
    if (!line) return [];

    const cached = tokenCache.get(line);
    if (cached) return cached;

    const tokenRegex = /("(?:\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(?:true|false|null)\b|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|[{}[\],:]|\s+|[^"{}[\],:\s]+|[\s\S])/g;

    const tokens: LineToken[] = [];
    let match: RegExpExecArray | null;

    while ((match = tokenRegex.exec(line)) !== null) {
        const text = match[0];
        const start = match.index;
        const end = start + text.length;
        let type: LineToken['type'] = 'plain';

        const firstChar = text.charCodeAt(0);
        if (firstChar === 34) {
            // String literals: "..." or "..." :
            type = match[2] ? 'key' : 'string';
        } else if (text === 'true' || text === 'false') {
            type = 'boolean';
        } else if (text === 'null') {
            type = 'null';
        } else if ((firstChar >= 48 && firstChar <= 57) || firstChar === 45) {
            // Numeric values: begins with 0-9 or -
            type = 'number';
        } else if (text.length === 1 && (firstChar === 123 || firstChar === 125 || firstChar === 91 || firstChar === 93 || firstChar === 44 || firstChar === 58)) {
            // Punctuation: { } [ ] , :
            type = 'punct';
        }

        tokens.push({ text, type, start, end });
    }

    if (tokenCache.size > 10000) {
        tokenCache.clear();
    }
    tokenCache.set(line, tokens);

    return tokens;
};

// Calculate visual character width in monospace font (ASCII = 1, CJK/full-width = 2)
const getVisualCharCount = (str: string): number => {
    let count = 0;
    const len = str.length;
    for (let i = 0; i < len; i++) {
        const code = str.charCodeAt(i);
        if (code > 255) {
            count += 2;
        } else if (code === 9) {
            count += 2;
        } else {
            count += 1;
        }
    }
    return count;
};

// Render single-line text: sliced based on lossless tokens and search highlight intervals
const renderLineContent = (
    line: string,
    lineIndex: number,
    lineMatches: SearchMatch[] | undefined,
    currentMatchIndex: number,
    cardId: string
) => {
    if (!line) return <span>&nbsp;</span>;

    const tokens = tokenizeJsonLine(line);

    if (!lineMatches || lineMatches.length === 0) {
        return tokens.map((t, idx) => (
            <span key={`l-${lineIndex}-t-${idx}`} className={getTokenClass(t.type)}>
                {t.text}
            </span>
        ));
    }

    // When search matches exist on line: slice tokens into highlight segments preserving quotes
    const elements: React.ReactNode[] = [];

    for (let tIdx = 0; tIdx < tokens.length; tIdx++) {
        const token = tokens[tIdx];
        const tokenKey = `l-${lineIndex}-tok-${tIdx}`;
        const tokenClass = getTokenClass(token.type);

        // Find search matches overlapping with current token
        const overlapping = lineMatches.filter(
            (m) => m.colStart < token.end && m.colStart + m.length > token.start
        );

        if (overlapping.length === 0) {
            elements.push(
                <span key={tokenKey} className={tokenClass}>
                    {token.text}
                </span>
            );
            continue;
        }

        // Slice token interior according to matches
        let currentPos = token.start;
        for (let i = 0; i < overlapping.length; i++) {
            const m = overlapping[i];
            const matchStart = Math.max(token.start, m.colStart);
            const matchEnd = Math.min(token.end, m.colStart + m.length);

            if (matchStart > currentPos) {
                const nonMatchSlice = token.text.slice(
                    currentPos - token.start,
                    matchStart - token.start
                );
                elements.push(
                    <span key={`${tokenKey}-p-${i}`} className={tokenClass}>
                        {nonMatchSlice}
                    </span>
                );
            }

            const matchSlice = token.text.slice(
                matchStart - token.start,
                matchEnd - token.start
            );
            const isActive = m.globalIndex === currentMatchIndex;

            elements.push(
                <mark
                    key={`${tokenKey}-m-${i}`}
                    id={isActive ? `active-match-${cardId}` : undefined}
                    className={`rounded-sm px-0.5 font-bold transition-all duration-150 select-text ${
                        isActive
                            ? 'bg-amber-400 text-gray-950 ring-2 ring-amber-500 shadow-sm z-10'
                            : 'bg-amber-400/40 text-amber-950 dark:text-amber-100'
                    }`}
                >
                    {matchSlice}
                </mark>
            );

            currentPos = matchEnd;
        }

        if (currentPos < token.end) {
            const tailSlice = token.text.slice(currentPos - token.start);
            elements.push(
                <span key={`${tokenKey}-tail`} className={tokenClass}>
                    {tailSlice}
                </span>
            );
        }
    }

    return elements;
};

interface VirtualLineProps {
    lineIndex: number;
    line: string;
    start: number;
    isWrap: boolean;
    cardId: string;
    lineMatches?: SearchMatch[];
    currentMatchIndex: number;
    measureElement: (node: HTMLDivElement | null) => void;
}

// React.memo isolates viewport row rendering for smooth 60fps scrolling
const VirtualLine = React.memo<VirtualLineProps>(({
    lineIndex,
    line,
    start,
    isWrap,
    cardId,
    lineMatches,
    currentMatchIndex,
    measureElement,
}) => {
    return (
        <div
            ref={measureElement}
            data-index={lineIndex}
            style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                transform: `translateY(${start}px)`,
            }}
            className="flex items-start hover:bg-gray-200/40 dark:hover:bg-base-300/40 transition-colors"
        >
            {/* Line Number Gutter */}
            <div className="w-11 shrink-0 text-right pr-2 select-none text-[10px] font-mono text-gray-400 dark:text-gray-500 border-r border-gray-200/70 dark:border-base-300 bg-gray-100/40 dark:bg-base-300/20 leading-5">
                {lineIndex + 1}
            </div>

            {/* Line Content */}
            <div
                className={`flex-1 pl-2.5 pr-4 font-mono text-[11px] leading-5 ${
                    isWrap ? 'whitespace-pre-wrap break-all' : 'whitespace-pre'
                }`}
            >
                {renderLineContent(line, lineIndex, lineMatches, currentMatchIndex, cardId)}
            </div>
        </div>
    );
});

VirtualLine.displayName = 'VirtualLine';


export const VirtualizedPayloadViewer: React.FC<VirtualizedPayloadViewerProps> = ({
    cardId,
    title,
    badge,
    badgeStyle,
    rawPayload,
    concisePayload,
    headersJson,
    viewMode,
    emptyPlaceholder,
    onCopy,
    isCopied,
    timingNode,
}) => {
    const { t } = useTranslation();
    const [searchTerm, setSearchTerm] = useState('');
    const [debouncedSearchTerm, setDebouncedSearchTerm] = useState('');
    const [caseSensitive, setCaseSensitive] = useState(false);
    const [isWrap, setIsWrap] = useState(true);
    const [isHeadersExpanded, setIsHeadersExpanded] = useState(false);
    const [currentMatchIndex, setCurrentMatchIndex] = useState(0);

    const [containerWidth, setContainerWidth] = useState<number>(0);
    const [fontMetrics, setFontMetrics] = useState<{ charWidth: number; lineHeight: number }>({
        charWidth: 6.62,
        lineHeight: 20,
    });

    const containerRef = useRef<HTMLDivElement>(null);
    const searchInputRef = useRef<HTMLInputElement>(null);
    const fontMeasureRef = useRef<HTMLSpanElement>(null);

    // Measure precise monospace character width and line height
    useEffect(() => {
        if (fontMeasureRef.current) {
            const rect = fontMeasureRef.current.getBoundingClientRect();
            const cw = rect.width / 50;
            const lh = rect.height || 20;
            if (cw > 4 && cw < 15) {
                setFontMetrics({ charWidth: cw, lineHeight: Math.round(lh) || 20 });
            }
        }
    }, []);

    // Dynamically observe container viewport width (resizes, 3-column scaling, sidebar collapse)
    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;

        const updateWidth = () => {
            const w = el.clientWidth;
            if (w > 0) {
                setContainerWidth((prev) => (Math.abs(prev - w) > 4 ? w : prev));
            }
        };

        updateWidth();

        const ro = new ResizeObserver(() => {
            updateWidth();
        });
        ro.observe(el);
        return () => ro.disconnect();
    }, []);

    // Debounced search input (120ms): ensures responsive typing and fast indexing
    useEffect(() => {
        const timer = setTimeout(() => {
            setDebouncedSearchTerm(searchTerm);
        }, 120);
        return () => clearTimeout(timer);
    }, [searchTerm]);

    // Active display content
    const activeContent = useMemo(() => {
        if (viewMode === 'concise') {
            const trimmed = concisePayload ? concisePayload.trim() : '';
            if (trimmed && trimmed !== '{}') {
                return concisePayload;
            }
            return rawPayload || '';
        }
        return rawPayload || '';
    }, [viewMode, concisePayload, rawPayload]);

    // Formatted JSON string
    const formattedContent = useMemo(() => {
        if (!activeContent) return '';
        try {
            const obj = JSON.parse(activeContent);
            return JSON.stringify(obj, null, 2);
        } catch {
            return activeContent;
        }
    }, [activeContent]);

    // Split into lines for virtualization (< 2ms for 20,000 lines)
    const lines = useMemo(() => {
        if (!formattedContent) return [];
        return formattedContent.split('\n');
    }, [formattedContent]);

    // Formatted Headers
    const prettyHeaders = useMemo(() => {
        if (!headersJson) return '';
        try {
            return JSON.stringify(JSON.parse(headersJson), null, 2);
        } catch {
            return headersJson;
        }
    }, [headersJson]);

    // Fast line-level search index (< 1.5ms for 20,000 lines)
    const { matches, matchesByLine } = useMemo(() => {
        const trimmed = debouncedSearchTerm.trim();
        if (!trimmed || lines.length === 0) {
            return { matches: [] as SearchMatch[], matchesByLine: new Map<number, SearchMatch[]>() };
        }

        const matchesList: SearchMatch[] = [];
        const map = new Map<number, SearchMatch[]>();
        const query = caseSensitive ? trimmed : trimmed.toLowerCase();

        for (let i = 0; i < lines.length; i++) {
            const line = caseSensitive ? lines[i] : lines[i].toLowerCase();
            let pos = 0;
            while ((pos = line.indexOf(query, pos)) !== -1) {
                const item: SearchMatch = {
                    lineIndex: i,
                    colStart: pos,
                    length: query.length,
                    globalIndex: matchesList.length,
                };
                matchesList.push(item);
                let lineArr = map.get(i);
                if (!lineArr) {
                    lineArr = [];
                    map.set(i, lineArr);
                }
                lineArr.push(item);
                pos += query.length;
            }
        }

        return { matches: matchesList, matchesByLine: map };
    }, [lines, debouncedSearchTerm, caseSensitive]);

    const matchesCount = matches.length;

    // Estimated line height table (O(1) lookup to stabilize scroll bar)
    const lineHeights = useMemo(() => {
        const count = lines.length;
        if (!isWrap || count === 0) {
            return null;
        }
        const cw = fontMetrics.charWidth || 6.62;
        const lh = fontMetrics.lineHeight || 20;
        // Subtract line number column (44px) + horizontal padding (26px) = 70px
        const usableWidth = Math.max(100, (containerWidth || 600) - 70);
        const charsPerLine = Math.max(10, Math.floor(usableWidth / cw));

        const heights = new Int32Array(count);
        for (let i = 0; i < count; i++) {
            const line = lines[i];
            if (!line || line.length <= charsPerLine) {
                heights[i] = lh;
            } else {
                const visualChars = getVisualCharCount(line);
                heights[i] = Math.max(1, Math.ceil(visualChars / charsPerLine)) * lh;
            }
        }
        return heights;
    }, [lines, isWrap, containerWidth, fontMetrics]);

    const estimateSize = useCallback(
        (index: number) => {
            if (!isWrap || !lineHeights) {
                return fontMetrics.lineHeight || 20;
            }
            return lineHeights[index] || fontMetrics.lineHeight || 20;
        },
        [isWrap, lineHeights, fontMetrics.lineHeight]
    );

    // Virtualized scroll core: renders only visible 30-40 rows
    const rowVirtualizer = useVirtualizer({
        count: lines.length,
        getScrollElement: () => containerRef.current,
        estimateSize,
        overscan: 10,
        useFlushSync: false,
        getItemKey: (index) => `${cardId}-${isWrap ? 'w' : 'nw'}-${index}`,
    });

    // Retain scroll offset during dynamic resizing to prevent jitter
    rowVirtualizer.shouldAdjustScrollPositionOnItemSizeChange = () => false;

    // Reset measurement cache when content, wrapping, or container width changes
    useEffect(() => {
        rowVirtualizer.measure();
    }, [formattedContent, isWrap, containerWidth]);

    // Reset highlight to first match on search query update
    useEffect(() => {
        setCurrentMatchIndex(0);
        if (matches.length > 0) {
            rowVirtualizer.scrollToIndex(matches[0].lineIndex, { align: 'center', behavior: 'auto' });
        }
    }, [debouncedSearchTerm, caseSensitive]);

    // Smoothly scroll active match into view
    const scrollToMatch = (targetIndex: number) => {
        if (matches.length > 0 && matches[targetIndex]) {
            rowVirtualizer.scrollToIndex(matches[targetIndex].lineIndex, {
                align: 'center',
                behavior: 'smooth',
            });
        }
    };

    const handleNext = () => {
        if (matchesCount > 0) {
            const nextIdx = (currentMatchIndex + 1) % matchesCount;
            setCurrentMatchIndex(nextIdx);
            scrollToMatch(nextIdx);
        }
    };

    const handlePrev = () => {
        if (matchesCount > 0) {
            const prevIdx = (currentMatchIndex - 1 + matchesCount) % matchesCount;
            setCurrentMatchIndex(prevIdx);
            scrollToMatch(prevIdx);
        }
    };

    const copyPayload = prettyHeaders
        ? `/* headers */\n${prettyHeaders}\n\n/* body */\n${formattedContent}`
        : formattedContent;

    return (
        <div
            className="payload-viewer-card flex flex-col h-full bg-slate-50/50 dark:bg-base-200 rounded-xl border border-gray-200 dark:border-base-300 overflow-hidden shadow-sm outline-none"
            tabIndex={-1}
            onKeyDown={(e) => {
                if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F')) {
                    e.preventDefault();
                    e.stopPropagation();
                    searchInputRef.current?.focus();
                    searchInputRef.current?.select();
                }
            }}
        >
            {/* Card Header */}
            <div className="px-3.5 py-2 border-b border-gray-200 dark:border-base-300 bg-white/95 dark:bg-base-200 flex items-center justify-between gap-2 shrink-0 select-none">
                <div className="flex items-center gap-2 min-w-0">
                    <span className={`px-2 py-0.5 rounded text-[10px] font-black uppercase tracking-wider border shrink-0 ${badgeStyle}`}>
                        {badge}
                    </span>
                    <h3 className="text-xs font-bold text-gray-800 dark:text-gray-200 truncate" title={title}>
                        {title}
                    </h3>
                </div>

                <div className="flex items-center gap-1.5 shrink-0">
                    <button
                        type="button"
                        onClick={() => onCopy(copyPayload)}
                        disabled={!formattedContent && !prettyHeaders}
                        className="btn btn-ghost btn-xs gap-1 h-7 px-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-base-300"
                        title={isCopied ? t('proxy.config.btn_copied', 'Copied') : t('proxy.config.btn_copy', 'Copy')}
                    >
                        {isCopied ? <CheckCircle size={12} className="text-emerald-500" /> : <Copy size={12} />}
                        <span className="text-[10px] font-medium">
                            {isCopied ? t('proxy.config.btn_copied', 'Copied') : t('proxy.config.btn_copy', 'Copy')}
                        </span>
                    </button>
                </div>
            </div>

            {/* Browser-Grade Search & Control Toolbar */}
            <div className="px-2.5 py-1.5 bg-gray-100/70 dark:bg-base-300/40 border-b border-gray-200 dark:border-base-300 flex items-center gap-1.5 shrink-0">
                <div className="relative flex-1 min-w-0 flex items-center">
                    <Search size={12} className="absolute left-2 text-gray-400 pointer-events-none" />
                    <input
                        ref={searchInputRef}
                        type="text"
                        placeholder={t('monitor.details.search_placeholder', 'Search payload... (Enter: next, Shift+Enter: previous)')}
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter') {
                                e.preventDefault();
                                if (e.shiftKey) {
                                    handlePrev();
                                } else {
                                    handleNext();
                                }
                            } else if (e.key === 'Escape') {
                                setSearchTerm('');
                                searchInputRef.current?.blur();
                            } else if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F')) {
                                e.preventDefault();
                                e.stopPropagation();
                                searchInputRef.current?.select();
                            }
                        }}
                        className="input input-xs input-bordered w-full pl-6 pr-6 text-[11px] h-7 bg-white dark:bg-base-100 border-gray-200 dark:border-base-300 text-gray-800 dark:text-gray-200 rounded-md focus:border-blue-500 font-mono"
                    />
                    {searchTerm && (
                        <button
                            type="button"
                            onClick={() => setSearchTerm('')}
                            className="absolute right-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-0.5"
                            title="Clear search"
                        >
                            <X size={12} />
                        </button>
                    )}
                </div>

                {/* Case-Sensitive Toggle */}
                <button
                    type="button"
                    onClick={() => setCaseSensitive((prev) => !prev)}
                    className={`h-7 px-1.5 rounded-md border text-[10px] font-bold flex items-center gap-0.5 transition-colors cursor-pointer select-none ${
                        caseSensitive
                            ? 'bg-blue-100 text-blue-700 border-blue-300 dark:bg-blue-900/40 dark:text-blue-300 dark:border-blue-800'
                            : 'bg-white dark:bg-base-100 text-gray-400 border-gray-200 dark:border-base-300 hover:text-gray-600 dark:hover:text-gray-300'
                    }`}
                    title={caseSensitive ? 'Case sensitive enabled' : 'Toggle case sensitive'}
                >
                    <CaseSensitive size={13} />
                </button>

                {/* Wrap / No-Wrap Toggle */}
                <button
                    type="button"
                    onClick={() => setIsWrap((prev) => !prev)}
                    className={`h-7 px-1.5 rounded-md border text-[10px] font-medium flex items-center gap-1 transition-colors cursor-pointer select-none ${
                        isWrap
                            ? 'bg-gray-200/80 text-gray-800 border-gray-300 dark:bg-base-200 dark:text-gray-200 dark:border-base-300'
                            : 'bg-white dark:bg-base-100 text-gray-400 border-gray-200 dark:border-base-300 hover:text-gray-600 dark:hover:text-gray-300'
                    }`}
                    title={isWrap ? 'Word wrap enabled (click to toggle)' : 'Single line horizontal scroll (click to toggle wrap)'}
                >
                    <WrapText size={12} />
                    <span className="hidden sm:inline text-[9px]">{isWrap ? 'Wrap' : 'No Wrap'}</span>
                </button>

                {/* Match Counter & Prev/Next Controls */}
                {searchTerm.trim() && (
                    <div className="flex items-center gap-1 shrink-0 bg-white dark:bg-base-100 border border-gray-200 dark:border-base-300 rounded-md px-1.5 py-0.5 h-7">
                        <span className={`text-[10px] font-mono font-bold ${
                            matchesCount > 0
                                ? 'text-amber-600 dark:text-amber-400'
                                : 'text-gray-400'
                        }`}>
                            {matchesCount > 0 ? `${currentMatchIndex + 1}/${matchesCount}` : 'No matches'}
                        </span>
                        <div className="flex items-center">
                            <button
                                type="button"
                                onClick={handlePrev}
                                disabled={matchesCount <= 1}
                                className="btn btn-ghost btn-xs p-0.5 h-5 min-h-0 text-gray-500 dark:text-gray-400 disabled:opacity-30"
                                title="Previous match (Shift+Enter)"
                            >
                                <ChevronUp size={12} />
                            </button>
                            <button
                                type="button"
                                onClick={handleNext}
                                disabled={matchesCount <= 1}
                                className="btn btn-ghost btn-xs p-0.5 h-5 min-h-0 text-gray-500 dark:text-gray-400 disabled:opacity-30"
                                title="Next match (Enter)"
                            >
                                <ChevronDown size={12} />
                            </button>
                        </div>
                    </div>
                )}
            </div>

            {/* Optional Header Section (Timing or Headers) */}
            <div className="shrink-0 bg-gray-50 dark:bg-base-200 border-b border-gray-200 dark:border-base-300">
                {timingNode}

                {prettyHeaders && (
                    <div className="border-t border-gray-200 dark:border-base-300">
                        <div
                            onClick={() => setIsHeadersExpanded((prev) => !prev)}
                            className="px-3 py-1.5 bg-gray-100/70 dark:bg-base-300/40 flex items-center justify-between cursor-pointer select-none hover:bg-gray-200/60 dark:hover:bg-base-300/70 transition-colors"
                        >
                            <div className="flex items-center gap-1.5">
                                <span className="text-[10px] font-mono font-bold uppercase tracking-wider text-gray-600 dark:text-gray-300">
                                    {t('monitor.details.headers', 'Headers')}
                                </span>
                                <span className="text-[9px] font-mono text-gray-400 dark:text-gray-500">
                                    ({prettyHeaders.split('\n').length} lines)
                                </span>
                            </div>
                            <button
                                type="button"
                                className="text-[10px] text-blue-600 dark:text-blue-400 font-medium flex items-center gap-0.5"
                            >
                                <span>{isHeadersExpanded ? 'Collapse' : 'Expand'}</span>
                                <ChevronDown size={12} className={`transition-transform duration-200 ${isHeadersExpanded ? 'rotate-180' : ''}`} />
                            </button>
                        </div>
                        {isHeadersExpanded && (
                            <div className="p-2.5 max-h-40 overflow-y-auto bg-white/60 dark:bg-base-100 font-mono text-[10px] leading-relaxed border-t border-gray-200 dark:border-base-200">
                                <pre className="whitespace-pre-wrap select-text m-0 text-gray-600 dark:text-gray-300 font-mono">
                                    {prettyHeaders}
                                </pre>
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* Virtualized Body Container */}
            <div className="flex-1 min-h-0 relative bg-gray-50/30 dark:bg-base-100">
                {/* Chrome-Style Scrollbar Minimap Ticks */}
                {matches.length > 0 && (
                    <div
                        className="absolute right-0 top-0 bottom-0 w-2.5 pointer-events-none z-20 overflow-hidden"
                        aria-hidden="true"
                    >
                        {matches.slice(0, 300).map((m) => {
                            const totalSize = rowVirtualizer.getTotalSize();
                            let topPct = (m.lineIndex / Math.max(1, lines.length)) * 100;
                            if (totalSize > 0) {
                                const offsetInfo = rowVirtualizer.getOffsetForIndex(m.lineIndex);
                                if (offsetInfo) {
                                    topPct = (offsetInfo[0] / totalSize) * 100;
                                }
                            }
                            const isActive = m.globalIndex === currentMatchIndex;
                            return (
                                <div
                                    key={m.globalIndex}
                                    style={{ top: `${topPct}%` }}
                                    className={`absolute right-0.5 w-1.5 rounded-sm transition-all ${
                                        isActive
                                            ? 'h-2 bg-amber-500 ring-2 ring-amber-300 z-30 shadow'
                                            : 'h-1 bg-amber-400/80 dark:bg-amber-400/70'
                                    }`}
                                />
                            );
                        })}
                    </div>
                )}

                {lines.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center p-8 text-center text-gray-400 dark:text-gray-500 select-none">
                        <span className="text-xs italic">{emptyPlaceholder}</span>
                    </div>
                ) : (
                    <div
                        ref={containerRef}
                        tabIndex={0}
                        className="h-full overflow-y-auto overflow-x-auto font-mono text-[11px] outline-none focus:ring-1 focus:ring-blue-500/20 select-text payload-viewer-scroll"
                    >
                        {/* Hidden monospace calibration element */}
                        <span
                            ref={fontMeasureRef}
                            className="font-mono text-[11px] leading-5 invisible absolute -top-[9999px] left-0 pointer-events-none select-none"
                            aria-hidden="true"
                        >
                            {"0123456789".repeat(5)}
                        </span>

                        <div
                            style={{
                                height: `${rowVirtualizer.getTotalSize()}px`,
                                width: isWrap ? '100%' : 'max-content',
                                minWidth: '100%',
                                position: 'relative',
                            }}
                        >
                            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                                const lineIndex = virtualRow.index;
                                const line = lines[lineIndex];

                                return (
                                    <VirtualLine
                                        key={virtualRow.key}
                                        lineIndex={lineIndex}
                                        line={line}
                                        start={virtualRow.start}
                                        isWrap={isWrap}
                                        cardId={cardId}
                                        lineMatches={matchesByLine.get(lineIndex)}
                                        currentMatchIndex={currentMatchIndex}
                                        measureElement={rowVirtualizer.measureElement}
                                    />
                                );
                            })}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};
