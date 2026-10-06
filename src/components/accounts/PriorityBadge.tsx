import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { useAccountStore } from '../../stores/useAccountStore';
import { DEFAULT_ACCOUNT_PRIORITY } from '../../types/account';

export { DEFAULT_ACCOUNT_PRIORITY };

export interface PriorityBadgeProps {
    priority?: number | null;
    accountId?: string;
    onUpdatePriority?: (priority: number) => Promise<void> | void;
    size?: 'sm' | 'md' | 'xs';
    className?: string;
    isEditing?: boolean;
    onEditChange?: (isEditing: boolean) => void;
}

export function PriorityBadge({
    priority,
    accountId,
    onUpdatePriority,
    size = 'sm',
    className,
    isEditing: controlledIsEditing,
    onEditChange,
}: PriorityBadgeProps) {
    const { t } = useTranslation();
    const [isInternalEditing, setIsInternalEditing] = useState(false);
    const isEditing = controlledIsEditing !== undefined ? controlledIsEditing : isInternalEditing;

    const currentPriority = priority ?? DEFAULT_ACCOUNT_PRIORITY;
    const [inputValue, setInputValue] = useState(String(currentPriority));
    const [hasError, setHasError] = useState(false);

    useEffect(() => {
        setInputValue(String(currentPriority));
        setHasError(false);
    }, [currentPriority, isEditing]);

    const setEditingState = (nextEditing: boolean) => {
        if (onEditChange) {
            onEditChange(nextEditing);
        }
        setIsInternalEditing(nextEditing);
    };

    const handleCancel = () => {
        setInputValue(String(currentPriority));
        setHasError(false);
        setEditingState(false);
    };

    const handleSave = async () => {
        const val = Number.parseInt(inputValue, 10);
        if (!Number.isInteger(val) || val < 1 || val > 100) {
            setHasError(true);
            return;
        }
        setHasError(false);
        setEditingState(false);
        if (val !== currentPriority) {
            if (onUpdatePriority) {
                await onUpdatePriority(val);
            } else if (accountId) {
                await useAccountStore.getState().updateAccountPriority(accountId, val);
            }
        }
    };

    const handleDoubleClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        setEditingState(true);
    };

    if (!isEditing && currentPriority === DEFAULT_ACCOUNT_PRIORITY) {
        return null;
    }

    const sizeClasses = {
        xs: 'text-[9px] px-1.5 py-0.2 rounded',
        sm: 'text-[9px] px-1.5 py-0.5 rounded-md',
        md: 'text-[10px] px-2 py-0.5 rounded-md',
    }[size];

    if (!isEditing) {
        return (
            <span
                className={cn(
                    'font-bold cursor-pointer select-none transition-colors',
                    'bg-gray-100 dark:bg-[#15334d] text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-[#1a3d5c]',
                    sizeClasses,
                    className
                )}
                title={t('accounts.priority_hint', 'Double-click to edit priority')}
                onDoubleClick={handleDoubleClick}
            >
                {t('accounts.priority', 'Priority')}: {currentPriority}
            </span>
        );
    }

    return (
        <div
            className="inline-flex items-center gap-1"
            onClick={(e) => e.stopPropagation()}
            onDoubleClick={(e) => e.stopPropagation()}
        >
            <input
                type="number"
                min={1}
                max={100}
                autoFocus
                value={inputValue}
                onChange={(e) => {
                    setInputValue(e.target.value);
                    setHasError(false);
                }}
                onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                        e.preventDefault();
                        void handleSave();
                    } else if (e.key === 'Escape') {
                        e.preventDefault();
                        handleCancel();
                    }
                }}
                onBlur={handleCancel}
                className={cn(
                    'w-12 px-1 py-0.5 rounded border text-[10px] font-bold outline-none',
                    hasError
                        ? 'border-rose-500 ring-1 ring-rose-500 bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300'
                        : 'border-blue-400 bg-white text-slate-950 dark:bg-slate-800 dark:text-slate-100',
                    className
                )}
                title={
                    hasError
                        ? t('accounts.priority_invalid', 'Priority must be an integer between 1 and 100')
                        : t('accounts.priority_edit_hint', 'Enter to save, Esc to cancel (1-100)')
                }
            />
            {hasError && (
                <span className="text-[9px] text-rose-500 font-semibold whitespace-nowrap">
                    {t('accounts.priority_range_hint', '1-100')}
                </span>
            )}
        </div>
    );
}

export default PriorityBadge;
