import { useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { useConfigStore } from '../../stores/useConfigStore';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { getValidationBlockedStatusLabel } from './accountValidationStatus';
import { SELECTED_CARD_CLASSES } from '../common/selectedState';
import type { AccountCardProps } from './AccountCard/types';
import { useAccountCardModels } from './AccountCard/useAccountCardModels';
import { AccountCardHeader } from './AccountCard/AccountCardHeader';
import { AccountCardQuota } from './AccountCard/AccountCardQuota';
import { AccountCardActions } from './AccountCard/AccountCardActions';

export type { AccountCardProps } from './AccountCard/types';

function AccountCard({ account, selected, onSelect, isCurrent: propIsCurrent, isRefreshing, isSwitching = false, isFocused = false, onSwitch, onRefresh, onViewDetails, onExport, onDelete, onToggleProxy, onViewDevice, onWarmup, onUpdateLabel, onUpdatePriority, onViewError }: AccountCardProps) {
    const { t } = useTranslation();
    const { config, showAllQuotas } = useConfigStore();
    const { instances } = useInstanceStore();
    const isDisabled = Boolean(account.disabled);
    const validationBlockedLabel = getValidationBlockedStatusLabel(account.validation_blocked_reason, t);

    const boundInstance = instances.find(
        (i) => i.config.bound_account_id === account.id ||
            (i.config.bound_email && i.config.bound_email.toLowerCase() === account.email.toLowerCase())
    );

    // Custom label editing state
    const [isEditingLabel, setIsEditingLabel] = useState(false);
    const [labelInput, setLabelInput] = useState(account.custom_label || '');

    // Use the prop directly from parent component
    const isCurrent = propIsCurrent;

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

    const { displayModels, weeklyItems, isModelProtected } = useAccountCardModels({
        account,
        showAllQuotas,
        pinnedModels: config?.pinned_quota_models?.models,
        quotaProtectionEnabled: Boolean(config?.quota_protection?.enabled),
    });

    const cardRef = useRef<HTMLDivElement>(null);

    return (
        <div
            id={`account-card-${account.id}`}
            ref={cardRef}
            className={cn(
            "flex flex-col p-2 sm:p-2.5 rounded-xl border border-slate-200/80 dark:border-slate-800/80 border-l-2 transition-all duration-200 bg-white dark:bg-[#0c2438]",
            isFocused
                ? "bg-teal-50/90 dark:bg-[#0e2c44] text-slate-900 dark:text-cyan-300 font-bold border-l-cyan-500 dark:border-l-cyan-400 border-slate-200/80 dark:border-slate-800/80 shadow-md ring-1 ring-cyan-500/30"
                : isCurrent
                ? cn(SELECTED_CARD_CLASSES, "hover:bg-slate-200/60 dark:hover:bg-[#0c2438]")
                : selected
                ? "bg-blue-50/90 dark:bg-[#0f273d] text-blue-950 dark:text-blue-100 border-l-blue-500 dark:border-l-blue-500 font-semibold shadow-xs ring-1 ring-blue-500/30"
                : "border-l-transparent text-gray-800 dark:text-gray-200 hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60 hover:text-slate-900 dark:hover:text-white hover:border-l-blue-500/70",
            (isRefreshing || isDisabled) && "opacity-70"
        )}>

            <AccountCardHeader
                account={account}
                selected={selected}
                isCurrent={isCurrent}
                isFocused={isFocused}
                isDisabled={isDisabled}
                validationBlockedLabel={validationBlockedLabel}
                boundInstance={boundInstance}
                onSelect={onSelect}
                onUpdatePriority={onUpdatePriority}
            />

            <AccountCardQuota
                account={account}
                isDisabled={isDisabled}
                validationBlockedLabel={validationBlockedLabel}
                displayModels={displayModels}
                weeklyItems={weeklyItems}
                isModelProtected={isModelProtected}
                onViewError={onViewError}
            />

            <AccountCardActions
                account={account}
                isRefreshing={isRefreshing}
                isSwitching={isSwitching}
                isDisabled={isDisabled}
                isEditingLabel={isEditingLabel}
                labelInput={labelInput}
                onSwitch={onSwitch}
                onRefresh={onRefresh}
                onViewDetails={onViewDetails}
                onViewDevice={onViewDevice}
                onExport={onExport}
                onDelete={onDelete}
                onToggleProxy={onToggleProxy}
                onWarmup={onWarmup}
                onUpdateLabel={onUpdateLabel}
                setIsEditingLabel={setIsEditingLabel}
                setLabelInput={setLabelInput}
                handleSaveLabel={handleSaveLabel}
                handleCancelLabel={handleCancelLabel}
                handleKeyDown={handleKeyDown}
            />
        </div >
    );
}

export default AccountCard;
