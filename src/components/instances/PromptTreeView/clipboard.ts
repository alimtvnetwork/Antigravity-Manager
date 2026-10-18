import { convertFileSrc } from '@tauri-apps/api/core';
import { copyToClipboard, ensureDocumentFocus } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { showToast } from '../../common/ToastContainer';
import type { ExtractedPromptImage, PromptRewriteResult } from './types';

/**
 * Strips all markdown image tags (![alt](url/dataUri)) and image references from prompt text,
 * returning clean text suitable for text-only copying and tokens counting.
 */
export function stripImagesFromPrompt(text: string): string {
    if (!text) return '';
    return text
        .replace(/!\[.*?\]\((?:data:image\/[^;]+;base64,[A-Za-z0-9+/=]+|https?:\/\/[^\s)]+|[^\s)]+?\.(?:png|jpe?g|gif|webp|svg))\)/gi, '')
        .replace(/!\[.*?\]\(.*?\)/gi, '')
        .replace(/\n{3,}/g, '\n\n')
        .trim();
}

/**
 * Resolves an image source URI for safe WebView rendering.
 * Handles data URIs, http(s) URLs, file:// protocols, relative project paths, and native absolute paths.
 * Uses Tauri's convertFileSrc when in Tauri environment to bypass WebView CORS and local file restrictions.
 */
export function resolveImageSrc(src: string, repoPath?: string): string {
    if (!src || !src.trim()) return '';
    const trimmed = src.trim();

    // Already a web url or data uri or blob uri
    if (trimmed.startsWith('data:') || trimmed.startsWith('http://') || trimmed.startsWith('https://') || trimmed.startsWith('blob:')) {
        return trimmed;
    }

    // Handle file:/// or file://
    let cleanPath = trimmed;
    if (cleanPath.startsWith('file:///')) {
        cleanPath = cleanPath.slice(7);
        // On Windows with drive letter, e.g. '/C:/...' or '/D:/...', strip leading slash
        if (/^\/[a-zA-Z]:[\\/]/.test(cleanPath)) {
            cleanPath = cleanPath.slice(1);
        }
    } else if (cleanPath.startsWith('file://')) {
        cleanPath = cleanPath.slice(7);
    }

    try {
        cleanPath = decodeURIComponent(cleanPath);
    } catch {
        // Keep raw cleanPath if decoding fails
    }

    // If cleanPath is relative and repoPath is provided, resolve against repoPath
    const isAbsolute = /^[a-zA-Z]:[\\/]/.test(cleanPath) || cleanPath.startsWith('/') || cleanPath.startsWith('\\\\');
    if (!isAbsolute && repoPath && repoPath.trim().length > 0) {
        const sep = repoPath.includes('/') ? '/' : '\\';
        const normRepo = repoPath.replace(/[/\\]+$/, '');
        const normChild = cleanPath.replace(/^[./\\]+/, '');
        cleanPath = `${normRepo}${sep}${normChild}`;
    }

    try {
        if (typeof window !== 'undefined' && (window as unknown as { __TAURI_INTERNALS__?: { convertFileSrc?: unknown } }).__TAURI_INTERNALS__?.convertFileSrc) {
            return convertFileSrc(cleanPath);
        }
    } catch {
        // Fallback to original or cleanPath
    }

    return cleanPath;
}

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
 * Converts a base64 string to a Blob.
 */
export function base64ToBlob(base64Data: string, mimeType = 'image/png'): Blob {
    const byteString = atob(base64Data);
    const arrayBuffer = new ArrayBuffer(byteString.length);
    const uint8Array = new Uint8Array(arrayBuffer);
    for (let i = 0; i < byteString.length; i++) {
        uint8Array[i] = byteString.charCodeAt(i);
    }
    return new Blob([uint8Array], { type: mimeType });
}

/**
 * Converts an image URI (Base64 data URI, HTTP URL, file path, or blob URI)
 * into a full Base64 data URI (data:image/png;base64,...) for embedding into HTML.
 */
