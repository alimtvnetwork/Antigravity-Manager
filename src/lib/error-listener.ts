import {
  useErrorStore,
  recordClickEvent,
} from '../stores/error-store';

let isInitialized = false;

function getElementXPath(element: HTMLElement): string {
  const dataXpath = element.getAttribute('data-xpath');
  if (dataXpath) {
    return dataXpath;
  }
  if (element.id) {
    return `//*[@id='${element.id}']`;
  }
  const name = element.getAttribute('name');
  if (name) {
    return `//${element.tagName.toLowerCase()}[@name='${name}']`;
  }
  const parts: string[] = [];
  let curr: HTMLElement | null = element;
  while (curr && curr.nodeType === Node.ELEMENT_NODE && curr !== document.body && curr !== document.documentElement) {
    let index = 1;
    let sibling = curr.previousElementSibling;
    while (sibling) {
      if (sibling.tagName === curr.tagName) {
        index++;
      }
      sibling = sibling.previousElementSibling;
    }
    const tag = curr.tagName.toLowerCase();
    parts.unshift(`${tag}[${index}]`);
    curr = curr.parentElement;
  }
  return `/${parts.join('/')}`;
}

function handleClickCapture(e: MouseEvent): void {
  const target = e.target as HTMLElement | null;
  if (!target) {
    return;
  }

  // Find nearest interactive ancestor (button, a, input, [role="button"], or element with id)
  const interactive = (target.closest('button, a, input, select, textarea, [role="button"], [id]') as HTMLElement) || target;

  const tag = interactive.tagName ? interactive.tagName.toLowerCase() : 'element';
  const targetId = interactive.id || interactive.getAttribute('id') || undefined;
  const name = interactive.getAttribute('name') || undefined;
  const ariaLabel = interactive.getAttribute('aria-label') || undefined;
  const title = interactive.getAttribute('title') || undefined;
  const text = (interactive.innerText || title || ariaLabel || name || targetId || '')
    .trim()
    .slice(0, 40);

  const xpath = getElementXPath(interactive);
  const route = typeof window !== 'undefined' ? window.location.pathname : '/';

  recordClickEvent({
    element: tag,
    text: text || undefined,
    action: 'click',
    targetId,
    name,
    xpath,
    route,
  });
}

function handleWindowError(event: ErrorEvent): void {
  // Prevent duplicate handling or non-actionable script loading errors
  if (!event.error && !event.message) {
    return;
  }

  const err = event.error || new Error(event.message);

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
