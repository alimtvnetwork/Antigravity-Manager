import type { ComponentType } from 'react';
import type { Account, ModelQuota } from '../../../types/account';
import { MODEL_CONFIG } from '../../../config/modelConfig';

export interface AccountCardProps {
    account: Account;
    selected: boolean;
    onSelect: () => void;
    isCurrent: boolean;
    isRefreshing: boolean;
    isSwitching?: boolean;
    isFocused?: boolean;
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
    onViewError: () => void;
}

// Use unified model configuration
export const DEFAULT_MODELS = Object.entries(MODEL_CONFIG).map(([id, config]) => ({
    id,
    label: config.label,
    protectedKey: config.protectedKey,
    Icon: config.Icon
}));

export interface DisplayModel {
    id: string;
    label: string;
    protectedKey: string;
    Icon: ComponentType;
    data: ModelQuota;
}

export interface WeeklyItem {
    id: string;
    label: string;
    percentage: number;
    resetTime: string | undefined;
    cycleTokens: number | undefined;
    Icon: ComponentType;
}
