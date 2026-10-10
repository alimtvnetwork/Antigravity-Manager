import type { TFunction } from 'i18next';
import { RefreshCw, Trash2 } from 'lucide-react';
import HelpTooltip from '../../components/common/HelpTooltip';
import CircuitBreaker from '../../components/settings/CircuitBreaker';
import type { AppConfig, CircuitBreakerConfig, SchedulingMode } from '../../types/config';
import type { FixedAccountControls, ProxyStatus, SchedulingConfigUpdater } from './types';
import { DEFAULT_MAX_WAIT_SECONDS, MAX_WAIT_SLIDER_MAX_SECONDS, MAX_WAIT_SLIDER_STEP_SECONDS } from './constants';
import { CollapsibleCard } from './CollapsibleCard';

interface AccountSchedulingSectionProps {
    config: AppConfig;
    t: TFunction;
    status: ProxyStatus;
    updateSchedulingConfig: SchedulingConfigUpdater;
    updateCircuitBreakerConfig: (config: CircuitBreakerConfig) => void;
    fixedAccount: FixedAccountControls;
    onClearSessionBindings: () => void;
    onClearRateLimits: () => void;
}

const SCHEDULING_MODES: SchedulingMode[] = ['CacheFirst', 'Balance', 'PerformanceFirst'];

