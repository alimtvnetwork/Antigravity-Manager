import { SettingsPageApi } from '../useSettingsPage';
import { Network, Globe, Sliders } from 'lucide-react';
import ProxyPoolSettings from '../../../components/settings/ProxyPoolSettings';

export function ProxyTab(props: SettingsPageApi) {
    const { t, saveConfig, formData, setFormData } = props;

    return (
                        <div className="space-y-4 animate-in fade-in duration-300">
                            <ProxyPoolSettings
                                config={formData.proxy?.proxy_pool || {
                                    enabled: false,
                                    proxies: [],
                                    health_check_interval: 300,
                                    auto_failover: true,
                                    strategy: 'priority'
                                }}
                                onChange={(newConfig, silent = false) => {
                                    const updatedFormData = {
                                        ...formData,
                                        proxy: {
                                            ...formData.proxy,
                                            proxy_pool: newConfig
                                        }
                                    };
                                    setFormData(updatedFormData);

                                    // [FIX] Silent updates (like health polling) should NOT trigger saveConfig
                                    // to prevent race conditions where old memory state rolls back new manual changes
                                    if (silent) {
                                        console.log('Proxy status sync (silent)');
                                        return;
                                    }

                                    // Hot reload: save immediately for manual changes
                                    saveConfig({ ...updatedFormData, auto_refresh: true })
                                        .then(() => {
                                            console.log('Proxy config saved');
                                        })
                                        .catch(err => console.error('Save failed:', err));
                                }}
                            />

                            {/* [FIX #1701] Global upstream proxy settings */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-blue-200 transition-all duration-300 shadow-sm relative overflow-hidden">
                                <div className="absolute top-0 right-0 w-24 h-24 bg-blue-500/5 -mr-12 -mt-12 rounded-full blur-2xl group-hover:bg-blue-500/10 transition-colors"></div>
                                <div className="flex items-center justify-between mb-5 relative z-10">
                                    <div className="flex items-center gap-4">
                                        <div className="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-900/20 flex items-center justify-center text-blue-500 group-hover:bg-blue-500 group-hover:text-white transition-all duration-300 shadow-sm">
                                            <Globe size={18} />
                                        </div>
                                        <div>
                                            <div className="font-bold text-gray-900 dark:text-gray-100 text-sm">{t('proxy.config.upstream_proxy.title')}</div>
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5 leading-tight max-w-[280px]">
                                                {t('proxy.config.upstream_proxy.desc_short')}
                                            </p>
                                        </div>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer scale-90">
                                        <input
                                            type="checkbox"
                                            className="sr-only peer"
                                            checked={formData.proxy?.upstream_proxy?.enabled ?? false}
                                            onChange={(e) => setFormData({
                                                ...formData,
                                                proxy: {
                                                    ...formData.proxy,
                                                    upstream_proxy: {
                                                        ...formData.proxy?.upstream_proxy,
                                                        enabled: e.target.checked
                                                    }
                                                }
                                            })}
                                        />
                                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500 shadow-inner"></div>
                                    </label>
                                </div>

                                {formData.proxy?.upstream_proxy?.enabled && (
                                    <div className="space-y-4 animate-in slide-in-from-top-2 duration-300 relative z-10">
                                        <div className="pt-4 border-t border-gray-50 dark:border-base-300">
                                            <label className="block text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-widest mb-2.5">
                                                {t('proxy.config.upstream_proxy.url')}
                                            </label>
                                            <div className="relative group/input">
                                                <input
                                                    type="text"
                                                    className="w-full px-4 py-2.5 bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 rounded-xl focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 outline-none text-sm font-medium transition-all shadow-inner"
                                                    placeholder={t('proxy.config.upstream_proxy.url_placeholder')}
                                                    value={formData.proxy?.upstream_proxy?.url || ''}
                                                    onChange={(e) => setFormData({
                                                        ...formData,
                                                        proxy: {
                                                            ...formData.proxy,
                                                            upstream_proxy: {
                                                                ...formData.proxy?.upstream_proxy,
                                                                url: e.target.value
                                                            }
                                                        }
                                                    })}
                                                />
                                            </div>
                                            <div className="mt-4 bg-amber-50/40 dark:bg-amber-900/10 rounded-xl p-3.5 border border-amber-100/50 dark:border-amber-800/20 text-[11px] text-amber-700 dark:text-amber-400 flex items-start gap-3 transition-colors hover:bg-amber-50/60">
                                                <div className="mt-0.5 p-1 bg-amber-100/80 dark:bg-amber-800/40 rounded-lg shadow-sm">
                                                    <Network size={12} className="text-amber-600 dark:text-amber-400" />
                                                </div>
                                                <div className="leading-relaxed">
                                                    <span className="font-bold mr-1.5 opacity-80 uppercase tracking-tighter">Tip:</span>
                                                    {t('proxy.config.upstream_proxy.socks5h_hint')}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>

                            {/* Proxy Routing & URL Filtering */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-blue-200 transition-all duration-300 shadow-sm relative overflow-hidden">
                                <div className="absolute top-0 right-0 w-24 h-24 bg-indigo-500/5 -mr-12 -mt-12 rounded-full blur-2xl group-hover:bg-indigo-500/10 transition-colors"></div>
                                <div className="flex items-center gap-4 mb-5 relative z-10">
                                    <div className="w-10 h-10 rounded-xl bg-indigo-50 dark:bg-indigo-900/20 flex items-center justify-center text-indigo-500 group-hover:bg-indigo-500 group-hover:text-white transition-all duration-300 shadow-sm">
                                        <Sliders size={18} />
                                    </div>
                                    <div>
                                        <div className="font-bold text-gray-900 dark:text-gray-100 text-sm">
                                            {t('proxy.config.routing_filtering.title', 'Routing & URL Filtering')}
                                        </div>
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5 leading-tight max-w-[320px]">
                                            {t('proxy.config.routing_filtering.desc', 'Configure default path rewrites, global URL reachability, and excluded route rules.')}
                                        </p>
                                    </div>
                                </div>

                                <div className="space-y-4 pt-4 border-t border-gray-50 dark:border-base-300 relative z-10">
                                    {/* Default Path Rewrite Toggle */}
                                    <div className="flex items-center justify-between py-1">
                                        <div>
                                            <div className="text-xs font-semibold text-gray-800 dark:text-gray-200">
                                                {t('proxy.config.default_path_rewrite.title', 'Default Path Rewrite')}
                                            </div>
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5">
                                                {t('proxy.config.default_path_rewrite.desc', 'Automatically map standard protocol endpoints to internal Antigravity routes.')}
                                            </p>
                                        </div>
                                        <label className="relative inline-flex items-center cursor-pointer scale-90">
                                            <input
                                                type="checkbox"
                                                className="sr-only peer"
                                                checked={formData.proxy?.default_path_rewrite ?? true}
                                                onChange={(e) => setFormData({
                                                    ...formData,
                                                    proxy: {
                                                        ...formData.proxy,
                                                        default_path_rewrite: e.target.checked
                                                    }
                                                })}
                                            />
                                            <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500 shadow-inner"></div>
                                        </label>
                                    </div>

                                    {/* Enable All URLs by Default Toggle */}
                                    <div className="flex items-center justify-between py-1">
                                        <div>
                                            <div className="text-xs font-semibold text-gray-800 dark:text-gray-200">
                                                {t('proxy.config.enable_all_urls.title', 'Enable All URLs by Default')}
                                            </div>
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5">
                                                {t('proxy.config.enable_all_urls.desc', 'Permit all inbound upstream target URLs without requiring explicit allowlisting.')}
                                            </p>
                                        </div>
                                        <label className="relative inline-flex items-center cursor-pointer scale-90">
                                            <input
                                                type="checkbox"
                                                className="sr-only peer"
                                                checked={formData.proxy?.enable_all_urls ?? true}
                                                onChange={(e) => setFormData({
                                                    ...formData,
                                                    proxy: {
                                                        ...formData.proxy,
                                                        enable_all_urls: e.target.checked
                                                    }
                                                })}
                                            />
                                            <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500 shadow-inner"></div>
                                        </label>
                                    </div>

                                    {/* Excluded URLs list */}
                                    <div className="pt-2">
                                        <label className="block text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-widest mb-1.5">
                                            {t('proxy.config.excluded_urls.label', 'Excluded URLs (Comma-separated)')}
                                        </label>
                                        <input
                                            type="text"
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 rounded-xl focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 outline-none text-xs font-mono transition-all shadow-inner"
                                            placeholder="https://example.com/api, https://telemetry.internal"
                                            value={(formData.proxy?.excluded_urls || []).join(', ')}
                                            onChange={(e) => {
                                                const list = e.target.value
                                                    .split(',')
                                                    .map((s) => s.trim())
                                                    .filter(Boolean);
                                                setFormData({
                                                    ...formData,
                                                    proxy: {
                                                        ...formData.proxy,
                                                        excluded_urls: list
                                                    }
                                                });
                                            }}
                                        />
                                        <p className="text-[10px] text-gray-400 dark:text-gray-500 mt-1">
                                            {t('proxy.config.excluded_urls.hint', 'Requests targeted at these URLs will be intercepted or dropped by the gateway.')}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
    );
}
