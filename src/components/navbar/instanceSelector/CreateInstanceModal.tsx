import { Laptop, X } from 'lucide-react';
import type { InstanceSelectorApi } from './useInstanceSelector';

export function CreateInstanceModal({ api }: { api: InstanceSelectorApi }) {
    const { t, isCreateOpen, setIsCreateOpen, newInstanceName, setNewInstanceName, createMode, setCreateMode, handleCreate } = api;

    if (!isCreateOpen) return null;

    return (
        <div
            className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={() => setIsCreateOpen(false)}
        >
            <div
                className="bg-white dark:bg-[#071a27] rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-[#15334d]"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <Laptop className="w-5 h-5 text-blue-600 dark:text-cyan-400" />
                        <h3 className="font-bold text-sm text-gray-900 dark:text-white">
                            {t('instances.create_modal_title', 'Create New Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={() => setIsCreateOpen(false)}
                        className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-slate-200 hover:bg-gray-100 dark:hover:bg-[#0c2438] transition-colors cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
                <div className="flex gap-2 mb-3">
                    <button
                        type="button"
                        onClick={() => setCreateMode('clone-default')}
                        className={`flex-1 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold border transition-all ${
                            createMode === 'clone-default'
                                ? 'bg-blue-50 dark:bg-[#092236] text-blue-900 dark:text-cyan-300 border-blue-500 dark:border-cyan-400 shadow-xs'
                                : 'bg-gray-50 dark:bg-[#061521] text-gray-600 dark:text-slate-300 border-gray-200 dark:border-[#15334d] hover:bg-gray-100 dark:hover:bg-[#091f30]'
                        }`}
                    >
                        {t('instances.clone_default', 'Clone from default')}
                    </button>
                    <button
                        type="button"
                        onClick={() => setCreateMode('new')}
                        className={`flex-1 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold border transition-all ${
                            createMode === 'new'
                                ? 'bg-blue-50 dark:bg-[#092236] text-blue-900 dark:text-cyan-300 border-blue-500 dark:border-cyan-400 shadow-xs'
                                : 'bg-gray-50 dark:bg-[#061521] text-gray-600 dark:text-slate-300 border-gray-200 dark:border-[#15334d] hover:bg-gray-100 dark:hover:bg-[#091f30]'
                        }`}
                    >
                        {t('instances.create_empty', 'New empty')}
                    </button>
                </div>
                <input
                    type="text"
                    placeholder={t('instances.name_placeholder', 'Profile name (e.g., Work, Client B)')}
                    value={newInstanceName}
                    onChange={(e) => setNewInstanceName(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                    className="w-full px-3.5 py-2 bg-gray-50 dark:bg-[#040e16] border border-gray-200 dark:border-[#15334d] text-gray-900 dark:text-slate-100 rounded-xl mb-4 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-blue-500/40 focus:border-blue-500 dark:focus:border-cyan-400 transition-all shadow-xs placeholder-gray-400 dark:placeholder-slate-500"
                    autoFocus
                />
                <div className="flex justify-end gap-2">
                    <button
                        type="button"
                        onClick={() => setIsCreateOpen(false)}
                        className="px-3.5 py-1.5 rounded-xl text-xs font-medium text-gray-600 dark:text-slate-300 border border-gray-200 dark:border-[#15334d] hover:bg-gray-100 dark:hover:bg-[#0c2438] transition-all cursor-pointer"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button
                        type="button"
                        onClick={handleCreate}
                        disabled={!newInstanceName.trim()}
                        className="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs rounded-xl shadow-xs transition-all disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                    >
                        {t('common.create', 'Create')}
                    </button>
                </div>
            </div>
        </div>
    );
}
