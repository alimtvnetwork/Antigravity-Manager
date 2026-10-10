import React, { useEffect, useState, useRef, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { request as invoke } from '../../../utils/request';
import { useAccountStore } from '../../../stores/useAccountStore';
import { isTauri } from '../../../utils/env';
import { useErrorStore } from '../../../stores/error-store';
import type { AppConfig } from '../../../types/config';
import {
    ERROR_FILTER_SENTINEL,
    LIVE_EVENT_DEBOUNCE_MS,
    LIVE_LOG_BUFFER_MAX,
    LOAD_DATA_TIMEOUT_MS,
    WEB_POLL_INTERVAL_MS,
} from './types';
import type { ProxyRequestLog, ProxyStats } from './types';

export interface UseProxyLogsOptions {
    captureHealthLogs: boolean;
    onConfigLoaded: (config: AppConfig) => void;
}

export interface QuickFilter {
    label: string;
    value: string;
}

export interface UseProxyLogsResult {
    logs: ProxyRequestLog[];
    setLogs: React.Dispatch<React.SetStateAction<ProxyRequestLog[]>>;
    stats: ProxyStats;
    setStats: React.Dispatch<React.SetStateAction<ProxyStats>>;
    loading: boolean;
    filter: string;
    setFilter: (value: string) => void;
    accountFilter: string;
    setAccountFilter: (value: string) => void;
    pageSize: number;
    setPageSize: (value: number) => void;
    currentPage: number;
    totalCount: number;
    setTotalCount: React.Dispatch<React.SetStateAction<number>>;
    totalPages: number;
    pageStart: number;
    pageEnd: number;
    pageSizeOptions: number[];
    filteredLogs: ProxyRequestLog[];
    quickFilters: QuickFilter[];
    uniqueAccounts: string[];
    filterInputRef: React.RefObject<HTMLInputElement | null>;
    loadData: (page?: number, searchFilter?: string, accountEmailFilter?: string) => Promise<void>;
    goToPage: (page: number) => void;
}

const PAGE_SIZE_OPTIONS = [50, 100, 200, 500];

export function useProxyLogs({ captureHealthLogs, onConfigLoaded }: UseProxyLogsOptions): UseProxyLogsResult {
    const { t } = useTranslation();
    const [logs, setLogs] = useState<ProxyRequestLog[]>([]);
    const [stats, setStats] = useState<ProxyStats>({ total_requests: 0, success_count: 0, error_count: 0 });
    const [filter, setFilter] = useState('');
    const [accountFilter, setAccountFilter] = useState('');

    // [FIX] 使用 ref 存储最新的筛选条件，避免 setInterval 闭包问题
    const filterRef = useRef(filter);
    const accountFilterRef = useRef(accountFilter);
    const currentPageRef = useRef(1);
    const filterInputRef = useRef<HTMLInputElement>(null);

    const { accounts, fetchAccounts } = useAccountStore();

    // Pagination state
    const [pageSize, setPageSize] = useState(100);
    const [currentPage, setCurrentPage] = useState(1);
    const [totalCount, setTotalCount] = useState(0);
    const [loading, setLoading] = useState(false);

    const uniqueAccounts = useMemo(() => {
        const emailSet = new Set<string>();
        logs.forEach(log => {
            if (log.account_email) {
                emailSet.add(log.account_email);
            }
        });
        accounts.forEach(acc => {
            emailSet.add(acc.email);
        });
        return Array.from(emailSet).sort();
    }, [logs, accounts]);

    const loadData = async (page = 1, searchFilter = filter, accountEmailFilter = accountFilter): Promise<void> => {
        if (loading) return;
        setLoading(true);

        try {
            // Add timeout control (10 seconds)
            const timeoutPromise = new Promise((_, reject) =>
                setTimeout(() => reject(new Error('Request timeout')), LOAD_DATA_TIMEOUT_MS)
            );

            const config = await Promise.race([
                invoke<AppConfig>('load_config'),
                timeoutPromise
            ]) as AppConfig;

            if (config && config.proxy) {
                onConfigLoaded(config);
                await invoke('set_proxy_monitor_enabled', { enabled: config.proxy.enable_logging });
                await invoke('set_proxy_capture_health_logs', { enabled: !!config.proxy.capture_health_logs });
            }

            const errorsOnly = searchFilter === ERROR_FILTER_SENTINEL;
            const baseFilter = errorsOnly ? '' : searchFilter;
            const actualFilter = accountEmailFilter
                ? (baseFilter ? `${baseFilter} ${accountEmailFilter}` : accountEmailFilter)
                : baseFilter;

            // Get count with filter
            const count = await Promise.race([
                invoke<number>('get_proxy_logs_count_filtered', {
                    filter: actualFilter,
                    errorsOnly: errorsOnly
                }),
                timeoutPromise
            ]) as number;
            setTotalCount(count);

            // Use filtered paginated query
            const offset = (page - 1) * pageSize;
            const history = await Promise.race([
                invoke<ProxyRequestLog[]>('get_proxy_logs_filtered', {
                    filter: actualFilter,
                    errorsOnly: errorsOnly,
                    limit: pageSize,
                    offset: offset
                }),
                timeoutPromise
            ]) as ProxyRequestLog[];

            if (Array.isArray(history)) {
                setLogs(history);
                // Clear pending logs to avoid duplicates (database data is authoritative)
                pendingLogsRef.current = [];
            }

            const currentStats = await Promise.race([
                invoke<ProxyStats>('get_proxy_stats'),
                timeoutPromise
            ]) as ProxyStats;

            if (currentStats) setStats(currentStats);
        } catch (e: any) {
            console.error("Failed to load proxy data", e);
            // Tracked in the error module; stale stats kept, retried on next poll.
            useErrorStore.getState().trackWarning(e, {
              source: 'ProxyMonitor.loadProxyData',
              triggerAction: 'load_proxy_data',
              context: { isTimeout: e.message === 'Request timeout' },
            });
            if (e.message === 'Request timeout') {
                // Show timeout error to user
                console.error('Loading monitor data timeout, please try again later');
            }
        } finally {
            setLoading(false);
        }
    };

    const totalPages = Math.ceil(totalCount / pageSize);
    const pageStart = totalCount === 0 ? 0 : (currentPage - 1) * pageSize + 1;
    const pageEnd = totalCount === 0 ? 0 : Math.min(currentPage * pageSize, totalCount);

    const goToPage = (page: number) => {
        if (page >= 1 && page <= totalPages && page !== currentPage) {
            setCurrentPage(page);
            currentPageRef.current = page; // [FIX] 同步 ref
            loadData(page, filter, accountFilter);
        }
    };

    const pendingLogsRef = useRef<ProxyRequestLog[]>([]);
    const listenerSetupRef = useRef(false);
    const isMountedRef = useRef(true);


    useEffect(() => {
        isMountedRef.current = true;
        loadData();
        fetchAccounts();

        let unlistenFn: (() => void) | null = null;
        let updateTimeout: number | null = null;

        const setupListener = async () => {
            if (!isTauri()) return;
            // Prevent duplicate listener registration (React 18 StrictMode)
            if (listenerSetupRef.current) {
                console.debug('[ProxyMonitor] Listener already set up, skipping...');
                return;
            }
            listenerSetupRef.current = true;

            console.debug('[ProxyMonitor] Setting up event listener for proxy://request');
            unlistenFn = await listen<ProxyRequestLog>('proxy://request', (event) => {
                if (!isMountedRef.current) return;

                const newLog = event.payload;

                // 移除 body 以减少内存占用
                const logSummary = {
                    ...newLog,
                    request_body: undefined,
                    upstream_request_body: undefined,
                    response_body: undefined
                };

                // Check if this log already exists (deduplicate at event level)
                const alreadyExists = pendingLogsRef.current.some(log => log.id === newLog.id);
                if (alreadyExists) {
                    console.debug('[ProxyMonitor] Duplicate event ignored:', newLog.id);
                    return;
                }

                pendingLogsRef.current.push(logSummary);

                // 防抖:每 500ms 批量更新一次
                if (updateTimeout) clearTimeout(updateTimeout);
                updateTimeout = window.setTimeout(async () => {
                    if (!isMountedRef.current) return;

                    const currentPending = pendingLogsRef.current;
                    if (currentPending.length > 0) {
                        setLogs(prev => {
                            // Deduplicate by id
                            const existingIds = new Set(prev.map(log => log.id));
                            const uniqueNewLogs = currentPending.filter(log => !existingIds.has(log.id));
                            // Merge and sort by timestamp descending (newest first)
                            const merged = [...uniqueNewLogs, ...prev];
                            merged.sort((a, b) => b.timestamp - a.timestamp);
                            return merged.slice(0, LIVE_LOG_BUFFER_MAX);
                        });

                        // Fetch stats and total count from backend instead of local calculation
                        try {
                            const [currentStats, count] = await Promise.all([
                                invoke<ProxyStats>('get_proxy_stats'),
                                invoke<number>('get_proxy_logs_count_filtered', { filter: '', errorsOnly: false })
                            ]);
                            if (isMountedRef.current) {
                                if (currentStats) setStats(currentStats);
                                setTotalCount(count);
                            }
                        } catch (e) {
                            console.error('Failed to fetch stats:', e);
                        }

                        pendingLogsRef.current = [];
                    }
                }, LIVE_EVENT_DEBOUNCE_MS);
            });
        };
        setupListener();

        // Web 模式補強：如果不是 Tauri 環境，則啟用定時輪詢
        let pollInterval: number | null = null;
        if (!isTauri()) {
            console.debug('[ProxyMonitor] Web mode detected, starting auto-poll (10s)');
            pollInterval = window.setInterval(() => {
                if (isMountedRef.current) {
                    if (!loading) {
                        // [FIX] 使用 ref.current 获取最新的筛选条件
                        loadData(currentPageRef.current, filterRef.current, accountFilterRef.current);
                    }
                }
            }, WEB_POLL_INTERVAL_MS);
        }

        return () => {
            isMountedRef.current = false;
            listenerSetupRef.current = false;
            if (unlistenFn) unlistenFn();
            if (updateTimeout) clearTimeout(updateTimeout);
            if (pollInterval) clearInterval(pollInterval);
        };
    }, []);

    useEffect(() => {
        const handleGlobalKeyDown = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F')) {
                const activeEl = document.activeElement;
                if (activeEl && activeEl.closest('.payload-viewer-card')) {
                    return;
                }
                e.preventDefault();
                filterInputRef.current?.focus();
                filterInputRef.current?.select();
            }
        };
        window.addEventListener('keydown', handleGlobalKeyDown);
        return () => window.removeEventListener('keydown', handleGlobalKeyDown);
    }, []);

    useEffect(() => {
        setCurrentPage(1);
        loadData(1, filter, accountFilter);
    }, [pageSize]);

    // Reload when filter changes (search based on all logs)
    useEffect(() => {
        setCurrentPage(1);
        loadData(1, filter, accountFilter);
        // [FIX] 同步 ref 值，供 setInterval 使用
        filterRef.current = filter;
        accountFilterRef.current = accountFilter;
        currentPageRef.current = 1;
    }, [filter, accountFilter]);

    // Logs are already filtered and sorted by backend

    const filteredLogs = useMemo(() => {
        let result = logs;
        if (!captureHealthLogs) {
            result = result.filter(log => {
                const isGetSuccess = log.method?.toUpperCase() === 'GET'
                    && typeof log.status === 'number'
                    && log.status >= 200 && log.status < 300;
                return !isGetSuccess;
            });
        }
        if (accountFilter) {
            result = result.filter(log => log.account_email === accountFilter);
        }
        return result;
    }, [logs, accountFilter, captureHealthLogs]);

    const quickFilters = [
        { label: t('monitor.filters.all'), value: '' },
        { label: 'claude', value: 'claude' },
        { label: 'flash', value: 'flash' },
        { label: 'pro', value: 'pro' },
        { label: 'agent', value: 'agent' },
        { label: t('monitor.filters.error'), value: ERROR_FILTER_SENTINEL },
        { label: t('monitor.filters.chat'), value: 'completions' },
        { label: t('monitor.filters.gemini'), value: 'gemini' },
        { label: t('monitor.filters.images'), value: 'images' }
    ];
    return {
        logs,
        setLogs,
        stats,
        setStats,
        loading,
        filter,
        setFilter,
        accountFilter,
        setAccountFilter,
        pageSize,
        setPageSize,
        currentPage,
        totalCount,
        setTotalCount,
        totalPages,
        pageStart,
        pageEnd,
        pageSizeOptions: PAGE_SIZE_OPTIONS,
        filteredLogs,
        quickFilters,
        uniqueAccounts,
        filterInputRef,
        loadData,
        goToPage,
    };
}
