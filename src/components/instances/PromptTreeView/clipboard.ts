import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { copyToClipboard, ensureDocumentFocus } from '../../../utils/clipboard';
import { extractAttachmentsFromPrompt, formatPromptWithAttachments, stripAttachmentsFromPrompt } from '../../../utils/promptAttachment';
import { useErrorStore } from '../../../stores/error-store';
import { showToast } from '../../common/ToastContainer';
import type { ExtractedPromptImage, PromptRewriteResult } from './types';

/**
 * Strips all markdown image tags and canonical attachment blocks from prompt text,
 * returning clean text suitable for text-only copying and tokens counting.
 */
export function stripImagesFromPrompt(text: string): string {
    return stripAttachmentsFromPrompt(text);
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
        if (/^file:\/\/\/[a-zA-Z]:[\\/]/.test(cleanPath)) {
            cleanPath = cleanPath.slice(8);
        } else {
            cleanPath = cleanPath.slice(7);
            if (/^\/[a-zA-Z]:[\\/]/.test(cleanPath)) {
                cleanPath = cleanPath.slice(1);
            }
        }
    } else if (cleanPath.startsWith('file://')) {
        cleanPath = cleanPath.slice(7);
        if (/^\/[a-zA-Z]:[\\/]/.test(cleanPath)) {
            cleanPath = cleanPath.slice(1);
        }
    }

    try {
        cleanPath = decodeURIComponent(cleanPath);
    } catch {
        // Keep raw cleanPath if decoding fails
    }

    if (/^\/[a-zA-Z]:[\\/]/.test(cleanPath)) {
        cleanPath = cleanPath.slice(1);
    }

    // If cleanPath is relative and repoPath is provided, resolve against repoPath
    const isAbsolute = /^[a-zA-Z]:[\\/]/.test(cleanPath) || cleanPath.startsWith('/') || cleanPath.startsWith('\\\\');
    if (!isAbsolute && repoPath && repoPath.trim().length > 0) {
        const sep = repoPath.includes('/') ? '/' : '\\';
        const normRepo = repoPath.replace(/[/\\]+$/, '');
        const normChild = cleanPath.replace(/^(\.[/\\]|[/\\])+/, '');
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
    }

    return {
        rewrittenPrompt: text,
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
export const IMAGE_EXTENSIONS_PATTERN = /\.(png|jpe?g|gif|webp|svg|bmp|ico|avif|jfif|tiff?)$/i;

/**
 * Checks if a URI or path is likely an image attachment.
 * Accepts:
 *  - Inline data URIs (data:image/...)
 *  - Blob URIs (blob:...)
 *  - Files with recognized image extensions (.png, .jpg, .webp, .svg, .gif, .bmp, .ico, .avif, .jfif, .tiff)
 *  - Gemini uploaded media files (.user_uploaded)
 */
export function isLikelyImageUri(uri: string): boolean {
    if (!uri || !uri.trim()) return false;
    const clean = uri.trim();
    if (clean.startsWith('data:image/') || clean.startsWith('blob:')) return true;
    const withoutQuery = clean.split(/[?#]/)[0].replace(/[/\\]+$/, '');
    const basename = withoutQuery.split(/[/\\]/).pop() || '';
    if (!basename || basename === '.user_uploaded' || basename === '.' || basename === '..') return false;
    if (IMAGE_EXTENSIONS_PATTERN.test(basename)) return true;
    if (withoutQuery.includes('.user_uploaded') && (IMAGE_EXTENSIONS_PATTERN.test(basename) || /^media_|^screenshot_|^img_/i.test(basename))) {
        return true;
    }
    return false;
}

export function extractAssets(text: string): { images: string[]; files: string[] } {
    const images: string[] = [];
    const files: string[] = [];
    const seenImages = new Set<string>();
    const seenFiles = new Set<string>();

    const imgRegex = /!\[(.*?)\]\((https?:\/\/.*?|data:image\/[^;]+;base64,[A-Za-z0-9+/=]+|[^\s)]+?\.(?:png|jpe?g|gif|webp|svg|bmp|ico|avif|jfif|tiff?))\)/gi;
    let match: RegExpExecArray | null;
    while ((match = imgRegex.exec(text)) !== null) {
        if (match[2]) {
            const raw = match[2].trim();
            const key = (extractLocalPathFromUri(raw) || raw).replace(/\\/g, '/').toLowerCase();
            if (!seenImages.has(key)) {
                seenImages.add(key);
                images.push(raw);
            }
        }
    }

    const bracketRegex = /@\[(?:file;\s*)?(.*?)\]/gi;
    let bracketMatch: RegExpExecArray | null;
    while ((bracketMatch = bracketRegex.exec(text)) !== null) {
        if (bracketMatch[1]) {
            let raw = bracketMatch[1].trim();
            if (/^file;\s*/i.test(raw)) {
                raw = raw.replace(/^file;\s*/i, '');
            }
            const isImg = isLikelyImageUri(raw);
            const key = (extractLocalPathFromUri(raw) || raw).replace(/\\/g, '/').toLowerCase();
            if (isImg) {
                if (!seenImages.has(key)) {
                    seenImages.add(key);
                    images.push(raw);
                }
            } else if (!seenFiles.has(key)) {
                seenFiles.add(key);
                files.push(raw);
            }
        }
    }

    const fileRegex = /(?:file:\/\/\/|[a-zA-Z]:\\|\/)[^\s)"\]]+/g;
    let fileMatch: RegExpExecArray | null;
    while ((fileMatch = fileRegex.exec(text)) !== null) {
        const raw = fileMatch[0].trim();
        const isImg = isLikelyImageUri(raw);
        const key = (extractLocalPathFromUri(raw) || raw).replace(/\\/g, '/').toLowerCase();

        if (isImg) {
            if (!seenImages.has(key)) {
                seenImages.add(key);
                images.push(raw);
            }
        } else {
            if (!seenFiles.has(key)) {
                seenFiles.add(key);
                files.push(raw);
            }
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

function isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
}

/**
 * Normalizes an image URI into a clean filesystem path if it represents a local file.
 * Handles `file://`, `file:///`, `http://asset.localhost/...`, `asset://localhost/...`,
 * URL-encoded paths, Windows extended prefix `\\?\`, leading slashes before drive letters,
 * and direct absolute paths (`C:\...`, `/...`).
 */
export function extractLocalPathFromUri(uri: string): string | null {
    if (!uri || !uri.trim()) return null;
    let clean = uri.trim();

    // Already a data URI or blob URI
    if (clean.startsWith('data:') || clean.startsWith('blob:')) {
        return null;
    }

    if (/^file;\s*/i.test(clean)) {
        clean = clean.replace(/^file;\s*/i, '');
    }

    // Check Tauri asset protocol URLs
    if (clean.startsWith('http://asset.localhost/')) {
        clean = clean.slice('http://asset.localhost/'.length);
    } else if (clean.startsWith('https://asset.localhost/')) {
        clean = clean.slice('https://asset.localhost/'.length);
    } else if (clean.startsWith('asset://localhost/')) {
        clean = clean.slice('asset://localhost/'.length);
    } else if (clean.startsWith('file:///')) {
        if (/^file:\/\/\/[a-zA-Z]:[\\/]/.test(clean)) {
            clean = clean.slice(8);
        } else {
            clean = clean.slice(7);
            if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
                clean = clean.slice(1);
            }
        }
    } else if (clean.startsWith('file://')) {
        clean = clean.slice(7);
        if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
            clean = clean.slice(1);
        }
    }

    // If it was an external remote web URL (not asset.localhost), do not treat as local path
    if (clean.startsWith('http://') || clean.startsWith('https://')) {
        return null;
    }

    try {
        clean = decodeURIComponent(clean);
    } catch {
        // Keep clean as-is if decoding fails
    }

    // Strip Windows extended-length prefix: \\?\C:\... or \?\C:\...
    clean = clean.replace(/^[\\/]+\?[\\/]+/, '');

    // Strip leading slash before Windows drive letter: /C:/... -> C:/...
    if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
        clean = clean.slice(1);
    }

    const isWindowsPath = /^[a-zA-Z]:[\\/]/.test(clean) || clean.startsWith('\\\\');
    const isUnixPath = clean.startsWith('/') && !clean.startsWith('//');

    if (isWindowsPath || isUnixPath) {
        return clean;
    }

    return null;
}

/**
 * Resolves a URI to a canonical absolute local path if it refers to a local file,
 * resolving relative paths (e.g. `./.antigravity/attachments/...`) against `repoPath`.
 */
export function resolveToAbsolutePath(uri: string, repoPath?: string): string | null {
    if (!uri || !uri.trim()) return null;
    let clean = uri.trim();
    if (clean.startsWith('data:') || clean.startsWith('blob:')) return null;
    if (clean.startsWith('http://') || clean.startsWith('https://')) {
        if (!clean.includes('asset.localhost')) return null;
    }
    const extracted = extractLocalPathFromUri(clean);
    if (extracted) return extracted;

    // Check if it's a relative path and repoPath is available
    const isAbsolute = /^[a-zA-Z]:[\\/]/.test(clean) || clean.startsWith('/') || clean.startsWith('\\\\');
    if (!isAbsolute && repoPath && repoPath.trim().length > 0) {
        const sep = repoPath.includes('/') ? '/' : '\\';
        const normRepo = repoPath.replace(/[/\\]+$/, '');
        const normChild = clean.replace(/^(\.[/\\]|[/\\])+/, '');
        return `${normRepo}${sep}${normChild}`;
    }
    return null;
}

/**
 * Converts an image URI (Base64 data URI, HTTP URL, file path, or blob URI)
 * into a full Base64 data URI (data:image/<ext>;base64,...) for embedding into HTML.
 * Uses native Tauri backend command `read_image_as_base64` for local files to eliminate
 * CORS/taint canvas security errors in WebView2.
 */
export async function convertImageUriToBase64(imageUri: string, repoPath?: string): Promise<string | null> {
    try {
        if (!imageUri || !imageUri.trim()) return null;
        const cleanUri = imageUri.trim();

        // 1. Already a base64 data URI
        if (cleanUri.startsWith('data:image/')) {
            return cleanUri;
        }

        // 2. Local filesystem path or Tauri asset URL -> invoke native backend
        const localPath = resolveToAbsolutePath(cleanUri, repoPath) || extractLocalPathFromUri(cleanUri);
        if (localPath && isTauriEnvironment()) {
            try {
                const b64DataUri = await invoke<string>('read_image_as_base64', { path: localPath });
                if (b64DataUri && b64DataUri.startsWith('data:image/')) {
                    return b64DataUri;
                }
            } catch (backendErr) {
                useErrorStore.getState().trackWarning(backendErr, {
                    source: 'clipboard.convertImageUriToBase64.read_image_as_base64',
                    triggerAction: 'read_image_as_base64',
                });
            }
        }

        // 3. Fallback to offscreen HTML Image canvas rendering (for remote web URLs or non-Tauri)
        const resolvedSrc = resolveImageSrc(cleanUri, repoPath);
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
 * Invokes native `read_image_as_base64` and decodes directly via `base64ToBlob`
 * to completely eliminate canvas tainting and CORS failures.
 */
/**
 * Transcodes any image source or data URI (JPEG, WebP, SVG, PNG) through an offscreen HTMLCanvasElement
 * to guarantee 100% valid PNG magic bytes (\x89PNG\r\n\x1a\n) before placing on the clipboard.
 */
export async function transcodeSrcToPngBlob(src: string): Promise<Blob | null> {
    if (!src || !src.trim()) return null;
    return await new Promise<Blob | null>((resolve) => {
        const timeout = setTimeout(() => resolve(null), 8000);
        const img = new Image();
        if (!src.startsWith('data:') && !src.startsWith('blob:')) {
            img.crossOrigin = 'anonymous';
        }
        img.onload = () => {
            clearTimeout(timeout);
            try {
                const canvas = document.createElement('canvas');
                const w = img.naturalWidth || img.width || 300;
                const h = img.naturalHeight || img.height || 150;
                canvas.width = w;
                canvas.height = h;
                const ctx = canvas.getContext('2d');
                if (!ctx) {
                    resolve(null);
                    return;
                }
                ctx.drawImage(img, 0, 0, w, h);
                canvas.toBlob((blob) => {
                    resolve(blob);
                }, 'image/png');
            } catch (err) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'clipboard.transcodeSrcToPngBlob.canvas',
                    triggerAction: 'canvas_to_blob',
                });
                resolve(null);
            }
        };
        img.onerror = () => {
            clearTimeout(timeout);
            resolve(null);
        };
        img.src = src;
    });
}

