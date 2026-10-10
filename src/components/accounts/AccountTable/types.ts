import type { Account } from '../../../types/account';
import { categorizeModel, getModelProtectionKey } from '../../../utils/modelCategory';
import { formatDateOnly } from '../../../utils/date';

export const formatDateTimeShort = (val?: number | string | Date | null) => formatDateOnly(val);

export interface AccountTableProps {
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
    focusedAccountId?: string | null;
    showAllEmails?: boolean;
}

export interface SortableRowProps {
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
    isDragDisabled?: boolean;
    modelFilter?: 'gemini' | 'claude';
    showAllEmails?: boolean;
}

export interface AccountRowContentProps {
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
    modelFilter?: 'gemini' | 'claude';
    showAllEmails?: boolean;
}

export function isModelProtected(protectedModels: string[] | undefined, modelName: string): boolean {
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
export function extractAccountResetTime(account: Account): number | null {
    let earliestTime: number | null = null;
    const models = account.quota?.models || [];
    for (const model of models) {
        if (model.reset_time) {
            const t = new Date(model.reset_time).getTime();
            if (!isNaN(t) && (earliestTime === null || t < earliestTime)) {
                earliestTime = t;
            }
        }
    }
    return earliestTime;
}
