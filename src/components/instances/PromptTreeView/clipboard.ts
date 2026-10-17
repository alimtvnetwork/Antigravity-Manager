import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { showToast } from '../../common/ToastContainer';

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
