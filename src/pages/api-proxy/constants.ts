import type { StickySessionConfig } from '../../types/config';

/** How long the "copied" check icon stays visible (ms). */
export const COPY_FEEDBACK_TIMEOUT_MS = 2000;

/** Polling intervals for background status refreshes (ms). */
export const STATUS_POLL_INTERVAL_MS = 3000;
export const CF_STATUS_POLL_INTERVAL_MS = 5000;

/** localStorage key for user-defined model-mapping presets. */
export const CUSTOM_PRESETS_STORAGE_KEY = 'antigravity_custom_presets';

/** IDs of the built-in presets (cannot be deleted). */
export const BUILT_IN_PRESET_IDS = ['default', 'performance', 'cost-effective', 'balanced'];
export const DEFAULT_PRESET_ID = 'default';

/** Proxy server defaults. */
export const DEFAULT_PROXY_PORT = 8045;
export const DEFAULT_REQUEST_TIMEOUT_SECONDS = 120;
export const MIN_REQUEST_TIMEOUT_SECONDS = 30;
export const MAX_REQUEST_TIMEOUT_SECONDS = 7200;

/** Account scheduling defaults. */
export const DEFAULT_SCHEDULING: StickySessionConfig = {
    mode: 'Balance',
    max_wait_seconds: 60,
};
export const DEFAULT_MAX_WAIT_SECONDS = 60;
export const MAX_WAIT_SLIDER_MAX_SECONDS = 300;
export const MAX_WAIT_SLIDER_STEP_SECONDS = 10;

/** Default log-retention settings when none are stored. */
export const DEFAULT_LOG_RETENTION = {
    max_body_age_hours: 24,
    max_age_days: 30,
    max_rows: 100000,
};

/** Default experimental thresholds used when the block is absent. */
export const DEFAULT_EXPERIMENTAL_THRESHOLDS = {
    l1: 0.4,
    l2: 0.55,
    l3: 0.7,
};

/** Built-in wildcard rule: only this key may use the wildcard pattern. */
export const BUILT_IN_WILDCARD_KEY = 'gemini-3.x-flash';
export const BUILT_IN_WILDCARD_TARGET = '3.x-flash-tiered';

/** Model-mapping key for the background-task target model. */
export const BACKGROUND_TASK_MAPPING_KEY = 'internal-background-task';

/** Default model-mapping table used by "reset mapping". */
export const DEFAULT_MODEL_MAPPINGS: Record<string, string> = {
    'gemini-3.6-flash': 'gemini-3.6-flash-tiered',
    'gemini-3.7-flash': 'gemini-3.7-flash-tiered',
    'gemini-3.8-flash': 'gemini-3.8-flash-tiered',
    'gemini-3.x-flash': '3.x-flash-tiered',
};

/** Credential validation rules. */
export const API_KEY_MIN_LENGTH = 10;
export const ADMIN_PASSWORD_MIN_LENGTH = 4;

/** Default model pre-selected in the quick-integration preview. */
export const DEFAULT_SELECTED_MODEL_ID = 'gemini-3-flash';

/** User-Agent override placeholder when the user enables the toggle. */
export const DEFAULT_USER_AGENT_OVERRIDE = 'antigravity/1.15.8 darwin/arm64';

/** Default z.ai base URL when none is configured. */
export const DEFAULT_ZAI_BASE_URL = 'https://api.z.ai/api/anthropic';

/** z.ai dispatcher default model families. */
export const ZAI_MODEL_FAMILIES = ['opus', 'sonnet', 'haiku'] as const;

/** Default thinking-store retention values. */
export const DEFAULT_THINKING_RETENTION_DAYS = 15;
export const DEFAULT_THINKING_MAX_MEMORY_TURNS = 600;
