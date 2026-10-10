import type { TFunction } from 'i18next';
import { ArrowRight, Plus, RefreshCw, Settings, Trash2, Zap } from 'lucide-react';
import HelpTooltip from '../../components/common/HelpTooltip';
import type { AppConfig, ZaiDispatchMode, ZaiModelDefaults } from '../../types/config';
import type { ZaiDispatcher } from './useZaiDispatcher';
import { DEFAULT_ZAI_BASE_URL, ZAI_MODEL_FAMILIES } from './constants';
import { CollapsibleCard } from './CollapsibleCard';

interface ZaiDispatcherSectionProps {
    config: AppConfig;
    t: TFunction;
    zai: ZaiDispatcher;
}

export function ZaiDispatcherSection({ config, t, zai }: ZaiDispatcherSectionProps) {
    const {
        zaiModelOptions,
        zaiModelMapping,
        zaiModelsLoading,
        zaiNewMappingFrom,
        setZaiNewMappingFrom,
        zaiNewMappingTo,
        setZaiNewMappingTo,
        refreshZaiModels,
        updateZaiDefaultModels,
        upsertZaiModelMapping,
        removeZaiModelMapping,
        updateZaiGeneralConfig,
    } = zai;

    return (
        <CollapsibleCard
            title={t('proxy.config.zai.title')}
            icon={<Zap size={18} className="text-amber-500" />}
            enabled={!!config.proxy.zai?.enabled}
            onToggle={(checked) => updateZaiGeneralConfig({ enabled: checked })}
        >
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-1">
                        <label className="text-[11px] font-medium text-gray-500 dark:text-gray-400">
                            {t('proxy.config.zai.base_url')}
                        </label>
                        <input
                            type="text"
                            value={config.proxy.zai?.base_url || DEFAULT_ZAI_BASE_URL}
                            onChange={(e) => updateZaiGeneralConfig({ base_url: e.target.value })}
                            className="input input-sm input-bordered w-full font-mono text-xs"
                        />
                    </div>
                    <div className="space-y-1">
                        <label className="text-[11px] font-medium text-gray-500 dark:text-gray-400">
                            {t('proxy.config.zai.dispatch_mode')}
                        </label>
                        <select
                            className="select select-sm select-bordered w-full text-xs"
                            value={config.proxy.zai?.dispatch_mode || 'off'}
                            onChange={(e) => updateZaiGeneralConfig({ dispatch_mode: e.target.value as ZaiDispatchMode })}
                        >
                            <option value="off">{t('proxy.config.zai.modes.off')}</option>
                            <option value="exclusive">{t('proxy.config.zai.modes.exclusive')}</option>
                            <option value="pooled">{t('proxy.config.zai.modes.pooled')}</option>
                            <option value="fallback">{t('proxy.config.zai.modes.fallback')}</option>
                        </select>
                    </div>
                </div>

                <div className="space-y-1">
                    <label className="text-[11px] font-medium text-gray-500 dark:text-gray-400 flex items-center justify-between">
                        <span>{t('proxy.config.zai.api_key')}</span>
                        {!(config.proxy.zai?.api_key) && (
                            <span className="text-amber-500 text-[10px] flex items-center gap-1">
                                <HelpTooltip text={t('proxy.config.zai.warning')} />
                                {t('common.required')}
                            </span>
                        )}
                    </label>
                    <input
                        type="password"
                        value={config.proxy.zai?.api_key || ''}
                        onChange={(e) => updateZaiGeneralConfig({ api_key: e.target.value })}
                        placeholder="sk-..."
                        className="input input-sm input-bordered w-full font-mono text-xs"
                    />
                </div>

                {/* Model Mapping Section */}
                <div className="pt-4 border-t border-gray-100 dark:border-base-200">
                    <div className="flex items-center justify-between mb-3">
                        <h4 className="text-[11px] font-bold text-gray-400 uppercase tracking-widest">
                            {t('proxy.config.zai.models.title')}
                        </h4>
                        <button
                            onClick={refreshZaiModels}
                            disabled={zaiModelsLoading || !config.proxy.zai?.api_key}
                            className="btn btn-ghost btn-xs gap-1"
                        >
                            <RefreshCw size={12} className={zaiModelsLoading ? 'animate-spin' : ''} />
                            {t('proxy.config.zai.models.refresh')}
                        </button>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        {ZAI_MODEL_FAMILIES.map((family) => (
                            <div key={family} className="space-y-1">
                                <label className="text-[10px] text-gray-500 capitalize">{family}</label>
                                <div className="flex gap-1">
                                    {zaiModelOptions.length > 0 && (
                                        <select
                                            className="select select-xs select-bordered max-w-[80px]"
                                            value=""
                                            onChange={(e) => e.target.value && updateZaiDefaultModels({ [family]: e.target.value } as Partial<ZaiModelDefaults>)}
                                        >
                                            <option value="">{t('proxy.config.zai.models.select_placeholder')}</option>
                                            {zaiModelOptions.map(m => <option key={m} value={m}>{m}</option>)}
                                        </select>
                                    )}
                                    <input
                                        type="text"
                                        className="input input-xs input-bordered w-full font-mono"
                                        value={config.proxy.zai?.models?.[family] || ''}
                                        onChange={(e) => updateZaiDefaultModels({ [family]: e.target.value } as Partial<ZaiModelDefaults>)}
                                    />
                                </div>
                            </div>
                        ))}
                    </div>

                    <details className="mt-3 group">
                        <summary className="cursor-pointer text-[10px] text-gray-500 hover:text-blue-500 transition-colors inline-flex items-center gap-1 select-none">
                            <Settings size={12} />
                            {t('proxy.config.zai.models.advanced_title')}
                        </summary>
                        <div className="mt-2 space-y-2 p-2 bg-gray-50 dark:bg-base-200/50 rounded-lg">
                            {/* Advanced Mapping Table */}
                            {Object.entries(zaiModelMapping).map(([from, to]) => (
                                <div key={from} className="flex items-center gap-2">
                                    <div className="flex-1 bg-white dark:bg-base-100 px-2 py-1 rounded border border-gray-200 dark:border-base-300 text-[10px] font-mono truncate" title={from}>{from}</div>
                                    <ArrowRight size={10} className="text-gray-400" />
                                    <div className="flex-[1.5] flex gap-1">
                                        {zaiModelOptions.length > 0 && (
                                            <select
                                                className="select select-xs select-ghost h-6 min-h-0 px-1"
                                                value=""
                                                onChange={(e) => e.target.value && upsertZaiModelMapping(from, e.target.value)}
                                            >
                                                <option value="">▼</option>
                                                {zaiModelOptions.map(m => <option key={m} value={m}>{m}</option>)}
                                            </select>
                                        )}
                                        <input
                                            type="text"
                                            className="input input-xs input-bordered w-full font-mono h-6"
                                            value={to}
                                            onChange={(e) => upsertZaiModelMapping(from, e.target.value)}
                                        />
                                    </div>
                                    <button onClick={() => removeZaiModelMapping(from)} className="text-gray-400 hover:text-red-500"><Trash2 size={12} /></button>
                                </div>
                            ))}

                            <div className="flex items-center gap-2 pt-2 border-t border-gray-200/50">
                                <input
                                    className="input input-xs input-bordered flex-1 font-mono"
                                    placeholder={t('proxy.config.zai.models.from_placeholder') || "From (e.g. claude-3-opus)"}
                                    value={zaiNewMappingFrom}
                                    onChange={e => setZaiNewMappingFrom(e.target.value)}
                                />
                                <input
                                    className="input input-xs input-bordered flex-1 font-mono"
                                    placeholder={t('proxy.config.zai.models.to_placeholder') || "To (e.g. glm-4)"}
                                    value={zaiNewMappingTo}
                                    onChange={e => setZaiNewMappingTo(e.target.value)}
                                />
                                <button
                                    className="btn btn-xs btn-primary"
                                    onClick={() => {
                                        if (zaiNewMappingFrom && zaiNewMappingTo) {
                                            upsertZaiModelMapping(zaiNewMappingFrom, zaiNewMappingTo);
                                            setZaiNewMappingFrom('');
                                            setZaiNewMappingTo('');
                                        }
                                    }}
                                >
                                    <Plus size={12} />
                                </button>
                            </div>
                        </div>
                    </details>
                </div>
            </div>
        </CollapsibleCard>
    );
}