/**
 * Converts an image URI (Base64 data URI, HTTP URL, or local file/blob URI)
 * into a true PNG binary Blob (image/png) for native clipboard insertion.
 * Invokes native `read_image_as_base64` and transcodes directly through an offscreen canvas
 * to ensure 100% valid PNG magic bytes and eliminate CORS failures.
 */
export async function convertImageUriToPngBlob(imageUri: string, repoPath?: string): Promise<Blob | null> {
    try {
        if (!imageUri || !imageUri.trim()) return null;
        const cleanUri = imageUri.trim();

        // 1. If it's a local file or asset URL, invoke native backend read_image_as_base64
        const localPath = resolveToAbsolutePath(cleanUri, repoPath) || extractLocalPathFromUri(cleanUri);
        if (localPath && isTauriEnvironment()) {
            try {
                const b64DataUri = await invoke<string>('read_image_as_base64', { path: localPath });
                if (b64DataUri && b64DataUri.startsWith('data:image/')) {
                    const transcoded = await transcodeSrcToPngBlob(b64DataUri);
                    if (transcoded) return transcoded;
                    if (b64DataUri.startsWith('data:image/png')) {
                        const commaIdx = b64DataUri.indexOf(',');
                        const base64Payload = commaIdx !== -1 ? b64DataUri.slice(commaIdx + 1) : b64DataUri;
                        return base64ToBlob(base64Payload, 'image/png');
                    }
                }
            } catch (backendErr) {
                useErrorStore.getState().trackWarning(backendErr, {
                    source: 'clipboard.convertImageUriToPngBlob.read_image_as_base64',
                    triggerAction: 'read_image_as_base64',
                });
            }
        }

        // 2. If it's already a data URI, transcode to genuine PNG
        if (cleanUri.startsWith('data:image/')) {
            const transcoded = await transcodeSrcToPngBlob(cleanUri);
            if (transcoded) return transcoded;
            if (cleanUri.startsWith('data:image/png')) {
                const commaIdx = cleanUri.indexOf(',');
                const base64Payload = commaIdx !== -1 ? cleanUri.slice(commaIdx + 1) : cleanUri;
                return base64ToBlob(base64Payload, 'image/png');
            }
        }

        // 3. Remote URL or resolved src
        const resolvedSrc = resolveImageSrc(cleanUri, repoPath);
        return await transcodeSrcToPngBlob(resolvedSrc);
    } catch {
        return null;
    }
}

