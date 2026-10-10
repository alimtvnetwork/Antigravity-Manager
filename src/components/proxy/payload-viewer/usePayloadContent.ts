import { useMemo } from 'react';
import { deepUnescapeJsonValue } from './tokenizer';

export interface PayloadContentInput {
    viewMode: 'concise' | 'full';
    concisePayload?: string;
    rawPayload?: string;
    headersJson?: string;
}

export interface PayloadContent {
    activeContent: string;
    formattedContent: string;
    lines: string[];
    prettyHeaders: string;
    copyPayload: string;
}

export function usePayloadContent({
    viewMode,
    concisePayload,
    rawPayload,
    headersJson,
}: PayloadContentInput): PayloadContent {
    // Active display content
    const activeContent = useMemo(() => {
        if (viewMode === 'concise') {
            const trimmed = concisePayload ? concisePayload.trim() : '';
            if (trimmed && trimmed !== '{}') {
                return concisePayload ?? '';
            }
            return rawPayload || '';
        }
        return rawPayload || '';
    }, [viewMode, concisePayload, rawPayload]);

    // Formatted JSON string (deep unescape and format)
    const formattedContent = useMemo(() => {
        if (!activeContent) return '';
        try {
            let obj: unknown = JSON.parse(activeContent);
            // Handle top-level double-escaped-as-string case
            if (typeof obj === 'string') {
                try {
                    obj = JSON.parse(obj);
                } catch {
                    // Justification: format probe — a non-JSON string here is the expected common case
                    // (plain-text payloads), not an error; the original string is formatted as-is below.
                }
            }
            const unescaped = deepUnescapeJsonValue(obj);
            return JSON.stringify(unescaped, null, 2);
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

    const copyPayload = prettyHeaders
        ? `/* headers */\n${prettyHeaders}\n\n/* body */\n${formattedContent}`
        : formattedContent;

    return { activeContent, formattedContent, lines, prettyHeaders, copyPayload };
}
