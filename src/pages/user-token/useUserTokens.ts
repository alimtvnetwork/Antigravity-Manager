import { useState, useEffect } from 'react';
import type { Dispatch, SetStateAction } from 'react';
import { useTranslation } from 'react-i18next';
import { useErrorStore } from '../../stores/error-store';
import { request as invoke } from '../../utils/request';
import { showToast } from '../../components/common/ToastContainer';
import { copyToClipboard } from '../../utils/clipboard';
import { formatDateTime } from '../../utils/date';
import type { UserToken, UserTokenStats } from './types';
import {
    CMD_LIST_USER_TOKENS,
    CMD_GET_USER_TOKEN_SUMMARY,
    CMD_CREATE_USER_TOKEN,
    CMD_DELETE_USER_TOKEN,
    CMD_UPDATE_USER_TOKEN,
    CMD_RENEW_USER_TOKEN,
    DEFAULT_EXPIRES_TYPE,
    EXPIRES_TYPE_DAY,
    EXPIRES_TYPE_WEEK,
    EXPIRES_TYPE_MONTH,
    EXPIRES_TYPE_CUSTOM,
    EXPIRES_TYPE_NEVER,
    MS_PER_SECOND,
    SECONDS_PER_DAY,
    EXPIRY_WARNING_DAYS,
} from './types';

export interface UseUserTokensResult {
    tokens: UserToken[];
    stats: UserTokenStats | null;
    loading: boolean;
    showCreateModal: boolean;
    setShowCreateModal: Dispatch<SetStateAction<boolean>>;
    creating: boolean;
    showEditModal: boolean;
    setShowEditModal: Dispatch<SetStateAction<boolean>>;
    editingToken: UserToken | null;
    editUsername: string;
    setEditUsername: Dispatch<SetStateAction<string>>;
    editDesc: string;
    setEditDesc: Dispatch<SetStateAction<string>>;
    editMaxIps: number;
    setEditMaxIps: Dispatch<SetStateAction<number>>;
    editCurfewStart: string;
    setEditCurfewStart: Dispatch<SetStateAction<string>>;
    editCurfewEnd: string;
    setEditCurfewEnd: Dispatch<SetStateAction<string>>;
    newUsername: string;
    setNewUsername: Dispatch<SetStateAction<string>>;
    newDesc: string;
    setNewDesc: Dispatch<SetStateAction<string>>;
    newExpiresType: string;
    setNewExpiresType: Dispatch<SetStateAction<string>>;
    newMaxIps: number;
    setNewMaxIps: Dispatch<SetStateAction<number>>;
    newCurfewStart: string;
    setNewCurfewStart: Dispatch<SetStateAction<string>>;
    newCurfewEnd: string;
    setNewCurfewEnd: Dispatch<SetStateAction<string>>;
    newCustomExpires: string;
    setNewCustomExpires: Dispatch<SetStateAction<string>>;
    updating: boolean;
    loadData: () => Promise<void>;
    handleCreate: () => Promise<void>;
    handleDelete: (id: string) => Promise<void>;
    handleEdit: (token: UserToken) => void;
    handleUpdate: () => Promise<void>;
    handleRenew: (id: string, type: string) => Promise<void>;
    handleCopyToken: (text: string) => Promise<void>;
    formatTime: (ts?: number) => string;
    getExpiresLabel: (type: string) => string;
    getExpiresStatus: (expiresAt?: number) => string;
}