/**
 * Writes pure PNG binary (image/png) directly to the system clipboard without any
 * competing text/plain or text/html MIME types. This ensures Microsoft OneNote,
 * Telegram, Discord, Teams, Photoshop, etc. recognize the clipboard payload as a real
 * image rather than stripping data URIs or pasting text.
 */
export async function writePureImageToClipboard(
    imageUriOrBlob: string | Blob,
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void,
    repoPath?: string
): Promise<boolean> {
    try {
        let pngBlob: Blob | null = null;
        if (imageUriOrBlob instanceof Blob) {
            if (imageUriOrBlob.type === 'image/png') {
                pngBlob = imageUriOrBlob;
            } else {
                const blobUrl = URL.createObjectURL(imageUriOrBlob);
                try {
                    pngBlob = await transcodeSrcToPngBlob(blobUrl);
                } finally {
                    URL.revokeObjectURL(blobUrl);
                }
            }
        } else if (typeof imageUriOrBlob === 'string') {
            pngBlob = await convertImageUriToPngBlob(imageUriOrBlob, repoPath);
        }
        if (!pngBlob) {
            throw new Error('Failed to convert image to valid PNG binary blob');
        }

        let nativeWritten = false;
        // 0. Native Tauri OS Clipboard Path: Write native bitmap/DIB on Windows, osascript on macOS, wl-copy/xclip on Linux
        if (isTauriEnvironment()) {
            try {
                let pathToPass = '';
                if (typeof imageUriOrBlob === 'string' && !imageUriOrBlob.startsWith('blob:') && !imageUriOrBlob.startsWith('data:')) {
                    pathToPass = resolveToAbsolutePath(imageUriOrBlob, repoPath) || extractLocalPathFromUri(imageUriOrBlob) || imageUriOrBlob;
                } else if (pngBlob) {
                    const reader = new FileReader();
                    const b64Promise = new Promise<string>((res) => {
                        reader.onloadend = () => res(reader.result as string);
                        reader.readAsDataURL(pngBlob!);
                    });
                    pathToPass = await b64Promise;
                }
                if (pathToPass) {
                    await invoke('copy_image_to_clipboard', { path: pathToPass });
                    nativeWritten = true;
                }
            } catch (tauriErr) {
                useErrorStore.getState().trackWarning(tauriErr, {
                    source: 'clipboard.writePureImageToClipboard.tauri',
                    triggerAction: 'copy_image_to_clipboard',
                });
            }
        }

        if (nativeWritten) {
            const msg = 'Image copied to clipboard (ready for OneNote, chat & IDEs)!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }

        let webWritten = false;
        ensureDocumentFocus();
        const hasFocus = typeof document !== 'undefined' && typeof document.hasFocus === 'function' ? document.hasFocus() : true;
        if (hasFocus && navigator.clipboard && typeof ClipboardItem !== 'undefined') {
            try {
                // Strictly single MIME type: 'image/png' ONLY. Zero conflicting text/plain, zero text/html.
                // Bundling text/html in the same item drops image/png on Chromium.
                const item = new ClipboardItem({
                    'image/png': pngBlob,
                });
                await navigator.clipboard.write([item]);
                webWritten = true;
            } catch (clipErr) {
                useErrorStore.getState().trackWarning(clipErr, {
                    source: 'clipboard.writePureImageToClipboard.navigator',
                    triggerAction: 'write_pure_image',
                });
            }
        }

        if (webWritten) {
            const msg = 'Image copied to clipboard (ready for OneNote, chat & IDEs)!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }

        // Fallback: Markdown image tag or URI via copyToClipboard
        const fallbackPayload = typeof imageUriOrBlob === 'string' ? imageUriOrBlob : 'image.png';
        const fallbackOk = await copyToClipboard(fallbackPayload);
        if (fallbackOk) {
            const msg = 'Copied image reference to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.writePureImageToClipboard',
            triggerAction: 'write_pure_image',
        });
        onError?.(err);
        return false;
    }
}

/**
 * Loads an image URI into an HTMLImageElement safely across Tauri and web environments.
 * Uses native Tauri read_image_as_base64 or convertFileSrc to bypass WebView2 local file CORS restrictions.
 * Resolves relative image paths against repoPath to prevent WebView2 canvas tainting.
 */
async function loadImageElement(uri: string, repoPath?: string): Promise<HTMLImageElement | null> {
    if (!uri || !uri.trim()) return null;
    const cleanUri = uri.trim();

    let loadSrc = cleanUri;
    const localPath = resolveToAbsolutePath(cleanUri, repoPath) || extractLocalPathFromUri(cleanUri);
    if (localPath && isTauriEnvironment() && isLikelyImageUri(localPath)) {
        try {
            const b64 = await invoke<string>('read_image_as_base64', { path: localPath });
            if (b64 && b64.startsWith('data:image/')) {
                loadSrc = b64;
            } else {
                loadSrc = convertFileSrc(localPath);
            }
        } catch {
            loadSrc = resolveImageSrc(cleanUri, repoPath);
        }
    } else {
        loadSrc = resolveImageSrc(cleanUri, repoPath);
    }

    if (!loadSrc.startsWith('data:image/')) {
        try {
            const res = await fetch(loadSrc);
            if (res.ok) {
                const blob = await res.blob();
                const b64 = await new Promise<string>((resB64) => {
                    const reader = new FileReader();
                    reader.onloadend = () => resB64(reader.result as string);
                    reader.readAsDataURL(blob);
                });
                if (b64 && b64.startsWith('data:image/')) {
                    loadSrc = b64;
                }
            }
        } catch {
            // Keep loadSrc as-is if fetch is unavailable
        }
    }

    return await new Promise<HTMLImageElement | null>((resolve) => {
        const timeout = setTimeout(() => resolve(null), 8000);
        const img = new Image();
        if (!loadSrc.startsWith('data:') && !loadSrc.startsWith('blob:')) {
            img.crossOrigin = 'anonymous';
        }
        img.onload = () => {
            clearTimeout(timeout);
            resolve(img);
        };
        img.onerror = () => {
            clearTimeout(timeout);
            resolve(null);
        };
        img.src = loadSrc;
    });
}

/**
 * Stitches all provided image URIs vertically into a single composite PNG canvas,
 * returning a clean binary PNG Blob.
 * This allows pasting all images at once into applications like OneNote or Antigravity.
 */
export async function stitchImagesToCompositePng(imageUris: string[], repoPath?: string): Promise<Blob | null> {
    if (!imageUris || imageUris.length === 0) return null;
    const cleanUris = imageUris.map((u) => u.trim()).filter(Boolean);
    if (cleanUris.length === 0) return null;

    try {
        const loadedImages: HTMLImageElement[] = [];
        const results = await Promise.all(cleanUris.map((u) => loadImageElement(u, repoPath)));
        for (const img of results) {
            if (img && (img.naturalWidth || img.width) > 0 && (img.naturalHeight || img.height) > 0) {
                loadedImages.push(img);
            }
        }

        if (loadedImages.length === 0) return null;

        const MAX_COMPOSITE_WIDTH = 4096;
        const MAX_COMPOSITE_HEIGHT = 16384;
        const GAP = 16;

        if (loadedImages.length === 1) {
            const single = loadedImages[0];
            const canvas = document.createElement('canvas');
            const naturalW = single.naturalWidth || single.width;
            const naturalH = single.naturalHeight || single.height;
            const scale = Math.min(
                1,
                MAX_COMPOSITE_WIDTH / naturalW,
                MAX_COMPOSITE_HEIGHT / naturalH
            );
            const w = Math.max(1, Math.round(naturalW * scale));
            const h = Math.max(1, Math.round(naturalH * scale));
            canvas.width = w;
            canvas.height = h;
            const ctx = canvas.getContext('2d');
            if (!ctx) return null;
            ctx.drawImage(single, 0, 0, w, h);
            return await new Promise<Blob | null>((resolve) => {
                canvas.toBlob((blob) => resolve(blob), 'image/png');
            });
        }

        let maxWidth = 0;
        let totalHeight = 0;

        for (let i = 0; i < loadedImages.length; i++) {
            const img = loadedImages[i];
            const w = img.naturalWidth || img.width;
            const h = img.naturalHeight || img.height;
            if (w > maxWidth) maxWidth = w;
            totalHeight += h;
            if (i < loadedImages.length - 1) {
                totalHeight += GAP;
            }
        }

        let scale = 1;
        if (maxWidth > MAX_COMPOSITE_WIDTH || totalHeight > MAX_COMPOSITE_HEIGHT) {
            const scaleW = MAX_COMPOSITE_WIDTH / maxWidth;
            const scaleH = MAX_COMPOSITE_HEIGHT / totalHeight;
            scale = Math.min(scaleW, scaleH);
        }

        const canvasWidth = Math.max(1, Math.round(maxWidth * scale));
        const canvasHeight = Math.max(1, Math.round(totalHeight * scale));
        const scaledGap = Math.round(GAP * scale);

        const canvas = document.createElement('canvas');
        canvas.width = canvasWidth;
        canvas.height = canvasHeight;
        const ctx = canvas.getContext('2d');
        if (!ctx) return null;

        ctx.clearRect(0, 0, canvasWidth, canvasHeight);

        let currentY = 0;
        for (const img of loadedImages) {
            const naturalW = img.naturalWidth || img.width;
            const naturalH = img.naturalHeight || img.height;
            const drawW = Math.round(naturalW * scale);
            const drawH = Math.round(naturalH * scale);
            const drawX = Math.max(0, Math.round((canvasWidth - drawW) / 2));
            ctx.drawImage(img, drawX, currentY, drawW, drawH);
            currentY += drawH + scaledGap;
        }

        return await new Promise<Blob | null>((resolve) => {
            canvas.toBlob((blob) => resolve(blob), 'image/png');
        });
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.stitchImagesToCompositePng',
            triggerAction: 'stitch_images',
        });
        return null;
    }
}

