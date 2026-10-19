export function formatDualBadge(agmCode: string | undefined, defaultAgm: string, gmCode: string | undefined, defaultGm: string): string {
    const rawAgm = (agmCode || defaultAgm || '').replace(/^AGM:/i, '').replace(/[\[\]]/g, '').trim();
    const rawGm = (gmCode || defaultGm || '').replace(/^GM:/i, '').replace(/[\[\]]/g, '').trim();
    if (rawAgm && rawGm && rawAgm !== rawGm) {
        return `${rawAgm} · ${rawGm}`;
    }
    return rawGm || rawAgm || defaultAgm;
}

// Helper to count words

export function countWords(str: string): number {
    const trimmed = str.trim();
    if (!trimmed) return 0;
    return trimmed.split(/\s+/).length;
}


// Helper to truncate text at word limit while strictly preserving original line breaks and newlines
export function getTruncatedText(text: string, maxWords: number): { displayText: string; isTruncated: boolean; totalWords: number } {
    const trimmed = text.trim();
    if (!trimmed) return { displayText: '', isTruncated: false, totalWords: 0 };
    const allWords = trimmed.split(/\s+/).filter(Boolean);
    const totalWords = allWords.length;
    if (totalWords <= maxWords) {
        return { displayText: text, isTruncated: false, totalWords };
    }

    const lines = text.split('\n');
    const selectedLines: string[] = [];
    let wordsCollected = 0;

    for (const line of lines) {
        const lineWords = line.trim().split(/\s+/).filter(Boolean);
        if (lineWords.length === 0) {
            if (selectedLines.length > 0 && selectedLines[selectedLines.length - 1] !== '') {
                selectedLines.push('');
            }
            continue;
        }

        if (wordsCollected + lineWords.length <= maxWords) {
            selectedLines.push(line);
            wordsCollected += lineWords.length;
        } else {
            const remaining = maxWords - wordsCollected;
            if (remaining > 0) {
                selectedLines.push(lineWords.slice(0, remaining).join(' ') + ' ...');
            } else if (selectedLines.length > 0) {
                selectedLines[selectedLines.length - 1] = selectedLines[selectedLines.length - 1] + ' ...';
            }
            break;
        }
    }

    return {
        displayText: selectedLines.join('\n').trim(),
        isTruncated: true,
        totalWords,
    };
}

// Pre-formatter to ensure inline markdown headings and paragraph line gaps have explicit spacing