export async function convertImageUriToBase64(imageUri: string): Promise<string | null> {
    try {
        if (!imageUri || !imageUri.trim()) return null;
        const cleanUri = imageUri.trim();

        // Already a base64 data URI
        if (cleanUri.startsWith('data:image/')) {
            return cleanUri;
        }

        const resolvedSrc = resolveImageSrc(cleanUri);
        return await new Promise<string | null>((resolve) => {
            const timeout = setTimeout(() => resolve(null), 5000);
            const img = new Image();
            img.crossOrigin = 'anonymous';
            img.onload = () => {
                clearTimeout(timeout);
                try {
                    const canvas = document.createElement('canvas');
                    canvas.width = img.naturalWidth || img.width || 300;
                    canvas.height = img.naturalHeight || img.height || 150;
                    const ctx = canvas.getContext('2d');
                    if (!ctx) {
                        resolve(null);
                        return;
                    }
                    ctx.drawImage(img, 0, 0);
                    const dataUrl = canvas.toDataURL('image/png');
                    resolve(dataUrl);
                } catch {
                    resolve(null);
                }
            };
            img.onerror = () => {
                clearTimeout(timeout);
                resolve(null);
            };
            img.src = resolvedSrc;
        });
    } catch {
        return null;
    }
}

/**
 * Converts an image URI (Base64 data URI, HTTP URL, or local file/blob URI)
 * into a true PNG binary Blob (image/png) for native clipboard insertion.
 */
export async function convertImageUriToPngBlob(imageUri: string): Promise<Blob | null> {
    try {
        if (!imageUri || !imageUri.trim()) return null;
        const cleanUri = imageUri.trim();

        // 1. If it's already a PNG data URI, decode base64 directly to binary blob
        if (cleanUri.startsWith('data:image/png;base64,')) {
            const base64 = cleanUri.slice('data:image/png;base64,'.length);
            return base64ToBlob(base64, 'image/png');
        }

        const resolvedSrc = resolveImageSrc(cleanUri);

        // 2. Offscreen rendering to HTMLCanvasElement to guarantee standard PNG binary encoding
        return await new Promise<Blob | null>((resolve) => {
            const timeout = setTimeout(() => resolve(null), 5000);
            const img = new Image();
            img.crossOrigin = 'anonymous';
            img.onload = () => {
                clearTimeout(timeout);
                try {
                    const canvas = document.createElement('canvas');
                    canvas.width = img.naturalWidth || img.width || 300;
                    canvas.height = img.naturalHeight || img.height || 150;
                    const ctx = canvas.getContext('2d');
                    if (!ctx) {
                        resolve(null);
                        return;
                    }
                    ctx.drawImage(img, 0, 0);
                    canvas.toBlob((blob) => {
                        resolve(blob);
                    }, 'image/png');
                } catch {
                    resolve(null);
                }
            };
            img.onerror = () => {
                clearTimeout(timeout);
                resolve(null);
            };
            img.src = resolvedSrc;
        });
    } catch {
        return null;
    }
}

/**
 * Writes a single image directly to the system clipboard as a native binary PNG (image/png).
 * This enables pasting directly into apps like Gemini Web, Photoshop, Paint, Discord, etc.
 */
export async function writeNativeImageToClipboard(
    imageUri: string,
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    try {
        if (!imageUri || !imageUri.trim()) {
            throw new Error('No image URI provided to copy');
        }
        const pngBlob = await convertImageUriToPngBlob(imageUri);
        if (!pngBlob) {
            throw new Error('Failed to convert image to PNG binary blob');
        }
        ensureDocumentFocus();
        const hasFocus = typeof document !== 'undefined' && typeof document.hasFocus === 'function' ? document.hasFocus() : true;
        if (hasFocus && navigator.clipboard && typeof ClipboardItem !== 'undefined') {
            try {
                const item = new ClipboardItem({
                    'image/png': pngBlob,
                });
                await navigator.clipboard.write([item]);
                const msg = 'Image copied to clipboard as native PNG!';
                onSuccess?.(msg);
                showToast({ message: msg, type: 'success' });
                return true;
            } catch {
                // Focus was lost or binary write rejected; fall through to copyToClipboard
            }
        }
        const fallbackOk = await copyToClipboard(imageUri);
        if (fallbackOk) {
            const msg = 'Copied image URI to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        return false;
    } catch (err) {
        // Fallback to copying image URI / reference via multi-stage clipboard
        const fallbackOk = await copyToClipboard(imageUri);
        if (fallbackOk) {
            const msg = 'Copied image URI to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.writeNativeImageToClipboard',
            triggerAction: 'write_native_image',
        });
        onError?.(err);
        return false;
    }
}

/**
 * Copies markdown prompt content and embedded images into system clipboard
 * simultaneously using triple/dual MIME types ('image/png', 'text/html', and 'text/plain')
 * with a resilient tri-stage fallback ladder (Async Clipboard API -> DOM execCommand -> Native IPC).
 */
/**
 * Copies markdown prompt content and embedded images into system clipboard
 * simultaneously using triple/dual MIME types ('image/png', 'text/html', and 'text/plain')
 * with embedded Base64 <img> tags so pasting into Microsoft OneNote, Word, or external IDEs
 * correctly retains and renders the images without broken external dependencies.
 */
