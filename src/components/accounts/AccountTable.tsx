/**
 * 账号表格组件
 * 支持拖拽排序功能，用户可以通过拖拽行来调整账号顺序
 */
import { useMemo, useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { useInstanceStore } from '../../stores/useInstanceStore';

import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    DragEndEvent,
    DragStartEvent,
    DragOverlay,
} from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    useSortable,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
    GripVertical,
    ArrowRightLeft,
    RefreshCw,
    Trash2,
    Download,
    Fingerprint,
    Info,
    Lock,
    Ban,
    Diamond,
    Gem,
    Circle,
    ToggleLeft,
    ToggleRight,
    Sparkles,
    Tag,
    X,
    Check,
    Clock,
    Repeat2,
    Terminal,
    MoreHorizontal,
    ArrowUpDown,
    ArrowUp,
    ArrowDown,
} from 'lucide-react';
import type { Account } from '../../types/account';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { Gemini, Claude } from '@lobehub/icons';

import { useConfigStore } from '../../stores/useConfigStore';
import { QuotaItem } from './QuotaItem';
import { categorizeModel, getModelProtectionKey, findQuotaModel } from '../../utils/modelCategory';
import { getValidationBlockedStatusLabel } from './accountValidationStatus';
import { getLiveLimitForModel } from '../../utils/liveLimit';
import { supabaseService, WorkspaceLease } from '../../services/supabaseService';
import { formatDateTime } from '../../utils/date';

// ============================================================================
// 类型定义
// ============================================================================

interface AccountTableProps {
    accounts: Account[];
    selectedIds: Set<string>;
    refreshingIds: Set<string>;
    onToggleSelect: (id: string) => void;
    onToggleAll: () => void;
    currentAccountId: string | null;
    currentAccountEmail?: string | null;
    switchingAccountId: string | null;
    onSwitch: (accountId: string, targetIde?: string) => void;
    onRefresh: (accountId: string) => void;
    onViewDevice: (accountId: string) => void;
    onViewDetails: (accountId: string) => void;
    onExport: (accountId: string) => void;
    onDelete: (accountId: string) => void;
    onToggleProxy: (accountId: string) => void;
    onWarmup?: (accountId: string) => void;
    onUpdateLabel?: (accountId: string, label: string) => void;
    onUpdatePriority?: (accountId: string, priority: number) => Promise<void> | void;
    /** 拖拽排序回调，当用户完成拖拽时触发 */
    onReorder?: (accountIds: string[]) => void;
    onViewError: (accountId: string) => void;
    quotaWindow?: '5h' | 'weekly';
    focusedAccountId?: string | null;
}

interface SortableRowProps {
    account: Account;
    selected: boolean;
    isRefreshing: boolean;
    isCurrent: boolean;
    isFocused?: boolean;
    isSwitching: boolean;
    isDragging?: boolean;
    onSelect: () => void;
    onSwitch: (targetIde?: string) => void;
    onRefresh: () => void;
    onViewDevice: () => void;
    onViewDetails: () => void;
    onExport: () => void;
    onDelete: () => void;
    onToggleProxy: () => void;
    onWarmup?: () => void;
    onUpdateLabel?: (label: string) => void;
    onUpdatePriority?: (priority: number) => Promise<void> | void;
    showPriority?: boolean;
    onViewError: () => void;
    quotaWindow?: '5h' | 'weekly';
    isDragDisabled?: boolean;
    modelFilter?: 'gemini' | 'claude';
}

interface AccountRowContentProps {
    account: Account;
    selected?: boolean;
    isCurrent: boolean;
    isFocused?: boolean;
    isRefreshing: boolean;
    isSwitching: boolean;
    isDisabled: boolean;
    onSwitch: (targetIde?: string) => void;
    onRefresh: () => void;
    onViewDevice: () => void;
    onViewDetails: () => void;
    onExport: () => void;
    onDelete: () => void;
    onToggleProxy: () => void;
    onWarmup?: () => void;
    onUpdateLabel?: (label: string) => void;
    onUpdatePriority?: (priority: number) => Promise<void> | void;
    showPriority?: boolean;
    onViewError: () => void;
    quotaWindow?: '5h' | 'weekly';
    modelFilter?: 'gemini' | 'claude';
}

// ============================================================================
// 辅助函数
// ============================================================================



function isModelProtected(protectedModels: string[] | undefined, modelName: string): boolean {
    if (!protectedModels || protectedModels.length === 0) return false;
    const lowerName = modelName.toLowerCase();

    if (lowerName === 'gemini-pro') {
        return protectedModels.some((model) =>
            categorizeModel(model) === 'gemini-pro' && getModelProtectionKey(model) === 'gemini-3-pro-high',
        );
    }
    if (lowerName === 'gemini-flash') {
        return protectedModels.some((model) =>
            categorizeModel(model) === 'gemini-flash' && getModelProtectionKey(model) === 'gemini-3-flash',
        );
    }
    if (lowerName === 'claude-sonnet') {
        return protectedModels.some((model) =>
            categorizeModel(model) === 'claude' && getModelProtectionKey(model) === 'claude',
        );
    }

    const protectionKey = getModelProtectionKey(lowerName);
    return protectionKey ? protectedModels.includes(protectionKey) : false;
}

/**
 * 提取账号的最快配额重置时间（毫秒时间戳）
 * 用于表格排序
 */
function extractAccountResetTime(account: Account, quotaWindow?: '5h' | 'weekly'): number | null {
    let earliestTime: number | null = null;

    if (quotaWindow === 'weekly') {
        const groups = account.quota?.quota_groups || [];
        for (const group of groups) {
            for (const bucket of group.buckets || []) {
                const isWeekly = bucket.window.toLowerCase().includes('week') || bucket.bucket_id.toLowerCase().includes('week');
                if (isWeekly && bucket.reset_time) {
                    const t = new Date(bucket.reset_time).getTime();
                    if (!isNaN(t) && (earliestTime === null || t < earliestTime)) {
                        earliestTime = t;
                    }
                }
            }
        }
    } else {
        // 5h 或常规视图下，从 models 中获取最近的 reset_time
        const models = account.quota?.models || [];
        for (const model of models) {
            if (model.reset_time) {
                const t = new Date(model.reset_time).getTime();
                if (!isNaN(t) && (earliestTime === null || t < earliestTime)) {
                    earliestTime = t;
                }
            }
        }
    }

    return earliestTime;
}

