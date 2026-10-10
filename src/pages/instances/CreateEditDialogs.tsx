import { Laptop, Pencil, X } from 'lucide-react';
import type { InstancePageApi } from './instancePageTypes';

export function CreateInstanceDialog({ api }: { api: InstancePageApi }) {
    const {
        t,
        isCreateOpen,
        setIsCreateOpen,
        newInstanceName,
        setNewInstanceName,
        newInstanceBoundAccount,
        setNewInstanceBoundAccount,
        newInstanceFromInstance,
        setNewInstanceFromInstance,
        newInstanceLaunchImmediately,
        setNewInstanceLaunchImmediately,
        accounts,
        instances,
        handleCreate,
    } = api;

    if (!isCreateOpen) return null;

    const close = () => {
        setIsCreateOpen(false);
        setNewInstanceName('');
        setNewInstanceBoundAccount('');
        setNewInstanceFromInstance('');
        setNewInstanceLaunchImmediately(false);
    };

    return (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4" onClick={close}>
            <div
                className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100 mb-4">
                    <div className="flex items-center gap-2.5">
                        <Laptop className="w-5 h-5 text-blue-600" />
                        <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                            {t('instances.create_modal_title', 'Create New Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={close}
                        className="btn btn-ghost btn-xs btn-circle text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                    Creates an isolated Antigravity profile folder with its own SQLite token store, extensions, and configuration.
                </p>
                <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">Profile Name</label>
                <input
                    type="text"
                    placeholder={t('instances.name_placeholder', 'Profile name (e.g., Personal, Client Work)')}
                    value={newInstanceName}
                    onChange={(e) => setNewInstanceName(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                    className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-sm"
                    autoFocus
                />
                <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">
                    Initial Account (Optional)
                </label>
                <select
                    value={newInstanceBoundAccount}
                    onChange={(e) => setNewInstanceBoundAccount(e.target.value)}
                    className="select select-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-xs"
                >
                    <option value="">Auto-assign next available account</option>
                    {(accounts as Array<{ id: string; email: string; quota?: { subscription_tier?: string } }>).map((acc) => (
                        <option key={acc.id} value={acc.id}>
                            {acc.email} ({acc.quota?.subscription_tier || 'FREE'})
                        </option>
                    ))}
                </select>

                <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">
                    Clone Settings & Extensions From (Optional)
                </label>
                <select
                    value={newInstanceFromInstance}
                    onChange={(e) => setNewInstanceFromInstance(e.target.value)}
                    className="select select-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-xs"
                >
                    <option value="">Blank profile (clean isolated sandbox)</option>
                    {instances.map((inst) => (
                        <option key={inst.config.id} value={inst.config.id}>
                            {inst.config.name} {inst.config.is_default ? '(Default)' : ''}
                        </option>
                    ))}
                </select>

                <div className="flex items-center gap-2 mb-5">
                    <input
                        type="checkbox"
                        id="launch-immediately-check"
                        checked={newInstanceLaunchImmediately}
                        onChange={(e) => setNewInstanceLaunchImmediately(e.target.checked)}
                        className="checkbox checkbox-sm checkbox-primary rounded cursor-pointer"
                    />
                    <label
                        htmlFor="launch-immediately-check"
                        className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer select-none"
                    >
                        Launch Antigravity window immediately after creation
                    </label>
                </div>

                <div className="flex justify-end gap-2.5">
                    <button
                        onClick={close}
                        className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400 rounded-[5px]"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button onClick={handleCreate} disabled={!newInstanceName.trim()} className="btn btn-primary btn-sm rounded-[5px]">
                        {t('common.create', 'Create Profile')}
                    </button>
                </div>
            </div>
        </div>
    );
}

export function EditInstanceDialog({ api }: { api: InstancePageApi }) {
    const { t, editTargetId, setEditTargetId, editInstanceName, setEditInstanceName, handleEdit } = api;

    if (!editTargetId) return null;

    const close = () => {
        setEditTargetId(null);
        setEditInstanceName('');
    };

    return (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4" onClick={close}>
            <div
                className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100 mb-4">
                    <div className="flex items-center gap-2.5">
                        <Pencil className="w-5 h-5 text-blue-600" />
                        <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                            {t('instances.edit_modal_title', 'Rename Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={close}
                        className="btn btn-ghost btn-xs btn-circle text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                    {t('instances.edit_modal_desc', 'Update display name for this isolated instance profile.')}
                </p>
                <input
                    type="text"
                    placeholder={t('instances.edit_placeholder', 'New profile name')}
                    value={editInstanceName}
                    onChange={(e) => setEditInstanceName(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleEdit()}
                    className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-5 text-sm"
                    autoFocus
                />
                <div className="flex justify-end gap-2.5">
                    <button
                        onClick={() => setEditTargetId(null)}
                        className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400 rounded-[5px]"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button
                        onClick={handleEdit}
                        disabled={!editInstanceName.trim()}
                        className="btn btn-primary btn-sm rounded-[5px]"
                    >
                        {t('common.save', 'Save')}
                    </button>
                </div>
            </div>
        </div>
    );
}
