import { useCallback, useRef } from 'react';
import type { RefObject } from 'react';
import type { Virtualizer } from '@tanstack/react-virtual';
import type { FontMetrics, SearchMatch } from './types';
import { LINE_GUTTER_PX, MATCH_SCROLL_MAX_FRAMES } from './types';
import { getVisualCharCount } from './tokenizer';
import { cancelVirtualizerScrollToIndex } from './VirtualLine';

export interface MatchNavigationInput {
    matches: SearchMatch[];
    cardId: string;
    isWrap: boolean;
    fontMetrics: FontMetrics;
    lines: string[];
    lineHeights: Int32Array | null;
    containerWidth: number;
    containerRef: RefObject<HTMLDivElement | null>;
    rowVirtualizer: Virtualizer<HTMLDivElement, Element>;
    currentMatchIndex: number;
    setCurrentMatchIndex: (v: number) => void;
}

export interface MatchNavigation {
    handleNext: () => void;
    handlePrev: () => void;
    scrollToMatch: (targetIndex: number) => void;
}

export function useMatchNavigation(input: MatchNavigationInput): MatchNavigation {
    const {
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
    } = input;
    const scrollGenRef = useRef(0);
    const matchesCount = matches.length;

    // Pixel offset of match point in virtual row: convert wrapRow by visual column width
    // to avoid scrollToIndex only centering whole line
    const getMatchContentOffset = useCallback((match: SearchMatch) => {
        const cw = fontMetrics.charWidth || 6.62;
        const lh = fontMetrics.lineHeight || 20;
        const line = lines[match.lineIndex] || '';
        const visualBefore = getVisualCharCount(line.slice(0, match.colStart));

        const measured = rowVirtualizer.measurementsCache[match.lineIndex];
        let lineStart = 0;
        if (measured && Number.isFinite(measured.start)) {
            lineStart = measured.start;
        } else if (lineHeights) {
            for (let i = 0; i < match.lineIndex && i < lineHeights.length; i++) {
                lineStart += lineHeights[i];
            }
        } else {
            lineStart = match.lineIndex * lh;
        }

        if (!isWrap) {
            return {
                top: lineStart,
                left: LINE_GUTTER_PX + visualBefore * cw,
            };
        }

        const usableWidth = Math.max(100, (containerRef.current?.clientWidth || containerWidth || 600) - 70);
        const charsPerLine = Math.max(10, Math.floor(usableWidth / cw));
        const wrapRow = Math.floor(visualBefore / charsPerLine);
        return {
            top: lineStart + wrapRow * lh,
            left: 0,
        };
    }, [fontMetrics, lines, rowVirtualizer, lineHeights, isWrap, containerWidth, containerRef]);

    // Small viewport + long wrapped lines: jump to match point then fine tune geometrically once mark enters DOM
    const ensureActiveMatchInView = useCallback((targetIndex: number) => {
        const match = matches[targetIndex];
        if (!match) return;

        const gen = ++scrollGenRef.current;
        const lh = fontMetrics.lineHeight || 20;

        const jumpByMath = () => {
            const el = containerRef.current;
            if (!el) return;
            const { top, left } = getMatchContentOffset(match);
            el.scrollTop = Math.max(0, Math.round(top - el.clientHeight / 2 + lh / 2));
            el.scrollLeft = isWrap ? 0 : Math.max(0, Math.round(left - el.clientWidth / 2));
        };

        cancelVirtualizerScrollToIndex(rowVirtualizer);
        jumpByMath();

        const step = (frame: number) => {
            if (scrollGenRef.current !== gen) return;

            const el = containerRef.current;
            if (!el) return;

            const activeMark = el.querySelector(
                `mark[data-card-id="${cardId}"][data-match-index="${targetIndex}"]`
            ) as HTMLElement | null;

            if (activeMark) {
                // mark in DOM: stop scrollToIndex retries and roll match point to center
                cancelVirtualizerScrollToIndex(rowVirtualizer);
                const containerRect = el.getBoundingClientRect();
                const markRect = activeMark.getBoundingClientRect();
                const pad = 8;
                const visible =
                    markRect.bottom > containerRect.top + pad &&
                    markRect.top < containerRect.bottom - pad &&
                    markRect.right > containerRect.left + pad &&
                    markRect.left < containerRect.right - pad;

                const dy = markRect.top + markRect.height / 2 - (containerRect.top + containerRect.height / 2);
                const dx = isWrap
                    ? 0
                    : markRect.left + markRect.width / 2 - (containerRect.left + containerRect.width / 2);
                const ySlop = Math.max(16, containerRect.height * 0.18);
                const xSlop = Math.max(24, containerRect.width * 0.22);

                if (visible && Math.abs(dy) <= ySlop && Math.abs(dx) <= xSlop) {
                    return;
                }

                el.scrollTop = Math.max(0, el.scrollTop + dy);
                if (!isWrap) {
                    el.scrollLeft = Math.max(0, el.scrollLeft + dx);
                }
            } else if (frame === 8 || frame === 18) {
                // Estimation offset: pull row into DOM
                rowVirtualizer.scrollToIndex(match.lineIndex, { align: 'start', behavior: 'auto' });
            } else if (frame < 8 || frame > 20) {
                jumpByMath();
            }

            if (frame < MATCH_SCROLL_MAX_FRAMES) {
                requestAnimationFrame(() => step(frame + 1));
            }
        };

        requestAnimationFrame(() => step(0));
    }, [matches, cardId, rowVirtualizer, fontMetrics.lineHeight, isWrap, getMatchContentOffset, containerRef]);

    // Smoothly scroll active match into view
    const scrollToMatch = useCallback((targetIndex: number) => {
        ensureActiveMatchInView(targetIndex);
    }, [ensureActiveMatchInView]);

    const handleNext = useCallback(() => {
        if (matchesCount > 0) {
            const nextIdx = (currentMatchIndex + 1) % matchesCount;
            setCurrentMatchIndex(nextIdx);
            scrollToMatch(nextIdx);
        }
    }, [matchesCount, currentMatchIndex, scrollToMatch, setCurrentMatchIndex]);

    const handlePrev = useCallback(() => {
        if (matchesCount > 0) {
            const prevIdx = (currentMatchIndex - 1 + matchesCount) % matchesCount;
            setCurrentMatchIndex(prevIdx);
            scrollToMatch(prevIdx);
        }
    }, [matchesCount, currentMatchIndex, scrollToMatch, setCurrentMatchIndex]);

    return { handleNext, handlePrev, scrollToMatch };
}
