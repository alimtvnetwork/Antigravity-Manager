/**
 * Utility functions for normalizing, extracting, and formatting Antigravity IDE
 * multimodal prompt attachments in the canonical bracketed format:
 *
 * Attachments:
 * 1. [Attachment 01 | Description | filename.png](file;path/to/filename.png)
 * 2. [Attachment 02 | Description | filename2.png](file;path/to/filename2.png)
 */

export type PromptAttachmentInput =
    | string
    | {
          path?: string;
          src?: string;
          dataUri?: string;
      };

export interface PromptAttachmentFormatOptions {
    repoPath?: string;
    isUserRequestTagPreserved?: boolean;
    isInlineImageExtractionEnabled?: boolean;
    isWindowsPlatformOverride?: boolean;
}

function isWindowsPlatform(
    isWindowsPlatformOverride?: boolean,
    candidatePath?: string,
    repoPath?: string
): boolean {
    if (isWindowsPlatformOverride !== undefined) {
        return isWindowsPlatformOverride;
    }
    const combined = `${candidatePath || ''} ${repoPath || ''}`;
    if (/^[a-zA-Z]:[\\/]/.test(combined) || combined.includes('\\') || /^\/[a-zA-Z]:[\\/]/.test(combined)) {
        return true;
    }
    const nodeProcess = typeof globalThis !== 'undefined'
        ? (globalThis as unknown as { process?: { platform?: string } }).process
        : undefined;
    if (nodeProcess?.platform === 'win32') {
        return true;
    }
    if (typeof navigator !== 'undefined') {
        return /win/i.test(navigator.userAgent || (navigator as unknown as { platform?: string }).platform || '');
    }
    return false;
}

function stripProtocolPrefix(raw: string): string {
    let clean = raw.trim();
    if (/^file;\s*/i.test(clean)) {
        clean = clean.replace(/^file;\s*/i, '');
    }
    if (clean.startsWith('agm-asset://')) {
        clean = clean.slice('agm-asset://'.length);
    }
    if (clean.startsWith('http://asset.localhost/')) {
        clean = clean.slice('http://asset.localhost/'.length);
    }
    if (clean.startsWith('https://asset.localhost/')) {
        clean = clean.slice('https://asset.localhost/'.length);
    }
    if (/^file:\/\/\/[a-zA-Z]:[\\/]/.test(clean)) {
        return clean.slice(8);
    }
    if (clean.startsWith('file:///')) {
        return clean.slice(7);
    }
    if (clean.startsWith('file://')) {
        return clean.slice(7);
    }
    return clean;
}

function safeDecodeUri(raw: string): string {
    try {
        return decodeURIComponent(raw);
    } catch {
        return raw;
    }
}

function resolveRelativePath(
    filePath: string,
    repoPath?: string,
    isWindowsTarget = false
): string {
    const isAbsolute =
        /^[a-zA-Z]:[\\/]/.test(filePath) ||
        filePath.startsWith('/') ||
        filePath.startsWith('\\\\');
    if (isAbsolute || !repoPath || repoPath.trim().length === 0) {
        return filePath;
    }
    const sep = isWindowsTarget ? '\\' : '/';
    const normRepo = repoPath.replace(/[/\\]+$/, '');
    const normChild = filePath.replace(/^(\.[/\\]|[/\\])+/, '');
    return `${normRepo}${sep}${normChild}`;
}

export function normalizeAttachmentPath(
    rawPathOrUri: string,
    repoPath?: string,
    isWindowsPlatformOverride?: boolean
): string {
    if (!rawPathOrUri || !rawPathOrUri.trim()) {
        return '';
    }
    let clean = stripProtocolPrefix(rawPathOrUri);
    if (/^file;\s*/i.test(clean)) {
        clean = clean.replace(/^file;\s*/i, '');
    }
    clean = safeDecodeUri(clean);
    clean = clean.replace(/^[\\/]+\?[\\/]+/, '');
    if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
        clean = clean.slice(1);
    }
    if (clean.startsWith('data:') || clean.startsWith('blob:')) {
        return '';
    }
    if (clean.startsWith('http://') || clean.startsWith('https://')) {
        return clean;
    }
    const isWindowsTarget = isWindowsPlatform(isWindowsPlatformOverride, clean, repoPath);
    clean = resolveRelativePath(clean, repoPath, isWindowsTarget);
    return isWindowsTarget ? clean.replace(/\//g, '\\') : clean.replace(/\\/g, '/');
}

