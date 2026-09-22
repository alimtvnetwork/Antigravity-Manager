import { create } from 'zustand';

export interface StackFrame {
  function: string;
  file: string;
  line: number;
  column?: number;
  isInternal: boolean;
}

export interface ClickEvent {
  id: string;
  element: string;
  text?: string;
  action: string;
  componentName?: string;
  route?: string;
  timestamp: number;
}

export interface ErrorContext {
  source?: string;
  triggerComponent?: string;
  triggerAction?: string;
  [key: string]: string | number | boolean | null | undefined;
}

export interface EnvelopeErrors {
  BackendMessage: string;
  DelegatedServiceErrorStack?: string[];
  Backend?: string[];
  Frontend?: string[];
}

export interface CapturedError {
  id: string;
  code: string;
  level: 'error' | 'warn' | 'info';
  message: string;
  details?: string;
  createdAt: string;
  context?: ErrorContext;
  file?: string;
  line?: number;
  function?: string;
  stackTrace?: string;
  parsedFrames?: StackFrame[];
  endpoint?: string;
  method?: string;
  requestBody?: string;
  responseStatus?: number;
  invocationChain?: string[];
  triggerComponent?: string;
  triggerAction?: string;
  backendLogs?: Array<{ timestamp: string; level: string; message: string }>;
  backendStackTrace?: string;
  uiClickPath?: ClickEvent[];
  uiClickPathString?: string;
  uiClickPathArrow?: string;
  route?: string;
  routeComponent?: string;
  requestedAt?: string;
  envelopeErrors?: EnvelopeErrors;
}

export interface CaptureErrorMeta {
  source?: string;
  triggerComponent?: string;
  triggerAction?: string;
  endpoint?: string;
  method?: string;
  status?: number;
  requestBody?: string;
  context?: ErrorContext;
}

// In-memory ring buffer for the last 10 user clicks
const MAX_CLICKS = 10;
const clickHistory: ClickEvent[] = [];

export function recordClickEvent(event: Omit<ClickEvent, 'id' | 'timestamp'>): void {
  const item: ClickEvent = {
    ...event,
    id: Math.random().toString(36).substring(2, 9),
    timestamp: Date.now(),
  };
  if (clickHistory.length >= MAX_CLICKS) {
    clickHistory.shift();
  }
  clickHistory.push(item);
}

export function getRecentClicks(): ClickEvent[] {
  return [...clickHistory];
}

export function formatClickPathArrow(clicks: ClickEvent[]): string {
  if (clicks.length === 0) {
    return 'No prior interactions recorded';
  }
  return clicks
    .map((c) => `${c.element}${c.text ? ` "${c.text}"` : ''}`)
    .join(' → ');
}

export function parseFullStackTrace(stack?: string): {
  frames: StackFrame[];
  primaryFrame: StackFrame | null;
  invocationChain: string[];
} {
  if (!stack) {
    return { frames: [], primaryFrame: null, invocationChain: [] };
  }
  const lines = stack.split('\n');
  const frames: StackFrame[] = [];
  const chain: string[] = [];

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('at ')) {
      const parsed = parseStackLine(trimmed);
      if (parsed) {
        frames.push(parsed);
        if (!parsed.isInternal) {
          chain.push(parsed.function);
        }
      }
    }
  }

  const primary = frames.find((f) => !f.isInternal) || frames[0] || null;
  return { frames, primaryFrame: primary, invocationChain: chain.slice(0, 8) };
}

function parseStackLine(line: string): StackFrame | null {
  const match = line.match(/^at\s+(?:async\s+)?([^\s(]+)?\s*\(?([^:)]+):(\d+):(\d+)\)?$/);
  if (!match) {
    return null;
  }
  const fnName = match[1] || '<anonymous>';
  const filePath = match[2] || '';
  const lineNum = parseInt(match[3], 10) || 0;
  const colNum = parseInt(match[4], 10) || 0;
  const isInternal = checkIsInternalFrame(filePath);

  return {
    function: fnName,
    file: filePath,
    line: lineNum,
    column: colNum,
    isInternal,
  };
}

