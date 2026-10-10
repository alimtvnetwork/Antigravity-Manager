import React from 'react';
import type { SearchMatch, VirtualLineProps } from './types';
import { getTokenClass, tokenizeJsonLine } from './tokenizer';

// Cancel TanStack scrollToIndex internal retry alignment, avoiding pulling back already aligned mark to row center
export const cancelVirtualizerScrollToIndex = (virtualizer: unknown): void => {
    (virtualizer as { currentScrollToIndex: number | null }).currentScrollToIndex = null;
};

// Render single-line text: sliced based on lossless tokens and search highlight intervals
export const renderLineContent = (
    line: string,
    lineIndex: number,
    lineMatches: SearchMatch[] | undefined,
    currentMatchIndex: number,
    cardId: string
): React.ReactNode => {
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
                    data-card-id={cardId}
                    data-match-index={m.globalIndex}
                    data-active-match={isActive ? 'true' : undefined}
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

// React.memo isolates viewport row rendering for smooth 60fps scrolling
export const VirtualLine = React.memo<VirtualLineProps>(({
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