function cleanPromptWhitespace(text: string): string {
    const match = /<\/USER_REQUEST>/i.exec(text);
    if (!match) {
        return text.trim();
    }
    const idx = match.index;
    const tag = match[0];
    const before = text.slice(0, idx).trimEnd();
    const after = text.slice(idx + tag.length).trim();
    const afterSuffix = after.length > 0 ? `\n${after}` : '';
    return `${before}\n${tag}${afterSuffix}`.trimStart();
}

export function stripAttachmentsFromPrompt(prompt: string): string {
    if (!prompt) {
        return '';
    }
    const withoutLegacyImages = prompt
        .replace(/!\[.*?\]\((?:data:image\/[^;]+;base64,[A-Za-z0-9+/=]+|https?:\/\/[^\s)]+|[^\s)]+?)\)/gi, '')
        .replace(/!\[.*?\]\(.*?\)/gi, '');
    const withoutAttachments = withoutLegacyImages.replace(
        /(?:\r?\n)*\bAttachments:\s*(?:\r?\n\s*\d+\.\s*(?:@?\[[^\]]*\]\s*\(file;[^)]+\)|@\[(?:file;\s*)?[^\]]+\]))+/gi,
        ''
    );
    const collapsed = withoutAttachments.replace(/\n{3,}/g, '\n\n');
    return cleanPromptWhitespace(collapsed);
}

const SKILL_DIR_PATTERN = /(^|[\\/])\.agents[\\/]skills([\\/]|$)/i;
const FILE_BASENAME_EXT_PATTERN = /\.[a-zA-Z0-9]{1,8}$/;
const IMAGE_BASENAME_PATTERN = /\.(png|jpe?g|gif|webp|svg|bmp|ico|avif|jfif|tiff?)$/i;

/**
 * True for real file/image attachment targets. False for skill refs such as
 * `@[.agents/skills/foo]` or `@[foo](file;.agents/skills/foo)` so copy/send
 * does not rewrite them into an Attachments block.
 */
export function isFileAttachmentTarget(raw: string): boolean {
    if (!raw || !raw.trim()) {
        return false;
    }
    let clean = raw.trim();
    if (/^file;\s*/i.test(clean)) {
        clean = clean.replace(/^file;\s*/i, '').trim();
    }
    if (!clean) {
        return false;
    }
    if (clean.startsWith('data:image/') || clean.startsWith('blob:')) {
        return true;
    }
    if (SKILL_DIR_PATTERN.test(clean)) {
        return false;
    }
    const withoutQuery = clean.split(/[?#]/)[0].replace(/[/\\]+$/, '');
    const basename = withoutQuery.split(/[/\\]/).pop() || '';
    if (!basename || basename === '.' || basename === '..') {
        return false;
    }
    if (IMAGE_BASENAME_PATTERN.test(basename)) {
        return true;
    }
    if (withoutQuery.includes('.user_uploaded') && /^media_|^screenshot_|^img_/i.test(basename)) {
        return true;
    }
    return FILE_BASENAME_EXT_PATTERN.test(basename);
}

function isEligibleAttachmentTarget(raw: string): boolean {
    if (!raw || !raw.trim()) {
        return false;
    }
    const clean = raw.trim();
    if (clean.startsWith('data:') || clean.startsWith('blob:')) {
        return false;
    }
    const isExternalWeb =
        (clean.startsWith('http://') || clean.startsWith('https://')) &&
        !clean.includes('asset.localhost');
    return !isExternalWeb && isFileAttachmentTarget(clean);
}

function deduplicatePaths(paths: string[], isWindows: boolean): string[] {
    const seen = new Set<string>();
    const result: string[] = [];
    for (const p of paths) {
        if (!p || p.trim().length === 0) {
            continue;
        }
        const key = isWindows ? p.toLowerCase() : p;
        if (!seen.has(key)) {
            seen.add(key);
            result.push(p);
        }
    }
    return result;
}

function collectPromptAttachmentRawTargets(prompt: string): string[] {
    const matches: { index: number; raw: string }[] = [];

    // 1. Matches canonical 3-tier syntax [label](file;<path>) or @[label](file;<path>)
    const fileLinkRegex = /@?\[[^\]]*\]\s*\(file;\s*([^)]+)\)/gi;
    let fileLinkMatch: RegExpExecArray | null;
    while ((fileLinkMatch = fileLinkRegex.exec(prompt)) !== null) {
        if (fileLinkMatch[1]) {
            const target = fileLinkMatch[1].trim();
            if (target) {
                matches.push({ index: fileLinkMatch.index, raw: target });
            }
        }
    }

    // 2. Matches bracketed syntax @[file;<path>] or @[<path>] (excluding when followed by (file;...))
    const bracketRegex = /@\[(?:file;\s*)?([^\]]+)\](?!\s*\(file;)/gi;
    let bracketMatch: RegExpExecArray | null;
    while ((bracketMatch = bracketRegex.exec(prompt)) !== null) {
        if (bracketMatch[1]) {
            let target = bracketMatch[1].trim();
            if (/^file;\s*/i.test(target)) {
                target = target.replace(/^file;\s*/i, '');
            }
            if (target) {
                matches.push({ index: bracketMatch.index, raw: target });
            }
        }
    }

    // 3. Matches markdown images ![...](<path>)
    const mdRegex = /!\[.*?\]\((.*?)\)/g;
    let mdMatch: RegExpExecArray | null;
    while ((mdMatch = mdRegex.exec(prompt)) !== null) {
        if (mdMatch[1]) {
            matches.push({ index: mdMatch.index, raw: mdMatch[1].trim() });
        }
    }
    matches.sort((a, b) => a.index - b.index);
    return matches.map((m) => m.raw);
}

