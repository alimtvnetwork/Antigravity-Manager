import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';

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
            onSuccess?.('Copied clean prompt text to clipboard (rich media unsupported in current context)');
            return true;
        }
        onError?.(new Error('Clipboard API unavailable'));
        return false;
    }

    try {
        let htmlContent = markdownText
            .replace(/\n\n/g, '</p><p>')
            .replace(/\n/g, '<br/>')
            .replace(/!\[(.*?)\]\((.*?)\)/g, '<img src="$2" alt="$1" style="max-width:100%; border-radius:8px; margin:8px 0;" />');

        htmlContent = `<!DOCTYPE html><html><body><p>${htmlContent}</p></body></html>`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([markdownText], { type: 'text/plain' });

        const item = new ClipboardItem({
            'text/html': htmlBlob,
            'text/plain': textBlob,
        });

        await navigator.clipboard.write([item]);
        onSuccess?.('Copied full prompt with embedded rich images to clipboard!');
        return true;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'PromptTree.copyPromptWithRichImages',
            triggerAction: 'clipboard.write',
        });
        const fallbackOk = await copyToClipboard(markdownText);
        if (fallbackOk) {
            onSuccess?.('Copied clean prompt text to clipboard (rich media unsupported in current context)');
            return true;
        }
        onError?.(err);
        return false;
    }
}