function checkIsInternalFrame(path: string): boolean {
  return (
    path.includes('node_modules') ||
    path.includes('@tauri-apps') ||
    path.includes('react-dom') ||
    path.includes('react.development') ||
    path.includes('scheduler')
  );
}

function extractErrorCode(raw: string): string {
  const match = raw.match(/\[(E\d{4})\]/);
  if (match) {
    return match[1];
  }
  return 'E9001';
}

function normalizeRawError(err: unknown): {
  code: string;
  level: 'error' | 'warn' | 'info';
  message: string;
  details?: string;
  backendStack?: string;
  envelopeErrors?: EnvelopeErrors;
  status?: number;
} {
  let target = err;
  if (typeof err === 'string') {
    const trimmed = err.trim();
    if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
      try {
        target = JSON.parse(trimmed);
      } catch {
        target = err;
      }
    }
  }

  if (typeof target === 'object' && target !== null) {
    const record = target as Record<string, any>;
    const statusObj = record.Status;
    const msg = record.message || statusObj?.Message || record.error || String(err);
    const code = record.code || extractErrorCode(msg);
    const level = (record.level as 'error' | 'warn' | 'info') || 'error';
    const status = record.status || statusObj?.Code || 500;
    const backendStack = record.backend_stack_trace || record.Errors?.Backend?.join('\n');

    return {
      code,
      level,
      message: msg,
      details: record.details,
      backendStack,
      envelopeErrors: record.Errors,
      status,
    };
  }

  const str = String(err);
  return {
    code: extractErrorCode(str),
    level: 'error',
    message: str,
  };
}

export function buildCapturedError(
  rawError: unknown,
  meta?: CaptureErrorMeta,
  context?: ErrorContext
): CapturedError {
  const norm = normalizeRawError(rawError);
  const stack = rawError instanceof Error ? rawError.stack : undefined;
  const parsed = parseFullStackTrace(stack);
  const clicks = getRecentClicks();
  const currentRoute = typeof window !== 'undefined' ? window.location.pathname : '/';

  return {
    id: typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : Math.random().toString(36),
    code: norm.code,
    level: norm.level,
    message: norm.message,
    details: norm.details,
    createdAt: new Date().toISOString(),
    context: {
      ...meta?.context,
      ...context,
      source: meta?.source,
      triggerComponent: meta?.triggerComponent,
      triggerAction: meta?.triggerAction,
    },
    file: parsed.primaryFrame?.file,
    line: parsed.primaryFrame?.line,
    function: parsed.primaryFrame?.function,
    stackTrace: stack,
    parsedFrames: parsed.frames,
    endpoint: meta?.endpoint,
    method: meta?.method,
    requestBody: meta?.requestBody,
    responseStatus: norm.status || meta?.status,
    invocationChain: parsed.invocationChain,
    triggerComponent: meta?.triggerComponent,
    triggerAction: meta?.triggerAction,
    backendStackTrace: norm.backendStack,
    uiClickPath: clicks,
    uiClickPathArrow: formatClickPathArrow(clicks),
    route: currentRoute,
    envelopeErrors: norm.envelopeErrors,
  };
}

export type ErrorModalTab = 'overview' | 'backend' | 'stack' | 'context';

interface ErrorStoreState {
  selectedError: CapturedError | null;
  isModalOpen: boolean;
  activeTab: ErrorModalTab;
  recentErrors: CapturedError[];
  errorQueue: CapturedError[];
  currentQueueIndex: number;

  captureError: (error: unknown, meta?: CaptureErrorMeta) => CapturedError;
  captureException: (error: Error | string, context?: ErrorContext) => CapturedError;
  openErrorModal: (error: CapturedError, initialTab?: ErrorModalTab) => void;
  setActiveTab: (tab: ErrorModalTab) => void;
  openErrorQueue: (errors: CapturedError[], startIndex?: number) => void;
  navigateQueue: (direction: 'prev' | 'next') => void;
  closeErrorModal: () => void;
  clearRecentErrors: () => void;
  removeError: (id: string) => void;
}