export function useUserTokens(): UseUserTokensResult {
    const { t } = useTranslation();
    const [tokens, setTokens] = useState<UserToken[]>([]);
    const [stats, setStats] = useState<UserTokenStats | null>(null);
    const [loading, setLoading] = useState(false);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [creating, setCreating] = useState(false);

    // Edit State
    const [showEditModal, setShowEditModal] = useState(false);
    const [editingToken, setEditingToken] = useState<UserToken | null>(null);
    const [editUsername, setEditUsername] = useState('');
    const [editDesc, setEditDesc] = useState('');
    const [editMaxIps, setEditMaxIps] = useState(0);
    const [editCurfewStart, setEditCurfewStart] = useState('');
    const [editCurfewEnd, setEditCurfewEnd] = useState('');
    const [updating, setUpdating] = useState(false);

    // Create Form State
    const [newUsername, setNewUsername] = useState('');
    const [newDesc, setNewDesc] = useState('');
    const [newExpiresType, setNewExpiresType] = useState<string>(DEFAULT_EXPIRES_TYPE); // day, week, month, never, custom
    const [newMaxIps, setNewMaxIps] = useState(0);
    const [newCurfewStart, setNewCurfewStart] = useState('');
    const [newCurfewEnd, setNewCurfewEnd] = useState('');
    const [newCustomExpires, setNewCustomExpires] = useState(''); // datetime-local value

    const loadData = async () => {
        setLoading(true);
        try {
            const [tokensData, statsData] = await Promise.all([
                invoke<UserToken[]>(CMD_LIST_USER_TOKENS),
                invoke<UserTokenStats>(CMD_GET_USER_TOKEN_SUMMARY)
            ]);
            setTokens(tokensData);
            setStats(statsData);
        } catch (e) {
            console.error('Failed to load user tokens', e);
            useErrorStore.getState().trackWarning(e, {
                source: 'UserToken.loadData',
                triggerAction: CMD_LIST_USER_TOKENS,
            });
            showToast(t('common.load_failed') || 'Failed to load data', 'error');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadData();
    }, []);

    const handleCreate = async () => {
        if (!newUsername) {
            showToast(t('user_token.username_required') || 'Username is required', 'error');
            return;
        }

        // 验证自定义时间
        if (newExpiresType === EXPIRES_TYPE_CUSTOM && !newCustomExpires) {
            showToast(t('user_token.custom_expires_required') || 'Please select a custom expiration time', 'error');
            return;
        }

        setCreating(true);
        try {
            // 计算自定义过期时间戳
            const customExpiresAt = newExpiresType === EXPIRES_TYPE_CUSTOM && newCustomExpires
                ? Math.floor(new Date(newCustomExpires).getTime() / MS_PER_SECOND)
                : undefined;

            await invoke(CMD_CREATE_USER_TOKEN, {
                request: {
                    username: newUsername,
                    expires_type: newExpiresType,
                    description: newDesc || null,
                    max_ips: newMaxIps,
                    curfew_start: newCurfewStart || null,
                    curfew_end: newCurfewEnd || null,
                    custom_expires_at: customExpiresAt || null
                }
            });
            showToast(t('common.create_success') || 'Created successfully', 'success');
            setShowCreateModal(false);
            setNewUsername('');
            setNewDesc('');
            setNewExpiresType(DEFAULT_EXPIRES_TYPE);
            setNewMaxIps(0);
            setNewCurfewStart('');
            setNewCurfewEnd('');
            setNewCustomExpires('');
            loadData();
        } catch (e) {
            console.error('Failed to create token', e);
            useErrorStore.getState().trackWarning(e, {
                source: 'UserToken.handleCreate',
                triggerAction: CMD_CREATE_USER_TOKEN,
            });
            showToast(String(e), 'error');
        } finally {
            setCreating(false);
        }
    };

    const handleDelete = async (id: string) => {
        try {
            await invoke(CMD_DELETE_USER_TOKEN, { id });
            showToast(t('common.delete_success') || 'Deleted successfully', 'success');
            loadData();
        } catch (e) {
            showToast(String(e), 'error');
        }
    };

    const handleEdit = (token: UserToken) => {
        setEditingToken(token);
        setEditUsername(token.username);
        setEditDesc(token.description || '');
        setEditMaxIps(token.max_ips ?? 0);  // 使用 ?? 确保 null/undefined 变为 0
        setEditCurfewStart(token.curfew_start ?? '');
        setEditCurfewEnd(token.curfew_end ?? '');
        setShowEditModal(true);
    };

    const handleUpdate = async () => {
        if (!editingToken) return;
        if (!editUsername) {
            showToast(t('user_token.username_required') || 'Username is required', 'error');
            return;
        }

        setUpdating(true);
        try {
            await invoke(CMD_UPDATE_USER_TOKEN, {
                id: editingToken.id,
                request: {
                    username: editUsername,
                    description: editDesc || undefined,
                    max_ips: editMaxIps,
                    // 使用双层包装: undefined = 不更新, null = 清空, string = 设置值
                    curfew_start: editCurfewStart === '' ? null : editCurfewStart,
                    curfew_end: editCurfewEnd === '' ? null : editCurfewEnd
                }
            });
            showToast(t('common.update_success') || 'Updated successfully', 'success');
            setShowEditModal(false);
            setEditingToken(null);
            loadData();
        } catch (e) {
            console.error('Failed to update token', e);
            useErrorStore.getState().trackWarning(e, {
                source: 'UserToken.handleUpdate',
                triggerAction: CMD_UPDATE_USER_TOKEN,
            });
            showToast(String(e), 'error');
        } finally {
            setUpdating(false);
        }
    };

    const handleRenew = async (id: string, type: string) => {
        try {
            await invoke(CMD_RENEW_USER_TOKEN, { id, expiresType: type });
            showToast(t('user_token.renew_success') || 'Renewed successfully', 'success');
            loadData();
        } catch (e) {
            showToast(String(e), 'error');
        }
    };

    const handleCopyToken = async (text: string) => {
        const success = await copyToClipboard(text);
        if (success) {
            showToast(t('common.copied') || 'Copied to clipboard', 'success');
        } else {
            showToast(t('common.copy_failed') || 'Failed to copy to clipboard', 'error');
        }
    };

    const formatTime = (ts?: number) => {
        return formatDateTime(ts);
    };

    const getExpiresLabel = (type: string) => {
        switch (type) {
            case EXPIRES_TYPE_DAY: return t('user_token.expires_day', { defaultValue: '1 Day' });
            case EXPIRES_TYPE_WEEK: return t('user_token.expires_week', { defaultValue: '1 Week' });
            case EXPIRES_TYPE_MONTH: return t('user_token.expires_month', { defaultValue: '1 Month' });
            case EXPIRES_TYPE_NEVER: return t('user_token.expires_never', { defaultValue: 'Never' });
            case EXPIRES_TYPE_CUSTOM: return t('user_token.expires_custom', { defaultValue: 'Custom' });
            default: return type;
        }
    };

    // Calculate expiration status style
    const getExpiresStatus = (expiresAt?: number) => {
        if (!expiresAt) return 'text-green-500';
        const now = Date.now() / MS_PER_SECOND;
        if (expiresAt < now) return 'text-red-500 font-bold';
        if (expiresAt - now < SECONDS_PER_DAY * EXPIRY_WARNING_DAYS) return 'text-orange-500'; // Less than 3 days
        return 'text-green-500';
    };

    return {
        tokens,
        stats,
        loading,
        showCreateModal,
        setShowCreateModal,
        creating,
        showEditModal,
        setShowEditModal,
        editingToken,
        editUsername,
        setEditUsername,
        editDesc,
        setEditDesc,
        editMaxIps,
        setEditMaxIps,
        editCurfewStart,
        setEditCurfewStart,
        editCurfewEnd,
        setEditCurfewEnd,
        newUsername,
        setNewUsername,
        newDesc,
        setNewDesc,
        newExpiresType,
        setNewExpiresType,
        newMaxIps,
        setNewMaxIps,
        newCurfewStart,
        setNewCurfewStart,
        newCurfewEnd,
        setNewCurfewEnd,
        newCustomExpires,
        setNewCustomExpires,
        updating,
        loadData,
        handleCreate,
        handleDelete,
        handleEdit,
        handleUpdate,
        handleRenew,
        handleCopyToken,
        formatTime,
        getExpiresLabel,
        getExpiresStatus,
    };
}
