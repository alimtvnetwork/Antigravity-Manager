import { useEffect, useRef } from 'react';
import { useConfigStore } from '../stores/useConfigStore';
import { useInstanceStore } from '../stores/useInstanceStore';
import { useErrorStore } from '../stores/error-store';
import { showToast } from '../components/common/ToastContainer';

interface ParsedShortcut {
    key: string;
    ctrlKey: boolean;
    shiftKey: boolean;
    altKey: boolean;
}

function parseShortcut(shortcut: string): ParsedShortcut {
    const parts = shortcut.split('+').map(p => p.trim().toLowerCase());
    const key = parts[parts.length - 1];
    return {
        key,
        ctrlKey: parts.includes('ctrl') || parts.includes('control') || parts.includes('meta') || parts.includes('cmd'),
        shiftKey: parts.includes('shift'),
        altKey: parts.includes('alt'),
    };
}

function isMatch(event: KeyboardEvent, target: ParsedShortcut): boolean {
    const eventKey = event.key.toLowerCase();
    if (eventKey !== target.key) {
        return false;
    }
    const ctrlMatches = target.ctrlKey ? (event.ctrlKey || event.metaKey) : (!event.ctrlKey && !event.metaKey);
    const shiftMatches = target.shiftKey === event.shiftKey;
    const altMatches = target.altKey === event.altKey;
    return ctrlMatches && shiftMatches && altMatches;
}

export function useFastForwardShortcut(): void {
    const shortcutStr = useConfigStore(state => state.config?.auto_profile_switcher?.fast_forward_shortcut || 'Ctrl+Shift+F');
    const isRotatingRef = useRef(false);

    useEffect(() => {
        const parsed = parseShortcut(shortcutStr);

        const handleKeyDown = async (event: KeyboardEvent) => {
            if (!isMatch(event, parsed)) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();

            if (isRotatingRef.current) {
                return;
            }

            isRotatingRef.current = true;
            const instanceStore = useInstanceStore.getState();
            const activeId = instanceStore.activeInstanceId || 'default';

            try {
                const result = await instanceStore.smartRotateProfileAccount(activeId);
                const resumeText = (result.resumedProjectsCount ?? 0) > 0
                    ? ` · Auto-resumed ${result.resumedProjectsCount} project(s) (<1h)`
                    : '';
                showToast(
                    `Fast-Forward [${shortcutStr}]: Transferred ${result.instanceName} to ${result.accountEmail}${resumeText}`,
                    'success'
                );
            } catch (err: any) {
                console.error('Fast-Forward shortcut execution error:', err);
                const captured = useErrorStore.getState().captureError(err, {
                    source: 'useFastForwardShortcut.ts',
                    triggerComponent: 'GlobalShortcut',
                    triggerAction: 'handleFastForwardShortcut',
                    context: { shortcut: shortcutStr, activeId },
                });
                useErrorStore.getState().openErrorModal(captured);
                showToast(`Fast-Forward failed: ${err?.message || err}`, 'error');
            } finally {
                isRotatingRef.current = false;
            }
        };

        window.addEventListener('keydown', handleKeyDown, true);
        return () => {
            window.removeEventListener('keydown', handleKeyDown, true);
        };
    }, [shortcutStr]);
}
