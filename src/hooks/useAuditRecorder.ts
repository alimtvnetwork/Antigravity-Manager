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
    type AuditStackFrame,
    type AuditTrigger,
} from '../components/audit/audit-types';

const SOURCE = 'useAuditRecorder';
const SCROLL_THROTTLE_MS = 2000;
const MAX_DESC_LEN = 100;
/** Frames kept per interaction: enough to name the UI call path, cheap to parse. */
const STACK_FRAME_LIMIT = 8;

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
    const interactive = el.closest('button, a, input, select, textarea, [role="button"], [role="tab"], [role="menuitem"], [role="switch"], [role="checkbox"]');
    return (interactive as HTMLElement) || (el as HTMLElement);
}

function extractStackTrace(): { callerInfo: string; stackTrace: string; stackFrames: AuditStackFrame[] } {
    try {
        const stack = new Error().stack || '';
        const lines = stack.split('\n').map((l) => l.trim()).filter(Boolean);
        const filtered = lines.filter(
            (l) =>
                !l.includes('useAuditRecorder') &&
                !l.includes('extractStackTrace') &&
                l !== 'Error'
        );
        const stackFrames = filtered.flatMap((line) => {
            const matched = line.match(/^at\s+(.+?)\s+\((.+)\)$/) || line.match(/^at\s+(.+)$/);
            if (!matched) return [];
            const functionName = matched[2] ? matched[1] : 'anonymous';
            const location = matched[2] || matched[1];
            if (!location.includes('/') && !location.includes('\\')) return [];
            return [{ function: functionName, location }];
        }).slice(0, STACK_FRAME_LIMIT);
        const callerInfo = stackFrames[0]
            ? `${stackFrames[0].function} ${stackFrames[0].location}`
            : filtered[0] || 'anonymous';
        const stackTrace = filtered.slice(0, STACK_FRAME_LIMIT).join('\n');
        return { callerInfo, stackTrace, stackFrames };
    } catch (error) {
        trackHookError(error, 'extractStackTrace');
        return { callerInfo: 'unknown', stackTrace: '', stackFrames: [] };
    }
}

/** Clean and format human text snippet (max 32 chars) */
function extractCleanText(el: Element | null): string {
    if (!el) return '';
    const raw = (el.textContent || '').trim().replace(/\s+/g, ' ');
    if (!raw) return '';
    // Ignore pure numbers or massive blocks
    if (raw.length > 32) return `${raw.slice(0, 30)}...`;
    return raw;
}

/** Check if class name is just Tailwind utility noise */
function isUtilityClassName(cls: string): boolean {
    if (!cls) return true;
    const utilities = /^(grid|group|truncate|flex|inline|block|relative|absolute|fixed|w-|h-|p-|m-|text-|bg-|border-|rounded|shadow-|overflow-|transition-|cursor-|select-|space-|gap-|col-|row-|items-|justify-|self-|shrink|grow|opacity-)/;
    return utilities.test(cls);
}

export interface SemanticInteractionDetails {
    methodName: string;
    cleanMethodLabel: string;
    stepSummary: string;
    elementHierarchy: string[];
    tag: string;
    text?: string;
    title?: string;
    ariaLabel?: string;
    container?: string;
    uiName?: string;
    uiId?: string;
}

/**
 * Resolves human-meaningful labels, step breadcrumbs, and containers for an interaction
 */
