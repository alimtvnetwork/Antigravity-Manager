export const STORAGE_KEY = 'apikey_fun_managed_keys_local';
export const DEFAULT_ENDPOINT = 'https://api.apikey.fan/v1';

export const TRAILING_SLASH_PATTERN = /\/+$/;
export const V1_SUFFIX_PATTERN = /\/v1$/i;

export const CMD_GET_OPENCODE_PROVIDERS = 'get_opencode_providers';
export const CMD_QUERY_TRANSIT_INFO = 'query_transit_info';
export const CMD_EXECUTE_CLI_SYNC = 'execute_cli_sync';
export const CMD_OPENCODE_REMOVE_PROVIDER = 'execute_opencode_remove_provider';
export const CMD_OPENCODE_OPENAI_SYNC = 'execute_opencode_openai_sync';

export const CLI_APP_CODEX = 'Codex';
export const CLI_APP_CLAUDE = 'Claude';

export const MS_PER_DAY = 24 * 60 * 60 * 1000;
export const BILLING_LOOKBACK_DAYS = 100;
export const FIAT_DECIMALS = 2;
export const BILLING_USD_DECIMALS = 4;
export const CENTS_PER_DOLLAR = 100;

const SHORT_KEY_MAX_LENGTH = 10;
const SHORT_KEY_VISIBLE_PREFIX = 3;
const SHORT_KEY_VISIBLE_SUFFIX = 3;
const LONG_KEY_VISIBLE_PREFIX = 6;
const LONG_KEY_VISIBLE_SUFFIX = 4;
const MASK_FILLER = '••••';

const DATE_PART_WIDTH = 2;
const DATE_PART_PAD = '0';

export function maskKey(value: string): string {
    const trimmed = value.trim();
    if (!trimmed) return '';
    if (trimmed.length <= SHORT_KEY_MAX_LENGTH) {
        return `${trimmed.slice(0, SHORT_KEY_VISIBLE_PREFIX)}${MASK_FILLER}${trimmed.slice(-SHORT_KEY_VISIBLE_SUFFIX)}`;
    }
    return `${trimmed.slice(0, LONG_KEY_VISIBLE_PREFIX)}${MASK_FILLER}${trimmed.slice(-LONG_KEY_VISIBLE_SUFFIX)}`;
}

export function formatDate(ts: number | undefined): string {
    if (!ts) return '--';
    const d = new Date(ts);
    const pad = (n: number) => String(n).padStart(DATE_PART_WIDTH, DATE_PART_PAD);
    return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function trimTrailingSlashes(value: string): string {
    return value.trim().replace(TRAILING_SLASH_PATTERN, '');
}

export function stripV1Suffix(url: string): string {
    return url.replace(V1_SUFFIX_PATTERN, '');
}

/** Loose shapes of the transit-station JSON responses (narrowed at each read site). */
export interface ModelsListResponse {
    data?: unknown;
}

export interface TransitUsagePayload {
    unit?: unknown;
    remaining?: unknown;
    balance?: unknown;
    quota?: { unit?: unknown; used?: unknown };
    usage?: {
        today?: { requests?: unknown; total_tokens?: unknown };
        total?: { actual_cost?: unknown; cost?: unknown; requests?: unknown; total_tokens?: unknown };
    };
    is_active?: unknown;
    isValid?: unknown;
}

export interface BillingSubscriptionPayload {
    hard_limit_usd?: unknown;
}

export interface BillingUsagePayload {
    total_usage?: unknown;
}

export const asString = (value: unknown): string | undefined =>
    typeof value === 'string' ? value : undefined;

export const asNumber = (value: unknown): number | undefined =>
    typeof value === 'number' ? value : undefined;

export const isNonEmptyString = (value: unknown): value is string =>
    typeof value === 'string' && value.trim().length > 0;

const extractModelId = (entry: unknown): string | undefined => {
    if (typeof entry === 'string') return entry;
    if (entry !== null && typeof entry === 'object' && 'id' in entry) {
        const id: unknown = (entry as { id?: unknown }).id;
        return typeof id === 'string' ? id : undefined;
    }
    return undefined;
};

export const normalizeModelIds = (entries: unknown[]): string[] =>
    entries.map(extractModelId).filter(isNonEmptyString).map(id => id.trim());
