import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

function reportClipboardStage(err: unknown, triggerAction: string): void {
    useErrorStore.getState().trackWarning(err, {
        source: 'clipboard.copyToClipboard',
        triggerAction,
    });
}

function copyWithExecCommand(text: string): boolean {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.setAttribute('readonly', 'true');
    textArea.style.position = 'fixed';
    textArea.style.left = '-9999px';
    textArea.style.top = '0';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    const successful = document.execCommand('copy');
    document.body.removeChild(textArea);
    return successful;
}

async function copyWithNativeHelper(text: string): Promise<void> {
    const tauriWindow = window as Window & { __TAURI_INTERNALS__?: unknown };
    if (tauriWindow.__TAURI_INTERNALS__ === undefined) {
        throw new Error('native clipboard helper is unavailable outside the desktop app');
    }
    await invoke('copy_text_to_clipboard', { text });
}

/**
 * Copy text to the clipboard.
 * The browser Clipboard API rejects with NotAllowedError when the webview
 * document is not focused. That failure is recorded, then execCommand and the
 * OS helper (`clip` / `pbcopy` / `wl-copy`) are tried. The promise does not
 * reject. A total failure is recorded with captureError and returns false.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
    if (navigator.clipboard && window.isSecureContext) {
        try {
            await navigator.clipboard.writeText(text);
            return true;
        } catch (err) {
            reportClipboardStage(err, 'navigator.clipboard.writeText');
        }
    }

    try {
        if (copyWithExecCommand(text)) {
            return true;
        }
        reportClipboardStage(
            new Error('document.execCommand("copy") returned false'),
            'document.execCommand',
        );
    } catch (err) {
        reportClipboardStage(err, 'document.execCommand');
    }

    try {
        await copyWithNativeHelper(text);
        return true;
    } catch (err) {
        useErrorStore.getState().captureError(err, {
            source: 'clipboard.copyToClipboard',
            triggerAction: 'copy_text_to_clipboard',
        });
        return false;
    }
}
