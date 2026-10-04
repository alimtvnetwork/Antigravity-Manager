import { useState, useRef, useEffect } from 'react';
import { ArrowRightLeft, RefreshCw, Trash2, Download, Info, Lock, Ban, Diamond, Gem, Circle, ToggleLeft, ToggleRight, Fingerprint } from 'lucide-react';
import { Account } from '../../types/account';
import { cn } from '../../utils/cn';
import { useTranslation } from 'react-i18next';
import { findQuotaModel } from '../../config/modelConfig';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { formatDateTime, formatDateOnly } from '../../utils/date';
import { QuotaProgressBar } from './QuotaProgressBar';
import { Gemini } from '@lobehub/icons';


interface AccountRowProps {
    account: Account;
    selected: boolean;
    onSelect: () => void;
    isCurrent: boolean;
    isRefreshing: boolean;
    isSwitching?: boolean;
    isFocused?: boolean;
    onSwitch: (target?: string) => void;
    onRefresh: () => void;
    onViewDevice: () => void;
    onViewDetails: () => void;
    onExport: () => void;
    onDelete: () => void;
    onToggleProxy: () => void;
}




function AccountRow({ account, selected, onSelect, isCurrent, isRefreshing, isSwitching = false, isFocused = false, onSwitch, onRefresh, onViewDetails, onExport, onDelete, onToggleProxy, onViewDevice }: AccountRowProps) {
    const { t } = useTranslation();
    const [showInstanceMenu, setShowInstanceMenu] = useState(false);
    const { instances, activeInstanceId } = useInstanceStore();
    const menuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                setShowInstanceMenu(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    // 按优先级查找配额模型
    const geminiProModel = findQuotaModel(account.quota?.models, 'gemini-pro');
    const geminiFlashModel = findQuotaModel(account.quota?.models, 'gemini-flash');
    const isDisabled = Boolean(account.disabled);

    const rowRef = useRef<HTMLTableRowElement | null>(null);

    useEffect(() => {
        if (isFocused && rowRef.current) {
            rowRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
    }, [isFocused]);

    return (
        <tr
            id={`account-row-${account.id}`}
            ref={rowRef}
            className={cn(
            "group transition-all duration-200 border-b border-slate-200/80 dark:border-slate-800/80 border-l-2",
            isFocused
                ? "bg-teal-50/90 dark:bg-[#0e2c44] text-slate-900 dark:text-cyan-300 font-bold border-l-cyan-500 dark:border-l-cyan-400 border-slate-200/80 dark:border-slate-800/80 shadow-md ring-1 ring-cyan-500/30"
                : isCurrent
                ? "bg-blue-50/70 dark:bg-[#091b2c] border-l-blue-600 dark:border-l-amber-400 border-blue-200 dark:border-amber-400/40 font-semibold text-blue-900 dark:text-amber-300 shadow-xs ring-1 ring-blue-400/30 dark:ring-amber-400/30 hover:bg-blue-100/60 dark:hover:bg-[#0c2438]"
                : selected
                ? "bg-blue-50/90 dark:bg-[#0f273d] text-blue-950 dark:text-blue-100 border-l-blue-500 dark:border-l-blue-500 font-semibold shadow-xs ring-1 ring-blue-500/30"
                : "border-l-transparent text-gray-800 dark:text-gray-200 hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60 hover:text-slate-900 dark:hover:text-white hover:border-l-blue-500/70",
            (isRefreshing || isDisabled) && "opacity-70"
        )}>
            {/* 序号 */}
            <td className="pl-6 py-1 w-12">
                <input
                    type="checkbox"
                    className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                    checked={selected}
                    onChange={() => onSelect()}
                    onClick={(e) => e.stopPropagation()}
                />
            </td>

            {/* 邮箱 */}
            <td className="px-4 py-1">
                <div className="flex items-center gap-3">
                    <span className={cn(
                        "font-medium text-sm truncate max-w-[180px] xl:max-w-none transition-colors",
                        isFocused || selected
                            ? "text-blue-950 dark:text-blue-200 font-bold"
                            : isCurrent
                            ? "text-blue-900 dark:text-amber-300 font-bold"
                            : "text-gray-900 dark:text-gray-100 group-hover:text-blue-600 dark:group-hover:text-blue-400"
                    )} title={account.email}>
                        {account.email}
                    </span>

                        {isCurrent && (
                            <span className="px-1.5 py-0.5 rounded-[5px] bg-blue-100 dark:bg-amber-400/15 text-blue-800 dark:text-amber-300 border border-blue-200 dark:border-amber-400/30 text-[9px] font-semibold shadow-xs">
                                {t('accounts.current').toUpperCase()}
                            </span>
                        )}

                        {isDisabled && (
                            <span
                                className="px-1.5 py-0.5 rounded-[5px] bg-rose-500/10 text-rose-600 dark:text-rose-400 border border-rose-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs"
                                title={account.disabled_reason || t('accounts.disabled_tooltip')}
                            >
                                <Ban className="w-2.5 h-2.5" />
                                <span>{t('accounts.disabled')}</span>
                            </span>
                        )}

                        {account.proxy_disabled && (
                            <span
                                className="px-1.5 py-0.5 rounded-[5px] bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs"
                                title={account.proxy_disabled_reason || t('accounts.proxy_disabled_tooltip')}
                            >
                                <Ban className="w-2.5 h-2.5" />
                                <span>{t('accounts.proxy_disabled')}</span>
                            </span>
                        )}

                        {account.quota?.is_forbidden && (
                            <span className="px-1.5 py-0.5 rounded-[5px] bg-rose-500/10 text-rose-600 dark:text-rose-400 border border-rose-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs" title={t('accounts.forbidden_tooltip')}>
                                <Lock className="w-2.5 h-2.5" />
                                <span>{t('accounts.forbidden')}</span>
                            </span>
                        )}

                        {/* 订阅类型徽章 */}
                        {account.quota?.subscription_tier && (() => {
                            const tier = account.quota.subscription_tier.toLowerCase();
                            if (tier.includes('ultra')) {
                                return (
                                    <span className="flex items-center gap-1 px-2 py-0.5 rounded-md bg-gradient-to-r from-purple-600 to-pink-600 text-white text-[10px] font-bold shadow-sm hover:opacity-90 transition-opacity cursor-default">
                                        <Gem className="w-2.5 h-2.5 fill-current" />
                                        ULTRA
                                    </span>
                                );
                            } else if (tier.includes('pro')) {
                                return (
                                    <span className="flex items-center gap-1 px-2 py-0.5 rounded-md bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-[10px] font-bold shadow-sm hover:opacity-90 transition-opacity cursor-default">
                                        <Diamond className="w-2.5 h-2.5 fill-current" />
                                        PRO
                                    </span>
                                );
                            } else {
                                return (
                                    <span className="flex items-center gap-1 px-2 py-0.5 rounded-md bg-gray-100 dark:bg-[#15334d] text-gray-600 dark:text-gray-400 text-[10px] font-bold shadow-sm border border-gray-200 dark:border-[#15334d] hover:bg-gray-200 transition-colors cursor-default">
                                        <Circle className="w-2.5 h-2.5" />
                                        FREE
                                    </span>
                                );
                            }
                        })()}
                    </div>
                </div>
            </td>

            {/* 4H 模型配额 */}
            <td className="px-2.5 py-1 align-middle min-w-[210px] w-1/2 bg-slate-50/40 dark:bg-slate-900/25 border-l border-slate-200/50 dark:border-slate-800/50">
                {account.quota?.is_forbidden ? (
                    <div className="flex items-center gap-1.5 text-[10px] text-rose-600 dark:text-rose-400 bg-rose-500/10 p-1 rounded-[5px] border border-rose-400/20">
                        <Ban className="w-3 h-3 shrink-0" />
                        <span className="truncate">{t('accounts.forbidden_msg')}</span>
                    </div>
                ) : (
                    <QuotaProgressBar
                        percentage={geminiProModel?.percentage ?? geminiFlashModel?.percentage ?? 0}
                        resetTime={geminiProModel?.reset_time ?? geminiFlashModel?.reset_time}
                        Icon={Gemini.Color}
                    />
                )}
            </td>

            {/* Weekly 配额 */}
            <td className="px-2.5 py-1 align-middle min-w-[210px] w-1/2 bg-slate-50/40 dark:bg-slate-900/25">
                {account.quota?.is_forbidden ? (
                    <span className="text-[10px] text-gray-400 italic">--</span>
                ) : (
                    <QuotaProgressBar
                        isWeekly
                        percentage={(() => {
                            const groups = account.quota?.quota_groups || [];
                            const group = groups.find((item) => {
                                const name = (item.display_name || '').toLowerCase();
                                return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
                            });
                            const bucket = (group?.buckets || []).find((item) =>
                                (item.window || '').toLowerCase().includes('week')
                                || (item.bucket_id || '').toLowerCase().includes('week')
                            );
                            return bucket ? Math.round((bucket.remaining_fraction || 0) * 100) : 0;
                        })()}
                        resetTime={(() => {
                            const groups = account.quota?.quota_groups || [];
                            const group = groups.find((item) => {
                                const name = (item.display_name || '').toLowerCase();
                                return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
                            });
                            const bucket = (group?.buckets || []).find((item) =>
                                (item.window || '').toLowerCase().includes('week')
                                || (item.bucket_id || '').toLowerCase().includes('week')
                            );
                            return bucket?.reset_time;
                        })()}
                        Icon={Gemini.Color}
                    />
                )}
            </td>

            {/* 最后使用 */}
            <td className="px-2 py-0.5 align-middle whitespace-nowrap w-[95px]">
                <span className="text-[10px] font-medium text-gray-600 dark:text-gray-400 font-mono" title={formatDateTime(account.last_used)}>
                    {formatDateOnly(account.last_used)}
                </span>
            </td>

            {/* 操作 */}
            <td className="px-4 py-1">
                <div className="flex items-center gap-0.5 opacity-60 group-hover:opacity-100 transition-opacity">
                    {/* 1. Refresh button (first) */}
                    <button
                        className={`p-1.5 text-gray-500 dark:text-gray-400 rounded-[5px] transition-all ${(isRefreshing || isDisabled) ? 'bg-slate-100 dark:bg-slate-800/50 text-slate-400 cursor-not-allowed' : 'hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
                        onClick={(e) => { e.stopPropagation(); onRefresh(); }}
                        title={isDisabled ? t('accounts.disabled_tooltip') : (isRefreshing ? t('common.refreshing') : t('common.refresh'))}
                        disabled={isRefreshing || isDisabled}
                    >
                        <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
                    </button>

                    {/* 2. 切换/实例选择按钮 (排在第二位) */}
                    <div className="relative inline-flex items-center" ref={menuRef}>
                        <button
                            className={`p-1.5 text-gray-500 dark:text-gray-400 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 cursor-not-allowed' : 'hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
                            onClick={(e) => { e.stopPropagation(); onSwitch(); }}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                setShowInstanceMenu(!showInstanceMenu);
                            }}
                            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : `${t('accounts.switch_to')} (Right-click to select instance)`)}
                            disabled={isSwitching || isDisabled}
                        >
                            <ArrowRightLeft className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
                        </button>

                        {showInstanceMenu && (
                            <div className="absolute top-full left-0 mt-1 w-52 rounded-xl shadow-xl bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 py-1.5 z-50 animate-in fade-in zoom-in-95">
                                <div className="px-3 py-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                                    Target Instance
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

                    {/* 3. 详情与其它操作 */}
                    <button
                        className="p-1.5 text-gray-500 dark:text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-900/30 rounded-[5px] transition-all"
                        onClick={(e) => { e.stopPropagation(); onViewDetails(); }}
                        title={t('common.details')}
                    >
                        <Info className="w-3.5 h-3.5" />
                    </button>
                    <button
                        className="p-1.5 text-gray-500 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 rounded-[5px] transition-all"
                        onClick={(e) => { e.stopPropagation(); onViewDevice(); }}
                        title={t('accounts.device_fingerprint')}
                    >
                        <Fingerprint className="w-3.5 h-3.5" />
                    </button>
                    <button
                        className="p-1.5 text-gray-500 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 rounded-[5px] transition-all"
                        onClick={(e) => { e.stopPropagation(); onExport(); }}
                        title={t('common.export')}
                    >
                        <Download className="w-3.5 h-3.5" />
                    </button>
                    <button
                        className={cn(
                            "p-1.5 rounded-[5px] transition-all",
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
                        className="p-1.5 text-gray-500 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-[5px] transition-all"
                        onClick={(e) => { e.stopPropagation(); onDelete(); }}
                        title={t('common.delete')}
                    >
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                </div>
            </td>
        </tr>
    );
}

export default AccountRow;
