import {
  useErrorStore,
  recordClickEvent,
  buildCapturedError,
} from '../stores/error-store';

let isInitialized = false;

function handleClickCapture(e: MouseEvent): void {
  const target = e.target as HTMLElement | null;
  if (!target) {
    return;
  }
  const tag = target.tagName ? target.tagName.toLowerCase() : 'element';
  const text = (target.innerText || target.getAttribute('title') || target.getAttribute('aria-label') || '')
    .trim()
    .slice(0, 30);

  const route = typeof window !== 'undefined' ? window.location.pathname : '/';

  recordClickEvent({
    element: tag,
    text: text || undefined,
    action: 'click',
    route,
  });
}

function handleWindowError(event: ErrorEvent): void {
  // Prevent duplicate handling or non-actionable script loading errors
  if (!event.error && !event.message) {
    return;
  }

  const err = event.error || new Error(event.message);
  const route = typeof window !== 'undefined' ? window.location.pathname : '/';

  useErrorStore.getState().captureError(err, {
    source: 'window.onerror',
    endpoint: event.filename,
    triggerAction: 'unhandled_window_error',
  });
}

function handleUnhandledRejection(event: PromiseRejectionEvent): void {
  const reason = event.reason;
  if (!reason) {
    return;
  }

  // If reason is an intentional cancellation, ignore
  if (typeof reason === 'object' && reason !== null && reason.name === 'AbortError') {
    return;
  }

  useErrorStore.getState().captureError(reason, {
    source: 'window.unhandledrejection',
    triggerAction: 'unhandled_promise_rejection',
  });
}

export function initGlobalErrorListeners(): () => void {
  if (isInitialized) {
    return () => {};
  }
  if (typeof window === 'undefined') {
    return () => {};
  }

  isInitialized = true;
  document.addEventListener('click', handleClickCapture, { capture: true, passive: true });
  window.addEventListener('error', handleWindowError);
  window.addEventListener('unhandledrejection', handleUnhandledRejection);

  return () => {
    document.removeEventListener('click', handleClickCapture, { capture: true });
    window.removeEventListener('error', handleWindowError);
    window.removeEventListener('unhandledrejection', handleUnhandledRejection);
    isInitialized = false;
  };
}
