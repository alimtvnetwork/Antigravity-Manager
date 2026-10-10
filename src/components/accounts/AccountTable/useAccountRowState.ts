import { useMemo, useState, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useConfigStore } from '../../../stores/useConfigStore';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { useErrorStore } from '../../../stores/error-store';
import { getValidationBlockedStatusLabel } from '../accountValidationStatus';
import { supabaseService, type WorkspaceLease } from '../../../services/supabaseService';
import { findQuotaModel, categorizeModel } from '../../../utils/modelCategory';
import { getLiveLimitForModel } from '../../../utils/liveLimit';
import { Gemini, Claude } from '@lobehub/icons';
import { isModelProtected } from './types';
import type { AccountRowContentProps } from './types';

/**
 * Encapsulates all state and derived data for an account table row.
 * Extracted from AccountRowContent to enable splitting the row into cell components.
 */
export function useAccountRowState(props: AccountRowContentProps) {
    const { account, onUpdateLabel, onUpdatePriority, modelFilter = 'gemini' } = props;
    const { t } = useTranslation();
    const { config } = useConfigStore();
    const validationBlockedLabel = getValidationBlockedStatusLabel(account.validation_blocked_reason, t);

    // 自定义标签编辑状态
    const [isEditingLabel, setIsEditingLabel] = useState(false);
    const [labelInput, setLabelInput] = useState(account.custom_label || '');
    const [showInstanceMenu, setShowInstanceMenu] = useState(false);
    const [showMoreMenu, setShowMoreMenu] = useState(false);
    const [showEmail, setShowEmail] = useState(false);
    const [isHoverUnmasked, setIsHoverUnmasked] = useState(false);
    const [moreMenuPos, setMoreMenuPos] = useState<{ top: number; left: number } | null>(null);
    const [editingPriority, setEditingPriority] = useState(false);
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
        }).catch((e) => {
            useErrorStore.getState().trackWarning(e, {
                source: 'AccountTable.listActiveLeases',
                triggerAction: 'list_active_leases',
            });
        });
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

    const openPriorityEditor = () => {
        if (!onUpdatePriority) return;
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

    return {
        t,
        config,
        validationBlockedLabel,
        isEditingLabel, setIsEditingLabel,
        labelInput, setLabelInput,
        showInstanceMenu, setShowInstanceMenu,
        showMoreMenu, setShowMoreMenu,
        showEmail, setShowEmail,
        isHoverUnmasked, setIsHoverUnmasked,
        moreMenuPos, setMoreMenuPos,
        editingPriority, setEditingPriority,
        instances, activeInstanceId,
        menuRef, moreBtnRef, moreMenuRef,
        boundInstance,
        leaseInfo,
        handleSaveLabel, handleCancelLabel, handleKeyDown,
        openPriorityEditor, openMoreMenu,
        displayModels, fourHourModel, weeklyCell,
    };
}

export type AccountRowState = ReturnType<typeof useAccountRowState>;
