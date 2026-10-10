import type { TFunction } from 'i18next';
import { CheckCircle, Copy } from 'lucide-react';
import { isTauri } from '../../utils/env';
import { cn } from '../../utils/cn';
import type { AppConfig } from '../../types/config';
import type { CloudflaredControls, SaveConfigFn } from './types';
import { CollapsibleCard } from './CollapsibleCard';

interface CloudflaredSectionProps {
    config: AppConfig;
    t: TFunction;
    saveConfig: SaveConfigFn;
    copied: string | null;
    cloudflared: CloudflaredControls;
}

function CloudflareIcon() {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-orange-500"><path d="M12 2L2 7l10 5 10-5-10-5z" /><path d="M2 17l10 5 10-5" /><path d="M2 12l10 5 10-5" /></svg>
    );
}

export function CloudflaredSection({
    config,
    t,
    saveConfig,
    copied,
    cloudflared,
}: CloudflaredSectionProps) {
    const {
        cfStatus,
        cfLoading,
        cfMode,
        cfToken,
        cfUseHttp2,
        setCfMode,
        setCfToken,
        setCfUseHttp2,
        handleCfInstall,
        handleCfToggle,
        handleCfCopyUrl,
    } = cloudflared;

    // 公网访问 (Cloudflared) - 仅在桌面端显示
    if (!isTauri()) return null;

    return (
        <CollapsibleCard
            title={t('proxy.cloudflared.title', { defaultValue: 'Public Access (Cloudflared)' })}
            icon={<CloudflareIcon />}
            enabled={cfStatus.running}
            onToggle={handleCfToggle}
            allowInteractionWhenDisabled={true}
            rightElement={
                cfLoading ? (
                    <span className="loading loading-spinner loading-xs"></span>
                ) : cfStatus.running && cfStatus.url ? (
                    <button
                        onClick={(e) => { e.stopPropagation(); handleCfCopyUrl(); }}
                        className="text-xs px-2 py-1 rounded bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 hover:bg-green-200 dark:hover:bg-green-900/50 transition-colors flex items-center gap-1"
                    >
                        {copied === 'cf-url' ? <CheckCircle size={12} /> : <Copy size={12} />}
                        {cfStatus.url.replace('https://', '').slice(0, 20)}...
                    </button>
                ) : null
            }
        >
            <div className="space-y-4">
                {/* 安装状态 */}
                {!cfStatus.installed ? (
                    <div className="flex items-center justify-between p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-xl border border-yellow-200 dark:border-yellow-800">
                        <div className="space-y-1">
                            <span className="text-sm font-bold text-yellow-800 dark:text-yellow-200">
                                {t('proxy.cloudflared.not_installed', { defaultValue: 'Cloudflared not installed' })}
                            </span>
                            <p className="text-xs text-yellow-600 dark:text-yellow-400">
                                {t('proxy.cloudflared.install_hint', { defaultValue: 'Click to download and install cloudflared binary' })}
                            </p>
                        </div>
                        <button
                            onClick={handleCfInstall}
                            disabled={cfLoading}
                            className="px-4 py-2 rounded-lg text-sm font-medium bg-yellow-500 text-white hover:bg-yellow-600 disabled:opacity-50 flex items-center gap-2"
                        >
                            {cfLoading ? <span className="loading loading-spinner loading-xs"></span> : null}
                            {t('proxy.cloudflared.install', { defaultValue: 'Install' })}
                        </button>
                    </div>
                ) : (
                    <>
                        {/* 版本信息 */}
                        <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                            <CheckCircle size={14} className="text-green-500" />
                            {t('proxy.cloudflared.installed', { defaultValue: 'Installed' })}: {cfStatus.version || 'Unknown'}
                        </div>

                        {/* 隧道模式选择 */}
                        <div className="grid grid-cols-2 gap-3">
                            <button
                                onClick={() => {
                                    setCfMode('quick');
                                    saveConfig({
                                        ...config,
                                        cloudflared: { ...config.cloudflared, mode: 'quick' }
                                    });
                                }}
                                disabled={cfStatus.running}
                                className={cn(
                                    "p-3 rounded-lg border-2 text-left transition-all",
                                    cfMode === 'quick'
                                        ? "border-orange-500 bg-orange-50 dark:bg-orange-900/20"
                                        : "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600",
                                    cfStatus.running && "opacity-60 cursor-not-allowed"
                                )}
                            >
                                <div className="text-sm font-bold text-gray-900 dark:text-base-content">
                                    {t('proxy.cloudflared.mode_quick', { defaultValue: 'Quick Tunnel' })}
                                </div>
                                <p className="text-[10px] text-gray-500 dark:text-gray-400 mt-1">
                                    {t('proxy.cloudflared.mode_quick_desc', { defaultValue: 'Auto-generated temporary URL (*.trycloudflare.com)' })}
                                </p>
                            </button>
                            <button
                                onClick={() => {
                                    setCfMode('auth');
                                    saveConfig({
                                        ...config,
                                        cloudflared: { ...config.cloudflared, mode: 'auth' }
                                    });
                                }}
                                disabled={cfStatus.running}
                                className={cn(
                                    "p-3 rounded-lg border-2 text-left transition-all",
                                    cfMode === 'auth'
                                        ? "border-orange-500 bg-orange-50 dark:bg-orange-900/20"
                                        : "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600",
                                    cfStatus.running && "opacity-60 cursor-not-allowed"
                                )}
                            >
                                <div className="text-sm font-bold text-gray-900 dark:text-base-content">
                                    {t('proxy.cloudflared.mode_auth', { defaultValue: 'Named Tunnel' })}
                                </div>
                                <p className="text-[10px] text-gray-500 dark:text-gray-400 mt-1">
                                    {t('proxy.cloudflared.mode_auth_desc', { defaultValue: 'Use your Cloudflare account with custom domain' })}
                                </p>
                            </button>
                        </div>

                        {/* Token输入 (仅auth模式) */}
                        {cfMode === 'auth' && (
                            <div className="space-y-2">
                                <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    {t('proxy.cloudflared.token', { defaultValue: 'Tunnel Token' })}
                                </label>
                                <input
                                    type="password"
                                    value={cfToken}
                                    onChange={(e) => setCfToken(e.target.value)}
                                    onBlur={() => {
                                        saveConfig({
                                            ...config,
                                            cloudflared: { ...config.cloudflared, token: cfToken }
                                        });
                                    }}
                                    disabled={cfStatus.running}
                                    placeholder="eyJhIjoiNj..."
                                    className="w-full px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-base-200 text-sm font-mono disabled:opacity-60"
                                />
                            </div>
                        )}

                        {/* HTTP2选项 */}
                        <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-base-200 rounded-lg">
                            <div className="space-y-0.5">
                                <span className="text-sm font-medium text-gray-900 dark:text-base-content">
                                    {t('proxy.cloudflared.use_http2', { defaultValue: 'Use HTTP/2' })}
                                </span>
                                <p className="text-[10px] text-gray-500 dark:text-gray-400">
                                    {t('proxy.cloudflared.use_http2_desc', { defaultValue: 'More compatible, recommended for China mainland' })}
                                </p>
                            </div>
                            <input
                                type="checkbox"
                                className="toggle toggle-sm"
                                checked={cfUseHttp2}
                                onChange={(e) => {
                                    const val = e.target.checked;
                                    setCfUseHttp2(val);
                                    const newConfig = {
                                        ...config,
                                        cloudflared: {
                                            ...config.cloudflared,
                                            use_http2: val
                                        }
                                    };
                                    saveConfig(newConfig);
                                }}
                                disabled={cfStatus.running}
                            />
                        </div>

                        {/* 运行状态和URL */}
                        {cfStatus.running && (
                            <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-xl border border-green-200 dark:border-green-800">
                                <div className="flex items-center gap-2 mb-2">
                                    <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
                                    <span className="text-sm font-bold text-green-800 dark:text-green-200">
                                        {t('proxy.cloudflared.running', { defaultValue: 'Tunnel Running' })}
                                    </span>
                                </div>
                                {cfStatus.url && (
                                    <div className="flex items-center gap-2">
                                        <code className="flex-1 px-3 py-2 bg-white dark:bg-base-100 rounded text-xs font-mono text-gray-800 dark:text-gray-200 border border-green-200 dark:border-green-800">
                                            {cfStatus.url}
                                        </code>
                                        <button
                                            onClick={handleCfCopyUrl}
                                            className="p-2 rounded-lg bg-green-500 text-white hover:bg-green-600 transition-colors"
                                        >
                                            {copied === 'cf-url' ? <CheckCircle size={16} /> : <Copy size={16} />}
                                        </button>
                                    </div>
                                )}
                            </div>
                        )}

                        {/* 错误信息 */}
                        {cfStatus.error && (
                            <div className="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800 text-sm text-red-700 dark:text-red-300">
                                {cfStatus.error}
                            </div>
                        )}
                    </>
                )}
            </div>
        </CollapsibleCard>
    );
}