/**
 * Copies all images simultaneously via composite PNG blob, plus rich HTML with markdown fallback.
 * Writes pure PNG binary blob of stitched composite images so pasting into OneNote or Antigravity
 * pastes all images cleanly at once without Chromium dropping image/png due to conflicting text MIME types.
 */
export async function copyAllImagesToClipboard(
    imageUris: string[],
    repoPathOrOnSuccess?: string | ((msg: string) => void),
    onSuccessOrOnError?: ((msg: string) => void) | ((err: unknown) => void),
    onError?: (err: unknown) => void
): Promise<boolean> {
    if (!imageUris || imageUris.length === 0) return false;
    const cleanUris = imageUris.map((u) => u.trim()).filter(Boolean);
    if (cleanUris.length === 0) return false;

    let repoPath: string | undefined;
    let onSuccess: ((msg: string) => void) | undefined;
    let actualOnError: ((err: unknown) => void) | undefined = onError;

    if (typeof repoPathOrOnSuccess === 'string') {
        repoPath = repoPathOrOnSuccess;
        if (typeof onSuccessOrOnError === 'function') {
            onSuccess = onSuccessOrOnError as (msg: string) => void;
        }
    } else if (typeof repoPathOrOnSuccess === 'function') {
        onSuccess = repoPathOrOnSuccess;
        if (typeof onSuccessOrOnError === 'function') {
            actualOnError = onSuccessOrOnError as (err: unknown) => void;
        }
    }

    try {
        // 0. Primary Native Path (Tauri): vertical composite image + CF_HDROP file drop list
        if (isTauriEnvironment()) {
            try {
                const resolvedPaths: string[] = [];
                for (const u of cleanUris) {
                    if (u.startsWith('data:image/')) {
                        resolvedPaths.push(u);
                    } else if (u.startsWith('http://') || u.startsWith('https://')) {
                        const b64 = await convertImageUriToBase64(u, repoPath);
                        resolvedPaths.push(b64 || u);
                    } else {
                        const abs = resolveToAbsolutePath(u, repoPath) || extractLocalPathFromUri(u) || u;
                        resolvedPaths.push(abs);
                    }
                }
                if (resolvedPaths.length > 0) {
                    await invoke('copy_images_to_clipboard', { paths: resolvedPaths });
                    const msg = `Copied ${cleanUris.length} image(s) to clipboard (ready for OneNote & IDE)!`;
                    onSuccess?.(msg);
                    showToast({ message: msg, type: 'success' });
                    return true;
                }
            } catch (tauriErr) {
                useErrorStore.getState().trackWarning(tauriErr, {
                    source: 'clipboard.copyAllImagesToClipboard.tauri',
                    triggerAction: 'copy_images_to_clipboard',
                });
            }
        }

        // 1. Fallback Web Path: Vertical composite PNG blob (pure binary to prevent Chromium drop bug)
        const compositeBlob = await stitchImagesToCompositePng(cleanUris, repoPath);
        if (compositeBlob) {
            let webWritten = false;
            ensureDocumentFocus();
            const hasFocus = typeof document !== 'undefined' && typeof document.hasFocus === 'function' ? document.hasFocus() : true;
            if (hasFocus && navigator.clipboard && typeof ClipboardItem !== 'undefined') {
                try {
                    const item = new ClipboardItem({
                        'image/png': compositeBlob,
                    });
                    await navigator.clipboard.write([item]);
                    webWritten = true;
                } catch (clipErr) {
                    useErrorStore.getState().trackWarning(clipErr, {
                        source: 'clipboard.copyAllImagesToClipboard.navigator',
                        triggerAction: 'write_composite_image',
                    });
                }
            }

            if (webWritten) {
                const msg = `Copied ${cleanUris.length} image(s) as composite PNG to clipboard (ready for OneNote & IDE)!`;
                onSuccess?.(msg);
                showToast({ message: msg, type: 'success' });
                return true;
            }
        }

        // 2. Secondary path: Rich HTML with embedded Base64 <img> tags
        const richHtmlOk = await copyAllImagesAsRichHtml(cleanUris, onSuccess, actualOnError);
        if (richHtmlOk) return true;

        // 3. Tertiary fallback: Markdown references
        const mdFallback = cleanUris.map((u, idx) => `![Image ${idx + 1}](${u})`).join('\n\n');
        const fallbackOk = await copyToClipboard(mdFallback);
        if (fallbackOk) {
            const msg = `Copied ${cleanUris.length} image Markdown reference(s) to clipboard!`;
            onSuccess?.(msg);
            showToast({ message: msg, type: 'info' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyAllImagesToClipboard',
            triggerAction: 'copy_all_images',
        });
        actualOnError?.(err);
        return false;
    }
}

/**
 * Writes a single image directly to the system clipboard as a native binary PNG (image/png).
 * This enables pasting directly into apps like Gemini Web, Photoshop, Paint, Discord, OneNote, etc.
 */
export async function writeNativeImageToClipboard(
    imageUri: string,
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    return writePureImageToClipboard(imageUri, onSuccess, onError);
}

/**
 * Copies a prompt turn or transcript event object as formatted JSON to clipboard
 * with immediate success toast feedback.
 */
export async function copyEventAsJson(
    eventData: unknown,
    eventName = 'Event',
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    try {
        if (!eventData) {
            throw new Error('No event data provided to copy');
        }
        const jsonText = typeof eventData === 'string' ? eventData : JSON.stringify(eventData, null, 2);
        const success = await copyToClipboard(jsonText);
        if (success) {
            const msg = `Copied ${eventName} data to clipboard!`;
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyEventAsJson',
            triggerAction: 'copy_event_as_json',
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
    mediaPaths?: string[],
    repoPath?: string
): Promise<boolean> {
    if (!markdownText || !markdownText.trim()) {
        return false;
    }

    const providedMedia = (mediaPaths || []).filter((p) => typeof p === 'string' && p.trim().length > 0);
    const extractedAttachments = extractAttachmentsFromPrompt(markdownText, repoPath);
    const effectiveMarkdown =
        providedMedia.length > 0 || extractedAttachments.length > 0
            ? formatPromptWithAttachments(markdownText, providedMedia, { repoPath })
            : markdownText;

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
                const base64Uri = await convertImageUriToBase64(rawSrc, repoPath);
                srcMap.set(rawSrc, base64Uri || rawSrc);
            }
        }

        const attachmentMatches = Array.from(effectiveMarkdown.matchAll(/@\[(?:file;\s*)?(.*?)\]/gi));
        for (const match of attachmentMatches) {
            let rawSrc = match[1].trim();
            if (/^file;\s*/i.test(rawSrc)) {
                rawSrc = rawSrc.replace(/^file;\s*/i, '');
            }
            if (rawSrc && isLikelyImageUri(rawSrc) && !srcMap.has(rawSrc)) {
                const base64Uri = await convertImageUriToBase64(rawSrc, repoPath);
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
            // Canonical attachment bracket syntax: @[file;path] or @[path] -> formatted paragraph with embedded Base64 image
            .replace(/@\[(?:file;\s*)?(.*?)\]/gi, (fullBracket, rawPath) => {
                let cleanSrc = rawPath.trim();
                if (/^file;\s*/i.test(cleanSrc)) {
                    cleanSrc = cleanSrc.replace(/^file;\s*/i, '');
                }
                const resolvedSrc = srcMap.get(cleanSrc) || cleanSrc;
                const isImg = Boolean(cleanSrc.match(/\.(png|jpg|jpeg|gif|webp|svg)$/i));
                if (isImg && resolvedSrc) {
                    return `${fullBracket}<p style="margin:12px 0;"><img src="${resolvedSrc}" alt="attachment" style="max-width:100%;height:auto;border-radius:6px;display:block;box-shadow:0 2px 8px rgba(0,0,0,0.15);" /></p>`;
                }
                return fullBracket;
            })
            // Paragraphs and line breaks
            .replace(/\n\n+/g, '</p><p style="margin:8px 0;line-height:1.6;">')
            .replace(/\n/g, '<br/>');

        const htmlContent = `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#222;"><p style="margin:8px 0;line-height:1.6;">${htmlBody}</p></body></html>`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([effectiveMarkdown], { type: 'text/plain' });

        // Extract first image to attach true binary image/png alongside html and text.
        // Do not write image-only via native helper first: that replaces the clipboard
        // and would report success without the prompt text.
        const { images } = extractAssets(effectiveMarkdown);
        let pngBlob: Blob | null = null;
        if (images.length > 0) {
            const firstImg = images[0];
            const base64First = srcMap.get(firstImg) || firstImg;
            pngBlob = await convertImageUriToPngBlob(base64First, repoPath);
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
    onError?: (err: unknown) => void,
    repoPath?: string
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
        const base64Uri = await convertImageUriToBase64(cleanUri, repoPath);
        const resolvedSrc = base64Uri || cleanUri;

        const htmlContent = `<!DOCTYPE html><html><head><meta charset="utf-8"></head><body style="font-family:system-ui,-apple-system,sans-serif;line-height:1.6;color:#222;"><p style="margin:8px 0;"><img src="${resolvedSrc}" alt="${altText}" style="max-width:100%;height:auto;border-radius:6px;display:block;box-shadow:0 2px 8px rgba(0,0,0,0.15);" /></p></body></html>`;
        const mdText = `![${altText}](${cleanUri})`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([mdText], { type: 'text/plain' });
        const pngBlob = await convertImageUriToPngBlob(resolvedSrc, repoPath);

        let nativeWritten = false;
        if (isTauriEnvironment()) {
            try {
                let pathToPass = '';
                if (resolvedSrc.startsWith('data:image/')) {
                    pathToPass = resolvedSrc;
                } else {
                    pathToPass = resolveToAbsolutePath(resolvedSrc, repoPath) || extractLocalPathFromUri(resolvedSrc) || resolvedSrc;
                }
                if (pathToPass) {
                    await invoke('copy_image_to_clipboard', { path: pathToPass });
                    nativeWritten = true;
                }
            } catch (nativeErr) {
                useErrorStore.getState().trackWarning(nativeErr, {
                    source: 'clipboard.copySingleImageForOneNoteOrIde.native',
                    triggerAction: 'copy_image_to_clipboard',
                });
            }
        }

        if (nativeWritten) {
            const msg = 'Copied image for OneNote/Word (rich media) and IDEs (Markdown)!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
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

/**
 * Normalizes an image path or local file URI into an RFC-8089 compliant clickable IDE markdown link: [label](file:///<path>).
 * Supports both signatures:
 *   formatIdeClickableFileLink(rawPath: string, repoPath?: string, altText?: string): string
 *   formatIdeClickableFileLink(rawPath: string, label?: string): string
 */
export function formatIdeClickableFileLink(
    rawPath: string,
    repoPathOrLabel?: string,
    altText?: string
): string {
    if (!rawPath || !rawPath.trim()) return '';
    let clean = rawPath.trim();

    // Strip agm-asset:// or file:// or file:/// prefixes
    if (clean.startsWith('agm-asset://')) {
        clean = clean.slice('agm-asset://'.length);
    } else if (clean.startsWith('file:///')) {
        if (/^file:\/\/\/[a-zA-Z]:[\\/]/.test(clean)) {
            clean = clean.slice(8);
        } else {
            clean = clean.slice(7);
            if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
                clean = clean.slice(1);
            }
        }
    } else if (clean.startsWith('file://')) {
        clean = clean.slice(7);
        if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
            clean = clean.slice(1);
        }
    }

    try {
        clean = decodeURIComponent(clean);
    } catch {
        // Keep raw clean if decoding fails
    }

    // Strip leading slash before Windows drive letter: /C:/... -> C:/...
    if (/^\/[a-zA-Z]:[\\/]/.test(clean)) {
        clean = clean.slice(1);
    }

    let repoPath: string | undefined;
    let label: string | undefined;

    if (altText !== undefined) {
        repoPath = repoPathOrLabel;
        label = altText;
    } else if (repoPathOrLabel !== undefined) {
        const isPath =
            repoPathOrLabel.includes('/') ||
            repoPathOrLabel.includes('\\') ||
            /^[a-zA-Z]:/.test(repoPathOrLabel);
        if (isPath) {
            repoPath = repoPathOrLabel;
        } else {
            label = repoPathOrLabel;
        }
    }

    let absPath = clean;
    const isAbs =
        /^[a-zA-Z]:[\\/]/.test(clean) ||
        clean.startsWith('/') ||
        clean.startsWith('\\\\');
    if (!isAbs && repoPath && repoPath.trim().length > 0) {
        const sep = repoPath.includes('/') ? '/' : '\\';
        const normRepo = repoPath.replace(/[/\\]+$/, '');
        const normChild = clean.replace(/^(\.[/\\]|[/\\])+/, '');
        absPath = `${normRepo}${sep}${normChild}`;
    }

    const forwardSlashed = absPath.replace(/\\/g, '/');
    const fileUri = forwardSlashed.startsWith('/')
        ? `file://${forwardSlashed}`
        : `file:///${forwardSlashed}`;
    const filename = forwardSlashed.split('/').pop() || 'image.png';
    const displayLabel = label || filename;

    return `[${displayLabel}](${fileUri})`;
}

/**
 * Formats a markdown prompt so all image references point to normalized local file URLs (`file:///...`).
 * Appends any turn mediaPaths not already present in the markdown.
 */
export function formatMarkdownWithLocalFileLinks(
    markdownText: string,
    mediaPaths?: string[],
    repoPath?: string
): string {
    if (!markdownText && (!mediaPaths || mediaPaths.length === 0)) return '';

    let text = markdownText || '';

    // Append missing mediaPaths
    if (mediaPaths && mediaPaths.length > 0) {
        const missing = mediaPaths.filter((p) => p && !text.includes(p));
        if (missing.length > 0) {
            text =
                text.trim() +
                '\n\n' +
                missing
                    .map((p, idx) => {
                        const link = formatIdeClickableFileLink(p, repoPath, `Turn Image ${idx + 1}`);
                        return `!${link}`;
                    })
                    .join('\n\n');
        }
    }

    // Replace all image tags with file:/// paths where possible
    return text.replace(/!\[(.*?)\]\((.*?)\)/g, (fullMatch, alt, src) => {
        const cleanSrc = src.trim();
        const localPath = resolveToAbsolutePath(cleanSrc, repoPath) || extractLocalPathFromUri(cleanSrc);
        if (localPath) {
            const normalizedPath = localPath.replace(/\\/g, '/');
            const fileUrl = normalizedPath.startsWith('/') ? `file://${normalizedPath}` : `file:///${normalizedPath}`;
            return `![${alt}](${fileUrl})`;
        }
        return fullMatch;
    });
}

/**
 * Copies prompt markdown referencing local file paths (`file:///...`) to clipboard.
 */
export async function copyPromptWithLocalFileLinks(
    markdownText: string,
    mediaPaths?: string[],
    repoPathOrOnSuccess?: string | ((msg: string) => void),
    onSuccessOrOnError?: ((msg: string) => void) | ((err: unknown) => void),
    onError?: (err: unknown) => void
): Promise<boolean> {
    let repoPath: string | undefined;
    let onSuccess: ((msg: string) => void) | undefined;
    let actualOnError: ((err: unknown) => void) | undefined = onError;

    if (typeof repoPathOrOnSuccess === 'string') {
        repoPath = repoPathOrOnSuccess;
        if (typeof onSuccessOrOnError === 'function') {
            onSuccess = onSuccessOrOnError as (msg: string) => void;
        }
    } else if (typeof repoPathOrOnSuccess === 'function') {
        onSuccess = repoPathOrOnSuccess;
        if (typeof onSuccessOrOnError === 'function') {
            actualOnError = onSuccessOrOnError as (err: unknown) => void;
        }
    }

    try {
        const formatted = formatMarkdownWithLocalFileLinks(markdownText, mediaPaths, repoPath);
        const ok = await copyToClipboard(formatted);
        if (ok) {
            const msg = 'Copied prompt with local file links to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyPromptWithLocalFileLinks',
            triggerAction: 'copy_prompt_local_file_links',
        });
        actualOnError?.(err);
        return false;
    }
}

/**
 * Formats a markdown prompt by converting all referenced images (local files or URLs) into full Base64 data URIs.
 * This creates a 100% portable, self-contained markdown document suitable for any LLM or web editor.
 */
export async function formatMarkdownWithBase64Images(markdownText: string, mediaPaths?: string[]): Promise<string> {
    if (!markdownText && (!mediaPaths || mediaPaths.length === 0)) return '';

    let text = markdownText || '';

    // Append missing mediaPaths
    if (mediaPaths && mediaPaths.length > 0) {
        const missing = mediaPaths.filter((p) => p && !text.includes(p));
        if (missing.length > 0) {
            text = text.trim() + '\n\n' + missing.map((p, idx) => `![Turn Image ${idx + 1}](${p})`).join('\n\n');
        }
    }

    const matches = Array.from(text.matchAll(/!\[(.*?)\]\((.*?)\)/g));
    let result = text;
    for (const match of matches) {
        const full = match[0];
        const alt = match[1];
        const rawSrc = match[2].trim();
        if (rawSrc.startsWith('data:image/')) {
            continue;
        }
        const b64 = await convertImageUriToBase64(rawSrc);
        if (b64) {
            result = result.replace(full, `![${alt}](${b64})`);
        }
    }

    return result;
}

/**
 * Copies prompt markdown with all images embedded as full Base64 data URIs to clipboard.
 */
export async function copyPromptWithBase64Images(
    markdownText: string,
    mediaPaths?: string[],
    onSuccess?: (msg: string) => void,
    onError?: (err: unknown) => void
): Promise<boolean> {
    try {
        const formatted = await formatMarkdownWithBase64Images(markdownText, mediaPaths);
        const ok = await copyToClipboard(formatted);
        if (ok) {
            const msg = 'Copied prompt with embedded Base64 images to clipboard!';
            onSuccess?.(msg);
            showToast({ message: msg, type: 'success' });
            return true;
        }
        return false;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyPromptWithBase64Images',
            triggerAction: 'copy_prompt_base64_images',
        });
        onError?.(err);
        return false;
    }
}

