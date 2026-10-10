export interface StageTimingInfo {
    cleanSec?: number;
    normSec?: number;
    thinkingSec?: number;
    ttftSec?: number;
    streamSec?: number;
    totalSec?: number;
    isOldRecordWithoutStages?: boolean;
}

export const parseTimingFromHeadersAndBody = (
    headersJson?: string,
    responseBody?: string,
    durationMs?: number
): StageTimingInfo | null => {
    let cleanSec: number | undefined;
    let normSec: number | undefined;
    let thinkingSec: number | undefined;
    let ttftSec: number | undefined;
    let streamSec: number | undefined;
    let totalSec: number | undefined;

    // 1. Check if responseBody has _timing object
    if (responseBody) {
        try {
            const bodyObj = JSON.parse(responseBody);
            if (bodyObj && typeof bodyObj === 'object' && bodyObj._timing) {
                const t = bodyObj._timing;
                if (typeof t.clean_s === 'number') cleanSec = t.clean_s;
                else if (typeof t.clean_ms === 'number') cleanSec = t.clean_ms / 1000;

                if (typeof t.norm_s === 'number') normSec = t.norm_s;
                else if (typeof t.norm_ms === 'number') normSec = t.norm_ms / 1000;

                if (typeof t.thinking_s === 'number') thinkingSec = t.thinking_s;
                else if (typeof t.thinking_ms === 'number') thinkingSec = t.thinking_ms / 1000;

                if (typeof t.ttft_s === 'number') ttftSec = t.ttft_s;
                else if (typeof t.ttft_ms === 'number') ttftSec = t.ttft_ms / 1000;

                if (typeof t.stream_s === 'number') streamSec = t.stream_s;
                else if (typeof t.stream_ms === 'number') streamSec = t.stream_ms / 1000;

                if (typeof t.total_s === 'number') totalSec = t.total_s;
                else if (typeof t.total_ms === 'number') totalSec = t.total_ms / 1000;
            }
        } catch {
            // Justification: optional timing-header probe — absent or malformed x-timing-* headers
            // are the expected common case (not an error); timing display simply stays unset.
        }
    }

    // 2. Parse from headersJson if any are still missing
    if (headersJson) {
        try {
            const headersObj = JSON.parse(headersJson);
            if (headersObj && typeof headersObj === 'object') {
                const getVal = (key: string): number | undefined => {
                    const matchKey = Object.keys(headersObj).find(
                        (k) => k.toLowerCase() === key.toLowerCase()
                    );
                    if (!matchKey) return undefined;
                    const v = headersObj[matchKey];
                    if (typeof v === 'number') return v;
                    if (typeof v === 'string') {
                        const parsed = parseFloat(v);
                        return isNaN(parsed) ? undefined : parsed;
                    }
                    if (Array.isArray(v) && v.length > 0) {
                        const parsed = parseFloat(String(v[0]));
                        return isNaN(parsed) ? undefined : parsed;
                    }
                    return undefined;
                };

                if (cleanSec === undefined) {
                    const ms = getVal('x-timing-clean-ms');
                    if (ms !== undefined) cleanSec = ms / 1000;
                }
                if (normSec === undefined) {
                    const ms = getVal('x-timing-norm-ms');
                    if (ms !== undefined) normSec = ms / 1000;
                }
                if (thinkingSec === undefined) {
                    const ms = getVal('x-timing-thinking-ms');
                    if (ms !== undefined) thinkingSec = ms / 1000;
                }
                if (ttftSec === undefined) {
                    const ms = getVal('x-timing-ttft-ms');
                    if (ms !== undefined) ttftSec = ms / 1000;
                }
                if (streamSec === undefined) {
                    const ms = getVal('x-timing-stream-ms');
                    if (ms !== undefined) streamSec = ms / 1000;
                }
                if (totalSec === undefined) {
                    const ms = getVal('x-timing-total-ms');
                    if (ms !== undefined) totalSec = ms / 1000;
                }
            }
        } catch {
            // Justification: optional timing-header probe — absent or malformed x-timing-* headers
            // are the expected common case (not an error); timing display simply stays unset.
        }
    }

    // 3. Fallback for totalSec if durationMs exists
    if (totalSec === undefined && durationMs !== undefined && durationMs > 0) {
        totalSec = durationMs / 1000;
    }

    // If we have neither totalSec nor any stages, return null
    if (totalSec === undefined && cleanSec === undefined && ttftSec === undefined) {
        return null;
    }

    const isOldRecordWithoutStages =
        cleanSec === undefined &&
        normSec === undefined &&
        thinkingSec === undefined &&
        ttftSec === undefined;

    return {
        cleanSec,
        normSec,
        thinkingSec,
        ttftSec,
        streamSec,
        totalSec,
        isOldRecordWithoutStages,
    };
};

export const formatSeconds = (sec?: number): string => {
    if (sec === undefined || sec === null || isNaN(sec)) return '-';
    if (sec < 0.001) {
        return `${sec.toFixed(4)}s`;
    }
    if (sec < 1) {
        return `${sec.toFixed(3)}s`;
    }
    return `${sec.toFixed(2)}s`;
};