export const useErrorStore = create<ErrorStoreState>((set, get) => ({
  selectedError: null,
  isModalOpen: false,
  activeTab: 'stack',
  recentErrors: [],
  errorQueue: [],
  currentQueueIndex: 0,

  captureError: (error: unknown, meta?: CaptureErrorMeta): CapturedError => {
    const captured = buildCapturedError(error, meta);
    set((state) => ({
      selectedError: captured,
      isModalOpen: true,
      activeTab: 'stack',
      recentErrors: [captured, ...state.recentErrors].slice(0, 50),
      errorQueue: [captured, ...state.errorQueue],
      currentQueueIndex: 0,
    }));
    return captured;
  },

  captureException: (error: Error | string, context?: ErrorContext): CapturedError => {
    const captured = buildCapturedError(error, undefined, context);
    set((state) => ({
      selectedError: captured,
      isModalOpen: true,
      activeTab: 'stack',
      recentErrors: [captured, ...state.recentErrors].slice(0, 50),
      errorQueue: [captured, ...state.errorQueue],
      currentQueueIndex: 0,
    }));
    return captured;
  },

  openErrorModal: (error: CapturedError, initialTab: ErrorModalTab = 'stack'): void => {
    set({
      selectedError: error,
      isModalOpen: true,
      activeTab: initialTab,
    });
  },

  setActiveTab: (tab: ErrorModalTab): void => {
    set({ activeTab: tab });
  },

  openErrorQueue: (errors: CapturedError[], startIndex = 0): void => {
    const validIndex = Math.max(0, Math.min(startIndex, errors.length - 1));
    set({
      errorQueue: errors,
      currentQueueIndex: validIndex,
      selectedError: errors[validIndex] || null,
      isModalOpen: errors.length > 0,
    });
  },

  navigateQueue: (direction: 'prev' | 'next'): void => {
    const { errorQueue, currentQueueIndex } = get();
    if (errorQueue.length <= 1) {
      return;
    }
    const delta = direction === 'next' ? 1 : -1;
    const nextIndex = (currentQueueIndex + delta + errorQueue.length) % errorQueue.length;
    set({
      currentQueueIndex: nextIndex,
      selectedError: errorQueue[nextIndex],
    });
  },

  closeErrorModal: (): void => {
    set({ isModalOpen: false });
  },

  clearRecentErrors: (): void => {
    set({
      recentErrors: [],
      errorQueue: [],
      selectedError: null,
      isModalOpen: false,
    });
  },

  removeError: (id: string): void => {
    const { recentErrors, errorQueue, selectedError, currentQueueIndex, isModalOpen } = get();
    const updatedRecent = recentErrors.filter((e) => e.id !== id);
    const updatedQueue = errorQueue.filter((e) => e.id !== id);

    let nextSelected: CapturedError | null = selectedError;
    let nextIndex = currentQueueIndex;
    let nextModalOpen = isModalOpen;

    const isSelectedTarget = Boolean(selectedError && selectedError.id === id);
    if (isSelectedTarget) {
      if (updatedQueue.length > 0) {
        nextIndex = Math.min(currentQueueIndex, updatedQueue.length - 1);
        nextSelected = updatedQueue[nextIndex];
      } else {
        nextSelected = null;
        nextIndex = 0;
        nextModalOpen = false;
      }
    } else if (nextSelected) {
      const idx = updatedQueue.findIndex((e) => e.id === nextSelected?.id);
      if (idx !== -1) {
        nextIndex = idx;
      } else {
        nextIndex = 0;
      }
    }

    set({
      recentErrors: updatedRecent,
      errorQueue: updatedQueue,
      selectedError: nextSelected,
      currentQueueIndex: nextIndex,
      isModalOpen: nextModalOpen,
    });
  },
}));
