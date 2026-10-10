import type { ReactNode } from 'react';
import type { TFunction } from 'i18next';
import {
    Check,
    AlertCircle,
    RefreshCw,
    Eye,
    RotateCcw,
    Trash2,
    Sparkles,
    Loader2,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { copyToClipboard } from '../../../utils/clipboard';
import { showToast } from '../../common/ToastContainer';
import GroupedSelect from '../../common/GroupedSelect';
import { Github as LobeGithub } from '@lobehub/icons';
import { JeikCodeIcon } from '../../common/JeikCodeIcon';
import type { CliAppType, CliStatus, ViewingConfig } from './types';

export interface CliAppCardProps {
    t: TFunction;
    app: CliAppType;
    icon: ReactNode;
    name: string;
    status: CliStatus | null;
    isAppLoading: boolean;
    isAppSyncing: boolean;
    selectedModel: string;
    onSelectModel: (value: string) => void;
    modelOptions: { value: string; label: string; group: string }[];
    syncAccounts: boolean;
    onToggleSyncAccounts: (v: boolean) => void;
    onSync: () => void;
    onRestore: () => void;
    onClear: () => void;
    onViewConfig: () => void;
    onOpenExternalUrl: (url: string) => void;
}

const MODEL_SELECT_APPS: CliAppType[] = ['Claude', 'Codex', 'Gemini', 'JeikCode', 'GrokBuild'];
const ALWAYS_VISIBLE_APPS: CliAppType[] = ['OpenCode', 'JeikCode', 'GrokBuild', 'Hermes', 'OpenClaw'];
const CLEARABLE_APPS: CliAppType[] = ['OpenCode', 'Hermes', 'OpenClaw'];
const VIEW_HIDDEN_UNLESS_SYNCED: CliAppType[] = ['OpenCode', 'Hermes', 'OpenClaw'];

export function CliAppCard({
    t,
    app,
    icon,
    name,
    status,
    isAppLoading,
    isAppSyncing,
    selectedModel,
    onSelectModel,
    modelOptions,
    syncAccounts,
    onToggleSyncAccounts,
    onSync,
    onRestore,
    onClear,
    onViewConfig,
    onOpenExternalUrl,
}: CliAppCardProps) {
    const showActions = status?.installed || ALWAYS_VISIBLE_APPS.includes(app);
    const showModelSelect =
        (status?.installed || app === 'OpenCode' || app === 'JeikCode' || app === 'GrokBuild') &&
        MODEL_SELECT_APPS.includes(app);
    const showSyncBadge =
        !isAppLoading && (status?.installed || (ALWAYS_VISIBLE_APPS.includes(app) && status?.current_base_url));
    const viewTitleKey =
        app === 'OpenCode'
            ? 'proxy.opencode_sync.btn_view'
            : app === 'Hermes'
              ? 'proxy.hermes_sync.btn_view'
              : app === 'OpenClaw'
                ? 'proxy.openclaw_sync.btn_view'
                : 'proxy.cli_sync.btn_view';
    const restoreTitleKey =
        app === 'OpenCode'
            ? 'proxy.opencode_sync.btn_restore'
            : app === 'Hermes'
              ? 'proxy.hermes_sync.btn_restore'
              : app === 'OpenClaw'
                ? 'proxy.openclaw_sync.btn_restore'
                : 'proxy.cli_sync.btn_restore';
    const clearTitleKey =
        app === 'Hermes'
            ? 'proxy.hermes_sync.btn_clear'
            : app === 'OpenClaw'
              ? 'proxy.openclaw_sync.btn_clear'
              : 'proxy.opencode_sync.btn_clear';
    const canSync =
        (app === 'OpenCode' || app === 'JeikCode' || app === 'GrokBuild' || app === 'Hermes' || app === 'OpenClaw' || status?.installed) &&
        !isAppSyncing &&
        !isAppLoading;

    return (
        <div className={cn(
            "flex flex-col rounded-xl border p-4 shadow-sm transition-all duration-300 group",
            app === 'JeikCode'
                ? "bg-gradient-to-b from-emerald-500/[0.04] via-white/80 to-white/60 dark:from-emerald-950/20 dark:via-gray-800/60 dark:to-gray-800/40 border-emerald-500/30 dark:border-emerald-500/40 hover:border-emerald-400 dark:hover:border-emerald-400 shadow-emerald-500/5 hover:shadow-emerald-500/15 ring-1 ring-emerald-500/10"
                : "bg-white/50 dark:bg-gray-800/40 border-gray-100 dark:border-white/5 hover:shadow-lg hover:border-blue-200/50 dark:hover:border-blue-500/30"
        )}>
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-y-3 gap-x-2 mb-3">
                <div className="flex items-center gap-3 min-w-0">
                    <div className={cn(
                        "w-10 h-10 shrink-0 flex items-center justify-center transition-transform duration-300 group-hover:scale-105 p-0 rounded-xl overflow-hidden shadow-sm border border-gray-200/60 dark:border-white/10",
                        app === 'JeikCode' && "shadow-md shadow-emerald-500/10 border-emerald-500/30 dark:border-emerald-500/40"
                    )}>
                        {app === 'JeikCode' ? (
                            <JeikCodeIcon size="100%" className="w-full h-full" />
                        ) : (
                            icon
                        )}
                    </div>
                    <div className="min-w-0 flex-1">
                        <h4 className="text-sm font-bold text-gray-900 dark:text-gray-100 leading-tight whitespace-nowrap">
                            {name}
                        </h4>
                        <div className="mt-1 flex items-center gap-1.5 flex-wrap">
                            {isAppLoading ? (
                                <div className="flex items-center gap-1 text-[10px] text-gray-400">
                                    <Loader2 size={10} className="animate-spin" />
                                    {t('proxy.cli_sync.status.detecting')}
                                </div>
                            ) : status?.installed ? (
                                <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-bold whitespace-nowrap">
                                    {t('proxy.cli_sync.status.installed', { version: status.version })}
                                </span>
                            ) : (
                                <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-400 font-medium whitespace-nowrap">
                                    {t('proxy.cli_sync.status.not_installed')}
                                </span>
                            )}
                            {app === 'JeikCode' && (
                                <span className="inline-flex items-center gap-1 text-[9px] px-1.5 py-0.5 rounded-full bg-gradient-to-r from-emerald-500/20 to-teal-500/20 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 font-bold shrink-0 shadow-2xs">
                                    <Sparkles size={10} className="text-emerald-500 animate-spin-once" />
                                    {t('proxy.cli_sync.jeikcode_recommended', { defaultValue: '推荐使用' })}
                                </span>
                            )}
                        </div>
                    </div>
                </div>

                {/* Show Sync Status if installed OR if it's OpenCode/JeikCode/GrokBuild/Hermes/OpenClaw */}
                {showSyncBadge && (
                    <div
                        className={cn(
                            "inline-flex items-center justify-center transition-all shrink-0 whitespace-nowrap shadow-sm",
                            status!.is_synced
                                ? "w-6 h-6 rounded-full bg-gradient-to-tr from-green-500 to-emerald-600 text-white shadow-emerald-500/20"
                                : "w-6 h-6 rounded-full bg-amber-100 dark:bg-amber-900/40 text-amber-600 dark:text-amber-500 border border-amber-200/60 dark:border-amber-800/40"
                        )}
                        title={status!.is_synced ? t('proxy.cli_sync.status.synced', { defaultValue: '已指向本项目' }) : t('proxy.cli_sync.status.not_synced', { defaultValue: '未同步' })}
                    >
                        {status!.is_synced ? (
                            <Check size={14} className="stroke-[3]" />
                        ) : (
                            <AlertCircle size={13} />
                        )}
                    </div>
                )}
            </div>

            {/* JeikCode 专属官方项目推荐卡片（贴满方格，充实饱满，点击跳转 GitHub） */}
            {app === 'JeikCode' && (
                <div
                    onClick={() => onOpenExternalUrl('https://github.com/jeikl/JeikCode')}
                    className="mb-3 p-3 rounded-xl border border-emerald-500/30 bg-gradient-to-br from-emerald-500/[0.09] via-teal-500/[0.05] to-cyan-500/[0.08] dark:from-emerald-950/40 dark:via-teal-950/25 dark:to-cyan-950/30 hover:border-emerald-400 hover:shadow-md transition-all duration-300 cursor-pointer group/link select-none w-full"
                    title="点击访问 JeikCode 官方 GitHub 仓库 (https://github.com/jeikl/JeikCode)"
                >
                    {/* 顶栏：Logo + 标题与作者 + 右侧 GitHub 放大图标 */}
                    <div className="flex items-center justify-between gap-2 mb-1.5">
                        <div className="flex items-center gap-1.5 min-w-0">
                            <JeikCodeIcon size={16} />
                            <div className="flex items-baseline gap-1 min-w-0">
                                <span className="text-[11px] font-black text-emerald-900 dark:text-emerald-200 tracking-tight whitespace-nowrap">
                                    {t('proxy.cli_sync.jeikcode_badge', { defaultValue: 'Best Matched' })}
                                </span>
                                <span className="text-[9px] text-emerald-700/80 dark:text-emerald-400/80 font-semibold whitespace-nowrap">
                                    · {t('proxy.cli_sync.jeikcode_author', { defaultValue: 'by Jeikl' })}
                                </span>
                            </div>
                        </div>
                        <span className="inline-flex items-center text-gray-700 dark:text-gray-300 group-hover/link:text-emerald-500 group-hover/link:scale-110 transition-all shrink-0">
                            <LobeGithub size={16} />
                        </span>
                    </div>

                    {/* 核心标语（独立一行，排版整洁，中英文均不截断） */}
                    <p className="text-[9.5px] text-emerald-800 dark:text-emerald-300 font-medium leading-tight mb-2">
                        {t('proxy.cli_sync.jeikcode_tagline', { defaultValue: '核心维护者深度打造 · 原生网关最佳兼容' })}
                    </p>

                    {/* 关键词高亮标签群（精炼短语，自适应折行，丰满耐看） */}
                    <div className="flex flex-wrap items-center gap-1.5">
                        <span className="inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-md bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30 shadow-2xs whitespace-nowrap">
                            ⚡ {t('proxy.cli_sync.jeikcode_tag_cache', { defaultValue: '95%+ 恐怖缓存' })}
                        </span>
                        <span className="inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-md bg-blue-500/15 text-blue-700 dark:text-blue-300 border border-blue-500/30 shadow-2xs whitespace-nowrap">
                            🗺️ {t('proxy.cli_sync.jeikcode_tag_codegraph', { defaultValue: '代码图谱' })}
                        </span>
                        <span className="inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-md bg-purple-500/15 text-purple-700 dark:text-purple-300 border border-purple-500/30 shadow-2xs whitespace-nowrap">
                            🚀 {t('proxy.cli_sync.jeikcode_tag_surpass', { defaultValue: '超越 CC / Codex' })}
                        </span>
                        <span className="inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-md bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs whitespace-nowrap">
                            🛠️ {t('proxy.cli_sync.jeikcode_tag_custom', { defaultValue: '自由定制' })}
                        </span>
                    </div>
                </div>
            )}

            <div className="mt-auto space-y-3">
                <div className="p-2.5 bg-gray-50/80 dark:bg-gray-900/40 rounded-lg border border-dashed border-gray-200 dark:border-white/10">
                    <div className="flex justify-between items-start mb-1">
                        <div className="text-[9px] text-gray-400 dark:text-gray-500 uppercase font-bold tracking-wider">{t('proxy.cli_sync.status.current_base_url')}</div>
                    </div>
                    <div className="text-[10px] font-mono truncate text-gray-500 dark:text-gray-400 italic">
                        {status?.current_base_url || '---'}
                    </div>
                </div>

                {/* Claude, Codex, Gemini, JeikCode, GrokBuild model selection */}
                {showModelSelect && (
                    <div className="space-y-1">
                        <div className="text-[9px] text-gray-400 dark:text-gray-500 uppercase font-bold tracking-wider px-1">
                            {t('proxy.cli_sync.model_select', { defaultValue: 'Select Model' })}
                        </div>
                        <GroupedSelect
                            value={selectedModel}
                            onChange={onSelectModel}
                            options={modelOptions}
                            className="w-full !h-8 !text-[11px] !rounded-lg"
                            allowCustomInput={true}
                        />
                    </div>
                )}

                {/* OpenCode-specific account sync option - Allow even if not installed */}
                {app === 'OpenCode' && (
                    <div className="flex items-center gap-2 p-2 bg-gray-50/50 dark:bg-gray-900/20 rounded-lg">
                        <input
                            type="checkbox"
                            id="opencode-sync-accounts"
                            checked={syncAccounts}
                            onChange={(e) => onToggleSyncAccounts(e.target.checked)}
                            className="checkbox checkbox-xs checkbox-primary"
                        />
                        <label htmlFor="opencode-sync-accounts" className="text-[10px] text-gray-600 dark:text-gray-400 cursor-pointer select-none">
                            {t('proxy.opencode_sync.sync_accounts', { defaultValue: 'Sync accounts to antigravity-accounts.json' })}
                        </label>
                    </div>
                )}

                <div className="flex items-center gap-2">
                    {showActions && (
                        <>
                            {/* For OpenCode, Hermes, and OpenClaw, if not synced, hide view button */}
                            {(!VIEW_HIDDEN_UNLESS_SYNCED.includes(app) || status?.is_synced) && (
                                <button
                                    onClick={onViewConfig}
                                    className="p-1 text-gray-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-colors"
                                    title={t(viewTitleKey, { defaultValue: 'View Config' })}
                                >
                                    <Eye size={14} />
                                </button>
                            )}
                            <button
                                onClick={onRestore}
                                className="p-1 text-gray-400 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/20 rounded transition-colors"
                                title={t(restoreTitleKey, { defaultValue: 'Restore' })}
                            >
                                <RotateCcw size={14} />
                            </button>
                            {/* OpenCode, Hermes, and OpenClaw Clear button */}
                            {CLEARABLE_APPS.includes(app) && (
                                <button
                                    onClick={onClear}
                                    className="p-1 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                                    title={t(clearTitleKey, { defaultValue: 'Clear' })}
                                >
                                    <Trash2 size={14} />
                                </button>
                            )}
                        </>
                    )}
                    <button
                        onClick={onSync}
                        disabled={!canSync}
                        className={cn(
                            "btn btn-sm flex-1 gap-2 rounded-xl transition-all font-bold shadow-sm",
                            status?.is_synced
                                ? "btn-ghost border-gray-200 dark:border-base-400 text-gray-500 hover:bg-gray-100"
                                : "btn-primary hover:shadow-lg shadow-blue-500/20"
                        )}
                    >
                        {isAppSyncing ? (
                            <Loader2 size={14} className="animate-spin" />
                        ) : (
                            <RefreshCw size={14} className={cn(isAppLoading && "animate-spin-once")} />
                        )}
                        {t('proxy.cli_sync.btn_sync')}
                    </button>
                </div>
            </div>
        </div>
    );
}

export async function copyViewingConfigContent(t: TFunction, vc: ViewingConfig): Promise<void> {
    const success = await copyToClipboard(vc.content);
    if (success) {
        showToast(t('proxy.cli_sync.modal.copy_success'), 'success');
    }
}
