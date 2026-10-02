import { useMemo, useState } from 'react';
import { Pin, Check, Search } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { PinnedQuotaModelsConfig } from '../../types/config';
import { MODEL_CONFIG } from '../../config/modelConfig';
import { useAccountStore } from '../../stores/useAccountStore';

interface PinnedQuotaModelsProps {
    config: PinnedQuotaModelsConfig;
    onChange: (config: PinnedQuotaModelsConfig) => void;
}

const PinnedQuotaModels = ({ config, onChange }: PinnedQuotaModelsProps) => {
    const { t } = useTranslation();
    const [query, setQuery] = useState('');
    const { accounts } = useAccountStore();

    const toggleModel = (model: string) => {
        const currentModels = config.models || [];
        if (currentModels.includes(model)) {
            if (currentModels.length <= 1) return;
            onChange({ ...config, models: currentModels.filter(m => m !== model) });
            return;
        }
        onChange({ ...config, models: [...currentModels, model] });
    };

    const modelOptions = useMemo(() => {
        const uniqueIds = new Set<string>();
        Object.entries(MODEL_CONFIG).forEach(([id, cfg]) => {
            uniqueIds.add(id.toLowerCase());
            if (cfg.protectedKey) uniqueIds.add(cfg.protectedKey.toLowerCase());
        });

        const addedDisplayLabels = new Set<string>();
        const baseModels = Object.entries(MODEL_CONFIG)
            .filter(([id, cfg]) => {
                if (id.includes('thinking')) return false;
                const labelKey = (cfg.shortLabel || cfg.label).toLowerCase();
                if (addedDisplayLabels.has(labelKey)) return false;
                addedDisplayLabels.add(labelKey);
                return true;
            })
            .map(([id, cfg]) => ({
                id,
                label: id,
                desc: cfg.shortLabel || cfg.label || t(cfg.i18nDescKey || cfg.i18nKey, cfg.label),
            }));

        const dynamicModels = accounts.flatMap(a => a.quota?.models || [])
            .filter(m => {
                const id = m.name.toLowerCase();
                if (id.includes('thinking')) return false;
                if (uniqueIds.has(id)) return false;
                uniqueIds.add(id);
                return true;
            })
            .map(m => ({
                id: m.name.toLowerCase(),
                label: m.name.toLowerCase(),
                desc: m.display_name || t('settings.pinned_quota_models.dynamic', 'Dynamic Extracted Model'),
            }));

        const options = [...baseModels, ...dynamicModels];
        (config.models || []).forEach(modelId => {
            if (!options.some(m => m.id === modelId)) {
                const quotaModel = accounts.flatMap(a => a.quota?.models || []).find(m => m.name.toLowerCase() === modelId.toLowerCase());
                const cfg = MODEL_CONFIG[modelId.toLowerCase()];
                options.push({
                    id: modelId,
                    label: modelId,
                    desc: quotaModel?.display_name || cfg?.shortLabel || cfg?.label || t('common.unknown', 'Unknown'),
                });
            }
        });
        return options;
    }, [accounts, config.models, t]);

    const needle = query.trim().toLowerCase();
    const visible = needle
        ? modelOptions.filter(model => model.label.toLowerCase().includes(needle) || model.desc.toLowerCase().includes(needle))
        : modelOptions;
    const pinned = new Set(config.models || []);
    const ordered = [...visible].sort((a, b) => Number(pinned.has(b.id)) - Number(pinned.has(a.id)));

    return (
        <div>
            <div className="flex items-center justify-between gap-3">
                <div className="flex items-center gap-2 min-w-0">
                    <Pin size={14} className="text-indigo-400 shrink-0" />
                    <div className="min-w-0">
                        <div className="text-sm font-semibold text-gray-900 dark:text-gray-100 leading-tight">
                            {t('settings.pinned_quota_models.title')}
                        </div>
                        <p className="text-[11px] text-gray-500 dark:text-gray-400 truncate">
                            {t('settings.pinned_quota_models.desc')}
                        </p>
                    </div>
                </div>
                <span className="text-[11px] text-gray-400 shrink-0">{pinned.size} pinned</span>
            </div>

            <div className="mt-3 relative">
                <Search size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-400" />
                <input
                    value={query}
                    onChange={(event) => setQuery(event.target.value)}
                    placeholder="Filter models"
                    className="w-full pl-8 pr-2 py-1.5 text-xs rounded-md border border-gray-200 dark:border-base-300 bg-white dark:bg-base-200 text-gray-900 dark:text-base-content focus:outline-none focus:ring-1 focus:ring-indigo-500"
                />
            </div>

            <div className="mt-2 flex flex-wrap gap-1.5">
                {ordered.map((model) => {
                    const isSelected = pinned.has(model.id);
                    return (
                        <button
                            key={model.id}
                            type="button"
                            title={model.desc}
                            onClick={() => toggleModel(model.id)}
                            className={`inline-flex items-center gap-1 max-w-full px-2 py-1 rounded-md border text-[11px] leading-none transition-colors ${isSelected
                                ? 'bg-indigo-50 dark:bg-indigo-500/15 border-indigo-400 text-indigo-700 dark:text-indigo-200'
                                : 'bg-gray-50 dark:bg-white/5 border-gray-200 dark:border-white/10 text-gray-600 dark:text-gray-400 hover:border-gray-400'
                                }`}
                        >
                            {isSelected && <Check size={11} strokeWidth={3} className="shrink-0" />}
                            <span className="truncate">{model.label}</span>
                        </button>
                    );
                })}
            </div>
        </div>
    );
};

export default PinnedQuotaModels;
