export interface UserToken {
    id: string;
    token: string;
    username: string;
    description?: string;
    enabled: boolean;
    expires_type: string;
    expires_at?: number;
    max_ips: number;
    curfew_start?: string;
    curfew_end?: string;
    created_at: number;
    updated_at: number;
    last_used_at?: number;
    total_requests: number;
    total_tokens_used: number;
}

export interface UserTokenStats {
    total_tokens: number;
    active_tokens: number;
    total_users: number;
    today_requests: number;
}

export const CMD_LIST_USER_TOKENS = 'list_user_tokens';
export const CMD_GET_USER_TOKEN_SUMMARY = 'get_user_token_summary';
export const CMD_CREATE_USER_TOKEN = 'create_user_token';
export const CMD_DELETE_USER_TOKEN = 'delete_user_token';
export const CMD_UPDATE_USER_TOKEN = 'update_user_token';
export const CMD_RENEW_USER_TOKEN = 'renew_user_token';

export const EXPIRES_TYPE_DAY = 'day';
export const EXPIRES_TYPE_WEEK = 'week';
export const EXPIRES_TYPE_MONTH = 'month';
export const EXPIRES_TYPE_CUSTOM = 'custom';
export const EXPIRES_TYPE_NEVER = 'never';
export const DEFAULT_EXPIRES_TYPE = EXPIRES_TYPE_MONTH;

export const MS_PER_SECOND = 1000;
export const SECONDS_PER_DAY = 86400;
export const EXPIRY_WARNING_DAYS = 3;

export const TOKEN_PREVIEW_LENGTH = 8;
export const TOKEN_PREVIEW_MASK = '••••••••';
export const USERNAME_AVATAR_LENGTH = 2;

export const TOKENS_PER_THOUSAND = 1000;
export const TOKEN_KILO_DECIMALS = 1;

export const DATETIME_LOCAL_TRUNCATE_LENGTH = 16;
