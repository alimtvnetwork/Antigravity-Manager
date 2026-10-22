import { useState, useRef, useEffect, useCallback } from 'react';

export interface UseAutoScrollOptions {
    bottomThresholdPx?: number;
    smooth?: boolean;
}

/**
 * Smart auto-scroller hook that anchors to the bottom of a container while content updates,
 * unless the user manually scrolls up to inspect older records.
 */
export function useAutoScroll<T extends HTMLElement = HTMLDivElement>(
    dependency: unknown,
    options: UseAutoScrollOptions = {}
) {
    const { bottomThresholdPx = 40, smooth = false } = options;
    const scrollRef = useRef<T | null>(null);
    const [isScrolledToBottom, setIsScrolledToBottom] = useState(true);

    const scrollToBottom = useCallback(() => {
        if (!scrollRef.current) return;
        scrollRef.current.scrollTo({
            top: scrollRef.current.scrollHeight,
            behavior: smooth ? 'smooth' : 'auto',
        });
    }, [smooth]);

    const handleScroll = useCallback(() => {
        if (!scrollRef.current) return;
        const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
        const distanceToBottom = scrollHeight - (scrollTop + clientHeight);
        const nearBottom = distanceToBottom <= bottomThresholdPx;
        setIsScrolledToBottom(nearBottom);
    }, [bottomThresholdPx]);

    useEffect(() => {
        if (isScrolledToBottom) {
            scrollToBottom();
        }
    }, [dependency, isScrolledToBottom, scrollToBottom]);

    return {
        scrollRef,
        isScrolledToBottom,
        scrollToBottom,
        handleScroll,
    };
}

export default useAutoScroll;
