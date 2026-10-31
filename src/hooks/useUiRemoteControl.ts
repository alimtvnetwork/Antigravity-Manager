import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { listen } from '@tauri-apps/api/event';
import { useErrorStore } from '../stores/error-store';
import { logUiTrace } from '../utils/uiLogger';

interface UiNavigatePayload {
    route: string;
    actor: string;
}

interface UiElementActionPayload {
    element_id: string;
    route?: string;
    actor: string;
}

/**
 * Listens for backend-driven UI remote control (`agm://ui-navigate`,
 * `agm://ui-scroll`, `agm://ui-click` Tauri events, emitted by
 * POST /ui/navigate, POST /ui/scroll, POST /ui/click). This is how
 * `agm ui goto|scroll|click` and the Telegram `/ui` commands drive the
 * running UI — one code path, no polling.
 *
 * Task-04 (auditing-and-cli-ui-integration): scroll/click listeners appended.
 * The CLI validates the element id against the static manifest and records
 * the audit entry (`actor=cli`); this hook performs the real DOM action, so
 * behavior matches a GUI interaction exactly. Missing elements are tracked
 * as warnings (never a bare catch).
 */
export function useUiRemoteControl() {
    const navigate = useNavigate();

    useEffect(() => {
        const unlistens: Array<() => void> = [];
        (async () => {
            try {
                unlistens.push(
                    await listen<UiNavigatePayload>('agm://ui-navigate', (event) => {
                        const route = event.payload?.route?.trim();
                        if (!route) return;
                        // React Router path: ensure leading slash.
                        const path = route.startsWith('/') ? route : `/${route}`;
                        navigate(path);
                    }),
                    await listen<UiElementActionPayload>('agm://ui-scroll', (event) => {
                        const id = event.payload?.element_id?.trim();
                        if (!id) return;
                        const el = document.getElementById(id);
                        if (!el) {
                            useErrorStore.getState().trackWarning(
                                new Error(`agm://ui-scroll target #${id} not found in the live DOM`),
                                {
                                    source: 'useUiRemoteControl.scroll',
                                    triggerAction: 'agm://ui-scroll',
                                },
                            );
                            return;
                        }
                        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
                        el.focus({ preventScroll: true });
                    }),
                    await listen<UiElementActionPayload>('agm://ui-click', (event) => {
                        const id = event.payload?.element_id?.trim();
                        if (!id) return;
                        const escapedId = id.replace(/"/g, '\\"');
                        const el =
                            document.getElementById(id) ||
                            (document.querySelector(`[data-ui-id="${escapedId}"]`) as HTMLElement | null) ||
                            (document.querySelector(`[data-ui-name="${escapedId}"]`) as HTMLElement | null) ||
                            (document.querySelector(`[data-testid="${escapedId}"]`) as HTMLElement | null) ||
                            (document.querySelector(`[title="${escapedId}"]`) as HTMLElement | null);
                        if (!el) {
                            useErrorStore.getState().trackWarning(
                                new Error(`agm://ui-click target #${id} not found in the live DOM`),
                                {
                                    source: 'useUiRemoteControl.click',
                                    triggerAction: 'agm://ui-click',
                                },
                            );
                            return;
                        }
                        logUiTrace({ step: 'remote_control', action: 'cli_click', target: id });
                        // Same code path as a user click: the audit recorder
                        // logs it with actor=cli on the backend side.
                        el.click();
                    }),
                );
            } catch (e) {
                useErrorStore.getState().trackWarning(e, {
                    source: 'useUiRemoteControl.listen',
                    triggerAction: 'subscribe agm://ui-navigate|ui-scroll|ui-click',
                });
            }
        })();
        return () => {
            for (const unlisten of unlistens) {
                try {
                    unlisten();
                } catch (e) {
                    useErrorStore.getState().trackWarning(e, {
                        source: 'useUiRemoteControl.cleanup',
                        triggerAction: 'unsubscribe',
                    });
                }
            }
        };
    }, [navigate]);
}