export function extractAttachmentsFromPrompt(
    prompt: string,
    repoPath?: string,
    isWindowsPlatformOverride?: boolean
): string[] {
    if (!prompt) {
        return [];
    }
    const rawTargets = collectPromptAttachmentRawTargets(prompt);
    const normalized: string[] = [];
    for (const raw of rawTargets) {
        if (!isEligibleAttachmentTarget(raw)) {
            continue;
        }
        const norm = normalizeAttachmentPath(raw, repoPath, isWindowsPlatformOverride);
        if (norm && norm.length > 0) {
            normalized.push(norm);
        }
    }
    const isWindows = isWindowsPlatform(isWindowsPlatformOverride, undefined, repoPath);
    return deduplicatePaths(normalized, isWindows);
}

function extractRawTarget(input: PromptAttachmentInput): string {
    if (typeof input === 'string') {
        const fileLinkMatch = /@?\[[^\]]*\]\s*\(file;\s*([^)]+)\)/i.exec(input);
        if (fileLinkMatch && fileLinkMatch[1]) {
            return fileLinkMatch[1].trim();
        }
        const bracketMatch = /@\[(?:file;\s*)?(.*?)\]/i.exec(input);
        if (bracketMatch) {
            let target = bracketMatch[1].trim();
            if (/^file;\s*/i.test(target)) {
                target = target.replace(/^file;\s*/i, '');
            }
            return target;
        }
        const mdMatch = /!\[.*?\]\((.*?)\)/.exec(input);
        if (mdMatch) {
            return mdMatch[1].trim();
        }
        let clean = input.trim();
        if (/^file;\s*/i.test(clean)) {
            clean = clean.replace(/^file;\s*/i, '');
        }
        return clean;
    }
    if (input && typeof input === 'object') {
        const val = input.path || input.src || input.dataUri || '';
        let clean = val.trim();
        if (/^file;\s*/i.test(clean)) {
            clean = clean.replace(/^file;\s*/i, '');
        }
        return clean;
    }
    return '';
}

function cleanLegacyMarkdownImages(prompt: string): string {
    if (!prompt) {
        return '';
    }
    return prompt.replace(/!\[(.*?)\]\((.*?)\)/g, (fullMatch, alt, src) => {
        const cleanSrc = src.trim();
        const isExternal =
            (cleanSrc.startsWith('http://') || cleanSrc.startsWith('https://')) &&
            !cleanSrc.includes('asset.localhost');
        const isAttachedImageAlt = /^Attached Image \d+$/i.test(alt.trim());
        if (isExternal && !isAttachedImageAlt) {
            return fullMatch;
        }
        return '';
    });
}

