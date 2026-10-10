import { createPortal } from 'react-dom';
import {
    ArrowRightLeft,
    RefreshCw,
    Trash2,
    Download,
    Fingerprint,
    Info,
    Lock,
    Ban,
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
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { Account } from '../../../types/account';
import type { AccountRowContentProps } from './types';
import type { AccountRowState } from './useAccountRowState';

interface ActionsCellProps extends Pick<AccountRowContentProps,
    'account' | 'selected' | 'isCurrent' | 'isFocused' | 'isRefreshing' | 'isSwitching' | 'isDisabled' |
    'onSwitch' | 'onRefresh' | 'onViewDevice' | 'onViewDetails' | 'onExport' | 'onDelete' | 'onToggleProxy' |
    'onWarmup' | 'onUpdateLabel' | 'onUpdatePriority' | 'onViewError'
> {
    rowState: AccountRowState;
}

/**
 * Actions column: refresh, switch, details, menus, export, delete, proxy toggle.
 * Extracted from AccountRowContent to keep it under the 500-line limit.
 */
export function ActionsCell(props: ActionsCellProps) {
    const { account, selected, isCurrent, isFocused, isRefreshing, isSwitching, isDisabled } = props;
    const { onSwitch, onRefresh, onViewDevice, onViewDetails, onExport, onDelete, onToggleProxy, onWarmup, onUpdateLabel, onUpdatePriority, onViewError } = props;
    const {
        t,
        isEditingLabel, setIsEditingLabel,
        labelInput, setLabelInput,
        showInstanceMenu, setShowInstanceMenu,
        showMoreMenu, setShowMoreMenu,
        moreMenuPos,
        editingPriority, setEditingPriority,
        instances,
        menuRef, moreBtnRef, moreMenuRef,
        boundInstance,
        handleSaveLabel, handleCancelLabel, handleKeyDown,
        openMoreMenu,
    } = props.rowState;

    return (
<td className={cn(
    "px-2 py-0.5 sticky right-0 z-10 w-[220px] xl:w-[280px] shadow-[-12px_0_12px_-12px_rgba(0,0,0,0.1)] dark:shadow-[-12px_0_12px_-12px_rgba(255,255,255,0.05)] text-center align-middle transition-colors border-b border-slate-200/90 dark:border-slate-800/90",
    // 动态高对比高亮处理
    isFocused
        ? "bg-teal-50/90 dark:bg-[#0e2c44]"
        : selected
        ? "bg-blue-50/90 dark:bg-[#0f273d]"
        : isCurrent
        ? "bg-blue-50/70 dark:bg-[#091b2c]"
        : "bg-white dark:bg-[#081826]",
    !isCurrent && !selected && !isFocused ? "group-hover:bg-slate-50/80 dark:group-hover:bg-[#0f273d]/60" : ""
)}>
    <div className="flex items-center justify-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
        <button
            type="button"
            className={`p-1 text-gray-500 dark:text-gray-400 rounded-[5px] transition-all ${(isRefreshing || isDisabled) ? 'cursor-not-allowed' : 'hover:text-blue-600 dark:hover:text-cyan-400 hover:bg-slate-100 dark:hover:bg-slate-800'}`}
            onClick={(event) => { event.stopPropagation(); onRefresh(); }}
            title={t('common.refresh')}
            disabled={isRefreshing || isDisabled}
        >
            <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
        </button>
        <div className="relative inline-flex items-center" ref={menuRef}>
            <button
                className={`p-1 text-gray-500 dark:text-gray-400 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 cursor-not-allowed' : 'hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
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
                                <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${inst.is_running ? 'bg-teal-500' : 'bg-gray-400'}`} />
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
            ref={moreBtnRef}
            type="button"
            className="p-1 text-gray-500 dark:text-gray-400 hover:text-slate-950 dark:hover:text-slate-100 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-[5px] transition-all"
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
                className="fixed z-[10000] min-w-[168px] rounded-[5px] border border-gray-200 bg-white py-1 text-slate-950 shadow-xl"
                style={{ top: moreMenuPos.top, left: moreMenuPos.left, transform: 'translateX(-100%)' }}
                onClick={(event) => event.stopPropagation()}
            >
                <button
                    type="button"
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    disabled={isSwitching || isDisabled}
                    onClick={() => { setShowMoreMenu(false); onSwitch('ide'); }}
                >
                    <Repeat2 className="w-3.5 h-3.5" />
                    {t('accounts.switch_to_ide', 'IDE switch')}
                </button>
                <button
                    type="button"
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    disabled={isSwitching || isDisabled}
                    onClick={() => { setShowMoreMenu(false); onSwitch('agy'); }}
                >
                    <Terminal className="w-3.5 h-3.5" />
                    {t('accounts.switch_to_agy', 'CLI switch')}
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
                <button
                    type="button"
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    onClick={() => setShowEmail((current) => !current)}
                >
                    {showEmail ? t('accounts.hide_email') : t('accounts.show_email')}
                </button>
                <div className="px-3 py-1.5 text-xs text-slate-600">
                    Cycle tokens: {weeklyCell.weeklyTokens ?? 0}
                </div>
            </div>,
            document.body,
        )}
        {onUpdateLabel && (
            <button
                className={cn(
                    "hidden lg:inline-flex p-1 rounded-[5px] transition-all",
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
                className={`hidden xl:inline-flex p-1 text-gray-500 dark:text-gray-400 rounded-[5px] transition-all ${(isRefreshing || isDisabled) ? 'bg-orange-50 dark:bg-orange-900/10 text-orange-600 dark:text-orange-400 cursor-not-allowed' : 'hover:text-orange-500 dark:hover:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/30'}`}
                onClick={(e) => { e.stopPropagation(); onWarmup(); }}
                title={isDisabled ? t('accounts.disabled_tooltip') : (isRefreshing ? t('common.loading') : t('accounts.warmup_this', 'Warmup Account'))}
                disabled={isRefreshing || isDisabled}
            >
                <Sparkles className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-pulse' : ''}`} />
            </button>
        )}
        <button
            className={cn(
                "p-1 rounded-[5px] transition-all",
                account.proxy_disabled
                    ? "text-gray-500 dark:text-gray-400 hover:text-cyan-600 dark:hover:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/30"
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
            className="p-1 text-gray-500 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-[5px] transition-all"
            onClick={(e) => { e.stopPropagation(); onDelete(); }}
            title={t('common.delete')}
        >
            <Trash2 className="w-3.5 h-3.5" />
        </button>
    </div>
</td>
    );
}