export function AccountSchedulingSection({
    config,
    t,
    status,
    updateSchedulingConfig,
    updateCircuitBreakerConfig,
    fixedAccount,
    onClearSessionBindings,
    onClearRateLimits,
}: AccountSchedulingSectionProps) {
    const { preferredAccountId, availableAccounts, handleSetPreferredAccount } = fixedAccount;

    return (
        <CollapsibleCard
            title={t('proxy.config.scheduling.title')}
            icon={<RefreshCw size={18} className="text-indigo-500" />}
        >
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <label className="text-xs font-medium text-gray-700 dark:text-gray-300 inline-flex items-center gap-1">
                                {t('proxy.config.scheduling.mode')}
                                <HelpTooltip
                                    text={t('proxy.config.scheduling.mode_tooltip')}
                                    placement="right"
                                />
                            </label>
                            <div className="flex items-center gap-3">
                                {/* [MOVED] Clear Rate Limit button moved to CircuitBreaker component */}
                                <button
                                    onClick={onClearSessionBindings}
                                    className="text-[10px] text-indigo-500 hover:text-indigo-600 transition-colors flex items-center gap-1"
                                    title={t('proxy.config.scheduling.clear_bindings_tooltip')}
                                >
                                    <Trash2 size={12} />
                                    {t('proxy.config.scheduling.clear_bindings')}
                                </button>
                            </div>
                        </div>
                        <div className="grid grid-cols-1 gap-2">
                            {SCHEDULING_MODES.map(mode => (
                                <label
                                    key={mode}
                                    className={`flex items-start gap-3 p-3 rounded-xl border cursor-pointer transition-all duration-200 ${(config.proxy.scheduling?.mode || 'Balance') === mode
                                        ? 'border-indigo-500 bg-indigo-50/30 dark:bg-indigo-900/10'
                                        : 'border-gray-100 dark:border-base-200 hover:border-indigo-200'
                                        }`}
                                >
                                    <input
                                        type="radio"
                                        className="radio radio-xs radio-primary mt-1"
                                        checked={(config.proxy.scheduling?.mode || 'Balance') === mode}
                                        onChange={() => updateSchedulingConfig({ mode })}
                                    />
                                    <div className="space-y-1">
                                        <div className="text-xs font-bold text-gray-900 dark:text-base-content">
                                            {t(`proxy.config.scheduling.modes.${mode}`)}
                                        </div>
                                        <div className="text-[10px] text-gray-500 line-clamp-2">
                                            {t(`proxy.config.scheduling.modes_desc.${mode}`, {
                                                defaultValue: mode === 'CacheFirst' ? 'Binds session to account, waits precisely if limited (Maximizes Prompt Cache hits).' :
                                                    mode === 'Balance' ? 'Binds session, auto-switches to available account if limited (Balanced cache & availability).' :
                                                        'No session binding, pure round-robin rotation (Best for high concurrency).'
                                            })}
                                        </div>
                                    </div>
                                </label>
                            ))}
                        </div>
                    </div>

                    <div className="space-y-4 pt-1">
                        <div className="bg-slate-100 dark:bg-slate-800/80 rounded-xl p-4 border border-slate-200 dark:border-slate-700">
                            <div className="flex items-center justify-between mb-2">
                                <label className="text-xs font-medium text-gray-700 dark:text-gray-300 inline-flex items-center gap-1">
                                    {t('proxy.config.scheduling.max_wait')}
                                    <HelpTooltip text={t('proxy.config.scheduling.max_wait_tooltip')} />
                                </label>
                                <span className="text-xs font-mono text-indigo-600 font-bold">
                                    {config.proxy.scheduling?.max_wait_seconds || DEFAULT_MAX_WAIT_SECONDS}s
                                </span>
                            </div>
                            <input
                                type="range"
                                min="0"
                                max={MAX_WAIT_SLIDER_MAX_SECONDS}
                                step={MAX_WAIT_SLIDER_STEP_SECONDS}
                                disabled={(config.proxy.scheduling?.mode || 'Balance') !== 'CacheFirst'}
                                className="range range-indigo range-xs"
                                value={config.proxy.scheduling?.max_wait_seconds || DEFAULT_MAX_WAIT_SECONDS}
                                onChange={(e) => updateSchedulingConfig({ max_wait_seconds: parseInt(e.target.value) })}
                            />
                            <div className="flex justify-between px-1 mt-1 text-[10px] text-gray-400 font-mono">
                                <span>0s</span>
                                <span>{MAX_WAIT_SLIDER_MAX_SECONDS}s</span>
                            </div>
                        </div>

                        <div className="p-3 bg-amber-50 dark:bg-amber-900/10 border border-amber-100 dark:border-amber-900/20 rounded-xl">
                            <p className="text-[10px] text-amber-700 dark:text-amber-500 leading-relaxed">
                                <strong>{t('common.info')}:</strong> {t('proxy.config.scheduling.subtitle')}
                            </p>
                        </div>

                        {/* [FIX #820] Fixed Account Mode */}
                        <div className="bg-indigo-50 dark:bg-indigo-900/20 rounded-xl p-4 border border-indigo-200 dark:border-indigo-800">
                            <div className="flex items-center justify-between mb-3">
                                <label className="text-xs font-medium text-gray-700 dark:text-gray-300 inline-flex items-center gap-1">
                                    🔒 {t('proxy.config.scheduling.fixed_account', { defaultValue: 'Fixed Account Mode' })}
                                    <HelpTooltip text={t('proxy.config.scheduling.fixed_account_tooltip', { defaultValue: 'When enabled, all API requests will use only the selected account instead of rotating between accounts.' })} />
                                </label>
                                <input
                                    type="checkbox"
                                    className="toggle toggle-sm toggle-primary"
                                    checked={preferredAccountId !== null}
                                    onChange={(e) => {
                                        if (e.target.checked) {
                                            // Enable fixed mode with first available account
                                            if (availableAccounts.length > 0) {
                                                handleSetPreferredAccount(availableAccounts[0].id);
                                            }
                                        } else {
                                            // Disable fixed mode
                                            handleSetPreferredAccount(null);
                                        }
                                    }}
                                    disabled={!status.running}
                                />
                            </div>
                            {preferredAccountId !== null && (
                                <select
                                    className="select select-bordered select-sm w-full text-xs"
                                    value={preferredAccountId || ''}
                                    onChange={(e) => handleSetPreferredAccount(e.target.value || null)}
                                    disabled={!status.running}
                                >
                                    {availableAccounts.map(account => (
                                        <option key={account.id} value={account.id}>
                                            {account.email}
                                        </option>
                                    ))}
                                </select>
                            )}
                            {!status.running && (
                                <p className="text-[10px] text-gray-500 mt-2">
                                    {t('proxy.config.scheduling.start_proxy_first', { defaultValue: 'Start the proxy service to configure fixed account mode.' })}
                                </p>
                            )}
                        </div>
                    </div>
                </div>

                {/* Circuit Breaker Section */}
                {config.circuit_breaker && (
                    <div className="pt-4 border-t border-gray-100 dark:border-gray-700/50">
                        <div className="flex items-center justify-between mb-4">
                            <label className="text-xs font-medium text-gray-700 dark:text-gray-300 inline-flex items-center gap-1">
                                {t('proxy.config.circuit_breaker.title', { defaultValue: 'Adaptive Circuit Breaker' })}
                                <HelpTooltip text={t('proxy.config.circuit_breaker.tooltip', { defaultValue: 'Prevent continuous failures by exponentially backing off when quota is exhausted.' })} />
                            </label>
                            <input
                                type="checkbox"
                                className="toggle toggle-sm toggle-warning"
                                checked={config.circuit_breaker.enabled}
                                onChange={(e) => updateCircuitBreakerConfig({ ...config.circuit_breaker, enabled: e.target.checked })}
                            />
                        </div>

                        {config.circuit_breaker.enabled && (
                            <CircuitBreaker
                                config={config.circuit_breaker}
                                onChange={updateCircuitBreakerConfig}
                                onClearRateLimits={onClearRateLimits}
                            />
                        )}
                    </div>
                )}
            </div>
        </CollapsibleCard>
    );
}