export function formatPromptForMarkdown(text: string): string {
    if (!text) return '';
    let formatted = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

    // Elevate inline <truncated ...> markers to standalone block callouts
    formatted = formatted.replace(/(<truncated\s+\d+\s+(?:bytes|lines)>)/gi, '\n\n$1\n\n');

    // Separate inline markdown headings attached to paragraph text:
    // e.g. "# High Priority Instruction Hi there." -> "# High Priority Instruction\n\nHi there."
    formatted = formatted.replace(
        /^(#{1,4}\s+[A-Za-z0-9_\-\s]{2,40}?)([\.\:\!\?])\s+([A-Z])/gm,
        '$1$2\n\n$3'
    );
    formatted = formatted.replace(
        /^(#{1,4}\s+High Priority Instruction|#{1,4}\s+Instruction|#{1,4}\s+Overview|#{1,4}\s+Notice|#{1,4}\s+Task|#{1,4}\s+Plan)\s+([A-Z])/gm,
        '$1\n\n$2'
    );

    return formatted;
}


export function formatCleanSeqCode(code: string | undefined | null, prefix: 'P' | 'C' = 'C'): string {
    if (!code) return prefix === 'P' ? '#P001' : 'C001';
    const stripped = code.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
    if (!stripped) return prefix === 'P' ? '#P001' : 'C001';
    if (prefix === 'P') {
        const withP = stripped.startsWith('P') || stripped.startsWith('#P') ? stripped : `P${stripped}`;
        return withP.startsWith('#') ? withP : `#${withP}`;
    }
    const withoutHash = stripped.replace(/^#/, '');
    return withoutHash.startsWith('C') ? withoutHash : `C${withoutHash}`;
}


export function formatSeqBadge(raw: string | undefined | null, fallback: string): string {
    if (!raw) return fallback;
    const clean = raw.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
    if (!clean) return fallback;
    if (fallback.startsWith('#P') || fallback === '#P001') {
        const withP = clean.startsWith('P') || clean.startsWith('#P') ? clean : `P${clean}`;
        return withP.startsWith('#') ? withP : `#${withP}`;
    }
    if (fallback.startsWith('C') || fallback === 'C001') {
        const withoutHash = clean.replace(/^#/, '');
        return withoutHash.startsWith('C') ? withoutHash : `C${withoutHash}`;
    }
    return clean.startsWith('#') || clean.startsWith('P') || clean.startsWith('C') ? clean : `#${clean}`;
}

// Helper to check if a conversation has zero prompt content and untitled title (true ghost node)

export function stripImagesFromPrompt(text: string): string {
    return text
        .replace(/!\[.*?\]\((?:https?:\/\/.*?|data:image\/.*?;base64,.*?|[^\s)]+)\)/g, '')
        .trim();
}

export interface RelativeRecency {
    label: string;
    dotClass: string;
    textClass: string;
    formattedTime: string;
}

/**
 * Computes human-friendly relative recency label (now, 30m, 6h, 12h, 1d) matching
 * Antigravity IDE history, along with glowing status dots and full timestamp tooltip.
 */
export function formatRelativeRecency(
    lastModified: string | number | undefined | null,
    isRunning?: boolean,
    isQueued?: boolean
): RelativeRecency {
    if (isRunning) {
        return {
            label: 'now',
            dotClass: 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse',
            textClass: 'text-emerald-300 font-semibold',
            formattedTime: 'Running now',
        };
    }
    if (isQueued) {
        return {
            label: 'queued',
            dotClass: 'bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.8)] animate-pulse',
            textClass: 'text-amber-300 font-semibold',
            formattedTime: 'Queued',
        };
    }

    if (!lastModified) {
        return {
            label: 'idle',
            dotClass: 'bg-zinc-500/60',
            textClass: 'text-zinc-400',
            formattedTime: 'Idle',
        };
    }

    let timestampMs = 0;
    if (typeof lastModified === 'number') {
        timestampMs = lastModified < 100_000_000_000 ? lastModified * 1000 : lastModified;
    } else {
        const num = Number(lastModified);
        if (!isNaN(num) && num > 0) {
            timestampMs = num < 100_000_000_000 ? num * 1000 : num;
        } else {
            const parsed = new Date(lastModified).getTime();
            if (!isNaN(parsed) && parsed > 0) {
                timestampMs = parsed;
            }
        }
    }

    if (timestampMs === 0) {
        return {
            label: 'idle',
            dotClass: 'bg-zinc-500/60',
            textClass: 'text-zinc-400',
            formattedTime: 'Idle',
        };
    }

    const now = Date.now();
    const diffMs = Math.max(0, now - timestampMs);
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHours = Math.floor(diffMin / 60);
    const diffDays = Math.floor(diffHours / 24);

    let label: string;
    let dotClass: string;
    let textClass: string;

    if (diffMin < 2) {
        label = 'now';
        dotClass = 'bg-emerald-400/80';
        textClass = 'text-emerald-300';
    } else if (diffMin < 60) {
        label = `${diffMin}m`;
        dotClass = diffMin <= 30 ? 'bg-sky-400/80' : 'bg-cyan-500/70';
        textClass = 'text-sky-300';
    } else if (diffHours < 24) {
        label = `${diffHours}h`;
        dotClass = diffHours <= 6 ? 'bg-indigo-400/70' : 'bg-purple-400/60';
        textClass = 'text-indigo-300';
    } else if (diffDays < 7) {
        label = `${diffDays}d`;
        dotClass = 'bg-zinc-500/60';
        textClass = 'text-zinc-400';
    } else {
        const weeks = Math.floor(diffDays / 7);
        label = `${weeks}w`;
        dotClass = 'bg-zinc-600/50';
        textClass = 'text-zinc-500';
    }

    const d = new Date(timestampMs);
    const formattedTime = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;

    return { label, dotClass, textClass, formattedTime };
}


// Helper for rich clipboard media copying with ClipboardItem
