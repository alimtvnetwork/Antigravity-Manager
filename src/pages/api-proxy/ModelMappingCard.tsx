import { useState } from 'react';
import type { TFunction } from 'i18next';
import {
    ArrowRight,
    BrainCircuit,
    Check,
    Edit2,
    Plus,
    RefreshCw,
    Sparkles,
    Target,
    Trash2,
    X,
} from 'lucide-react';
import GroupedSelect, { SelectOption } from '../../components/common/GroupedSelect';
import type { AppConfig } from '../../types/config';
import type { PresetManager, ProxyConfigUpdater } from './types';
import { BACKGROUND_TASK_MAPPING_KEY, BUILT_IN_PRESET_IDS, BUILT_IN_WILDCARD_KEY } from './constants';

interface ModelMappingCardProps {
    config: AppConfig;
    t: TFunction;
    updateProxyConfig: ProxyConfigUpdater;
    customMappingOptions: SelectOption[];
    presetManager: PresetManager;
    onMappingUpdate: (type: 'custom', key: string, value: string) => Promise<void>;
    onRemoveCustomMapping: (key: string) => Promise<void>;
    onResetMapping: () => void;
}

export function ModelMappingCard({
    config,
    t,
    updateProxyConfig,
    customMappingOptions,
    presetManager,
    onMappingUpdate,
    onRemoveCustomMapping,
    onResetMapping,
}: ModelMappingCardProps) {
    const [editingKey, setEditingKey] = useState<string | null>(null);
    const [editingValue, setEditingValue] = useState<string>('');
    const [customMappingValue, setCustomMappingValue] = useState(''); // 自定义映射表单的选中值

    const {
        defaultPresets,
        customPresets,
        selectedPreset,
        setSelectedPreset,
        setNewPresetName,
        setIsPresetManagerOpen,
        handleApplyPresets,
        handleDeletePreset,
    } = presetManager;

    return (
        <div className="bg-white dark:bg-base-100 rounded-xl shadow-xs border border-gray-200/80 dark:border-base-200 overflow-hidden">
            <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-700/50 bg-gray-50/50 dark:bg-gray-800/50">
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="flex-1">
                        <h2 className="text-base font-bold flex items-center gap-2 text-gray-900 dark:text-base-content">
                            <BrainCircuit size={18} className="text-blue-500" />
                            {t('proxy.router.title')}
                        </h2>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1 max-w-xl leading-relaxed">
                            {t('proxy.router.subtitle_simple')}
                        </p>
                    </div>
                    <div className="flex flex-wrap items-center gap-2.5 bg-white dark:bg-base-100 p-1.5 rounded-xl border border-gray-100 dark:border-gray-700/50 shadow-sm">
                        {/* 仅暴露真实配额模型开关 */}
                        <label
                            className="flex items-center gap-2 px-3 py-1 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50/80 dark:bg-base-200 cursor-pointer hover:bg-gray-100 dark:hover:bg-base-300 transition-colors h-9 select-none"
                            title={t('proxy.router.only_raw_quota_models_tooltip')}
                        >
                            <span className="text-xs font-medium text-gray-700 dark:text-gray-300 whitespace-nowrap">
                                {t('proxy.router.only_raw_quota_models')}
                            </span>
                            <input
                                type="checkbox"
                                className="toggle toggle-sm bg-gray-300 dark:bg-gray-700 border-gray-400 dark:border-gray-600 checked:bg-blue-600 checked:border-blue-600 cursor-pointer"
                                checked={config.proxy.only_raw_quota_models ?? false}
                                onChange={(e) => updateProxyConfig({ only_raw_quota_models: e.target.checked })}
                            />
                        </label>

                        {/* 预设选择下拉框 */}
                        <div className="relative min-w-[140px]">
                            <select
                                value={selectedPreset}
                                onChange={(e) => setSelectedPreset(e.target.value)}
                                className="select select-sm w-full bg-gray-50 dark:bg-base-200 border-gray-200 dark:border-gray-700 text-xs font-medium focus:ring-1 focus:ring-blue-500 h-9 min-h-0 rounded-lg"
                            >
                                <optgroup label={t('proxy.router.built_in_presets')}>
                                    {defaultPresets.map(preset => (
                                        <option key={preset.id} value={preset.id}>
                                            {preset.name}
                                        </option>
                                    ))}
                                </optgroup>
                                {customPresets.length > 0 && (
                                    <optgroup label={t('proxy.router.custom_presets')}>
                                        {customPresets.map(preset => (
                                            <option key={preset.id} value={preset.id}>
                                                {preset.name}
                                            </option>
                                        ))}
                                    </optgroup>
                                )}
                            </select>
                        </div>

                        <button
                            onClick={handleApplyPresets}
                            className="btn btn-sm btn-primary h-9 min-h-0 gap-1.5 shadow-sm text-xs"
                        >
                            <Check size={14} />
                            {t('proxy.router.apply_selected')}
                        </button>

                        <button
                            onClick={() => {
                                setNewPresetName('');
                                setIsPresetManagerOpen(true);
                            }}
                            className="btn btn-sm btn-outline border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 h-9 min-h-0 text-xs gap-1.5"
                            title={t('proxy.router.add_preset')}
                        >
                            <Plus size={14} />
                        </button>

                        {selectedPreset && !BUILT_IN_PRESET_IDS.includes(selectedPreset) ? (
                            <button
                                onClick={() => handleDeletePreset(selectedPreset)}
                                className="btn btn-sm btn-ghost text-error hover:bg-red-50 dark:hover:bg-red-900/20 h-9 min-h-0 p-2 text-xs"
                                title={t('proxy.router.delete_preset')}
                            >
                                <Trash2 size={14} />
                            </button>
                        ) : null}

                        <button
                            onClick={onResetMapping}
                            className="btn btn-sm btn-ghost text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 h-9 min-h-0 p-2 text-xs"
                            title={t('proxy.router.reset_mapping')}
                        >
                            <RefreshCw size={14} />
                        </button>
                    </div>
                </div>
            </div>

            <div className="p-4 space-y-4">
                {/* Background Task Model Mapping */}
                <div className="bg-gray-50/50 dark:bg-white/5 p-3 rounded-xl border border-gray-100 dark:border-white/5">
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                        <div>
                            <h3 className="text-xs font-bold text-gray-900 dark:text-white flex items-center gap-1.5">
                                <Sparkles size={14} className="text-blue-500" />
                                {t('proxy.router.background_task_title')}
                            </h3>
                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5">
                                {t('proxy.router.background_task_desc')}
                            </p>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-48 sm:w-60">
                                <GroupedSelect
                                    value={config.proxy.custom_mapping?.[BACKGROUND_TASK_MAPPING_KEY] || ''}
                                    onChange={(val) => onMappingUpdate('custom', BACKGROUND_TASK_MAPPING_KEY, val)}
                                    options={customMappingOptions}
                                    placeholder={t('proxy.router.select_target_model') || 'Select Target Model'}
                                    className="font-mono text-xs h-8 dark:bg-gray-800"
                                    allowCustomInput={true}
                                />
                            </div>
                            {config.proxy.custom_mapping && config.proxy.custom_mapping[BACKGROUND_TASK_MAPPING_KEY] && (
                                <button
                                    onClick={() => onRemoveCustomMapping(BACKGROUND_TASK_MAPPING_KEY)}
                                    className="p-1.5 text-gray-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded transition-colors"
                                    title={t('proxy.router.use_default')}
                                >
                                    <RefreshCw size={12} />
                                </button>
                            )}
                        </div>
                    </div>
                </div>

                <div className="flex items-center justify-between mb-3">
                    <div className="flex flex-col gap-1">
                        <h3 className="text-[10px] font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2">
                            <ArrowRight size={14} /> {t('proxy.router.custom_mappings')}
                        </h3>
                        <p className="text-[9px] text-gray-500 dark:text-gray-400 leading-relaxed">
                            {t('proxy.router.custom_mapping_tip')}
                            <span className="text-amber-600 dark:text-amber-400">{t('proxy.router.custom_mapping_warning')}</span>
                        </p>
                        <p className="text-[9px] text-amber-600 dark:text-amber-400 leading-relaxed">
                            {t('proxy.router.wildcard_rule_notice') || '通配符规则 gemini-3.x-flash：x 必须大于 8，统一转为 3.x-flash-tiered'}
                        </p>
                    </div>
                </div>
                <div className="flex flex-col gap-4">
                    {/* 当前映射列表 (置顶 2 列) */}
                    <div className="w-full flex flex-col">
                        <div className="flex items-center justify-between mb-2">
                            <span className="text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                                {t('proxy.router.current_list')}
                            </span>
                        </div>
                        <div className="overflow-y-auto max-h-[180px] border border-gray-100 dark:border-white/5 rounded-lg bg-gray-50/10 dark:bg-white/5 p-3" data-custom-mapping-list>
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-2">
                                {config.proxy.custom_mapping && Object.entries(config.proxy.custom_mapping).length > 0 ? (
                                    Object.entries(config.proxy.custom_mapping).map(([key, val]) => (
                                        <div key={key} className={`flex items-center justify-between p-1.5 rounded-md transition-all border group ${editingKey === key ? 'bg-blue-50/80 dark:bg-blue-900/15 border-blue-300/50 dark:border-blue-500/30 shadow-sm' : 'border-transparent hover:bg-gray-100 dark:hover:bg-white/5 hover:border-gray-200 dark:hover:border-white/10'}`}>
                                            <div className="flex items-center gap-2 overflow-hidden flex-1">
                                                <div className="flex items-center gap-1.5 min-w-0">
                                                    <span className="font-mono text-[10px] font-bold text-blue-600 dark:text-blue-400 truncate max-w-[140px]" title={key}>{key}</span>
                                                    {key.toLowerCase() === BUILT_IN_WILDCARD_KEY && (
                                                        <span className="badge badge-warning badge-outline text-[11px] font-bold font-mono py-0.5 px-1.5 h-5 shrink-0 ml-0.5 cursor-help shadow-xs" title={t('proxy.router.wildcard_rule_notice') || "x 必须大于 8，自动转为 3.x-flash-tiered"}>
                                                            x &gt; 8
                                                        </span>
                                                    )}
                                                </div>
                                                <ArrowRight size={10} className="text-gray-300 dark:text-gray-600 shrink-0" />

                                                {editingKey === key ? (
                                                    <div className="flex-1 mr-2">
                                                        <GroupedSelect
                                                            value={editingValue}
                                                            onChange={setEditingValue}
                                                            options={customMappingOptions}
                                                            placeholder="Select..."
                                                            className="font-mono text-[10px] h-7 dark:bg-gray-800 border-blue-200 dark:border-blue-800"
                                                            allowCustomInput={true}
                                                        />
                                                    </div>
                                                ) : (
                                                    <span className="font-mono text-[10px] text-gray-500 dark:text-gray-400 truncate cursor-pointer hover:text-blue-500"
                                                        onClick={() => { setEditingKey(key); setEditingValue(val); }}
                                                        title={val}>{val}</span>
                                                )}
                                            </div>

                                            <div className="flex items-center gap-1.5 shrink-0">
                                                {editingKey === key ? (
                                                    <div className="flex items-center gap-1 bg-white dark:bg-gray-800 rounded-md border border-blue-200 dark:border-blue-800 p-0.5 shadow-sm">
                                                        <button
                                                            className="btn btn-ghost btn-xs text-primary hover:bg-blue-50 dark:hover:bg-blue-900/30 p-0 h-6 w-6 min-h-0"
                                                            onClick={() => {
                                                                onMappingUpdate('custom', key, editingValue);
                                                                setEditingKey(null);
                                                            }}
                                                            title={t('common.save') || 'Save'}
                                                        >
                                                            <Check size={14} strokeWidth={3} />
                                                        </button>
                                                        <div className="w-[1px] h-3 bg-gray-200 dark:bg-gray-700" />
                                                        <button
                                                            className="btn btn-ghost btn-xs text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 p-0 h-6 w-6 min-h-0"
                                                            onClick={() => setEditingKey(null)}
                                                            title={t('common.cancel') || 'Cancel'}
                                                        >
                                                            <X size={14} strokeWidth={3} />
                                                        </button>
                                                    </div>
                                                ) : (
                                                    <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                                        <button
                                                            className="btn btn-ghost btn-xs text-gray-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-white/10 p-0 h-6 w-6 min-h-0"
                                                            onClick={() => { setEditingKey(key); setEditingValue(val); }}
                                                            title={t('common.edit') || 'Edit'}
                                                        >
                                                            <Edit2 size={12} />
                                                        </button>
                                                        <button
                                                            className="btn btn-ghost btn-xs text-error hover:bg-red-50 dark:hover:bg-red-900/20 p-0 h-6 w-6 min-h-0"
                                                            onClick={() => onRemoveCustomMapping(key)}
                                                            title={t('common.delete') || 'Delete'}
                                                        >
                                                            <Trash2 size={12} />
                                                        </button>
                                                    </div>
                                                )}
                                            </div>
                                        </div>
                                    ))
                                ) : (
                                    <div className="col-span-full text-center py-4 text-gray-400 dark:text-gray-600 italic text-[11px]">{t('proxy.router.no_custom_mapping')}</div>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* 添加映射表单 (置底单行) */}
                    <div className="w-full bg-gray-50/50 dark:bg-white/5 p-2.5 rounded-xl border border-gray-100 dark:border-white/5 shadow-inner">
                        <div className="flex flex-col sm:flex-row items-center gap-3">
                            <div className="flex items-center gap-1.5 shrink-0">
                                <Target size={14} className="text-gray-400 dark:text-gray-500" />
                                <span className="text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider">{t('proxy.router.add_mapping')}</span>
                            </div>
                            <div className="flex-1 flex flex-col sm:flex-row gap-2 w-full">
                                <input
                                    id="custom-key"
                                    type="text"
                                    placeholder={t('proxy.router.original_placeholder') || "Original (e.g. gpt-4 or gpt-4*)"}
                                    className="input input-xs input-bordered flex-1 font-mono text-[11px] bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 shadow-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all placeholder:text-gray-400 dark:placeholder:text-gray-600 h-8"
                                />
                                <div className="w-full sm:w-48">
                                    <GroupedSelect
                                        value={customMappingValue}
                                        onChange={setCustomMappingValue}
                                        options={customMappingOptions}
                                        placeholder={t('proxy.router.select_target_model') || 'Select Target Model'}
                                        className="font-mono text-[11px] h-8 dark:bg-gray-800"
                                        allowCustomInput={true}
                                    />
                                </div>
                            </div>
                            <button
                                className="btn btn-xs sm:w-20 gap-1.5 shadow-md hover:shadow-lg transition-all bg-blue-600 hover:bg-blue-700 text-white border-none h-8"
                                onClick={() => {
                                    const k = (document.getElementById('custom-key') as HTMLInputElement).value;
                                    const v = customMappingValue;
                                    if (k && v) {
                                        onMappingUpdate('custom', k, v);
                                        (document.getElementById('custom-key') as HTMLInputElement).value = '';
                                        setCustomMappingValue(''); // 清空选择
                                    }
                                }}
                            >
                                <Plus size={14} />
                                {t('common.add')}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