function resolveSemanticInteraction(el: Element | null, trigger: AuditTrigger): SemanticInteractionDetails {
    const rawTarget = el as HTMLElement | null;
    const interactive = findInteractiveTarget(el);
    const target = interactive || rawTarget;
    const tag = (target?.tagName || rawTarget?.tagName || 'div').toLowerCase();
    const route = currentRoute();

    // 1. Explicit attributes
    const explicitMethod = target?.closest?.(`[${AUDIT_ATTR_METHOD}]`)?.getAttribute(AUDIT_ATTR_METHOD);
    const uiName = target?.getAttribute('data-ui-name') || target?.closest?.('[data-ui-name]')?.getAttribute('data-ui-name') || undefined;
    const uiId = target?.getAttribute('data-ui-id') || target?.closest?.('[data-ui-id]')?.getAttribute('data-ui-id') || undefined;
    const title = target?.getAttribute('title') || rawTarget?.getAttribute('title') || target?.closest?.('[title]')?.getAttribute('title') || undefined;
    const ariaLabel = target?.getAttribute('aria-label') || rawTarget?.getAttribute('aria-label') || target?.closest?.('[aria-label]')?.getAttribute('aria-label') || undefined;

    // 2. Visible text
    const text = extractCleanText(rawTarget) || extractCleanText(target) || undefined;

    // 3. Container & domain identification
    const instanceCard = target?.closest?.('[data-instance-id]') || rawTarget?.closest?.('[data-instance-id]');
    const instanceId = instanceCard?.getAttribute('data-instance-id') || undefined;
    const instanceHeader = instanceCard?.querySelector('h3, h4, .font-bold')?.textContent?.trim();
    const instanceName = instanceHeader ? instanceHeader.split(/\s+/).slice(0, 3).join(' ') : (instanceId ? `#${instanceId}` : undefined);

    const projectContainer = target?.closest?.('[data-project-name]') || rawTarget?.closest?.('[data-project-name]');
    const projectName = projectContainer?.getAttribute('data-project-name') || undefined;

    const modalContainer = target?.closest?.('[role="dialog"], [data-modal], .modal');
    const modalTitle = modalContainer?.querySelector('h2, h3, .font-semibold, .font-bold')?.textContent?.trim()?.slice(0, 30);

    const isFleetTable = Boolean(target?.closest?.('table') && (target?.closest?.('.fleet-table') || route === '/instances'));
    const isNavbar = Boolean(target?.closest?.('nav, header'));

    // Determine container label
    let container = 'Workspace Area';
    if (modalTitle) {
        container = `Modal: ${modalTitle}`;
    } else if (instanceName) {
        container = `Instance Card: ${instanceName}`;
    } else if (isNavbar) {
        container = 'Navigation Bar';
    } else if (isFleetTable) {
        container = 'Shared Cluster / Fleet Nodes';
    } else if (route === '/instances') {
        container = 'Instances Grid';
    }

    // 4. Resolve clean target name
    let cleanTargetName = '';
    if (uiName) {
        cleanTargetName = uiName
            .replace(/-(btn|tab|menu|card|item|view)$/, '')
            .replace(/-/g, ' ')
            .replace(/\b\w/g, (c) => c.toUpperCase());
    } else if (title && title.length <= 40) {
        cleanTargetName = title;
    } else if (ariaLabel && ariaLabel.length <= 40) {
        cleanTargetName = ariaLabel;
    } else if (text && text.length <= 30 && !isUtilityClassName(text)) {
        cleanTargetName = `"${text}"`;
    } else if (projectName) {
        cleanTargetName = `Project "${projectName}"`;
    } else if (instanceName) {
        cleanTargetName = `Instance "${instanceName}"`;
    } else if (tag === 'button') {
        cleanTargetName = title || ariaLabel || (text ? `"${text}" Button` : 'Action Button');
    } else if (tag === 'a') {
        cleanTargetName = text ? `Link "${text}"` : 'Navigation Link';
    } else if (tag === 'input' || tag === 'select' || tag === 'textarea') {
        cleanTargetName = `${tag.toUpperCase()} Field`;
    } else {
        cleanTargetName = container;
    }

    // 5. Build method name and clean method label
    let methodName = explicitMethod || '';
    let cleanMethodLabel = '';
    const actionVerb = trigger === 'click' ? 'Click' : trigger === 'input' ? 'Input' : 'Action';

    if (!methodName) {
        if (uiName) {
            methodName = `${trigger}:${uiName}`;
            cleanMethodLabel = `${actionVerb}: ${cleanTargetName}`;
        } else if (title && title.length <= 35) {
            methodName = `${trigger}:button("${title}")`;
            cleanMethodLabel = `${actionVerb}: "${title}"`;
        } else if (ariaLabel && ariaLabel.length <= 35) {
            methodName = `${trigger}:button("${ariaLabel}")`;
            cleanMethodLabel = `${actionVerb}: "${ariaLabel}"`;
        } else if (text && text.length <= 25 && !isUtilityClassName(text)) {
            methodName = `${trigger}:"${text}"`;
            cleanMethodLabel = `${actionVerb}: "${text}"`;
        } else if (instanceName) {
            methodName = `${trigger}:instance-card("${instanceName}")`;
            cleanMethodLabel = `${actionVerb}: Instance "${instanceName}"`;
        } else if (route === '/instances') {
            methodName = `${trigger}:instances-grid-area`;
            cleanMethodLabel = `${actionVerb}: Instances Grid Area`;
        } else {
            methodName = `${trigger}:${cleanTargetName.toLowerCase().replace(/\s+/g, '-')}`;
            cleanMethodLabel = `${actionVerb}: ${cleanTargetName}`;
        }
    } else {
        cleanMethodLabel = methodName
            .replace(/^(handle|on)/, '')
            .replace(/([A-Z])/g, ' $1')
            .trim();
    }

    // 6. Build execution breadcrumb steps hierarchy
    const elementHierarchy: string[] = [`Route: ${route}`];
    if (container && !elementHierarchy.includes(container)) {
        elementHierarchy.push(container);
    }
    if (projectName && !elementHierarchy.includes(`Project: ${projectName}`)) {
        elementHierarchy.push(`Project: ${projectName}`);
    }
    elementHierarchy.push(`${tag.toUpperCase()}: ${cleanTargetName}`);

    // 7. Full human step summary sentence
    let stepSummary = `User ${trigger}ed ${cleanTargetName} on ${route}`;
    if (instanceName) {
        stepSummary = `User ${trigger}ed ${cleanTargetName} in Instance "${instanceName}" on ${route}`;
    } else if (container && container !== 'Workspace Area') {
        stepSummary = `User ${trigger}ed ${cleanTargetName} in ${container} on ${route}`;
    }

    return {
        methodName,
        cleanMethodLabel,
        stepSummary,
        elementHierarchy,
        tag,
        text,
        title,
        ariaLabel,
        container,
        uiName,
        uiId,
    };
}

