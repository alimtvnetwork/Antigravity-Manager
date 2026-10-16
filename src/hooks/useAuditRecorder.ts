/**
 * useAuditRecorder — global interaction listener for the audit recorder.
 *
 * Mount once (e.g. in `Layout` next to `useFastForwardShortcut`) and it
 * captures clicks, input commits, scrolls, and navigations while a
 * recording session is active, reporting each interaction to the backend
 * `audit_record` command.
 *
 * Lightweight by design: a handful of delegated document-level listeners,
 * throttled scroll reporting, and `getState()` reads so captured events
 * never cause React re-renders.
 *
 * Privacy: input VALUES are never captured — only the input type and
 * value length. Password fields are described as `input[type=password]`
 * with no length at all.
 */
import { useEffect, useRef } from 'react';
import { useErrorStore } from '../stores/error-store';
import { auditRecord } from '../components/audit/audit-client';
import { useAuditStore } from '../components/audit/audit-store';
import {
    AUDIT_ATTR_COMPONENT,
    AUDIT_ATTR_METHOD,
    type AuditRecordPayload,
    type AuditTrigger,
} from '../components/audit/audit-types';

const SOURCE = 'useAuditRecorder';
const SCROLL_THROTTLE_MS = 2000;
const MAX_DESC_LEN = 80;

function isRecordingActive(): boolean {
    return useAuditStore.getState().isRecording;
}

