import type { TFunction } from 'i18next';

export interface ProxyRequestLog {
    id: string;
    timestamp: number;
    method: string;
    url: string;
    status: number;
    duration: number;
    model?: string;
    mapped_model?: string;
    error?: string;
    request_body?: string;
    upstream_request_body?: string;
    response_body?: string;
    request_headers?: string;
    upstream_request_headers?: string;
    response_headers?: string;
    input_tokens?: number;
    output_tokens?: number;
    cached_tokens?: number;
    account_email?: string;
    protocol?: string;  // "openai" | "anthropic" | "gemini"
}

export interface ProxyStats {
    total_requests: number;
    success_count: number;
    error_count: number;
}

export interface ProxyMonitorProps {
    className?: string;
}

// Log Table Component
export interface LogTableProps {
    logs: ProxyRequestLog[];
    loading: boolean;
    onLogClick: (log: ProxyRequestLog) => void;
    t: TFunction;
}

export interface ColumnWidths {
    status: number;
    method: number;
    model: number;
    protocol: number;
    account: number;
    path: number;
    usage: number;
    duration: number;
    time: number;
}

export const DEFAULT_COL_WIDTHS: ColumnWidths = {
    status: 65,
    method: 65,
    model: 240,
    protocol: 80,
    account: 150,
    path: 180,
    usage: 125,
    duration: 85,
    time: 85,
};

export type PayloadViewMode = 'concise' | 'full';

// ---------------------------------------------------------------------------
// Shared constants (previously magic strings / numbers in ProxyMonitor.tsx)
// ---------------------------------------------------------------------------

/** localStorage key for persisted log-table column widths. */
export const PROXY_LOG_COL_WIDTHS_STORAGE_KEY = 'proxy_log_col_widths';

/** Minimum resizable column width in pixels. */
export const MIN_COL_WIDTH_PX = 45;

/** Sentinel filter value meaning "only error requests". */
export const ERROR_FILTER_SENTINEL = '__ERROR__';

/** Timeout for each backend invoke inside loadData. */
export const LOAD_DATA_TIMEOUT_MS = 10000;

/** Debounce for batching live proxy://request events into one state update. */
export const LIVE_EVENT_DEBOUNCE_MS = 500;

/** Poll interval for refreshing logs when running in web (non-Tauri) mode. */
export const WEB_POLL_INTERVAL_MS = 10000;

/** Max live logs kept in memory before the backend becomes authoritative. */
export const LIVE_LOG_BUFFER_MAX = 100;

/** How long the "copied" feedback state stays visible on copy buttons. */
export const COPY_FEEDBACK_TIMEOUT_MS = 2000;

/** How long the "saved" feedback state stays visible on the settings save button. */
export const CONFIG_SAVE_SUCCESS_TIMEOUT_MS = 2000;

/** How long the "cache cleared" feedback stays visible in the settings panel. */
export const CACHE_CLEAR_SUCCESS_TIMEOUT_MS = 2500;