export async function copyPromptWithRichImages(
    markdownText: string,
    onSuccess?: (message: string) => void,
    onError?: (err: unknown) => void,
    mediaPaths?: string[]
): Promise<boolean> {
    if (!markdownText || !markdownText.trim()) {
        return false;
    }

    // Gather turn media paths not already in markdown
    let effectiveMarkdown = markdownText;
    if (mediaPaths && mediaPaths.length > 0) {
        const missingPaths = mediaPaths.filter((p) => p && !effectiveMarkdown.includes(p));
        if (missingPaths.length > 0) {
            effectiveMarkdown += '\n\n' + missingPaths.map((p, idx) => `![Turn Image ${idx + 1}](${p})`).join('\n\n');
        }
    }

    if (!navigator.clipboard || typeof ClipboardItem === 'undefined') {
        const fallbackOk = await copyToClipboard(effectiveMarkdown);
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
        // Pre-convert any non-base64 image sources to embedded Base64 data URIs for OneNote/Word
        const imageMatches = Array.from(effectiveMarkdown.matchAll(/!\[(.*?)\]\((.*?)\)/g));
        const srcMap = new Map<string, string>();
        for (const match of imageMatches) {
            const rawSrc = match[2].trim();
            if (rawSrc.startsWith('data:image/')) {
                srcMap.set(rawSrc, rawSrc);
            } else if (!srcMap.has(rawSrc)) {
                const base64Uri = await convertImageUriToBase64(rawSrc);
                srcMap.set(rawSrc, base64Uri || rawSrc);
            }
        }

        // Extract embedded image URLs and format rich HTML with embedded Base64 <img> tags
        let htmlBody = effectiveMarkdown
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
            // Markdown image syntax: ![alt](url) -> formatted paragraph with embedded Base64 image
            .replace(/!\[(.*?)\]\((.*?)\)/g, (_m, alt, src) => {
                const cleanSrc = src.trim();
                const resolvedSrc = srcMap.get(cleanSrc) || cleanSrc;
                return `<p style="margin:12px 0;"><img src="${resolvedSrc}" alt="${alt || 'image'}" style="max-width:100%;height:auto;border-radius:6px;display:block;box-shadow:0 2px 8px rgba(0,0,0,0.15);" /></p>`;
            })
            // Paragraphs and line breaks
            .replace(/\n\n+/g, '</p><p style="margin:8px 0;line-height:1.6;">')
            .replace(/\n/g, '<br/>');

        const htmlContent = `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#222;"><p style="margin:8px 0;line-height:1.6;">${htmlBody}</p></body></html>`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([effectiveMarkdown], { type: 'text/plain' });

        // Extract first image to attach true binary image/png alongside html and text
        const { images } = extractAssets(effectiveMarkdown);
        let pngBlob: Blob | null = null;
        if (images.length > 0) {
            const firstImg = images[0];
            const base64First = srcMap.get(firstImg) || firstImg;
            pngBlob = await convertImageUriToPngBlob(base64First);
        }

        let isWritten = false;
        ensureDocumentFocus();
        const hasFocus = typeof document !== 'undefined' && typeof document.hasFocus === 'function' ? document.hasFocus() : true;
        if (hasFocus && navigator.clipboard && typeof ClipboardItem !== 'undefined') {
            if (pngBlob) {
                try {
                    const item = new ClipboardItem({
                        'image/png': pngBlob,
                        'text/html': htmlBlob,
                        'text/plain': textBlob,
                    });
                    await navigator.clipboard.write([item]);
                    isWritten = true;
                } catch {
                    // Triple MIME write failed (some browser versions reject mixed binary + rich HTML items)
                    // Fall through to dual HTML/plain-text item
                }
            }

            if (!isWritten) {
                try {
                    const item = new ClipboardItem({
                        'text/html': htmlBlob,
                        'text/plain': textBlob,
                    });
                    await navigator.clipboard.write([item]);
                    isWritten = true;
                } catch {
                    // Binary clipboard write failed or document focus was lost
                }
            }
        }

        if (isWritten) {
            const successMsg = 'Copied full prompt with embedded rich images to clipboard!';
            onSuccess?.(successMsg);
            showToast({ message: successMsg, type: 'success' });
            return true;
        }

        const fallbackOk = await copyToClipboard(effectiveMarkdown);
        if (fallbackOk) {
            const msg = 'Copied clean prompt text to clipboard (rich media unsupported in current context)';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        return false;
    } catch (err) {
        // Fallback to resilient text copy ladder (Clipboard API -> execCommand -> native helper)
        const fallbackOk = await copyToClipboard(effectiveMarkdown);
        if (fallbackOk) {
            const msg = 'Copied clean prompt text to clipboard (rich media unsupported in current context)';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        useErrorStore.getState().trackWarning(err, {
            source: 'PromptTree.copyPromptWithRichImages',
            triggerAction: 'clipboard.write',
        });
        onError?.(err);
        return false;
    }
}

/**
 * Copies prompt images or a single image as clean Markdown text
 * (![Image](data:image/png;base64,...) or relative path text) so it can be pasted as clean text anywhere.
 */
export async function copyImageAsMarkdownText(
    sourceTextOrUri: string,
    altText = 'Image',
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    if (!sourceTextOrUri || !sourceTextOrUri.trim()) {
        return false;
    }

    try {
        const trimmed = sourceTextOrUri.trim();
        let markdownOutput = '';

        // Check if input is a prompt containing markdown image tags (![alt](src))
        const mdImageMatches = Array.from(trimmed.matchAll(/!\[(.*?)\]\((.*?)\)/g));
        if (mdImageMatches.length > 0) {
            markdownOutput = mdImageMatches.map((m) => m[0]).join('\n\n');
        } else {
            // Check if input contains referenced image assets
            const { images } = extractAssets(trimmed);
            if (images.length > 0) {
                markdownOutput = images
                    .map((img, idx) => `![${altText}${images.length > 1 ? ` ${idx + 1}` : ''}](${img})`)
                    .join('\n\n');
            } else {
                // Input is a direct image URI or path
                markdownOutput = `![${altText}](${trimmed})`;
            }
        }

        const success = await copyToClipboard(markdownOutput);
        if (success) {
            const msg = 'Copied image as Markdown text!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyImageAsMarkdownText',
            triggerAction: 'copy_image_as_markdown_text',
        });
        onError?.(err);
        return false;
    }
}

/**
 * Strips all markdown image tags (![alt](url/dataUri)) and optional trailing image-only lines from prompt text,
 * copying a clean plain-text version to clipboard.
 */
export async function copyPromptCleanText(
    markdownText: string,
    onSuccess?: (message: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    if (!markdownText || !markdownText.trim()) {
        return false;
    }

    try {
        // Strip markdown images using centralized stripImagesFromPrompt
        const cleanText = stripImagesFromPrompt(markdownText);

        const success = await copyToClipboard(cleanText);
        if (success) {
            const msg = 'Copied clean prompt text (without images) to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyPromptCleanText',
            triggerAction: 'copy_prompt_clean_text',
        });
        onError?.(err);
        return false;
    }
}

/**
 * Copies multiple image URIs into clipboard memory as rich HTML (<p><img src="..." /></p>)
 * with embedded Base64 data URIs and attached binary PNG blob, alongside plain text URIs.
 */
export async function copyAllImagesAsRichHtml(
    images: string[],
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    if (!images || images.length === 0) {
        return false;
    }
    const cleanImages = images.map((img) => img.trim()).filter(Boolean);
    if (cleanImages.length === 0) {
        return false;
    }

    try {
        // Convert all images to Base64 data URIs for rich HTML clipboard pasting
        const base64List: string[] = [];
        for (const img of cleanImages) {
            const b64 = await convertImageUriToBase64(img);
            base64List.push(b64 || img);
        }

        const htmlBody = base64List
            .map(
                (src) =>
                    `<p style="margin:12px 0;"><img src="${src}" style="max-width:100%;height:auto;border-radius:6px;display:block;box-shadow:0 2px 8px rgba(0,0,0,0.15);" /></p>`
            )
            .join('');
        const fullHtml = `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#222;"><p style="margin:8px 0;line-height:1.6;">${htmlBody}</p></body></html>`;
        const plainText = cleanImages.join('\n');

        let firstPngBlob: Blob | null = null;
        if (base64List.length > 0) {
            firstPngBlob = await convertImageUriToPngBlob(base64List[0]);
        }

        let isWritten = false;
        ensureDocumentFocus();
        const hasFocus = typeof document !== 'undefined' && typeof document.hasFocus === 'function' ? document.hasFocus() : true;
        if (hasFocus && navigator.clipboard && typeof ClipboardItem !== 'undefined') {
            const htmlBlob = new Blob([fullHtml], { type: 'text/html' });
            const textBlob = new Blob([plainText], { type: 'text/plain' });

            if (firstPngBlob) {
                try {
                    const item = new ClipboardItem({
                        'image/png': firstPngBlob,
                        'text/html': htmlBlob,
                        'text/plain': textBlob,
                    });
                    await navigator.clipboard.write([item]);
                    isWritten = true;
                } catch {
                    // Fall back to dual HTML/plain text item
                }
            }

            if (!isWritten) {
                try {
                    const item = new ClipboardItem({
                        'text/html': htmlBlob,
                        'text/plain': textBlob,
                    });
                    await navigator.clipboard.write([item]);
                    isWritten = true;
                } catch {
                    // Focus was lost or rich HTML item rejected; fall through to copyToClipboard
                }
            }
        }

        if (isWritten) {
            const msg = `Copied ${cleanImages.length} image(s) as rich HTML with embedded media!`;
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }

        const success = await copyToClipboard(plainText);
        if (success) {
            const msg = `Copied ${cleanImages.length} image URL(s) to clipboard!`;
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        return false;
    } catch (err) {
        const plainText = cleanImages.join('\n');
        const fallbackOk = await copyToClipboard(plainText);
        if (fallbackOk) {
            const msg = `Copied ${cleanImages.length} image URL(s) to clipboard!`;
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyAllImagesAsRichHtml',
            triggerAction: 'copy_all_images_html',
        });
        onError?.(err);
        return false;
    }
}

/**
 * Copies a single image into clipboard memory formatted for Microsoft OneNote / Word (rich HTML & binary PNG blob)
 * and external IDEs like VS Code / Cursor (plain-text Markdown image tag).
 */
export async function copySingleImageForOneNoteOrIde(
    imageUri: string,
    altText = 'Image',
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    if (!imageUri || !imageUri.trim()) {
        return false;
    }

    const cleanUri = imageUri.trim();

    if (!navigator.clipboard || typeof ClipboardItem === 'undefined') {
        const mdText = `![${altText}](${cleanUri})`;
        const fallbackOk = await copyToClipboard(mdText);
        if (fallbackOk) {
            const msg = 'Copied image as Markdown text (rich clipboard unsupported in current environment)';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        onError?.(new Error('Clipboard API unavailable'));
        return false;
    }

    try {
        const base64Uri = await convertImageUriToBase64(cleanUri);
        const resolvedSrc = base64Uri || cleanUri;

        const htmlContent = `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#222;"><p style="margin:8px 0;"><img src="${resolvedSrc}" alt="${altText}" style="max-width:100%;height:auto;border-radius:6px;display:block;box-shadow:0 2px 8px rgba(0,0,0,0.15);" /></p></body></html>`;
        const mdText = `![${altText}](${cleanUri})`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([mdText], { type: 'text/plain' });
        const pngBlob = await convertImageUriToPngBlob(resolvedSrc);

        let isWritten = false;
        ensureDocumentFocus();
        const hasFocus = typeof document !== 'undefined' && typeof document.hasFocus === 'function' ? document.hasFocus() : true;

        if (hasFocus && navigator.clipboard && typeof ClipboardItem !== 'undefined') {
            if (pngBlob) {
                try {
                    const item = new ClipboardItem({
                        'image/png': pngBlob,
                        'text/html': htmlBlob,
                        'text/plain': textBlob,
                    });
                    await navigator.clipboard.write([item]);
                    isWritten = true;
                } catch {
                    // Triple MIME write failed; fall through to dual HTML/plain-text
                }
            }

            if (!isWritten) {
                try {
                    const item = new ClipboardItem({
                        'text/html': htmlBlob,
                        'text/plain': textBlob,
                    });
                    await navigator.clipboard.write([item]);
                    isWritten = true;
                } catch {
                    // Fall back to PNG binary only
                    if (pngBlob) {
                        try {
                            const item = new ClipboardItem({
                                'image/png': pngBlob,
                            });
                            await navigator.clipboard.write([item]);
                            isWritten = true;
                        } catch {
                            // Binary write failed
                        }
                    }
                }
            }
        }

        if (isWritten) {
            const msg = 'Copied image for OneNote/Word (rich media) and IDEs (Markdown)!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }

        const fallbackOk = await copyToClipboard(mdText);
        if (fallbackOk) {
            const msg = 'Copied image as Markdown text to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        return false;
    } catch (err) {
        const mdText = `![${altText}](${cleanUri})`;
        const fallbackOk = await copyToClipboard(mdText);
        if (fallbackOk) {
            const msg = 'Copied image as Markdown text to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copySingleImageForOneNoteOrIde',
            triggerAction: 'copy_image_onenote_ide',
        });
        onError?.(err);
        return false;
    }
}