// ============================================================================
// 子组件
// ============================================================================

/**
 * 可拖拽的表格行组件
 * 使用 @dnd-kit/sortable 实现拖拽功能
 */
function SortableAccountRow({
    account,
    selected,
    isRefreshing,
    isCurrent,
    isFocused = false,
    isSwitching,
    isDragging,
    onSelect,
    onSwitch,
    onRefresh,
    onViewDevice,
    onViewDetails,
    onExport,
    onDelete,
    onToggleProxy,
    onWarmup,
    onUpdateLabel,
    onUpdatePriority,
    showPriority = false,
    onViewError,
    quotaWindow,
    isDragDisabled = false,
    modelFilter = 'gemini',
}: SortableRowProps) {
    const { t } = useTranslation();
    const rowRef = useRef<HTMLTableRowElement | null>(null);

    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging: isSortableDragging,
    } = useSortable({ id: account.id, disabled: isDragDisabled });

    const setMergedRef = (node: HTMLTableRowElement | null) => {
        setNodeRef(node);
        rowRef.current = node;
    };

    useEffect(() => {
        if (isFocused && rowRef.current) {
            rowRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
    }, [isFocused]);

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isSortableDragging ? 0.5 : 1,
        zIndex: isSortableDragging ? 1000 : 'auto',
    };

    return (
        <tr
            id={`account-row-${account.id}`}
            ref={setMergedRef}
            style={style as React.CSSProperties}
            className={cn(
                "group transition-all duration-200 border-b border-gray-100 dark:border-base-200 border-l-4",
                isFocused
                    ? "bg-emerald-50/90 dark:bg-[#15334d] text-slate-900 dark:text-[#43d6a2] font-bold border-l-[#43d6a2] dark:border-l-[#43d6a2] border-emerald-400 dark:border-[#43d6a2]/50 shadow-xl ring-2 ring-[#43d6a2]/60 dark:ring-[#43d6a2]/40"
                    : isCurrent
                    ? "bg-emerald-50/60 dark:bg-[#0c2438] border-l-[#16a97a] dark:border-l-[#16a97a] font-semibold text-slate-900 dark:text-white shadow-xs ring-1 ring-[#16a97a]/30 hover:bg-emerald-100/60 dark:hover:bg-[#15334d]/60"
                    : selected
                    ? "bg-blue-50/90 dark:bg-[#0c2438] text-blue-950 dark:text-blue-100 border-l-[#2878f0] dark:border-l-[#2878f0] font-semibold shadow-md ring-1 ring-[#2878f0]/40 dark:ring-[#2878f0]/30"
                    : isDragging
                    ? "bg-blue-100 dark:bg-blue-900/30 shadow-lg"
                    : "border-l-transparent text-gray-800 dark:text-gray-200 hover:bg-slate-50 dark:hover:bg-[#0c2438]/80 hover:text-slate-900 dark:hover:text-white hover:border-l-[#2878f0] dark:hover:border-l-[#2878f0]"
            )}
        >
            {/* 拖拽手柄 */}
            <td className="pl-2 py-0.5 w-7 align-middle">
                <div
                    {...(!isDragDisabled ? attributes : {})}
                    {...(!isDragDisabled ? listeners : {})}
                    className={cn(
                        "flex items-center justify-center w-5 h-5 rounded transition-colors",
                        isDragDisabled
                            ? "text-gray-200 dark:text-gray-700 cursor-not-allowed opacity-40"
                            : "cursor-grab active:cursor-grabbing text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                    )}
                    title={isDragDisabled ? t('accounts.drag_disabled_during_sort', '已激活列排序，拖拽排序已暂停') : t('accounts.drag_to_reorder')}
                >
                    <GripVertical className="w-3.5 h-3.5" />
                </div>
            </td>
            {/* 复选框 */}
            <td className="px-1.5 py-0.5 w-8 align-middle">
                <input
                    type="checkbox"
                    className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                    checked={selected}
                    onChange={onSelect}
                    disabled={isRefreshing}
                />
            </td>
            <AccountRowContent
                account={account}
                selected={selected}
                isCurrent={isCurrent}
                isFocused={isFocused}
                isRefreshing={isRefreshing}
                isSwitching={isSwitching}
                isDisabled={Boolean(account.disabled)}
                onSwitch={onSwitch}
                onRefresh={onRefresh}
                onViewDevice={onViewDevice}
                onViewDetails={onViewDetails}
                onExport={onExport}
                onDelete={onDelete}
                onToggleProxy={onToggleProxy}
                onWarmup={onWarmup}
                onUpdateLabel={onUpdateLabel}
                onUpdatePriority={onUpdatePriority}
                showPriority={showPriority}
                onViewError={onViewError}
                quotaWindow={quotaWindow}
                modelFilter={modelFilter}
            />
        </tr>
    );
}

/**
 * 账号行内容组件
 * 渲染邮箱、配额、最后使用时间和操作按钮等列
 */
function AccountRowContent({
    account,
    selected = false,
    isCurrent,
    isFocused = false,
    isRefreshing,
    isSwitching,
    isDisabled,
    onSwitch,
    onRefresh,
    onViewDevice,
    onViewDetails,
    onExport,
    onDelete,
    onToggleProxy,
    onWarmup,
    onUpdateLabel,
    onUpdatePriority,
    showPriority = false,
    onViewError,
    modelFilter = 'gemini',
}: AccountRowContentProps) {
    const { t } = useTranslation();
    const { config } = useConfigStore();
    const validationBlockedLabel = getValidationBlockedStatusLabel(account.validation_blocked_reason, t);

    // 自定义标签编辑状态
    const [isEditingLabel, setIsEditingLabel] = useState(false);
    const [labelInput, setLabelInput] = useState(account.custom_label || '');
    const [showInstanceMenu, setShowInstanceMenu] = useState(false);
    const [showMoreMenu, setShowMoreMenu] = useState(false);
    const [moreMenuPos, setMoreMenuPos] = useState<{ top: number; left: number } | null>(null);
    const [editingPriority, setEditingPriority] = useState(false);
    const [priorityInput, setPriorityInput] = useState(String(account.priority ?? 50));
    const { instances, activeInstanceId } = useInstanceStore();
    const menuRef = useRef<HTMLDivElement>(null);
    const moreBtnRef = useRef<HTMLButtonElement>(null);
    const moreMenuRef = useRef<HTMLDivElement>(null);

    const boundInstance = useMemo(() => {
        return instances.find((inst) => inst.config.bound_email === account.email || inst.config.bound_account_id === account.id);
    }, [instances, account.email, account.id]);

    const [leaseInfo, setLeaseInfo] = useState<WorkspaceLease | null>(null);

    useEffect(() => {
        let isMounted = true;
        supabaseService.listActiveLeases().then(leases => {
            if (isMounted) {
                const found = leases.find(l => l.account_id === account.id);
                setLeaseInfo(found || null);
            }
        }).catch(() => {});
        return () => { isMounted = false; };
    }, [account.id]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            const target = event.target as Node;
            if (menuRef.current && !menuRef.current.contains(target)) {
                setShowInstanceMenu(false);
            }
            const inMore = moreMenuRef.current?.contains(target) || moreBtnRef.current?.contains(target);
            if (!inMore) {
                setShowMoreMenu(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);


    const handleSaveLabel = () => {
        if (onUpdateLabel) {
            onUpdateLabel(labelInput.trim());
        }
        setIsEditingLabel(false);
    };

    const handleCancelLabel = () => {
        setLabelInput(account.custom_label || '');
        setIsEditingLabel(false);
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            handleSaveLabel();
        } else if (e.key === 'Escape') {
            handleCancelLabel();
        }
    };

    const savePriority = () => {
        const value = Number.parseInt(priorityInput, 10);
        if (!onUpdatePriority || !Number.isInteger(value) || value < 1 || value > 100) {
            setPriorityInput(String(account.priority ?? 50));
            setEditingPriority(false);
            return;
        }
        void onUpdatePriority(value);
        setEditingPriority(false);
    };

    const openPriorityEditor = () => {
        if (!onUpdatePriority) return;
        setPriorityInput(String(account.priority ?? 50));
        setEditingPriority(true);
    };

    const openMoreMenu = () => {
        const rect = moreBtnRef.current?.getBoundingClientRect();
        if (!rect) return;
        setMoreMenuPos({ top: rect.bottom + 4, left: rect.right });
        setShowMoreMenu(true);
    };

    const displayModels = useMemo(() => {
        // 1. Gemini (shared quota pool)
        const geminiQuotaModel = findQuotaModel(account.quota?.models, 'gemini-pro')
            || findQuotaModel(account.quota?.models, 'gemini-flash')
            || account.quota?.models?.find(m => {
                const cat = categorizeModel(m.name);
                return cat === 'gemini-pro' || cat === 'gemini-flash' || cat === 'gemini-pro-image' || cat === 'gemini-flash-image';
            })
            || account.quota?.models?.find(m => m.name.toLowerCase().includes('gemini'));

        const isGeminiProtected = Boolean(
            config?.quota_protection?.enabled && (
                isModelProtected(account.protected_models, 'gemini-pro') ||
                isModelProtected(account.protected_models, 'gemini-flash')
            )
        );

        const geminiLiveLimit = getLiveLimitForModel(account, 'gemini-3.1-pro-high', 'gemini-pro')
            || getLiveLimitForModel(account, 'gemini-3-flash', 'gemini-flash');

        // 2. Claude (Claude Sonnet pool)
        const claudeQuotaModel = findQuotaModel(account.quota?.models, 'claude')
            || account.quota?.models?.find(m => categorizeModel(m.name) === 'claude')
            || account.quota?.models?.find(m => m.name.toLowerCase().includes('claude'));

        const isClaudeProtected = Boolean(
            config?.quota_protection?.enabled && isModelProtected(account.protected_models, 'claude')
        );

        const claudeLiveLimit = getLiveLimitForModel(account, 'claude-sonnet-4-6', 'claude');

        const baseModels = [
            {
                id: 'gemini',
                label: 'Gemini',
                percentage: geminiQuotaModel?.percentage ?? 0,
                resetTime: geminiQuotaModel?.reset_time,
                isProtected: isGeminiProtected,
                liveLimit: geminiLiveLimit,
                Icon: Gemini.Color,
            },
            {
                id: 'claude',
                label: 'Claude',
                percentage: claudeQuotaModel?.percentage ?? 0,
                resetTime: claudeQuotaModel?.reset_time,
                isProtected: isClaudeProtected,
                liveLimit: claudeLiveLimit,
                Icon: Claude.Color,
            },
        ];

        return baseModels.filter((m) => m.id === modelFilter);
    }, [account.quota?.models, account.protected_models, config?.quota_protection?.enabled, modelFilter]);

    const fourHourModel = displayModels[0];
    const weeklyCell = useMemo(() => {
        const groups = account.quota?.quota_groups || [];
        const group = groups.find((item) => {
            const name = (item.display_name || '').toLowerCase();
            if (modelFilter === 'claude') return name.includes('claude');
            return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
        });
        const bucket = (group?.buckets || []).find((item) =>
            (item.window || '').toLowerCase().includes('week')
            || (item.bucket_id || '').toLowerCase().includes('week')
        );
        return {
            percentage: bucket ? Math.round((bucket.remaining_fraction || 0) * 100) : 0,
            resetTime: bucket?.reset_time,
            weeklyTokens: bucket?.cycle_tokens ?? null,
        };
    }, [account.quota?.quota_groups, modelFilter]);


    return (
        <>
            {/* 邮箱列 */}
            <td className="px-2 py-0.5 align-middle">
                <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
                    <span
                        className={cn(
                        "font-medium text-xs break-all transition-colors",
                        isFocused || selected
                            ? "text-blue-950 dark:text-blue-200 font-bold"
                            : isCurrent
                            ? "text-slate-950 dark:text-white font-bold"
                            : "text-gray-900 dark:text-gray-100 group-hover:text-blue-600 dark:group-hover:text-blue-400"
                        )}
                        title={account.email}
                        onDoubleClick={(event) => {
                            event.stopPropagation();
                            openPriorityEditor();
                        }}
                    >
                        {account.email}
                    </span>

                    <div className="flex items-center gap-1 shrink-0">
                        {isCurrent ? (
                            <span className="px-1.5 py-0.2 rounded bg-[#16a97a]/15 dark:bg-[#16a97a]/25 text-[#16a97a] dark:text-[#43d6a2] text-[9px] font-bold shadow-xs border border-[#16a97a]/30 dark:border-[#16a97a]/40">
                                {t('accounts.current').toUpperCase()}
                            </span>
                        ) : null}
                        {isDisabled ? (
                            <span
                                className="px-1.5 py-0.2 rounded bg-rose-100 dark:bg-rose-900/50 text-rose-700 dark:text-rose-300 text-[9px] font-bold flex items-center gap-0.5 shadow-xs border border-rose-200/50"
                            >
                                <Ban className="w-2.5 h-2.5" />
                                <span>{t('accounts.disabled')}</span>
                            </span>
                        ) : null}

                        {account.proxy_disabled ? (
                            <span
                                className="px-1.5 py-0.2 rounded bg-orange-100 dark:bg-orange-900/50 text-orange-700 dark:text-orange-300 text-[9px] font-bold flex items-center gap-0.5 shadow-xs border border-orange-200/50"
                            >
                                <Ban className="w-2.5 h-2.5" />
                                <span>{t('accounts.proxy_disabled')}</span>
                            </span>
                        ) : null}

                        {account.quota?.is_forbidden ? (
                            <span className="px-1.5 py-0.2 rounded bg-red-100 dark:bg-red-900/50 text-red-600 dark:text-red-400 text-[9px] font-bold flex items-center gap-0.5 shadow-xs border border-red-200/50">
                                <Lock className="w-2.5 h-2.5" />
                                <span>{t('accounts.forbidden')}</span>
                            </span>
                        ) : null}
                        {account.validation_blocked ? (
                            <span className="px-1.5 py-0.2 rounded bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-400 text-[9px] font-bold flex items-center gap-0.5 shadow-xs border border-amber-200/50">
                                <Clock className="w-2.5 h-2.5" />
                                <span>{validationBlockedLabel}</span>
                            </span>
                        ) : null}

                        {/* 订阅类型徽章 */}
                        {account.quota?.subscription_tier && (() => {
                            const tier = account.quota.subscription_tier.toLowerCase();
                            if (tier.includes('ultra')) {
                                return (
                                    <span className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-gradient-to-r from-purple-600 to-pink-600 text-white text-[9px] font-bold shadow-xs cursor-default">
                                        <Gem className="w-2.5 h-2.5 fill-current" />
                                        {t('accounts.ultra')}
                                    </span>
                                );
                            }
                            if (tier.includes('pro')) {
                                return (
                                    <span className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-[9px] font-bold shadow-xs cursor-default">
                                        <Diamond className="w-2.5 h-2.5 fill-current" />
                                        {t('accounts.pro')}
                                    </span>
                                );
                            }
                            return (
                                <span className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-gray-100 dark:bg-[#15334d] text-gray-600 dark:text-gray-400 text-[9px] font-bold shadow-xs border border-gray-200 dark:border-[#15334d] hover:bg-gray-200 transition-colors cursor-default">
                                    <Circle className="w-2.5 h-2.5" />
                                    {t('accounts.free')}
                                </span>
                            );
                        })()}
                        {/* 绑定实例徽章 */}
                        {boundInstance && (
                            <span
                                className="flex items-center gap-1 px-1.5 py-0.2 rounded bg-indigo-50 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300 text-[9px] font-bold shadow-xs border border-indigo-200/50 dark:border-indigo-800/50 cursor-default"
                                title={`Bound to profile: ${boundInstance.config.name}`}
                            >
                                <span className={cn(
                                    "w-1.5 h-1.5 rounded-full shrink-0",
                                    boundInstance.is_running ? "bg-emerald-500 animate-pulse" : "bg-indigo-400"
                                )} />
                                <span>{boundInstance.config.name}</span>
                            </span>
                        )}
                        {editingPriority ? (
                            <input
                                type="number"
                                min={1}
                                max={100}
                                autoFocus
                                value={priorityInput}
                                onChange={(event) => setPriorityInput(event.target.value)}
                                onClick={(event) => event.stopPropagation()}
                                onDoubleClick={(event) => event.stopPropagation()}
                                onKeyDown={(event) => {
                                    if (event.key === 'Enter') savePriority();
                                    if (event.key === 'Escape') {
                                        setPriorityInput(String(account.priority ?? 50));
                                        setEditingPriority(false);
                                    }
                                }}
                                onBlur={savePriority}
                                className="w-12 px-1 py-0.5 rounded border border-blue-400 bg-white text-slate-950 text-[10px] font-bold"
                            />
                        ) : showPriority ? (
                            <span
                                className="px-1.5 py-0.2 rounded bg-gray-100 dark:bg-base-300 text-gray-500 dark:text-gray-400 text-[9px] font-bold cursor-text"
                                title={t('accounts.priority_hint')}
                                onDoubleClick={(event) => {
                                    event.stopPropagation();
                                    openPriorityEditor();
                                }}
                            >
                                {t('accounts.priority')}: {account.priority ?? 50}
                            </span>
                        ) : null}
                        {/* 远程节点租赁徽章 */}
                        {leaseInfo && (
                            <span
                                className="flex items-center gap-1 px-1.5 py-0.2 rounded bg-purple-50 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300 text-[9px] font-bold shadow-xs border border-purple-200/50 dark:border-purple-800/50 cursor-default"
                                title={`Leased by Node: ${leaseInfo.node_alias} (${leaseInfo.profile_name})`}
                            >
                                <Lock className="w-2.5 h-2.5" />
                                <span>{leaseInfo.node_alias}</span>
                            </span>
                        )}
                        {/* 自定义标签 */}
                        {account.custom_label ? (
                            isEditingLabel ? null : (
                                <span className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 text-[9px] font-bold shadow-xs border border-orange-200/50 dark:border-orange-800/50">
                                    <Tag className="w-2.5 h-2.5" />
                                    {account.custom_label}
                                </span>
                            )
                        ) : null}
                        {/* 标签编辑输入框 */}
                        {isEditingLabel && (
                            <div className="flex items-center gap-1">
                                <input
                                    type="text"
                                    className="px-1.5 py-0.2 text-[10px] w-20 border border-orange-300 dark:border-orange-700 rounded focus:outline-none focus:ring-1 focus:ring-orange-500 bg-white dark:bg-base-200"
                                    placeholder={t('accounts.custom_label_placeholder', 'Label')}
                                    value={labelInput}
                                    onChange={(e) => setLabelInput(e.target.value)}
                                    onKeyDown={handleKeyDown}
                                    autoFocus
                                    maxLength={15}
                                    onClick={(e) => e.stopPropagation()}
                                />
                                <button
                                    className="p-0.5 text-green-600 hover:bg-green-50 dark:hover:bg-green-900/30 rounded transition-all"
                                    onClick={(e) => { e.stopPropagation(); handleSaveLabel(); }}
                                >
                                    <Check className="w-3 h-3" />
                                </button>
                                <button
                                    className="p-0.5 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 rounded transition-all"
                                    onClick={(e) => { e.stopPropagation(); handleCancelLabel(); }}
                                >
                                    <X className="w-3 h-3" />
                                </button>
                            </div>
                        )}
                    </div>

                </div>
            </td>

            {/* 模型配额列 (永久可见，禁止隐藏) */}
            <td className="px-2 py-0.5 align-middle">
                {isDisabled || account.quota?.is_forbidden || account.validation_blocked ? (
                    <div className={cn(
                        "flex items-center justify-center gap-2 py-1 px-3 rounded-lg border group/error",
                        account.validation_blocked ? "bg-amber-50/50 dark:bg-amber-900/10 border-amber-100/50 dark:border-amber-900/20" : "bg-red-50/50 dark:bg-red-900/10 border-red-100/50 dark:border-red-900/20"
                    )}>
                        <div className={cn(
                            "flex items-center gap-1.5",
                            account.validation_blocked ? "text-amber-600 dark:text-amber-400" : "text-red-600 dark:text-red-400"
                        )}>
                            {account.validation_blocked ? <Clock className="w-3 h-3" /> : (account.quota?.is_forbidden ? <Lock className="w-3 h-3" /> : <Ban className="w-3 h-3" />)}
                            <span className={cn(
                                "text-[10px] font-bold",
                                account.validation_blocked ? "text-amber-700/80 dark:text-amber-400" : "text-red-700/80 dark:text-red-400"
                            )}>
                                {account.validation_blocked ? validationBlockedLabel : (isDisabled ? t('accounts.status.disabled') : t('accounts.forbidden_msg'))}
                            </span>
                        </div>
                        <div className={cn(
                            "w-px h-3",
                            account.validation_blocked ? "bg-amber-200 dark:bg-amber-800/50" : "bg-red-200 dark:bg-red-800/50"
                        )} />
                        <button
                            onClick={(e) => { e.stopPropagation(); onViewError(); }}
                            className="text-[10px] font-medium text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-0.5"
                        >
                            {t('accounts.view_error')}
                        </button>
                    </div>
                ) : (
                    <div className="grid grid-cols-2 gap-1.5 py-0">
                        <QuotaItem
                            label="4h"
                            percentage={fourHourModel?.percentage ?? 0}
                            resetTime={fourHourModel?.resetTime}
                            isProtected={fourHourModel?.isProtected}
                            liveLimit={fourHourModel?.liveLimit}
                            Icon={fourHourModel?.Icon || (modelFilter === 'claude' ? Claude.Color : Gemini.Color)}
                        />
                        <QuotaItem
                            label={t('accounts.table.weekly_quota', 'Weekly')}
                            percentage={weeklyCell.percentage}
                            resetTime={weeklyCell.resetTime}
                            weeklyTokens={weeklyCell.weeklyTokens}
                            Icon={modelFilter === 'claude' ? Claude.Color : Gemini.Color}
                        />
                    </div>
                )}
            </td>

            {/* 最后使用时间列 */}
            <td className="px-2 py-0.5 align-middle whitespace-nowrap">
                <span className="text-[11px] font-medium text-gray-600 dark:text-gray-400 font-mono">
                    {formatDateTime(account.last_used)}
                </span>
            </td>

            {/* 操作列 */}
            <td className={cn(
                "px-2 py-0.5 sticky right-0 z-10 w-[220px] xl:w-[280px] shadow-[-12px_0_12px_-12px_rgba(0,0,0,0.1)] dark:shadow-[-12px_0_12px_-12px_rgba(255,255,255,0.05)] text-center align-middle transition-colors",
                // 动态高对比高亮处理
                isFocused
                    ? "bg-emerald-50/90 dark:bg-[#15334d]"
                    : selected
                    ? "bg-blue-50/90 dark:bg-[#0c2438]"
                    : isCurrent
                    ? "bg-emerald-50/60 dark:bg-[#0c2438]"
                    : "bg-white dark:bg-[#071a27]",
                !isCurrent && !selected && !isFocused ? "group-hover:bg-slate-50 dark:group-hover:bg-[#0c2438]/80" : ""
            )}>
                <div className="flex items-center justify-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
                    {/* 切换/实例选择操作组 */}
                    <div className="relative inline-flex items-center" ref={menuRef}>
                        <button
                            className={`p-1 text-gray-500 dark:text-gray-400 rounded transition-all ${(isSwitching || isDisabled) ? 'bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 cursor-not-allowed' : 'hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
                            onClick={(e) => { e.stopPropagation(); onSwitch(); }}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                setShowInstanceMenu(!showInstanceMenu);
                            }}
                            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : `${t('accounts.switch_to_classic', '切换到 Antigravity')} (右键选择实例)`)}
                            disabled={isSwitching || isDisabled}
                        >
                            <ArrowRightLeft className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
                        </button>

                        {showInstanceMenu && (
                            <div className="absolute top-full left-0 mt-1 w-52 rounded-xl shadow-xl bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 py-1.5 z-50 animate-in fade-in zoom-in-95">
                                <div className="px-3 py-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                                    目标实例 / Target Instance
                                </div>
                                {instances.map((inst) => (
                                    <button
                                        key={inst.config.id}
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            setShowInstanceMenu(false);
                                            onSwitch(`instance:${inst.config.id}`);
                                        }}
                                        className="w-full flex items-center justify-between px-3 py-1.5 text-xs text-left hover:bg-gray-50 dark:hover:bg-base-100 text-gray-700 dark:text-gray-300"
                                    >
                                        <div className="flex items-center gap-1.5 truncate">
                                            <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${inst.is_running ? 'bg-emerald-500' : 'bg-gray-400'}`} />
                                            <span className="truncate">{inst.config.name}</span>
                                        </div>
                                        {inst.config.id === activeInstanceId && (
                                            <span className="text-[10px] text-blue-600 font-medium">Active</span>
                                        )}
                                    </button>
                                ))}
                            </div>
                        )}
                    </div>

                    <button
                        className={`flex p-1 text-gray-500 dark:text-gray-400 rounded transition-all ${(isSwitching || isDisabled) ? 'bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 cursor-not-allowed' : 'hover:text-sky-600 dark:hover:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-900/30'}`}
                        onClick={(e) => { e.stopPropagation(); onSwitch('ide'); }}
                        title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_ide', 'Switch to Antigravity IDE'))}
                        disabled={isSwitching || isDisabled}
                    >
                        <Repeat2 className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
                    </button>
                    <button
                        className={`flex p-1 text-gray-500 dark:text-gray-400 rounded transition-all ${(isSwitching || isDisabled) ? 'bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 cursor-not-allowed' : 'hover:text-emerald-600 dark:hover:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-900/30'}`}
                        onClick={(e) => { e.stopPropagation(); onSwitch('agy'); }}
                        title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_agy', 'Switch to Antigravity CLI (agy)'))}
                        disabled={isSwitching || isDisabled}
                    >
                        <Terminal className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
                    </button>

                    <button
                        ref={moreBtnRef}
                        type="button"
                        className="p-1 text-gray-500 dark:text-gray-400 hover:text-slate-950 hover:bg-white rounded transition-all"
                        onClick={(event) => {
                            event.stopPropagation();
                            if (showMoreMenu) {
                                setShowMoreMenu(false);
                                return;
                            }
                            openMoreMenu();
                        }}
                        title={t('common.more', 'More')}
                    >
                        <MoreHorizontal className="w-3.5 h-3.5" />
                    </button>
                    {showMoreMenu && moreMenuPos && createPortal(
                        <div
                            ref={moreMenuRef}
                            className="fixed z-[10000] min-w-[168px] rounded-lg border border-gray-200 bg-white py-1 text-slate-950 shadow-xl"
                            style={{ top: moreMenuPos.top, left: moreMenuPos.left, transform: 'translateX(-100%)' }}
                            onClick={(event) => event.stopPropagation()}
                        >
                            <button
                                type="button"
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                                disabled={isRefreshing || isDisabled}
                                onClick={() => { setShowMoreMenu(false); onRefresh(); }}
                            >
                                <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
                                {t('common.refresh')}
                            </button>
                            <button
                                type="button"
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                                onClick={() => { setShowMoreMenu(false); onViewDetails(); }}
                            >
                                <Info className="w-3.5 h-3.5" />
                                {t('common.details')}
                            </button>
                            <button
                                type="button"
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                                onClick={() => { setShowMoreMenu(false); onViewDevice(); }}
                            >
                                <Fingerprint className="w-3.5 h-3.5" />
                                {t('accounts.device_fingerprint')}
                            </button>
                            <button
                                type="button"
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                                onClick={() => { setShowMoreMenu(false); onExport(); }}
                            >
                                <Download className="w-3.5 h-3.5" />
                                {t('common.export')}
                            </button>
                        </div>,
                        document.body,
                    )}
                    {onUpdateLabel && (
                        <button
                            className={cn(
                                "hidden lg:inline-flex p-1 rounded transition-all",
                                account.custom_label
                                    ? "text-orange-500 hover:text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-900/30"
                                    : "text-gray-500 dark:text-gray-400 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/30"
                            )}
                            onClick={(e) => { e.stopPropagation(); setIsEditingLabel(true); }}
                            title={t('accounts.edit_label', 'Edit Label')}
                        >
                            <Tag className="w-3.5 h-3.5" />
                        </button>
                    )}

                    {onWarmup && (
                        <button
                            className={`hidden xl:inline-flex p-1 text-gray-500 dark:text-gray-400 rounded transition-all ${(isRefreshing || isDisabled) ? 'bg-orange-50 dark:bg-orange-900/10 text-orange-600 dark:text-orange-400 cursor-not-allowed' : 'hover:text-orange-500 dark:hover:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/30'}`}
                            onClick={(e) => { e.stopPropagation(); onWarmup(); }}
                            title={isDisabled ? t('accounts.disabled_tooltip') : (isRefreshing ? t('common.loading') : t('accounts.warmup_this', 'Warmup Account'))}
                            disabled={isRefreshing || isDisabled}
                        >
                            <Sparkles className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-pulse' : ''}`} />
                        </button>
                    )}
                    <button
                        className={cn(
                            "p-1 rounded transition-all",
                            account.proxy_disabled
                                ? "text-gray-500 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/30"
                                : "text-gray-500 dark:text-gray-400 hover:text-orange-600 dark:hover:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/30"
                        )}
                        onClick={(e) => { e.stopPropagation(); onToggleProxy(); }}
                        title={account.proxy_disabled ? t('accounts.enable_proxy') : t('accounts.disable_proxy')}
                    >
                        {account.proxy_disabled ? (
                            <ToggleRight className="w-3.5 h-3.5" />
                        ) : (
                            <ToggleLeft className="w-3.5 h-3.5" />
                        )}
                    </button>
                    <button
                        className="p-1 text-gray-500 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 rounded transition-all"
                        onClick={(e) => { e.stopPropagation(); onDelete(); }}
                        title={t('common.delete')}
                    >
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                </div>
            </td>
        </>
    );
}

