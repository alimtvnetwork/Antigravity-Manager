import React from 'react';
import { useTranslation } from 'react-i18next';
import {
    Wand2,
    Zap,
    Code,
    Cpu,
    CodeXml,
    RefreshCw,
    CheckSquare,
    MinusSquare,
    Square
} from 'lucide-react';
import type { CliSyncApp, OpenCodeProfileInfo } from './types';
import { CLI_APP_CODEX, CLI_APP_CLAUDE } from './utils';

export interface SyncSectionProps {
    models: string[];
    apiKey: string;
    baseUrl: string;
    querying: boolean;
    syncingKey: string | null;
    handleSyncCli: (app: CliSyncApp) => Promise<void>;
    handleSyncOpenCode: () => Promise<void>;
    getModelsForKey: (targetKey: string, targetUrl?: string) => string[] | undefined;
    profileInfo: (key: string, url: string, modelIds?: string[]) => OpenCodeProfileInfo;
}

export const SyncSection: React.FC<SyncSectionProps> = ({
    models,
    apiKey,
    baseUrl,
    querying,
    syncingKey,
    handleSyncCli,
    handleSyncOpenCode,
    getModelsForKey,
    profileInfo,
}) => {
    const { t } = useTranslation();

    return (
        <div className="mt-4 flex flex-col sm:flex-row items-start sm:items-center justify-between p-3.5 rounded-xl bg-gradient-to-r from-blue-50/80 to-purple-50/80 dark:from-blue-900/20 dark:to-purple-900/20 border border-blue-500/20 shadow-sm relative overflow-hidden group/banner transition-all hover:shadow-md">
            {/* Subtle Background Effect */}
            <div className="absolute -right-6 -top-6 text-purple-500/10 dark:text-purple-400/5 rotate-12 transition-transform group-hover/banner:rotate-45 duration-700">
                <Wand2 size={80} />
            </div>

            <div className="flex items-center gap-3 z-10 mb-3 sm:mb-0">
                <div className="p-2 rounded-lg bg-base-100/80 shadow-sm border border-base-300 backdrop-blur-sm">
                    <Zap size={16} className="text-amber-500" />
                </div>
                <div className="flex flex-col">
                    <span className="text-sm font-bold text-base-content">
                        {t('apiKeyFun.cli.quickConfig', { defaultValue: '一键配置本地开发环境' })}
                    </span>
                    <span className="text-[10px] text-base-content/60 font-medium">
                        {t('apiKeyFun.cli.syncDesc', { defaultValue: '同步至官方标准配置' })}
                    </span>
                </div>
            </div>

            {(() => {
                const hasModels = models.length > 0;
                const hasGpt = models.some(m => {
                    const lower = m.toLowerCase();
                    return lower.includes('gpt') || lower.includes('o1') || lower.includes('o3') || lower.includes('deepseek') || lower.includes('qwen');
                });
                const hasClaude = models.some(m => m.toLowerCase().includes('claude'));

                // 默认都显示，除非明确检测到只支持其中一种
                const showCodex = !hasModels || hasGpt || (!hasGpt && !hasClaude);
                const showClaude = !hasModels || hasClaude || (!hasGpt && !hasClaude);

                return (
                    <div className="flex items-center gap-2 w-full sm:w-auto z-10 pl-11 sm:pl-0">
                        {showCodex && (
                            <button
                                onClick={() => handleSyncCli(CLI_APP_CODEX)}
                                className="flex-1 sm:flex-none btn btn-sm px-5 font-medium rounded-full bg-blue-500 hover:bg-blue-600 text-white border-none shadow-md shadow-blue-500/20 transition-all group"
                                disabled={!apiKey.trim()}
                                title={t('apiKeyFun.cli.codexTooltip', { defaultValue: 'Sync API Key & BaseURL to Codex / ChatGPT CLI' })}
                            >
                                <Code size={14} className="mr-1.5 opacity-90 group-hover:scale-110 group-hover:opacity-100 transition-all" />
                                Codex
                            </button>
                        )}
                        {showClaude && (
                            <button
                                onClick={() => handleSyncCli(CLI_APP_CLAUDE)}
                                className="flex-1 sm:flex-none btn btn-sm px-5 font-medium rounded-full bg-purple-500 hover:bg-purple-600 text-white border-none shadow-md shadow-purple-500/20 transition-all group"
                                disabled={!apiKey.trim()}
                                title={t('apiKeyFun.cli.claudeTooltip', { defaultValue: 'Sync API Key & BaseURL to Claude Code' })}
                            >
                                <Cpu size={14} className="mr-1.5 opacity-90 group-hover:scale-110 group-hover:opacity-100 transition-all" />
                                Claude
                            </button>
                        )}
                        {(() => {
                            const activeModels = getModelsForKey(apiKey, baseUrl);
                            const activeInfo = profileInfo(apiKey, baseUrl, activeModels);
                            const isCurrentKeySyncing = syncingKey === apiKey.trim();

                            return (
                                <div className="flex items-center gap-1.5 flex-1 sm:flex-none">
                                    <button
                                        onClick={handleSyncOpenCode}
                                        role="checkbox"
                                        aria-checked={activeInfo.status === 'synced' ? true : activeInfo.status === 'partial' ? 'mixed' : false}
                                        aria-label={`OpenCode ${activeInfo.suffix}`}
                                        className={`btn btn-sm px-4 font-medium rounded-full border-none shadow-md transition-all group flex items-center gap-2 ${
                                            activeInfo.status === 'synced'
                                                ? 'bg-teal-600 hover:bg-teal-700 text-white shadow-teal-600/20'
                                                : activeInfo.status === 'partial'
                                                    ? 'bg-amber-500 hover:bg-amber-600 text-white shadow-amber-500/20'
                                                    : 'bg-teal-500 hover:bg-teal-600 text-white shadow-teal-500/20'
                                        }`}
                                        disabled={!apiKey.trim() || !baseUrl.trim() || querying || Boolean(syncingKey)}
                                        title={
                                            activeInfo.status === 'synced'
                                                ? t('apiKeyFun.opencode.syncedActiveTooltip', { defaultValue: 'OpenCode: profile is active and fully synced. Click to deactivate' })
                                                : activeInfo.status === 'partial'
                                                    ? t('apiKeyFun.opencode.partialTooltip', { defaultValue: 'OpenCode: settings or models differ. Click to sync' })
                                                    : t('apiKeyFun.cli.opencodeTooltip', { defaultValue: 'Activate as separate profile in OpenCode' })
                                        }
                                    >
                                        {isCurrentKeySyncing ? (
                                            <RefreshCw size={13} className="animate-spin mr-0.5" />
                                        ) : activeInfo.status === 'synced' ? (
                                            <CheckSquare size={14} className="text-white shrink-0" />
                                        ) : activeInfo.status === 'partial' ? (
                                            <MinusSquare size={14} className="text-white shrink-0" />
                                        ) : (
                                            <Square size={14} className="text-white/70 shrink-0" />
                                        )}
                                        <CodeXml size={14} className="opacity-90 group-hover:scale-110 group-hover:opacity-100 transition-all" />
                                        <span>OpenCode</span>
                                        {activeInfo.suffix && (
                                            <span className="text-[10px] opacity-85 font-mono">
                                                ({activeInfo.suffix})
                                            </span>
                                        )}
                                    </button>
                                </div>
                            );
                        })()}
                    </div>
                );
            })()}
        </div>
    );
};
