import { Pencil, X, AlertTriangle } from 'lucide-react';
import type { InstanceSelectorApi } from './useInstanceSelector';

export function EditInstanceModal({ api }: { api: InstanceSelectorApi }) {
    const { t, isEditOpen, setIsEditOpen, editInstanceName, setEditInstanceName, handleEdit } = api;

    if (!isEditOpen) return null;

    return (
        <div
            className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={() => setIsEditOpen(false)}
        >
            <div
                className="bg-white dark:bg-[#0c2438] rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-[#15334d]"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <Pencil className="w-5 h-5 text-blue-600" />
                        <h3 className="font-bold text-sm text-gray-900 dark:text-slate-100">
                            {t('instances.edit_modal_title', 'Rename Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={() => setIsEditOpen(false)}
                        className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
                <input
                    type="text"
                    placeholder={t('instances.edit_placeholder', 'Profile name')}
                    value={editInstanceName}
                    onChange={(e) => setEditInstanceName(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleEdit()}
                    className="input input-sm w-full bg-gray-50 dark:bg-[#071a27] border border-gray-200 dark:border-[#15334d] text-gray-900 dark:text-slate-100 rounded-lg mb-4 text-xs"
                    autoFocus
                />
                <div className="flex justify-end gap-2">
                    <button
                        type="button"
                        onClick={() => setIsEditOpen(false)}
                        className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button
                        type="button"
                        onClick={handleEdit}
                        disabled={!editInstanceName.trim()}
                        className="btn btn-primary btn-xs"
                    >
                        {t('common.save', 'Save')}
                    </button>
                </div>
            </div>
        </div>
    );
}

export function DeleteInstanceModal({ api }: { api: InstanceSelectorApi }) {
    const { t, isDeleteOpen, deleteTarget, handleDelete, closeDelete } = api;

    if (!isDeleteOpen || !deleteTarget) return null;

    return (
        <div
            className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={closeDelete}
        >
            <div
                className="bg-white dark:bg-[#0c2438] rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-[#15334d]"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between mb-3 text-red-600">
                    <div className="flex items-center gap-2">
                        <AlertTriangle className="w-5 h-5" />
                        <h3 className="font-bold text-sm text-gray-900 dark:text-slate-100">
                            {t('instances.delete_modal_title', 'Delete Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={closeDelete}
                        className="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
                <p className="text-xs text-gray-600 dark:text-gray-300 mb-4 leading-relaxed">
                    {t(
                        'instances.delete_confirm_desc',
                        `Are you sure you want to delete profile "${deleteTarget.config.name}"? All isolated data and workspace sessions will be deleted.`
                    )}
                </p>
                <div className="flex justify-end gap-2">
                    <button
                        type="button"
                        onClick={closeDelete}
                        className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button type="button" onClick={handleDelete} className="btn btn-error btn-xs text-white">
                        {t('common.delete', 'Delete')}
                    </button>
                </div>
            </div>
        </div>
    );
}
