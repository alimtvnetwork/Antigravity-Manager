import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { showToast } from '../../common/ToastContainer';
import type { ExtractedPromptImage, PromptRewriteResult } from './types';

/**
 * Computes a fast deterministic 8-character hex hash for image data URI deduplication and filenames.
 */
export function computeImageHash(data: string): string {
    let hash = 0;
    for (let i = 0; i < Math.min(data.length, 16384); i++) {
        hash = ((hash << 5) - hash) + data.charCodeAt(i);
        hash |= 0;
    }
    return Math.abs(hash).toString(16).padStart(8, '0');
}

/**
 * Extracts inline Base64 data URIs and remote image URLs from markdown text.
 * Rewrites inline multi-megabyte Base64 payloads into compact relative attachment links
 * for safe CLI and IPC dispatch.
 */
export function extractPromptImages(text: string): PromptRewriteResult {
    if (!text) {
        return { rewrittenPrompt: '', hasImages: false, images: [] };
    }

    const images: ExtractedPromptImage[] = [];
    const imageRegex = /!\[(.*?)\]\((data:image\/([a-zA-Z0-9+.-]+);base64,[a-zA-Z0-9+/=]+|https?:\/\/[^\s)]+)\)/g;

    let rewritten = text;
    let match: RegExpExecArray | null;

    const matcher = new RegExp(imageRegex);
    while ((match = matcher.exec(text)) !== null) {
        const alt = match[1] || 'image';
        const sourceUri = match[2];
        const isBase64 = sourceUri.startsWith('data:image/');
        const mimeSubtype = match[3];

        let mimeType = 'image/png';
        let ext = 'png';

        if (isBase64) {
            mimeType = mimeSubtype ? `image/${mimeSubtype}` : 'image/png';
            ext = mimeSubtype ? mimeSubtype.replace('+xml', '') : 'png';
            if (ext === 'jpeg') ext = 'jpg';
        } else {
            const extMatch = sourceUri.match(/\.(png|jpe?g|webp|gif|svg)(?:\?.*)?$/i);
            if (extMatch) {
                ext = extMatch[1].toLowerCase();
                if (ext === 'jpg') ext = 'jpeg';
                mimeType = `image/${ext}`;
            }
        }

        const hash = computeImageHash(sourceUri);
        const byteLength = isBase64
            ? Math.round((sourceUri.length * 3) / 4)
            : sourceUri.length;

        images.push({
            alt,
            dataUri: sourceUri,
            mimeType,
            byteLength,
            hash,
        });

        // For inline Base64 data URIs, rewrite prompt to compact relative path
        if (isBase64) {
            const relativePath = `./.antigravity/attachments/attachment-${hash}.${ext}`;
            rewritten = rewritten.replace(match[0], `![${alt}](${relativePath})`);
        }
    }

    return {
        rewrittenPrompt: rewritten,
        hasImages: images.length > 0,
        images,
    };
}

/**
 * Re-exports rewriting helper for IPC dispatch workflows.
 */
export function rewritePromptForIpcDispatch(promptText: string): PromptRewriteResult {
    return extractPromptImages(promptText);
}

/**
 * Extracts embedded image URLs/data URIs and referenced file paths from prompt text.
 * Fixed regex ensures match[2] (the actual URL or data URI) is captured rather than match[1] (alt text).
 */
export function extractAssets(text: string): { images: string[]; files: string[] } {
    const images: string[] = [];
    const files: string[] = [];

    const imgRegex = /!\[(.*?)\]\((https?:\/\/.*?|data:image\/[^;]+;base64,[A-Za-z0-9+/=]+|[^\s)]+?\.(?:png|jpe?g|gif|webp|svg))\)/gi;
    let match: RegExpExecArray | null;
    while ((match = imgRegex.exec(text)) !== null) {
        if (match[2]) {
            images.push(match[2].trim());
        }
    }

    const fileRegex = /(?:file:\/\/\/|[a-zA-Z]:\\|\/)[^\s)"]+/g;
    let fileMatch: RegExpExecArray | null;
    while ((fileMatch = fileRegex.exec(text)) !== null) {
        if (!fileMatch[0].match(/\.(png|jpg|jpeg|gif|webp|svg)$/i)) {
            files.push(fileMatch[0]);
        } else {
            images.push(fileMatch[0]);
        }
    }

    return { images, files };
}

/**
 * Copies markdown prompt content and embedded images into system clipboard
 * simultaneously using dual MIME types ('text/html' and 'text/plain')
 * with a resilient tri-stage fallback ladder (Async Clipboard API -> DOM execCommand -> Native IPC).
 */
export async function copyPromptWithRichImages(
    markdownText: string,
    onSuccess?: (message: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    if (!markdownText || !markdownText.trim()) {
        return false;
    }

    if (!navigator.clipboard || typeof ClipboardItem === 'undefined') {
        const fallbackOk = await copyToClipboard(markdownText);
        if (fallbackOk) {
            const msg = 'Copied clean prompt text to clipboard (rich media unsupported in current context)';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        onError?.(new Error('Clipboard API unavailable'));
        return false;
    }

    try {
        // Extract embedded image URLs and format rich HTML
        let htmlBody = markdownText
            // Escape special HTML chars in text while preserving markdown tokens
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            // Formatted code blocks
            .replace(/```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g, (_m, lang, code) => {
                const langClass = lang ? ` class="language-${lang}"` : '';
                return `<pre style="background:#1e1e2e;color:#cdd6f4;padding:12px;border-radius:6px;font-family:monospace;overflow-x:auto;"><code${langClass}>${code}</code></pre>`;
            })
            // Inline code
            .replace(/`([^`]+)`/g, '<code style="background:#282a36;color:#50fa7b;padding:2px 4px;border-radius:3px;font-family:monospace;">$1</code>')
            // Markdown image syntax: ![alt](url)
            .replace(/!\[(.*?)\]\((.*?)\)/g, (_m, alt, src) => {
                const cleanSrc = src.trim();
                return `<div style="margin:12px 0;"><img src="${cleanSrc}" alt="${alt || 'image'}" style="max-width:100%;height:auto;border-radius:8px;box-shadow:0 2px 8px rgba(0,0,0,0.15);display:block;" /></div>`;
            })
            // Paragraphs and line breaks
            .replace(/\n\n+/g, '</p><p style="margin:8px 0;line-height:1.6;">')
            .replace(/\n/g, '<br/>');

        const htmlContent = `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#222;"><p style="margin:8px 0;line-height:1.6;">${htmlBody}</p></body></html>`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([markdownText], { type: 'text/plain' });

        const item = new ClipboardItem({
            'text/html': htmlBlob,
            'text/plain': textBlob,
        });

        await navigator.clipboard.write([item]);
        const successMsg = 'Copied full prompt with embedded rich images to clipboard!';
        onSuccess?.(successMsg);
        showToast({ message: successMsg, type: 'success' });
        return true;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'PromptTree.copyPromptWithRichImages',
            triggerAction: 'clipboard.write',
        });
        const fallbackOk = await copyToClipboard(markdownText);
        if (fallbackOk) {
            const msg = 'Copied clean prompt text to clipboard (rich media unsupported in current context)';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        onError?.(err);
        return false;
    }
}
