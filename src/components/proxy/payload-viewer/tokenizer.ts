import type { LineToken, LineTokenType } from './types';
import { TOKEN_CACHE_MAX_SIZE } from './types';

export const getTokenClass = (type: LineTokenType): string => {
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
export const tokenizeJsonLine = (line: string): LineToken[] => {
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
        let type: LineTokenType = 'plain';

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

    if (tokenCache.size > TOKEN_CACHE_MAX_SIZE) {
        tokenCache.clear();
    }
    tokenCache.set(line, tokens);

    return tokens;
};

// Calculate visual character width in monospace font (ASCII = 1, CJK/full-width = 2)
export const getVisualCharCount = (str: string): number => {
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

/**
 * Recursively deep-unescapes JSON content nested inside JSON string properties.
 * E.g. `"response": "{\"error\":{\"code\":400...}}"` becomes a real nested object,
 * eliminating escaped backslashes so JSON.stringify pretty-prints cleanly.
 */
export function deepUnescapeJsonValue(val: unknown): unknown {
    if (typeof val === 'string') {
        const trimmed = val.trim();
        if ((trimmed.startsWith('{') && trimmed.endsWith('}')) || (trimmed.startsWith('[') && trimmed.endsWith(']'))) {
            try {
                const parsed: unknown = JSON.parse(trimmed);
                return deepUnescapeJsonValue(parsed);
            } catch {
                return val;
            }
        }
        return val;
    }
    if (Array.isArray(val)) {
        return val.map(deepUnescapeJsonValue);
    }
    if (val && typeof val === 'object') {
        const res: Record<string, unknown> = {};
        for (const [k, v] of Object.entries(val)) {
            res[k] = deepUnescapeJsonValue(v);
        }
        return res;
    }
    return val;
}
