import React, { useState, useRef, useEffect, useMemo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useVirtualizer } from '@tanstack/react-virtual';
import { Copy, CheckCircle, ChevronDown } from 'lucide-react';
import type {
    FontMetrics,
    VirtualizedPayloadViewerProps,
} from './payload-viewer/types';
import { DEFAULT_FONT_METRICS, MINIMAP_MAX_TICKS } from './payload-viewer/types';
import { getVisualCharCount } from './payload-viewer/tokenizer';
import { VirtualLine } from './payload-viewer/VirtualLine';
import { usePayloadContent } from './payload-viewer/usePayloadContent';
import { usePayloadSearch } from './payload-viewer/usePayloadSearch';
import { useMatchNavigation } from './payload-viewer/useMatchNavigation';
import { ViewerToolbar } from './payload-viewer/ViewerToolbar';

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
    const [isWrap, setIsWrap] = useState(true);
    const [isHeadersExpanded, setIsHeadersExpanded] = useState(false);
    const [isHeadersCopied, setIsHeadersCopied] = useState(false);
    const [containerWidth, setContainerWidth] = useState<number>(0);
    const [fontMetrics, setFontMetrics] = useState<FontMetrics>(DEFAULT_FONT_METRICS);

    const containerRef = useRef<HTMLDivElement | null>(null);
    const fontMeasureRef = useRef<HTMLSpanElement | null>(null);

    const { formattedContent, lines, prettyHeaders, copyPayload } = usePayloadContent({
        viewMode,
        concisePayload,
        rawPayload,
        headersJson,
    });

    const {
        searchTerm,
        setSearchTerm,
        debouncedSearchTerm,
        caseSensitive,
        setCaseSensitive,
        matches,
        matchesByLine,
        matchesCount,
        currentMatchIndex,
        setCurrentMatchIndex,
        searchInputRef,
    } = usePayloadSearch(lines);

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
    }, [formattedContent, isWrap, containerWidth, rowVirtualizer]);

    const { handleNext, handlePrev, scrollToMatch } = useMatchNavigation({
        matches,
        cardId,
        isWrap,
        fontMetrics,
        lines,
        lineHeights,
        containerWidth,
        containerRef,
        rowVirtualizer,
        currentMatchIndex,
        setCurrentMatchIndex,
    });

    // Reset highlight to first match on search query update
    const scrollToMatchRef = useRef(scrollToMatch);
    scrollToMatchRef.current = scrollToMatch;
    useEffect(() => {
        setCurrentMatchIndex(0);
        if (matches.length > 0) {
            scrollToMatchRef.current(0);
        }
    }, [debouncedSearchTerm, caseSensitive, matches.length, formattedContent, setCurrentMatchIndex]);

    // Independent Headers Copy
    const handleCopyHeaders = useCallback(async (e?: React.MouseEvent) => {
        if (e) e.stopPropagation();
        if (!prettyHeaders) return;
        try {
            await navigator.clipboard.writeText(prettyHeaders);
            setIsHeadersCopied(true);
            setTimeout(() => setIsHeadersCopied(false), 2000);
        } catch {
            if (onCopy) {
                await onCopy(prettyHeaders);
                setIsHeadersCopied(true);
                setTimeout(() => setIsHeadersCopied(false), 2000);
            }
        }
    }, [prettyHeaders, onCopy]);

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
            <ViewerToolbar
                t={t}
                searchTerm={searchTerm}
                setSearchTerm={setSearchTerm}
                caseSensitive={caseSensitive}
                setCaseSensitive={setCaseSensitive}
                isWrap={isWrap}
                setIsWrap={setIsWrap}
                matchesCount={matchesCount}
                currentMatchIndex={currentMatchIndex}
                onNext={handleNext}
                onPrev={handlePrev}
                searchInputRef={searchInputRef}
            />

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
                            <div className="flex items-center gap-2">
                                <button
                                    type="button"
                                    onClick={handleCopyHeaders}
                                    className="btn btn-ghost btn-xs h-5 min-h-0 px-1.5 gap-1 text-[10px] text-gray-500 hover:text-blue-500 dark:text-gray-400 dark:hover:text-blue-400 font-normal hover:bg-white/60 dark:hover:bg-base-200"
                                    title={t('common.copy', 'Copy')}
                                >
                                    {isHeadersCopied ? <CheckCircle size={11} className="text-green-500" /> : <Copy size={11} />}
                                    <span>{isHeadersCopied ? (t('common.copied', 'Copied')) : (t('common.copy', 'Copy'))}</span>
                                </button>
                                <button
                                    type="button"
                                    className="text-[10px] text-blue-600 dark:text-blue-400 font-medium flex items-center gap-0.5"
                                >
                                    <span>{isHeadersExpanded ? t('common.collapse', 'Collapse') : t('common.expand', 'Expand')}</span>
                                    <ChevronDown size={12} className={`transition-transform duration-200 ${isHeadersExpanded ? 'rotate-180' : ''}`} />
                                </button>
                            </div>
                        </div>
                        {isHeadersExpanded && (
                            <div className="relative group/head p-2.5 max-h-44 overflow-y-auto bg-white/60 dark:bg-base-100 font-mono text-[10px] leading-relaxed border-t border-gray-200 dark:border-base-200">
                                <button
                                    type="button"
                                    onClick={handleCopyHeaders}
                                    className="absolute top-2 right-2 p-1 rounded bg-white/80 dark:bg-base-200 border border-gray-200 dark:border-base-300 text-gray-500 hover:text-blue-500 opacity-0 group-hover/head:opacity-100 transition-opacity shadow-xs"
                                    title={t('common.copy', 'Copy')}
                                >
                                    {isHeadersCopied ? <CheckCircle size={12} className="text-green-500" /> : <Copy size={12} />}
                                </button>
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
                        {matches.slice(0, MINIMAP_MAX_TICKS).map((m) => {
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
                                overflowAnchor: 'none',
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