/** Compact, secret-free description of an element for uiTarget */
function describeElement(el: Element | null): string {
    const target = findInteractiveTarget(el);
    if (!target) return 'unknown';
    const semantic = resolveSemanticInteraction(el, 'click');
    const uiPrefix = semantic.uiName ? `[${semantic.uiName}] ` : '';
    const id = target.id ? `#${target.id}` : (semantic.uiId ? `[${semantic.uiId}]` : '');
    const cleanLabel = semantic.title || semantic.ariaLabel || semantic.text;
    const labelDesc = cleanLabel ? `("${cleanLabel.slice(0, 24)}")` : '';
    const desc = `${uiPrefix}${semantic.tag}${id}${labelDesc}`;
    return desc.slice(0, MAX_DESC_LEN);
}

/** Nearest opt-in component name, else 'unknown'. */
function componentOf(el: Element | null): string {
    const target = findInteractiveTarget(el);
    const tagged = target?.closest?.(`[${AUDIT_ATTR_COMPONENT}]`);
    return tagged?.getAttribute(AUDIT_ATTR_COMPONENT) || 'unknown';
}

/** `route | component | element` per the backend ui_target convention. */
function uiTargetOf(el: Element | null): string {
    const target = findInteractiveTarget(el);
    const semantic = resolveSemanticInteraction(el, 'click');
    const comp = componentOf(target);
    const effectiveComp = comp !== 'unknown' ? comp : semantic.container;
    return `${currentRoute()} | ${effectiveComp} | ${describeElement(target)}`;
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
    // The backend `RecordInteractionInput` requires top-level `route`
    // (serde camelCase, no default). The recorder historically sent it only
    // nested inside `params`, so every frontend-initiated record was rejected
    // with a missing-field deserialization error. Inject it here so all
    // call sites are fixed at once.
    const backendPayload: AuditRecordPayload = {
        ...payload,
        route: payload.route ?? currentRoute(),
    };
    // Intentionally not awaited: reporting must never block the interaction.
    // auditRecord routes its own failures into the error module.
    void auditRecord(backendPayload).catch((error) => {
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
                const rawEl = event.target as HTMLElement | null;
                const targetEl = rawEl?.closest?.('button, a, input, select, textarea, [role="button"]') || rawEl;
                const semantic = resolveSemanticInteraction(rawEl, 'click');
                const { callerInfo, stackTrace, stackFrames } = extractStackTrace();
                const methodName = semantic.methodName;

                report({
                    actor: actorOf(event),
                    methodName,
                    uiTarget: uiTargetOf(rawEl),
                    trigger: 'click',
                    params: {
                        caller: callerInfo,
                        callerInfo,
                        stack: stackTrace,
                        stackTrace,
                        stack_frames: stackFrames,
                        metadata: {
                            callerInfo,
                            stackTrace,
                        },
                        stepSummary: semantic.stepSummary,
                        cleanMethodLabel: semantic.cleanMethodLabel,
                        elementHierarchy: semantic.elementHierarchy,
                        container: semantic.container,
                        uiName: semantic.uiName || undefined,
                        uiId: semantic.uiId || undefined,
                        button: event.button,
                        tag: semantic.tag,
                        title: semantic.title || undefined,
                        ariaLabel: semantic.ariaLabel || undefined,
                        textSnippet: semantic.text || undefined,
                        href: targetEl instanceof HTMLAnchorElement ? targetEl.href : undefined,
                        route: currentRoute(),
                    },
                    // Promoted by the backend into the dedicated stack_trace column.
                    stackTrace: stackFrames,
                    result: 'ok',
                });
            } catch (error) {
                trackHookError(error, 'handleClick');
            }
        };

        const handleInputCommit = (event: Event) => {
            try {
                if (!isRecordingActive()) return;
                const rawEl = event.target as HTMLElement | null;
                const targetEl = rawEl?.closest?.('input, select, textarea') || rawEl;
                const semantic = resolveSemanticInteraction(rawEl, 'input');
                const isPassword = targetEl instanceof HTMLInputElement && targetEl.type === 'password';
                const { callerInfo, stackTrace, stackFrames } = extractStackTrace();
                const methodName = semantic.methodName;

                report({
                    actor: actorOf(event),
                    methodName,
                    uiTarget: uiTargetOf(rawEl),
                    trigger: 'input',
                    params: {
                        caller: callerInfo,
                        callerInfo,
                        stack: stackTrace,
                        stackTrace,
                        stack_frames: stackFrames,
                        metadata: {
                            callerInfo,
                            stackTrace,
                        },
                        stepSummary: semantic.stepSummary,
                        cleanMethodLabel: semantic.cleanMethodLabel,
                        elementHierarchy: semantic.elementHierarchy,
                        container: semantic.container,
                        uiName: semantic.uiName || undefined,
                        uiId: semantic.uiId || undefined,
                        tag: semantic.tag,
                        title: semantic.title || undefined,
                        ariaLabel: semantic.ariaLabel || undefined,
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
                    // Promoted by the backend into the dedicated stack_trace column.
                    stackTrace: stackFrames,
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
                const route = currentRoute();
                report({
                    actor: 'user',
                    methodName: `scroll:${route}`,
                    uiTarget: `${route} | window | scroll`,
                    trigger: 'scroll',
                    params: {
                        scrollX: window.scrollX,
                        scrollY: window.scrollY,
                        route,
                        stepSummary: `User scrolled the ${route} page`,
                        cleanMethodLabel: `Scroll: ${route} Page`,
                        elementHierarchy: [`Route: ${route}`, 'Window Viewport Scroll'],
                        container: 'Window Viewport',
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
