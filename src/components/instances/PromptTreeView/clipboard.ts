import { copyToClipboard } from '../../../utils/clipboard';

export async function copyPromptWithRichImages(markdownText: string): Promise<boolean> {
    try {
        if (!navigator.clipboard || typeof ClipboardItem === 'undefined') {
            await navigator.clipboard.writeText(markdownText);
            return true;
        }

        // Convert markdown images to HTML <img> tags
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
        return true;
    } catch {
        // Fallback to standard text copy if rich clipboard write fails
        try {
            await navigator.clipboard.writeText(markdownText);
            return true;
        } catch {
            return false;
        }
    }
}

