import type React from 'react';

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

export interface SearchMatch {
    lineIndex: number;
    colStart: number;
    length: number;
    globalIndex: number;
}

export type LineTokenType =
    | 'key'
    | 'string'
    | 'number'
    | 'boolean'
    | 'null'
    | 'punct'
    | 'plain';

export interface LineToken {
    text: string;
    type: LineTokenType;
    start: number;
    end: number;
}

export interface VirtualLineProps {
    lineIndex: number;
    line: string;
    start: number;
    isWrap: boolean;
    cardId: string;
    lineMatches?: SearchMatch[];
    currentMatchIndex: number;
    measureElement: (node: HTMLDivElement | null) => void;
}

export interface FontMetrics {
    charWidth: number;
    lineHeight: number;
}

export const DEFAULT_FONT_METRICS: FontMetrics = {
    charWidth: 6.62,
    lineHeight: 20,
};

// Gutter slot 44px + text left padding 10px, used for horizontal positioning when nowrap
export const LINE_GUTTER_PX = 54;

// Debounce delay for the search input (ms)
export const SEARCH_DEBOUNCE_MS = 120;

// Max frames for the active-match scroll settle loop
export const MATCH_SCROLL_MAX_FRAMES = 30;

// Cap for the global token cache pool
export const TOKEN_CACHE_MAX_SIZE = 10000;

// Cap for minimap tick rendering
export const MINIMAP_MAX_TICKS = 300;
