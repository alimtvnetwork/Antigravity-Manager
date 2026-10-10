import { useState, useEffect, useCallback, useRef } from 'react';
import type { MouseEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { useErrorStore } from '../../stores/error-store';
import { showToast } from '../../components/common/ToastContainer';
import { copyToClipboard } from '../../utils/clipboard';
import { request } from '../../utils/request';
import { generateUUID } from '../../utils/uuid';
import { getProfileInfo, type OpencodeProviderSummary } from '../../utils/opencodeProfiles';
import type { ManagedApiKey, UsageSummary, CliSyncApp, UseApiKeyFunResult } from './types';
import {
    STORAGE_KEY,
    DEFAULT_ENDPOINT,
    CMD_GET_OPENCODE_PROVIDERS,
    CMD_QUERY_TRANSIT_INFO,
    CMD_EXECUTE_CLI_SYNC,
    CMD_OPENCODE_REMOVE_PROVIDER,
    CMD_OPENCODE_OPENAI_SYNC,
    CLI_APP_CODEX,
    MS_PER_DAY,
    BILLING_LOOKBACK_DAYS,
    FIAT_DECIMALS,
    BILLING_USD_DECIMALS,
    CENTS_PER_DOLLAR,
    maskKey,
    trimTrailingSlashes,
    stripV1Suffix,
    asString,
    asNumber,
    normalizeModelIds,
    type ModelsListResponse,
    type TransitUsagePayload,
    type BillingSubscriptionPayload,
    type BillingUsagePayload,
} from './utils';

const FALLBACK_QUERY_ERROR = 'Query failed. Please verify network or key validity.';

export function useApiKeyFun(): UseApiKeyFunResult {
    const { t } = useTranslation();

    const [apiKey, setApiKey] = useState('');
    const [baseUrl, setBaseUrl] = useState(DEFAULT_ENDPOINT);
    const [showApiKey, setShowApiKey] = useState(false);

    // Querying states
    const [querying, setQuerying] = useState(false);
    const [usage, setUsage] = useState<UsageSummary | null>(null);
    const [models, setModels] = useState<string[]>([]);
    const [modelsSource, setModelsSource] = useState<{ key: string; endpoint: string } | null>(null);
    const [queryError, setQueryError] = useState<string | null>(null);
    const [modelsError, setModelsError] = useState<string | null>(null);
    const [opencodeProviders, setOpencodeProviders] = useState<OpencodeProviderSummary[]>([]);
    const [syncingKey, setSyncingKey] = useState<string | null>(null);
    const isTogglingRef = useRef(false);
    const querySeqRef = useRef(0);

    const providersSeqRef = useRef(0);
    const fetchOpencodeProviders = useCallback(async () => {
        const seq = ++providersSeqRef.current;
        const providers = await request<OpencodeProviderSummary[]>(CMD_GET_OPENCODE_PROVIDERS);
        if (!Array.isArray(providers)) throw new Error('Invalid OpenCode providers response');
        if (seq === providersSeqRef.current) setOpencodeProviders(providers);
        return providers;
    }, []);

    useEffect(() => {
        fetchOpencodeProviders().catch(err => useErrorStore.getState().trackWarning(err, {
            source: 'ApiKeyFun.providers',
            triggerAction: 'fetchOpencodeProviders',
        }));
    }, [fetchOpencodeProviders]);

    // Key Management
    const [managedKeys, setManagedKeys] = useState<ManagedApiKey[]>(() => {
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            return raw ? JSON.parse(raw) : [];
        } catch (e) {
            useErrorStore.getState().trackWarning(e, {
                source: 'ApiKeyFun.loadManagedKeys',
                triggerAction: 'localStorage.getItem',
            });
            return [];
        }
    });

    // Inline Rename state
    const [editingId, setEditingId] = useState<string | null>(null);
    const [editNameValue, setEditNameValue] = useState('');
    const initialKeyLoaded = useRef(false);

    // Save keys to localStorage
    useEffect(() => {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(managedKeys));
        } catch (e) {
            useErrorStore.getState().trackWarning(e, {
                source: 'ApiKeyFun.saveManagedKeys',
                triggerAction: 'localStorage.setItem',
            });
        }
    }, [managedKeys]);

    const handleCopy = async (text: string) => {
        const success = await copyToClipboard(text);
        if (success) {
            showToast(t('common.copied') || 'Copied to clipboard', 'success');
        }
    };

    // Auto balance & models query
    const runQuery = useCallback(async (keyToQuery: string, urlToQuery: string) => {
        const key = keyToQuery.trim();
        const endpoint = trimTrailingSlashes(urlToQuery);
        const seq = ++querySeqRef.current;
        if (!key) {
            setQuerying(false);
            return;
        }

        setQuerying(true);
        setQueryError(null);
        setUsage(null);
        setModels([]);
        setModelsSource(null);

        let fetchedModels: string[] = [];
        try {
            // 1. Fetch available models
            let rawText = '';
            setModelsError(null);
            try {
                rawText = await request<string>(CMD_QUERY_TRANSIT_INFO, {
                    url: `${endpoint}/models`,
                    key
                });
                if (seq !== querySeqRef.current) return;
                const modelsData: unknown = JSON.parse(rawText);
                if (
                    modelsData !== null &&
                    typeof modelsData === 'object' &&
                    !Array.isArray(modelsData) &&
                    Array.isArray((modelsData as ModelsListResponse).data)
                ) {
                    fetchedModels = normalizeModelIds((modelsData as ModelsListResponse).data as unknown[]);
                } else if (Array.isArray(modelsData)) {
                    fetchedModels = normalizeModelIds(modelsData);
                } else {
                    setModelsError(t('apiKeyFun.errors.parseFormat', { defaultValue: '解析格式异常: {{err}}', err: Object.keys(modelsData ?? {}).join(',') }));
                }
            } catch (err) {
                if (seq !== querySeqRef.current) return;
                useErrorStore.getState().trackWarning(err, {
                    source: 'ApiKeyFun.models',
                    triggerAction: CMD_QUERY_TRANSIT_INFO,
                });
                setModelsError(t('apiKeyFun.errors.fetchFailed', { defaultValue: '获取失败: {{err}}', err: err instanceof Error ? err.message : String(err) }));
            }
            if (seq !== querySeqRef.current) return;
            setModels(fetchedModels);
            setModelsSource({ key, endpoint });

            // 2. Fetch balance (Try sub2api /usage first, then New API billing)
            let usageSummary: UsageSummary | null = null;

            try {
                const usageText = await request<string>(CMD_QUERY_TRANSIT_INFO, {
                    url: `${endpoint}/usage`,
                    key
                });
                const data = JSON.parse(usageText) as TransitUsagePayload;

                const unit = asString(data.unit) || asString(data.quota?.unit) || 'USD';
                const remainingRaw = asNumber(data.remaining) ?? asNumber(data.balance);
                const remaining = remainingRaw !== undefined ? remainingRaw.toFixed(FIAT_DECIMALS) : '--';
                const usedRaw: unknown = data.quota?.used ?? data.usage?.total?.actual_cost ?? data.usage?.total?.cost;
                const used = typeof usedRaw === 'number' ? usedRaw.toFixed(FIAT_DECIMALS) : '--';
                const activeFlag: unknown = data.is_active ?? data.isValid ?? true;

                usageSummary = {
                    remaining: remaining !== '--' ? (unit === 'USD' ? `$${remaining}` : `${remaining} ${unit}`) : '--',
                    used: used !== '--' ? (unit === 'USD' ? `$${used}` : `${used} ${unit}`) : '--',
                    todayRequests: String(data.usage?.today?.requests ?? '--'),
                    todayTokens: String(data.usage?.today?.total_tokens ?? '--'),
                    totalRequests: String(data.usage?.total?.requests ?? '--'),
                    totalTokens: String(data.usage?.total?.total_tokens ?? '--'),
                    unit,
                    isValid: typeof activeFlag === 'boolean' ? activeFlag : true
                };
            } catch (e) {
                console.log('Skipping sub2api endpoint, trying standard billing endpoints...', e);
            }

            // Fallback to standard One-API / New-API dashboard billing
            if (!usageSummary) {
                try {
                    const subText = await request<string>(CMD_QUERY_TRANSIT_INFO, {
                        url: `${endpoint}/dashboard/billing/subscription`,
                        key
                    });
                    const subData = JSON.parse(subText) as BillingSubscriptionPayload;

                    const start = new Date(Date.now() - BILLING_LOOKBACK_DAYS * MS_PER_DAY).toISOString().split('T')[0];
                    const end = new Date(Date.now() + MS_PER_DAY).toISOString().split('T')[0];
                    const usageText = await request<string>(CMD_QUERY_TRANSIT_INFO, {
                        url: `${endpoint}/dashboard/billing/usage?start_date=${start}&end_date=${end}`,
                        key
                    });
                    const usageData = JSON.parse(usageText) as BillingUsagePayload;

                    const totalUsageUSD = (asNumber(usageData.total_usage) ?? 0) / CENTS_PER_DOLLAR;
                    const limitUSD = asNumber(subData.hard_limit_usd) ?? 0;
                    const remainingUSD = (limitUSD - totalUsageUSD).toFixed(BILLING_USD_DECIMALS);

                    usageSummary = {
                        remaining: `$${remainingUSD}`,
                        used: `$${totalUsageUSD.toFixed(BILLING_USD_DECIMALS)}`,
                        todayRequests: '--',
                        todayTokens: '--',
                        totalRequests: '--',
                        totalTokens: '--',
                        unit: 'USD',
                        isValid: true
                    };
                } catch (billingErr) {
                    console.log('Billing query failed', billingErr);
                }
            }

            if (seq !== querySeqRef.current) return;

            if (usageSummary) {
                setUsage(usageSummary);
                // Update or add to managed keys automatically
                setManagedKeys(prev => {
                    const existingIndex = prev.findIndex(item => item.key === key);
                    const now = Date.now();
                    if (existingIndex >= 0) {
                        const updated = [...prev];
                        updated[existingIndex] = {
                            ...updated[existingIndex],
                            lastRemaining: usageSummary?.remaining,
                            lastStatus: 'ok',
                            lastUsedAt: now,
                            baseUrl: endpoint, // optionally update baseUrl
                            models: fetchedModels.length > 0 ? fetchedModels
                                : updated[existingIndex].baseUrl === endpoint ? updated[existingIndex].models : undefined
                        };
                        return updated;
                    } else {
                        // Automatically save new key
                        return [{
                            id: generateUUID(),
                            key,
                            name: maskKey(key),
                            baseUrl: endpoint,
                            createdAt: now,
                            lastUsedAt: now,
                            lastStatus: 'ok',
                            lastRemaining: usageSummary?.remaining,
                            models: fetchedModels
                        }, ...prev];
                    }
                });
            } else {
                throw new Error(t('apiKeyFun.errors.queryFailed', { defaultValue: '无法获取有效的额度数据或模型列表，请确认 API Key 是否有效，以及接口地址是否正确。' }));
            }

        } catch (error) {
            if (seq !== querySeqRef.current) return;
            console.error('Balance query failed', error);
            const message: unknown = error instanceof Error ? error.message : (error as { message?: unknown })?.message;
            setQueryError(typeof message === 'string' && message ? message : FALLBACK_QUERY_ERROR);
            setManagedKeys(prev => {
                const existingIndex = prev.findIndex(item => item.key === key);
                const now = Date.now();
                if (existingIndex >= 0) {
                    const updated = [...prev];
                    updated[existingIndex] = {
                        ...updated[existingIndex],
                        lastStatus: 'bad',
                        lastUsedAt: now,
                        baseUrl: endpoint,
                        models: fetchedModels.length > 0 ? fetchedModels
                            : updated[existingIndex].baseUrl === endpoint ? updated[existingIndex].models : undefined
                    };
                    return updated;
                } else {
                    return [{
                        id: generateUUID(),
                        key,
                        name: maskKey(key),
                        baseUrl: endpoint,
                        createdAt: now,
                        lastUsedAt: now,
                        lastStatus: 'bad',
                        models: fetchedModels
                    }, ...prev];
                }
            });
        } finally {
            if (seq === querySeqRef.current) {
                setQuerying(false);
            }
        }
    }, [t]);

    // Load first key on mount and automatically run query
    useEffect(() => {
        if (!initialKeyLoaded.current) {
            initialKeyLoaded.current = true;
            if (managedKeys.length > 0) {
                const initialKey = managedKeys[0].key;
                const initialUrl = managedKeys[0].baseUrl || DEFAULT_ENDPOINT;
                setApiKey(initialKey);
                setBaseUrl(initialUrl);
                if (managedKeys[0].models && managedKeys[0].models.length > 0) {
                    setModels(managedKeys[0].models);
                    setModelsSource({ key: initialKey.trim(), endpoint: trimTrailingSlashes(initialUrl) });
                }
                runQuery(initialKey, initialUrl);
            }
        }
    }, [managedKeys, runQuery]);

    const handleSyncCli = async (app: CliSyncApp) => {
        if (!apiKey.trim()) return;
        const rawKey = apiKey.trim();
        const cleanUrl = trimTrailingSlashes(baseUrl);
        const baseWithoutV1 = stripV1Suffix(cleanUrl);
        const proxyUrl = app === CLI_APP_CODEX ? `${baseWithoutV1}/v1` : baseWithoutV1;
        const syncKey = rawKey;

        try {
            await request(CMD_EXECUTE_CLI_SYNC, {
                appType: app,
                proxyUrl: proxyUrl,
                apiKey: syncKey
            });
            showToast(t('apiKeyFun.syncSuccess', { defaultValue: 'Successfully synced to {{app}}', app }), 'success');
        } catch (error) {
            showToast(t('apiKeyFun.syncError', { defaultValue: 'Failed to sync: {{error}}', error: String(error) }), 'error');
        }
    };

    const getModelsForKey = useCallback((targetKey: string, targetUrl?: string): string[] | undefined => {
        const trimmed = targetKey.trim();
        if (!trimmed) return undefined;
        const normTarget = targetUrl ? trimTrailingSlashes(targetUrl) : null;
        if (modelsSource?.key === trimmed && (!normTarget || modelsSource.endpoint === normTarget) && models.length > 0) {
            return models.map(m => m.trim()).filter(Boolean);
        }
        const found = managedKeys.find(m => m.key.trim() === trimmed);
        if (found?.models && found.models.length > 0) {
            if (!normTarget || !found.baseUrl || trimTrailingSlashes(found.baseUrl) === normTarget) {
                return found.models.map(m => m.trim()).filter(Boolean);
            }
        }
        return undefined;
    }, [modelsSource, models, managedKeys]);

    const profileInfo = useCallback((key: string, url: string, modelIds?: string[]) =>
        getProfileInfo(opencodeProviders, key, url, modelIds), [opencodeProviders]);

    const handleToggleOpenCodeProfile = async (targetKey: string, targetUrl: string, explicitModels?: string[]) => {
        const trimmedKey = targetKey.trim();
        if (!trimmedKey || !targetUrl.trim() || isTogglingRef.current || (querying && trimmedKey === apiKey.trim())) return;
        isTogglingRef.current = true;
        setSyncingKey(trimmedKey);

        try {
            const keyModels = explicitModels ?? getModelsForKey(trimmedKey, targetUrl);
            const providers = await fetchOpencodeProviders();
            const info = getProfileInfo(providers, trimmedKey, targetUrl, keyModels);
            if (info.status === 'synced') {
                // Deactivate / remove profile
                await request(CMD_OPENCODE_REMOVE_PROVIDER, {
                    providerId: info.providerId
                });
                showToast(t('apiKeyFun.opencode.removedToast', { defaultValue: 'Removed from OpenCode: {{name}}', name: info.providerName }), 'success');
            } else {
                // 'not_present' or 'partial' -> Sync / update profile
                const proxyUrl = trimTrailingSlashes(targetUrl);
                const cleaned = (keyModels ?? []).map(id => id.trim()).filter(Boolean);
                if (cleaned.length === 0 && !info.existing?.models.length) {
                    throw new Error(t('apiKeyFun.opencode.modelsRequired'));
                }
                const modelInputs = cleaned.length > 0
                    ? cleaned.map(id => ({ id }))
                    : undefined;

                await request(CMD_OPENCODE_OPENAI_SYNC, {
                    proxyUrl,
                    apiKey: trimmedKey,
                    providerId: info.providerId,
                    providerName: info.providerName,
                    models: modelInputs
                });
                showToast(t('apiKeyFun.opencode.syncedToast', { defaultValue: 'Synced to OpenCode: {{name}}', name: info.providerName }), 'success');
            }
            await fetchOpencodeProviders();
        } catch (error) {
            showToast(t('apiKeyFun.syncError', { defaultValue: 'Failed to sync: {{error}}', error: String(error) }), 'error');
        } finally {
            isTogglingRef.current = false;
            setSyncingKey(null);
        }
    };

    const handleSyncOpenCode = async () => {
        if (!apiKey.trim() || !baseUrl.trim() || querying || Boolean(syncingKey)) return;
        await handleToggleOpenCodeProfile(apiKey, baseUrl);
    };

    const handleDeleteKey = (id: string, e: MouseEvent) => {
        e.stopPropagation();
        setManagedKeys(prev => prev.filter(item => item.id !== id));
        showToast(t('common.delete_success') || 'Deleted', 'success');
    };

    const startRename = (item: ManagedApiKey, e: MouseEvent) => {
        e.stopPropagation();
        setEditingId(item.id);
        setEditNameValue(item.name);
    };

    const saveRename = (id: string) => {
        const trimmed = editNameValue.trim();
        if (trimmed) {
            setManagedKeys(prev => prev.map(item => item.id === id ? { ...item, name: trimmed } : item));
        }
        setEditingId(null);
    };

    const handleApiKeyChange = (value: string) => {
        ++querySeqRef.current;
        setQuerying(false);
        setUsage(null);
        setModels([]);
        setModelsSource(null);
        setQueryError(null);
        setModelsError(null);
        setApiKey(value);
    };

    const handleSelectKey = (item: ManagedApiKey) => {
        setApiKey(item.key);
        setBaseUrl(item.baseUrl || DEFAULT_ENDPOINT);
        if (item.models && item.models.length > 0) {
            setModels(item.models);
            setModelsSource({ key: item.key.trim(), endpoint: trimTrailingSlashes(item.baseUrl || DEFAULT_ENDPOINT) });
        }
        runQuery(item.key, item.baseUrl || DEFAULT_ENDPOINT);
    };

    return {
        apiKey,
        baseUrl,
        showApiKey,
        setShowApiKey,
        querying,
        usage,
        models,
        queryError,
        modelsError,
        syncingKey,
        managedKeys,
        editingId,
        setEditingId,
        editNameValue,
        setEditNameValue,
        runQuery,
        fetchOpencodeProviders,
        getModelsForKey,
        profileInfo,
        handleCopy,
        handleSyncCli,
        handleToggleOpenCodeProfile,
        handleSyncOpenCode,
        handleDeleteKey,
        startRename,
        saveRename,
        handleApiKeyChange,
        handleSelectKey,
    };
}