function currentRoute(): string {
    if (typeof window === 'undefined') return '/';
    // createBrowserRouter (HTML5 history) — fall back to hash for safety.
    return window.location.hash?.replace(/^#/, '') || window.location.pathname || '/';
}

/** Climbs to nearest interactive element or falls back to original target */
function findInteractiveTarget(el: Element | null): HTMLElement | null {
    if (!el) return null;
    const interactive = el.closest('button, a, input, select, textarea, [role="button"]');
    return (interactive as HTMLElement) || (el as HTMLElement);
}

function extractStackTrace(): { callerInfo: string; stackTrace: string } {
    try {
        const stack = new Error().stack || '';
        const lines = stack.split('\n').map((l) => l.trim()).filter(Boolean);
        // Skip first line (Error) and hook internal frames
        const filtered = lines.filter(
            (l) =>
                !l.includes('useAuditRecorder') &&
                !l.includes('extractStackTrace') &&
                !l.includes('Error') &&
                !l.includes('report')
        );
        const callerInfo = filtered[0] || 'anonymous';
        const stackTrace = filtered.slice(0, 8).join('\n');
        return { callerInfo, stackTrace };
    } catch {
        return { callerInfo: 'unknown', stackTrace: '' };
    }
}

/** Compact, secret-free description of an element: `button#save.primary`. */
function describeElement(el: Element | null): string {
    const target = findInteractiveTarget(el);
    if (!target) return 'unknown';
    const tag = target.tagName.toLowerCase();
    const id = target.id ? `#${target.id}` : '';
    const firstClass = target.className && typeof target.className === 'string'
        ? target.className.trim().split(/\s+/)[0]
        : '';
    const cls = firstClass ? `.${firstClass}` : '';
    const label = target.getAttribute('aria-label');
    const text = !label && (tag === 'button' || tag === 'a' || target.getAttribute('role') === 'button')
        ? (target.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 24)
        : '';
    const desc = `${tag}${id}${cls}${label ? `[aria:${label.slice(0, 24)}]` : ''}${text ? `("${text}")` : ''}`;
    return desc.slice(0, MAX_DESC_LEN);
}

/** Nearest opt-in component name, else 'unknown'. */
function componentOf(el: Element | null): string {
    const target = findInteractiveTarget(el);
    const tagged = target?.closest?.(`[${AUDIT_ATTR_COMPONENT}]`);
    return tagged?.getAttribute(AUDIT_ATTR_COMPONENT) || 'unknown';
}

/** Handler name: opt-in attribute wins, otherwise an inferred descriptor. */
function methodOf(el: Element | null, trigger: AuditTrigger): string {
    const target = findInteractiveTarget(el);
    const tagged = target?.closest?.(`[${AUDIT_ATTR_METHOD}]`);
    const explicit = tagged?.getAttribute(AUDIT_ATTR_METHOD);
    if (explicit) return explicit;
    return `${trigger}:${describeElement(target)}`;
}

/** `route | component | element` per the backend ui_target convention. */
function uiTargetOf(el: Element | null): string {
    const target = findInteractiveTarget(el);
    return `${currentRoute()} | ${componentOf(target)} | ${describeElement(target)}`;
}

function trackHookError(error: unknown, triggerAction: string, context?: Record<string, unknown>): void {
    useErrorStore.getState().trackWarning(error, {
        source: SOURCE,
        triggerComponent: 'GlobalInteractionListener',
        triggerAction,
        context,
    });
}

/** Fire-and-forget report; every failure is tracked, never swallowed. */
function report(payload: AuditRecordPayload): void {
    // Intentionally not awaited: reporting must never block the interaction.
    // auditRecord routes its own failures into the error module.
    void auditRecord(payload).catch((error) => {
        trackHookError(error, 'report', { methodName: payload.methodName });
    });
}

function actorOf(event: Event): 'user' | 'system' {
    // Synthetic/programmatic dispatches are still worth recording, but
    // attributed honestly.
    return event.isTrusted ? 'user' : 'system';
}

export function useAuditRecorder(): void {
    const lastScrollReport = useRef(0);
    const scrollEndTimer = useRef<number | null>(null);

    useEffect(() => {
        // Reconcile with the backend once on mount (e.g. a recording left
        // running across a reload).
        void useAuditStore.getState().syncRecordingState().catch((error) => {
            trackHookError(error, 'syncRecordingState');
        });

        const handleClick = (event: MouseEvent) => {
            try {
                if (!isRecordingActive()) return;
                const targetEl = (event.target as HTMLElement)?.closest?.('button, a, input, select, textarea, [role="button"]') || (event.target as HTMLElement | null);
                const explicitMethod = targetEl?.closest?.(`[${AUDIT_ATTR_METHOD}]`)?.getAttribute(AUDIT_ATTR_METHOD);
                const { callerInfo, stackTrace } = extractStackTrace();
                const methodName = explicitMethod || methodOf(targetEl, 'click');

                report({
                    actor: actorOf(event),
                    methodName,
                    uiTarget: uiTargetOf(targetEl),
                    trigger: 'click',
                    params: {
                        caller: callerInfo,
                        callerInfo,
                        stack: stackTrace,
                        stackTrace,
                        metadata: {
                            callerInfo,
                            stackTrace,
                        },
                        button: event.button,
                        tag: targetEl?.tagName?.toLowerCase(),
                        ariaLabel: targetEl?.getAttribute?.('aria-label') || undefined,
                        textSnippet: (targetEl?.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 24) || undefined,
                        href: targetEl instanceof HTMLAnchorElement ? targetEl.href : undefined,
                        route: currentRoute(),
                    },
                    result: 'ok',
                });
            } catch (error) {
                trackHookError(error, 'handleClick');
            }
        };

        const handleInputCommit = (event: Event) => {
            try {
                if (!isRecordingActive()) return;
                const targetEl = (event.target as HTMLElement)?.closest?.('input, select, textarea') || (event.target as HTMLElement | null);
                const isPassword = targetEl instanceof HTMLInputElement && targetEl.type === 'password';
                const { callerInfo, stackTrace } = extractStackTrace();
                const explicitMethod = targetEl?.closest?.(`[${AUDIT_ATTR_METHOD}]`)?.getAttribute(AUDIT_ATTR_METHOD);
                const methodName = explicitMethod || methodOf(targetEl, 'input');

                report({
                    actor: actorOf(event),
                    methodName,
                    uiTarget: uiTargetOf(targetEl),
                    trigger: 'input',
                    params: {
                        caller: callerInfo,
                        callerInfo,
                        stack: stackTrace,
                        stackTrace,
                        metadata: {
                            callerInfo,
                            stackTrace,
                        },
                        tag: targetEl?.tagName?.toLowerCase(),
                        inputType: targetEl instanceof HTMLInputElement ? targetEl.type : undefined,
                        // Values are NEVER captured (secrets). Length only,
                        // and not even that for password fields.
                        valueLength: isPassword
                            ? undefined
                            : targetEl instanceof HTMLInputElement || targetEl instanceof HTMLTextAreaElement
                                ? targetEl.value.length
                                : undefined,
                        route: currentRoute(),
                    },
                    result: 'ok',
                });
            } catch (error) {
                trackHookError(error, 'handleInputCommit');
            }
        };

        const reportScroll = (immediate: boolean) => {
            try {
                if (!isRecordingActive()) return;
                const now = Date.now();
                if (!immediate && now - lastScrollReport.current < SCROLL_THROTTLE_MS) return;
                lastScrollReport.current = now;
                report({
                    actor: 'user',
                    methodName: `scroll:${currentRoute()}`,
                    uiTarget: `${currentRoute()} | unknown | window`,
                    trigger: 'scroll',
                    params: {
                        scrollX: window.scrollX,
                        scrollY: window.scrollY,
                        route: currentRoute(),
                    },
                    result: 'ok',
                });
            } catch (error) {
                trackHookError(error, 'reportScroll');
            }
        };

        const handleScroll = () => {
            // Throttled while scrolling + one final report when scrolling stops.
            reportScroll(false);
            if (scrollEndTimer.current !== null) {
                window.clearTimeout(scrollEndTimer.current);
            }
            scrollEndTimer.current = window.setTimeout(() => {
                scrollEndTimer.current = null;
                reportScroll(true);
            }, 900);
        };

        const handleNavigation = (kind: 'pushState' | 'replaceState' | 'popstate' | 'hashchange') => {
            try {
                if (!isRecordingActive()) return;
                report({
                    actor: 'user',
                    methodName: `history.${kind}`,
                    uiTarget: `${currentRoute()} | unknown | window`,
                    trigger: 'navigation',
                    params: { kind, route: currentRoute() },
                    result: 'ok',
                });
            } catch (error) {
                trackHookError(error, 'handleNavigation', { kind });
            }
        };

        // Wrap history methods so SPA navigations are observed without
        // touching the router setup.
        const origPushState = window.history.pushState.bind(window.history);
        const origReplaceState = window.history.replaceState.bind(window.history);
        window.history.pushState = (...args) => {
            origPushState(...args);
            handleNavigation('pushState');
        };
        window.history.replaceState = (...args) => {
            origReplaceState(...args);
            handleNavigation('replaceState');
        };

        const onPopState = () => handleNavigation('popstate');
        const onHashChange = () => handleNavigation('hashchange');

        document.addEventListener('click', handleClick, true);
        // `change` fires on commit (selects, checkboxes, blurred text inputs)
        // — far less noisy than per-keystroke `input`, and never sees values.
        document.addEventListener('change', handleInputCommit, true);
        window.addEventListener('scroll', handleScroll, { passive: true, capture: true });
        window.addEventListener('popstate', onPopState);
        window.addEventListener('hashchange', onHashChange);

        return () => {
            document.removeEventListener('click', handleClick, true);
            document.removeEventListener('change', handleInputCommit, true);
            window.removeEventListener('scroll', handleScroll, true);
            window.removeEventListener('popstate', onPopState);
            window.removeEventListener('hashchange', onHashChange);
            window.history.pushState = origPushState;
            window.history.replaceState = origReplaceState;
            if (scrollEndTimer.current !== null) {
                window.clearTimeout(scrollEndTimer.current);
                scrollEndTimer.current = null;
            }
        };
    }, []);
}
