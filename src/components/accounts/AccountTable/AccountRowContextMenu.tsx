import { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { ArrowRightLeft, Ban, RefreshCw, Trash2 } from 'lucide-react';
import type { Account } from '../../../types/account';

export interface AccountRowContextMenuProps {
    x: number;
    y: number;
    account: Account;
    onClose: () => void;
    onSwitch: () => void;
    onRefresh: () => void;
    onDelete: () => void;
    onToggleDisable: () => void;
}

export function AccountRowContextMenu({
    x,
    y,
    account,
    onClose,
    onSwitch,
    onRefresh,
    onDelete,
    onToggleDisable,
}: AccountRowContextMenuProps) {
    const { t } = useTranslation();
    const menuRef = useRef<HTMLDivElement>(null);

    const left = Math.max(8, Math.min(x, window.innerWidth - 190));
    const top = Math.max(8, Math.min(y, window.innerHeight - 240));

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };

        const handleClickOutside = (e: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        const handleScroll = () => {
            onClose();
        };

        window.addEventListener('keydown', handleKeyDown);
        document.addEventListener('mousedown', handleClickOutside);
        window.addEventListener('scroll', handleScroll, true);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            document.removeEventListener('mousedown', handleClickOutside);
            window.removeEventListener('scroll', handleScroll, true);
        };
    }, [onClose]);

    return createPortal(
        <div
            ref={menuRef}
            className="fixed z-[10000] min-w-[180px] rounded-lg border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] py-1 shadow-2xl text-xs text-slate-800 dark:text-slate-200 select-none"
            style={{ top, left }}
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => {
                e.preventDefault();
                e.stopPropagation();
            }}
        >
            {/* Switch Account */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-amber-500/10 hover:text-amber-600 dark:hover:text-amber-400"
                onClick={(e) => {
                    e.stopPropagation();
                    onSwitch();
                    onClose();
                }}
            >
                <ArrowRightLeft className="w-3.5 h-3.5 text-amber-500 dark:text-amber-400" />
                <span>{t('accounts.switch_account', 'Switch Account')}</span>
            </button>

            {/* Disable / Enable Account */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a]"
                onClick={(e) => {
                    e.stopPropagation();
                    onToggleDisable();
                    onClose();
                }}
            >
                <Ban className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400" />
                <span>
                    {account.disabled
                        ? t('accounts.enable_account', 'Enable Account')
                        : t('accounts.disable_account', 'Disable Account')}
                </span>
            </button>

            {/* Refresh Quota */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors hover:bg-slate-100 dark:hover:bg-[#251b3a]"
                onClick={(e) => {
                    e.stopPropagation();
                    onRefresh();
                    onClose();
                }}
            >
                <RefreshCw className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400" />
                <span>{t('accounts.refresh_quota', 'Refresh Quota')}</span>
            </button>

            {/* Separator */}
            <div className="my-1 border-t border-slate-200 dark:border-[#40285e]" />

            {/* Delete Account */}
            <button
                type="button"
                className="flex w-full items-center gap-2.5 px-3 py-1.5 text-left transition-colors text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30"
                onClick={(e) => {
                    e.stopPropagation();
                    onDelete();
                    onClose();
                }}
            >
                <Trash2 className="w-3.5 h-3.5" />
                <span>{t('common.delete', 'Delete Account')}</span>
            </button>
        </div>,
        document.body
    );
}
