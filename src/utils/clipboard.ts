import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export function ensureDocumentFocus(): void {
    if (typeof window !== 'undefined' && typeof window.focus === 'function' && !document.hasFocus()) {
        try {
            window.focus();
        } catch {
            // best-effort window focus
        }
    }
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
 * Copy text to the clipboard with resilient multi-stage fallback:
 * 1. Ensures document focus and tries modern Clipboard API.
 * 2. If Clipboard API rejects (e.g. document not focused), falls back to document.execCommand('copy') which forces textarea focus.
 * 3. If DOM execCommand fails, invokes native OS helper (`clip` / `pbcopy` / `wl-copy`) which never requires focus.
 * Only captures an error if all 3 fallback stages fail.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
    ensureDocumentFocus();

    // Stage 1: Modern Async Clipboard API
    if (navigator.clipboard && window.isSecureContext && document.hasFocus()) {
        try {
            await navigator.clipboard.writeText(text);
            return true;
        } catch {
            // Document focus may have been lost during async tick, fall through to execCommand
        }
    }

    // Stage 2: document.execCommand (forces element focus)
    try {
        if (copyWithExecCommand(text)) {
            return true;
        }
    } catch {
        // Fall through to native OS helper
    }

    // Stage 3: Native desktop helper (clip / pbcopy / wl-copy)
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