/**
 * Formats an individual attachment item into canonical 3-tier sequence, description, and filename label:
 * [Attachment XX | Description | filename.png]
 */
export function formatAttachmentLabel(path: string, index: number, customDesc?: string): string {
    const seq = String(index + 1).padStart(2, '0');
    const normalized = path.replace(/\\/g, '/');
    const filename = normalized.split('/').pop() || `image_${seq}.png`;

    let desc = customDesc?.trim();
    if (!desc) {
        if (/^(?:media|screenshot|img)_\d+/i.test(filename)) {
            desc = 'Screenshot Attachment';
        } else {
            const rawName = filename.replace(/\.[^.]+$/, '');
            const words = rawName
                .replace(/[-_]+/g, ' ')
                .replace(/\s+/g, ' ')
                .trim()
                .split(' ')
                .filter(Boolean)
                .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
                .join(' ');
            if (!words) {
                desc = 'Screenshot Attachment';
            } else if (/screenshot/i.test(words)) {
                desc = words;
            } else {
                desc = `${words} Screenshot`;
            }
        }
    }

    return `[Attachment ${seq} | ${desc} | ${filename}]`;
}

export function buildAttachmentsBlock(paths: string[]): string {
    const lines = paths.map((p, idx) => `${idx + 1}. ${formatAttachmentLabel(p, idx)}(file;${p})`);
    return `Attachments:\n${lines.join('\n')}`;
}

function injectAttachmentsIntoPrompt(
    cleanedPrompt: string,
    attachmentsBlock: string,
    isUserRequestTagPreserved: boolean
): string {
    const match = /<\/USER_REQUEST>/i.exec(cleanedPrompt);
    if (isUserRequestTagPreserved && match) {
        const idx = match.index;
        const tag = match[0];
        const before = cleanedPrompt.slice(0, idx).trimEnd();
        const after = cleanedPrompt.slice(idx + tag.length);
        const prefix = before.length > 0 ? `${before}\n\n` : '';
        return `${prefix}${attachmentsBlock}\n${tag}${after}`;
    }
    if (cleanedPrompt.length === 0) {
        return attachmentsBlock;
    }
    return `${cleanedPrompt.trimEnd()}\n\n${attachmentsBlock}`;
}

export function formatPromptWithAttachments(
    prompt: string,
    attachments: PromptAttachmentInput[],
    options?: PromptAttachmentFormatOptions
): string {
    const isUserRequestTagPreserved = options?.isUserRequestTagPreserved ?? true;
    const isInlineImageExtractionEnabled = options?.isInlineImageExtractionEnabled ?? true;
    const isWindows = isWindowsPlatform(options?.isWindowsPlatformOverride, undefined, options?.repoPath);

    const extractedFromPrompt = isInlineImageExtractionEnabled
        ? extractAttachmentsFromPrompt(prompt, options?.repoPath, options?.isWindowsPlatformOverride)
        : [];

    const providedNormalized: string[] = [];
    for (const item of attachments) {
        const raw = extractRawTarget(item);
        if (!isEligibleAttachmentTarget(raw)) {
            continue;
        }
        const norm = normalizeAttachmentPath(raw, options?.repoPath, options?.isWindowsPlatformOverride);
        if (norm && norm.length > 0) {
            providedNormalized.push(norm);
        }
    }

    const allPaths = deduplicatePaths([...extractedFromPrompt, ...providedNormalized], isWindows);

    const strippedLegacy = cleanLegacyMarkdownImages(prompt);
    const withoutOldBlocks = strippedLegacy.replace(
        /(?:\r?\n)*\bAttachments:\s*(?:\r?\n\s*\d+\.\s*(?:@?\[[^\]]*\]\s*\(file;[^)]+\)|@\[(?:file;\s*)?[^\]]+\]))+/gi,
        ''
    );
    const cleanedPrompt = cleanPromptWhitespace(withoutOldBlocks.replace(/\n{3,}/g, '\n\n'));

    if (allPaths.length === 0) {
        return cleanedPrompt;
    }

    const attachmentsBlock = buildAttachmentsBlock(allPaths);
    return injectAttachmentsIntoPrompt(cleanedPrompt, attachmentsBlock, isUserRequestTagPreserved);
}
