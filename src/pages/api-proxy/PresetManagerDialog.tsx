import type { TFunction } from 'i18next';
import { Save, Trash2 } from 'lucide-react';
import ModalDialog from '../../components/common/ModalDialog';
import type { PresetManager } from './types';

interface PresetManagerDialogProps {
    t: TFunction;
    presetManager: PresetManager;
}

export function PresetManagerDialog({ t, presetManager }: PresetManagerDialogProps) {
    const {
        customPresets,
        newPresetName,
        setNewPresetName,
        isPresetManagerOpen,
        setIsPresetManagerOpen,
        handleSaveCurrentAsPreset,
        handleDeletePreset,
    } = presetManager;

    return (
        <ModalDialog
            isOpen={isPresetManagerOpen}
            title={t('proxy.router.manage_presets_title')}
            onConfirm={() => setIsPresetManagerOpen(false)}
            confirmText={t('common.close')}
            type="info"
        >
            <div className="space-y-6">
                {/* Save Current Section */}
                <div className="space-y-3 p-4 bg-blue-50/50 dark:bg-blue-900/10 rounded-xl border border-blue-100 dark:border-blue-900/20">
                    <h3 className="text-sm font-bold text-gray-800 dark:text-gray-200 flex items-center gap-2">
                        <Save size={16} className="text-blue-500" />
                        {t('proxy.router.save_current_as_preset')}
                    </h3>
                    <div className="flex gap-2">
                        <input
                            type="text"
                            value={newPresetName}
                            onChange={(e) => setNewPresetName(e.target.value)}
                            placeholder={t('proxy.router.preset_name_placeholder')}
                            className="input input-sm flex-1 border-gray-300 focus:border-blue-500"
                        />
                        <button
                            onClick={handleSaveCurrentAsPreset}
                            disabled={!newPresetName.trim()}
                            className="btn btn-sm btn-primary text-white"
                        >
                            {t('common.save')}
                        </button>
                    </div>
                    <p className="text-[10px] text-gray-500 dark:text-gray-400">
                        {t('proxy.router.save_hint')}
                    </p>
                </div>

                {/* Existing Presets List */}
                <div className="space-y-3">
                    <h3 className="text-sm font-bold text-gray-800 dark:text-gray-200 px-1">
                        {t('proxy.router.your_presets')}
                    </h3>
                    <div className="max-h-[300px] overflow-y-auto space-y-2 pr-1">
                        {customPresets.length === 0 ? (
                            <div className="text-center py-8 text-gray-400 dark:text-gray-600 bg-gray-50 dark:bg-base-200 rounded-xl border border-dashed border-gray-200 dark:border-gray-700">
                                <p>{t('proxy.router.no_custom_presets')}</p>
                            </div>
                        ) : (
                            customPresets.map(preset => (
                                <div key={preset.id} className="flex items-center justify-between p-3 bg-white dark:bg-base-200 border border-gray-100 dark:border-gray-700 rounded-xl hover:shadow-sm transition-all group">
                                    <div className="flex-1 min-w-0">
                                        <div className="font-bold text-sm text-gray-800 dark:text-gray-200 truncate">{preset.name}</div>
                                        <div className="text-[10px] text-gray-400 dark:text-gray-500 truncate">
                                            {Object.keys(preset.mappings).length} {t('proxy.router.mappings_count')}
                                        </div>
                                    </div>
                                    <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            onClick={() => handleDeletePreset(preset.id)}
                                            className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                                            title={t('common.delete')}
                                        >
                                            <Trash2 size={16} />
                                        </button>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </ModalDialog>
    );
}
