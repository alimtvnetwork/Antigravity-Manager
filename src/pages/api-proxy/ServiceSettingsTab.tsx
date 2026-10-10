import type { TFunction } from 'i18next';
import { CheckCircle, Copy, Edit2, RefreshCw, X } from 'lucide-react';
import HelpTooltip from '../../components/common/HelpTooltip';
import type { AppConfig, ProxyConfig } from '../../types/config';
import type { CredentialEditing, ProxyConfigUpdater, ProxyStatus } from './types';
import {
    DEFAULT_REQUEST_TIMEOUT_SECONDS,
    MIN_REQUEST_TIMEOUT_SECONDS,
    MAX_REQUEST_TIMEOUT_SECONDS,
    DEFAULT_LOG_RETENTION,
    DEFAULT_USER_AGENT_OVERRIDE,
    DEFAULT_PROXY_PORT,
} from './constants';

interface ServiceSettingsTabProps {
    config: AppConfig;
    t: TFunction;
    status: ProxyStatus;
    updateProxyConfig: ProxyConfigUpdater;
    copied: string | null;
    copyToClipboardHandler: (text: string, label: string) => void;
    credentials: CredentialEditing;
}

export function ServiceSettingsTab({
    config,
    t,
    status,
    updateProxyConfig,
    copied,
    copyToClipboardHandler,
    credentials,
}: ServiceSettingsTabProps) {
    const {
        isEditingApiKey,
        tempApiKey,
        setTempApiKey,
        handleEditApiKey,
        handleSaveApiKey,
        handleCancelEditApiKey,
        isEditingAdminPassword,
        tempAdminPassword,
        setTempAdminPassword,
        handleEditAdminPassword,
        handleSaveAdminPassword,
        handleCancelEditAdminPassword,
        handleGenerateApiKey,
    } = credentials;

    return (
        <div className="p-4 space-y-4">
            {/* 监听端口、超时和自启动 */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                <div>
                    <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                        <span className="inline-flex items-center gap-1">
                            {t('proxy.config.port')}
                            <HelpTooltip
                                text={t('proxy.config.port_tooltip')}
                                ariaLabel={t('proxy.config.port')}
                                placement="right"
                            />
                        </span>
                    </label>
                    <input
                        type="number"
                        value={config.proxy.port}
                        onChange={(e) => updateProxyConfig({ port: parseInt(e.target.value) })}
                        min={8000}
                        max={65535}
                        disabled={status.running}
                        className="w-full px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 text-xs text-gray-900 dark:text-base-content focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                    />
                    <p className="mt-0.5 text-[10px] text-gray-500 dark:text-gray-400">
                        {t('proxy.config.port_hint')}
                    </p>

                    {/* 监听端口下方空位：服务运行状态指示卡片 */}
                    <div className="mt-2.5 p-2 rounded-lg border border-gray-200/80 dark:border-base-300/80 bg-gray-50/70 dark:bg-base-200/50 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className={`w-2.5 h-2.5 rounded-full ${status.running ? 'bg-emerald-500 shadow-xs shadow-emerald-500/50 animate-pulse' : 'bg-gray-400 dark:bg-gray-500'}`} />
                            <span className={`text-xs font-semibold ${status.running ? 'text-emerald-600 dark:text-emerald-400' : 'text-gray-500 dark:text-gray-400'}`}>
                                {status.running
                                    ? `${t('proxy.status.running')} (${status.active_accounts} ${t('common.accounts')})`
                                    : t('proxy.status.stopped')}
                            </span>
                        </div>
                        {status.running && (
                            <span className="text-[10px] font-mono text-gray-400 dark:text-gray-500 truncate max-w-[130px]" title={status.base_url || `http://127.0.0.1:${config.proxy.port || DEFAULT_PROXY_PORT}`}>
                                {status.base_url || `http://127.0.0.1:${config.proxy.port || DEFAULT_PROXY_PORT}`}
                            </span>
                        )}
                    </div>
                </div>
                <div>
                    <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                        <span className="inline-flex items-center gap-1">
                            {t('proxy.config.request_timeout')}
                            <HelpTooltip
                                text={t('proxy.config.request_timeout_tooltip')}
                                ariaLabel={t('proxy.config.request_timeout')}
                                placement="top"
                            />
                        </span>
                    </label>
                    <input
                        type="number"
                        value={config.proxy.request_timeout || DEFAULT_REQUEST_TIMEOUT_SECONDS}
                        onChange={(e) => {
                            const value = parseInt(e.target.value);
                            const timeout = Math.max(MIN_REQUEST_TIMEOUT_SECONDS, Math.min(MAX_REQUEST_TIMEOUT_SECONDS, value));
                            updateProxyConfig({ request_timeout: timeout });
                        }}
                        min={MIN_REQUEST_TIMEOUT_SECONDS}
                        max={MAX_REQUEST_TIMEOUT_SECONDS}
                        disabled={status.running}
                        className="w-full px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 text-xs text-gray-900 dark:text-base-content focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                    />
                    <p className="mt-0.5 text-[10px] text-gray-500 dark:text-gray-400">
                        {t('proxy.config.request_timeout_hint')}
                    </p>
                </div>
                <div className="flex items-center">
                    <label className="flex items-center cursor-pointer gap-3">
                        <input
                            type="checkbox"
                            className="toggle toggle-sm bg-gray-200 dark:bg-gray-700 border-gray-300 dark:border-gray-600 checked:bg-blue-500 checked:border-blue-500 disabled:opacity-50 disabled:bg-gray-100 dark:disabled:bg-gray-800"
                            checked={config.proxy.auto_start}
                            onChange={(e) => updateProxyConfig({ auto_start: e.target.checked })}
                        />
                        <span className="text-xs font-medium text-gray-900 dark:text-base-content inline-flex items-center gap-1">
                            {t('proxy.config.auto_start')}
                            <HelpTooltip
                                text={t('proxy.config.auto_start_tooltip')}
                                ariaLabel={t('proxy.config.auto_start')}
                                placement="right"
                            />
                        </span>
                    </label>
                </div>
            </div>


            <div className="border-t border-gray-200 dark:border-base-300 pt-3 mt-3">
                <div className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">{t('proxy.config.log_retention_title')}</div>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                    {([['max_body_age_hours', 'log_retention_body_hours'], ['max_age_days', 'log_retention_age_days'], ['max_rows', 'log_retention_rows']] as const).map(([field, label]) => (
                        <label key={field} className="text-xs text-gray-600 dark:text-gray-400">
                            {t(`proxy.config.${label}`)}
                            <input type="number" min={1}
                                value={config.proxy.log_retention?.[field] ?? DEFAULT_LOG_RETENTION[field]}
                                onChange={(e) => updateProxyConfig({ log_retention: { ...(config.proxy.log_retention || DEFAULT_LOG_RETENTION), [field]: Math.max(1, Number(e.target.value)) } })}
                                className="w-full mt-1 px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 text-xs" />
                        </label>
                    ))}
                </div>
            </div>

            {/* 局域网访问 & 访问授权 - 合并到同一行 */}
            <div className="border-t border-gray-200 dark:border-base-300 pt-3 mt-3">
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                    {/* 允许局域网访问 */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <span className="text-xs font-medium text-gray-700 dark:text-gray-300 inline-flex items-center gap-1">
                                {t('proxy.config.allow_lan_access')}
                                <HelpTooltip
                                    text={t('proxy.config.allow_lan_access_tooltip')}
                                    ariaLabel={t('proxy.config.allow_lan_access')}
                                    placement="right"
                                />
                            </span>
                            <input
                                type="checkbox"
                                className="toggle toggle-sm bg-gray-200 dark:bg-gray-700 border-gray-300 dark:border-gray-600 checked:bg-blue-500 checked:border-blue-500"
                                checked={config.proxy.allow_lan_access || false}
                                onChange={(e) => updateProxyConfig({ allow_lan_access: e.target.checked })}
                            />
                        </div>
                        <p className="text-[10px] text-gray-500 dark:text-gray-400">
                            {(config.proxy.allow_lan_access || false)
                                ? t('proxy.config.allow_lan_access_hint_enabled')
                                : t('proxy.config.allow_lan_access_hint_disabled')}
                        </p>
                        {(config.proxy.allow_lan_access || false) && (
                            <p className="text-[10px] text-amber-600 dark:text-amber-500">
                                {t('proxy.config.allow_lan_access_warning')}
                            </p>
                        )}
                        {status.running && (
                            <p className="text-[10px] text-blue-600 dark:text-blue-400">
                                {t('proxy.config.allow_lan_access_restart_hint')}
                            </p>
                        )}
                    </div>

                    {/* 访问授权 */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <label className="text-xs font-medium text-gray-700 dark:text-gray-300">
                                <span className="inline-flex items-center gap-1">
                                    {t('proxy.config.auth.title')}
                                    <HelpTooltip
                                        text={t('proxy.config.auth.title_tooltip')}
                                        ariaLabel={t('proxy.config.auth.title')}
                                        placement="top"
                                    />
                                </span>
                            </label>
                            <label className="flex items-center cursor-pointer gap-2">
                                <span className="text-[11px] text-gray-600 dark:text-gray-400 inline-flex items-center gap-1">
                                    {(config.proxy.auth_mode || 'off') !== 'off' ? t('proxy.config.auth.enabled') : t('common.disabled')}
                                    <HelpTooltip
                                        text={t('proxy.config.auth.enabled_tooltip')}
                                        ariaLabel={t('proxy.config.auth.enabled')}
                                        placement="left"
                                    />
                                </span>
                                <input
                                    type="checkbox"
                                    className="toggle toggle-sm bg-gray-200 dark:bg-gray-700 border-gray-300 dark:border-gray-600 checked:bg-blue-500 checked:border-blue-500 disabled:opacity-50 disabled:bg-gray-100 dark:disabled:bg-gray-800"
                                    checked={(config.proxy.auth_mode || 'off') !== 'off'}
                                    onChange={(e) => {
                                        const nextMode = e.target.checked ? 'all_except_health' : 'off';
                                        updateProxyConfig({ auth_mode: nextMode });
                                    }}
                                />
                            </label>
                        </div>

                        <div>
                            <label className="block text-[11px] text-gray-600 dark:text-gray-400 mb-1">
                                <span className="inline-flex items-center gap-1">
                                    {t('proxy.config.auth.mode')}
                                    <HelpTooltip
                                        text={t('proxy.config.auth.mode_tooltip')}
                                        ariaLabel={t('proxy.config.auth.mode')}
                                        placement="top"
                                    />
                                </span>
                            </label>
                            <select
                                value={config.proxy.auth_mode || 'off'}
                                onChange={(e) =>
                                    updateProxyConfig({
                                        auth_mode: e.target.value as ProxyConfig['auth_mode'],
                                    })
                                }
                                className="w-full px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 text-xs text-gray-900 dark:text-base-content focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            >
                                <option value="off">{t('proxy.config.auth.modes.off')}</option>
                                <option value="strict">{t('proxy.config.auth.modes.strict')}</option>
                                <option value="all_except_health">{t('proxy.config.auth.modes.all_except_health')}</option>
                                <option value="auto">{t('proxy.config.auth.modes.auto')}</option>
                            </select>
                            <p className="mt-0.5 text-[10px] text-gray-500 dark:text-gray-400">
                                {t('proxy.config.auth.hint')}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            {/* API 密钥 */}
            <div>
                <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                    <span className="inline-flex items-center gap-1">
                        {t('proxy.config.api_key')}
                        <HelpTooltip
                            text={t('proxy.config.api_key_tooltip')}
                            ariaLabel={t('proxy.config.api_key')}
                            placement="right"
                        />
                    </span>
                </label>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={isEditingApiKey ? tempApiKey : (config.proxy.api_key)}
                        onChange={(e) => isEditingApiKey && setTempApiKey(e.target.value)}
                        readOnly={!isEditingApiKey}
                        className={`flex-1 px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg text-xs font-mono ${isEditingApiKey
                            ? 'bg-white dark:bg-base-200 text-gray-900 dark:text-base-content'
                            : 'bg-gray-50 dark:bg-base-300 text-gray-600 dark:text-gray-400'
                            }`}
                    />
                    {isEditingApiKey ? (
                        <>
                            <button
                                onClick={handleSaveApiKey}
                                className="px-2.5 py-1.5 border border-green-300 dark:border-green-700 rounded-lg bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors text-green-600 dark:text-green-400"
                                title={t('proxy.config.btn_save')}
                            >
                                <CheckCircle size={14} />
                            </button>
                            <button
                                onClick={handleCancelEditApiKey}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('common.cancel')}
                            >
                                <X size={14} />
                            </button>
                        </>
                    ) : (
                        <>
                            <button
                                onClick={handleEditApiKey}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('proxy.config.btn_edit')}
                            >
                                <Edit2 size={14} />
                            </button>
                            <button
                                onClick={handleGenerateApiKey}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('proxy.config.btn_regenerate')}
                            >
                                <RefreshCw size={14} />
                            </button>
                            <button
                                onClick={() => copyToClipboardHandler(config.proxy.api_key, 'api_key')}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('proxy.config.btn_copy')}
                            >
                                {copied === 'api_key' ? (
                                    <CheckCircle size={14} className="text-green-500" />
                                ) : (
                                    <Copy size={14} />
                                )}
                            </button>
                        </>
                    )}
                </div>
                <p className="mt-0.5 text-[10px] text-amber-600 dark:text-amber-500">
                    {t('proxy.config.warning_key')}
                </p>
            </div>

            {/* Web UI 管理密码 */}
            <div className="border-t border-gray-200 dark:border-base-300 pt-3 mt-3">
                <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                    <span className="inline-flex items-center gap-1">
                        {t('proxy.config.admin_password', { defaultValue: 'Web UI Login Password' })}
                        <HelpTooltip
                            text={t('proxy.config.admin_password_tooltip', { defaultValue: 'Used for logging into the Web Management Console. If empty, it defaults to the API Key.' })}
                            ariaLabel={t('proxy.config.admin_password')}
                            placement="right"
                        />
                    </span>
                </label>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={isEditingAdminPassword ? tempAdminPassword : (config.proxy.admin_password || t('proxy.config.admin_password_default', { defaultValue: '(Same as API Key)' }))}
                        onChange={(e) => isEditingAdminPassword && setTempAdminPassword(e.target.value)}
                        readOnly={!isEditingAdminPassword}
                        placeholder={t('proxy.config.admin_password_placeholder', { defaultValue: 'Enter new password or leave empty to use API Key' })}
                        className={`flex-1 px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg text-xs font-mono ${isEditingAdminPassword
                            ? 'bg-white dark:bg-base-200 text-gray-900 dark:text-base-content'
                            : 'bg-gray-50 dark:bg-base-300 text-gray-600 dark:text-gray-400'
                            }`}
                    />
                    {isEditingAdminPassword ? (
                        <>
                            <button
                                onClick={handleSaveAdminPassword}
                                className="px-2.5 py-1.5 border border-green-300 dark:border-green-700 rounded-lg bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors text-green-600 dark:text-green-400"
                                title={t('proxy.config.btn_save')}
                            >
                                <CheckCircle size={14} />
                            </button>
                            <button
                                onClick={handleCancelEditAdminPassword}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('common.cancel')}
                            >
                                <X size={14} />
                            </button>
                        </>
                    ) : (
                        <>
                            <button
                                onClick={handleEditAdminPassword}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('proxy.config.btn_edit')}
                            >
                                <Edit2 size={14} />
                            </button>
                            <button
                                onClick={() => copyToClipboardHandler(config.proxy.admin_password || config.proxy.api_key, 'admin_password')}
                                className="px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                title={t('proxy.config.btn_copy')}
                            >
                                {copied === 'admin_password' ? (
                                    <CheckCircle size={14} className="text-green-500" />
                                ) : (
                                    <Copy size={14} />
                                )}
                            </button>
                        </>
                    )}
                </div>
                <p className="mt-0.5 text-[10px] text-gray-500 dark:text-gray-400">
                    {t('proxy.config.admin_password_hint', { defaultValue: 'For safety in Docker/Browser environments, you can set a separate login password from your API Key.' })}
                </p>
            </div>

            {/* User-Agent Overrides */}
            <div className="border-t border-gray-200 dark:border-base-300 pt-3 mt-3">
                <div className="flex items-center justify-between mb-2">
                    <label className="text-xs font-medium text-gray-700 dark:text-gray-300 inline-flex items-center gap-1">
                        {t('proxy.config.request.user_agent', { defaultValue: 'User-Agent Override' })}
                        <HelpTooltip text={t('proxy.config.request.user_agent_tooltip', { defaultValue: 'Override the User-Agent header sent to upstream APIs.' })} />
                    </label>
                    <input
                        type="checkbox"
                        className="toggle toggle-sm bg-gray-200 dark:bg-gray-700 border-gray-300 dark:border-gray-600 checked:bg-blue-500 checked:border-blue-500"
                        checked={!!config.proxy.user_agent_override}
                        onChange={(e) => {
                            const enabled = e.target.checked;
                            if (enabled) {
                                // Restore saved override from config or use default
                                const restoredValue = config.proxy.saved_user_agent || DEFAULT_USER_AGENT_OVERRIDE;
                                updateProxyConfig({
                                    user_agent_override: restoredValue,
                                    saved_user_agent: restoredValue
                                });
                            } else {
                                // Disable active override but keep saved value
                                updateProxyConfig({ user_agent_override: undefined });
                            }
                        }}
                    />
                </div>

                {!!config.proxy.user_agent_override && (
                    <div className="space-y-2 animate-in fade-in slide-in-from-top-1 duration-200">
                        <input
                            type="text"
                            value={config.proxy.user_agent_override}
                            onChange={(e) => {
                                const newValue = e.target.value;
                                updateProxyConfig({
                                    user_agent_override: newValue,
                                    saved_user_agent: newValue
                                });
                            }}
                            className="w-full px-2.5 py-1.5 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 text-xs font-mono text-gray-900 dark:text-base-content focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            placeholder={t('proxy.config.request.user_agent_placeholder', { defaultValue: 'Enter custom User-Agent string...' })}
                        />
                        <div className="bg-gray-50 dark:bg-base-300 rounded p-2 text-[10px] text-gray-500 font-mono break-all">
                            <span className="font-bold select-none mr-2">{t('common.example', { defaultValue: 'Example' })}:</span>
                            {DEFAULT_USER_AGENT_OVERRIDE}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