// ============================================================================
// 主组件
// ============================================================================

/**
 * 账号表格组件
 * 支持拖拽排序、多选、批量操作等功能
 */
function AccountTable({
    accounts,
    selectedIds,
    refreshingIds,
    onToggleSelect,
    onToggleAll,
    currentAccountId,
    currentAccountEmail,
    switchingAccountId,
    focusedAccountId,
    onSwitch,
    onRefresh,
    onViewDevice,
    onViewDetails,
    onExport,
    onDelete,
    onToggleProxy,
    onReorder,
    onWarmup,
    onUpdateLabel,
    onUpdatePriority,
    onViewError,
    quotaWindow,
}: AccountTableProps) {
    const { t } = useTranslation();

    const isAccountCurrent = (acc: Account) => {
        const isIdMatch = Boolean(currentAccountId && acc.id === currentAccountId);
        if (isIdMatch) {
            return true;
        }
        const isEmailMatch = Boolean(
            currentAccountEmail && acc.email && acc.email.toLowerCase() === currentAccountEmail.toLowerCase()
        );
        if (isEmailMatch) {
            return true;
        }
        return false;
    };

    const [modelFilter, setModelFilter] = useState<'gemini' | 'claude'>('gemini');
    const showPriority = useMemo(() => {
        const values = new Set(accounts.map((account) => account.priority ?? 50));
        return values.size > 1;
    }, [accounts]);
    const [activeId, setActiveId] = useState<string | null>(null);
    // 排序状态配置: 支持按配额重置时间 (reset_time) 或最后使用时间 (last_used) 排序
    const [sortConfig, setSortConfig] = useState<{
        key: 'reset_time' | 'last_used' | null;
        direction: 'asc' | 'desc' | null;
    }>({
        key: null,
        direction: null,
    });

    const isSortingActive = sortConfig.key !== null && sortConfig.direction !== null;

    const handleSortToggle = (key: 'reset_time' | 'last_used') => {
        setSortConfig(prev => {
            if (prev.key !== key) {
                return { key, direction: 'asc' };
            }
            if (prev.direction === 'asc') {
                return { key, direction: 'desc' };
            }
            return { key: null, direction: null };
        });
    };

    // 根据排序状态对 accounts 进行拦截排序
    const sortedAccounts = useMemo(() => {
        if (!isSortingActive) return accounts;

        return [...accounts].sort((a, b) => {
            if (sortConfig.key === 'reset_time') {
                const timeA = extractAccountResetTime(a, quotaWindow);
                const timeB = extractAccountResetTime(b, quotaWindow);

                // 没有 reset_time 的排到后面
                if (timeA === null && timeB === null) return 0;
                if (timeA === null) return 1;
                if (timeB === null) return -1;

                return sortConfig.direction === 'asc' ? timeA - timeB : timeB - timeA;
            }

            if (sortConfig.key === 'last_used') {
                const timeA = a.last_used || 0;
                const timeB = b.last_used || 0;

                return sortConfig.direction === 'asc' ? timeA - timeB : timeB - timeA;
            }

            return 0;
        });
    }, [accounts, sortConfig, quotaWindow, isSortingActive]);

    // 配置拖拽传感器
    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: { distance: 8 }, // 需要移动 8px 才触发拖拽
        }),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    const accountIds = useMemo(() => sortedAccounts.map(a => a.id), [sortedAccounts]);
    const activeAccount = useMemo(() => sortedAccounts.find(a => a.id === activeId), [sortedAccounts, activeId]);

    const handleDragStart = (event: DragStartEvent) => {
        if (isSortingActive) return;
        setActiveId(event.active.id as string);
    };

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveId(null);

        if (isSortingActive) return;

        if (over && active.id !== over.id) {
            const oldIndex = accountIds.indexOf(active.id as string);
            const newIndex = accountIds.indexOf(over.id as string);

            if (oldIndex !== -1 && newIndex !== -1 && onReorder) {
                onReorder(arrayMove(accountIds, oldIndex, newIndex));
            }
        }
    };

    if (accounts.length === 0) {
        return (
            <div className="bg-white dark:bg-base-100 rounded-2xl p-12 shadow-sm border border-gray-100 dark:border-base-200 text-center">
                <p className="text-gray-400 mb-2">{t('accounts.empty.title')}</p>
                <p className="text-sm text-gray-400">{t('accounts.empty.desc')}</p>
            </div>
        );
    }

    return (
        <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
        >
            <div className="overflow-x-auto">
                <table className="w-full">
                    <thead>
                        <tr className="border-b border-gray-100 dark:border-base-200 bg-gray-50 dark:bg-base-200">
                            <th className="pl-2 py-1 text-left w-7">
                                <span className="sr-only">{t('accounts.drag_to_reorder')}</span>
                            </th>
                            <th className="px-1.5 py-1 text-left w-8">
                                <input
                                    type="checkbox"
                                    className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                                    checked={accounts.length > 0 && selectedIds.size === accounts.length}
                                    onChange={onToggleAll}
                                />
                            </th>
                            <th className="px-2 py-1 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-[260px] whitespace-nowrap">{t('accounts.table.email')}</th>
                            <th className="px-2 py-1 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider min-w-[320px] whitespace-nowrap">
                                <div className="flex items-center justify-between gap-1 w-full">
                                    <button
                                        type="button"
                                        onClick={() => handleSortToggle('reset_time')}
                                        className={cn(
                                            "inline-flex items-center gap-1 hover:text-blue-600 dark:hover:text-blue-400 transition-colors uppercase font-medium",
                                            sortConfig.key === 'reset_time' && "text-blue-600 dark:text-blue-400 font-semibold"
                                        )}
                                        title={t('accounts.table.sort_by_reset_time', '点击按配额重置时间排序')}
                                    >
                                        <span>4h / {t('accounts.table.weekly_quota', 'Weekly')}</span>
                                        {sortConfig.key === 'reset_time' ? (
                                            sortConfig.direction === 'asc' ? <ArrowUp className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" /> : <ArrowDown className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" />
                                        ) : (
                                            <ArrowUpDown className="w-3 h-3 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 opacity-60 hover:opacity-100" />
                                        )}
                                    </button>

                                    {/* Gemini / Claude 视图切换药丸按钮 */}
                                    <div className="inline-flex items-center p-0.5 rounded-lg bg-gray-200/90 dark:bg-slate-900 border border-gray-300/80 dark:border-slate-800 text-[10px] font-semibold">
                                        <button
                                            type="button"
                                            onClick={() => setModelFilter('gemini')}
                                            className={cn(
                                                "px-2 py-0.5 rounded-md transition-all cursor-pointer font-medium",
                                                modelFilter === 'gemini'
                                                    ? "bg-white dark:bg-slate-800 text-amber-600 dark:text-amber-400 font-bold border border-amber-500/30 dark:border-amber-400/30 shadow-xs"
                                                    : "text-gray-600 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-200"
                                            )}
                                            title="Only show Gemini"
                                        >
                                            Gemini
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => setModelFilter('claude')}
                                            className={cn(
                                                "px-2 py-0.5 rounded-md transition-all cursor-pointer font-medium",
                                                modelFilter === 'claude'
                                                    ? "bg-white dark:bg-slate-800 text-amber-600 dark:text-amber-400 font-bold border border-amber-500/30 dark:border-amber-400/30 shadow-xs"
                                                    : "text-gray-600 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-200"
                                            )}
                                            title="Only show Claude"
                                        >
                                            Claude
                                        </button>
                                    </div>
                                </div>
                            </th>
                            <th className="px-2 py-1 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-[150px] whitespace-nowrap">
                                <button
                                    type="button"
                                    onClick={() => handleSortToggle('last_used')}
                                    className={cn(
                                        "inline-flex items-center gap-1 hover:text-blue-600 dark:hover:text-blue-400 transition-colors uppercase font-medium",
                                        sortConfig.key === 'last_used' && "text-blue-600 dark:text-blue-400 font-semibold"
                                    )}
                                    title={t('accounts.table.sort_by_last_used', '点击按最后使用时间排序')}
                                >
                                    <span>{t('accounts.table.last_used')}</span>
                                    {sortConfig.key === 'last_used' ? (
                                        sortConfig.direction === 'asc' ? <ArrowUp className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" /> : <ArrowDown className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" />
                                    ) : (
                                        <ArrowUpDown className="w-3 h-3 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 opacity-60 hover:opacity-100" />
                                    )}
                                </button>
                            </th>
                            <th className="px-2 py-1 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider whitespace-nowrap sticky right-0 w-[220px] xl:w-[280px] bg-gray-50 dark:bg-base-200 z-20 shadow-[-12px_0_12px_-12px_rgba(0,0,0,0.1)] dark:shadow-[-12px_0_12px_-12px_rgba(255,255,255,0.05)] text-center">{t('accounts.table.actions')}</th>
                        </tr>
                    </thead>
                    <SortableContext items={accountIds} strategy={verticalListSortingStrategy}>
                        <tbody className="divide-y divide-gray-100 dark:divide-base-200">
                            {sortedAccounts.map((account) => (
                                <SortableAccountRow
                                    key={account.id}
                                    account={account}
                                    selected={selectedIds.has(account.id)}
                                    isRefreshing={refreshingIds.has(account.id)}
                                    isCurrent={isAccountCurrent(account)}
                                    isFocused={Boolean(focusedAccountId && account.id === focusedAccountId)}
                                    isSwitching={account.id === switchingAccountId}
                                    isDragging={account.id === activeId}
                                    onSelect={() => onToggleSelect(account.id)}
                                    onSwitch={(targetIde?: string) => onSwitch(account.id, targetIde)}
                                    onRefresh={() => onRefresh(account.id)}
                                    onViewDevice={() => onViewDevice(account.id)}
                                    onViewDetails={() => onViewDetails(account.id)}
                                    onExport={() => onExport(account.id)}
                                    onDelete={() => onDelete(account.id)}
                                    onToggleProxy={() => onToggleProxy(account.id)}
                                    onWarmup={onWarmup ? () => onWarmup(account.id) : undefined}
                                    onUpdateLabel={onUpdateLabel ? (label: string) => onUpdateLabel(account.id, label) : undefined}
                                    onUpdatePriority={onUpdatePriority ? (priority: number) => onUpdatePriority(account.id, priority) : undefined}
                                    showPriority={showPriority}
                                    onViewError={() => onViewError(account.id)}
                                    quotaWindow={quotaWindow}
                                    isDragDisabled={isSortingActive}
                                    modelFilter={modelFilter}
                                />
                            ))}
                        </tbody>
                    </SortableContext>
                </table >
            </div >

            {/* 拖拽悬浮预览层 */}
            <DragOverlay>
                {
                    activeAccount ? (
                        <table className="w-full bg-white dark:bg-base-100 shadow-2xl rounded-lg border border-blue-200 dark:border-blue-800">
                            <tbody>
                                <tr className="bg-blue-50 dark:bg-blue-900/30">
                                    <td className="pl-2 py-0.5 w-7">
                                        <div className="flex items-center justify-center w-5 h-5 text-blue-500">
                                            <GripVertical className="w-3.5 h-3.5" />
                                        </div>
                                    </td>
                                    <td className="px-1.5 py-0.5 w-8">
                                        <input
                                            type="checkbox"
                                            className="checkbox checkbox-xs rounded border-2"
                                            checked={selectedIds.has(activeAccount.id)}
                                            readOnly
                                        />
                                    </td>
                                    <AccountRowContent
                                        account={activeAccount}
                                        selected={selectedIds.has(activeAccount.id)}
                                        isCurrent={isAccountCurrent(activeAccount)}
                                        isRefreshing={refreshingIds.has(activeAccount.id)}
                                        isSwitching={activeAccount.id === switchingAccountId}
                                        onSwitch={() => { }}
                                        onRefresh={() => { }}
                                        onViewDevice={() => { }}
                                        onViewDetails={() => { }}
                                        onExport={() => { }}
                                        onDelete={() => { }}
                                        onToggleProxy={() => { }}
                                        isDisabled={Boolean(activeAccount.disabled)}
                                        onViewError={() => { }}
                                        quotaWindow={quotaWindow}
                                    />
                                </tr>
                            </tbody>
                        </table>
                    ) : null
                }
            </DragOverlay>
        </DndContext>
    );
}

export default AccountTable;
