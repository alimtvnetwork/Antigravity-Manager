import { useEffect, type ReactNode } from 'react';
import { ChevronDown, Laptop, Pencil, X } from 'lucide-react';
import { cn } from '../../utils/cn';
import type { InstancePageApi } from './instancePageTypes';

const fieldClass =
    'w-full h-10 px-3.5 text-sm font-medium rounded-xl bg-slate-50 dark:bg-[#090611] text-slate-900 dark:text-slate-100 border border-slate-200/80 dark:border-[#40285e] placeholder:text-slate-400 dark:placeholder:text-slate-500 outline-none focus:outline-none focus:ring-0 focus:border-[#7c3aed]/65 dark:focus:border-[#a855f7]/75 shadow-[inset_0_1px_0_rgb(255_255_255_/_0.04)] focus:shadow-[0_0_0_1px_rgb(124_58_237_/_0.28),0_8px_20px_-12px_rgb(124_58_237_/_0.45)] transition-[border-color,background-color,box-shadow] duration-200';

function FieldLabel({ children }: { children: string }) {
    return (
        <label className="block text-[11px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400 mb-1.5">
            {children}
        </label>
    );
}

function SelectField({
    value,
    onChange,
    children,
}: {
    value: string;
    onChange: (value: string) => void;
    children: ReactNode;
}) {
    return (
        <div className="relative mb-4">
            <select
                value={value}
                onChange={(e) => onChange(e.target.value)}
                className={cn(fieldClass, 'appearance-none pr-9 cursor-pointer')}
            >
                {children}
            </select>
            <ChevronDown className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400" />
        </div>
    );
}

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

    useEffect(() => {
        if (isCreateOpen && !newInstanceFromInstance) {
            const defaultInst = instances.find((i) => i.config.is_default || i.config.id === 'default');
            setNewInstanceFromInstance(defaultInst?.config.id || 'default');
        }
    }, [isCreateOpen, newInstanceFromInstance, instances, setNewInstanceFromInstance]);

    if (!isCreateOpen) return null;

    const close = () => {
        setIsCreateOpen(false);
        setNewInstanceName('');
        setNewInstanceBoundAccount('');
        const defaultInst = instances.find((i) => i.config.is_default || i.config.id === 'default');
        setNewInstanceFromInstance(defaultInst?.config.id || 'default');
        setNewInstanceLaunchImmediately(false);
    };

    return (
        <div
            className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[99999] p-4 overflow-y-auto"
            onClick={close}
        >
            <div
                className="w-full max-w-md rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between px-5 pt-4 pb-3 border-b border-slate-200/80 dark:border-[#40285e]">
                    <div className="flex items-center gap-2.5">
                        <span className="flex h-8 w-8 items-center justify-center rounded-xl bg-blue-500/10 dark:bg-cyan-400/10 text-blue-600 dark:text-cyan-300">
                            <Laptop className="w-4 h-4" />
                        </span>
                        <h3 className="font-bold text-base text-slate-900 dark:text-white">
                            {t('instances.create_modal_title', 'Create New Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={close}
                        className="agm-hover-shade p-1.5 rounded-lg text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <div className="px-5 py-4">
                    <p className="text-xs text-slate-500 dark:text-slate-400 mb-4 leading-relaxed">
                        Creates an isolated Antigravity profile folder with its own SQLite token store, extensions, and
                        configuration.
                    </p>

                    <FieldLabel>Profile Name</FieldLabel>
                    <input
                        type="text"
                        placeholder={t('instances.name_placeholder', 'Profile name (e.g., Work, Personal)')}
                        value={newInstanceName}
                        onChange={(e) => setNewInstanceName(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                        className={cn(fieldClass, 'mb-4')}
                        autoFocus
                    />

                    <FieldLabel>Initial Account (Optional)</FieldLabel>
                    <SelectField value={newInstanceBoundAccount} onChange={setNewInstanceBoundAccount}>
                        <option value="">Auto-assign next available account</option>
                        {(accounts as Array<{ id: string; email: string; quota?: { subscription_tier?: string } }>).map(
                            (acc) => (
                                <option key={acc.id} value={acc.id}>
                                    {acc.email} ({acc.quota?.subscription_tier || 'FREE'})
                                </option>
                            )
                        )}
                    </SelectField>

                    <FieldLabel>Clone Settings & Extensions From (Optional)</FieldLabel>
                    <SelectField value={newInstanceFromInstance} onChange={setNewInstanceFromInstance}>
                        <option value="">Blank profile (clean isolated sandbox)</option>
                        {instances.map((inst) => (
                            <option key={inst.config.id} value={inst.config.id}>
                                {inst.config.name} {inst.config.is_default ? '(Default)' : ''}
                            </option>
                        ))}
                    </SelectField>

                    <label
                        htmlFor="launch-immediately-check"
                        className="agm-hover-shade mb-5 flex items-center gap-2.5 px-3 py-2 rounded-xl border border-slate-200/80 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#090611]/80 cursor-pointer select-none"
                    >
                        <input
                            type="checkbox"
                            id="launch-immediately-check"
                            checked={newInstanceLaunchImmediately}
                            onChange={(e) => setNewInstanceLaunchImmediately(e.target.checked)}
                            className="h-3.5 w-3.5 rounded border-slate-300 dark:border-[#40285e] text-violet-600 focus:ring-0 focus:outline-none cursor-pointer"
                        />
                        <span className="text-xs text-slate-700 dark:text-slate-300">
                            Launch Antigravity window immediately after creation
                        </span>
                    </label>

                    <div className="flex justify-end">
                        <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] overflow-hidden divide-x divide-slate-200 dark:divide-[#40285e]">
                            <button
                                type="button"
                                onClick={close}
                                className="agm-hover-shade px-4 py-1.5 text-xs font-semibold text-slate-600 dark:text-slate-300 cursor-pointer"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleCreate}
                                disabled={!newInstanceName.trim()}
                                className="agm-hover-shade px-4 py-1.5 text-xs font-semibold bg-blue-600 text-white disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                            >
                                {t('common.create', 'Create')}
                            </button>
                        </div>
                    </div>
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
        <div
            className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[99999] p-4 overflow-y-auto"
            onClick={close}
        >
            <div
                className="w-full max-w-md rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between px-5 pt-4 pb-3 border-b border-slate-200/80 dark:border-[#40285e]">
                    <div className="flex items-center gap-2.5">
                        <span className="flex h-8 w-8 items-center justify-center rounded-xl bg-blue-500/10 dark:bg-cyan-400/10 text-blue-600 dark:text-cyan-300">
                            <Pencil className="w-4 h-4" />
                        </span>
                        <h3 className="font-bold text-base text-slate-900 dark:text-white">
                            {t('instances.edit_modal_title', 'Rename Profile')}
                        </h3>
                    </div>
                    <button
                        type="button"
                        onClick={close}
                        className="agm-hover-shade p-1.5 rounded-lg text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
                <div className="px-5 py-4">
                    <p className="text-xs text-slate-500 dark:text-slate-400 mb-4">
                        {t('instances.edit_modal_desc', 'Update display name for this isolated instance profile.')}
                    </p>
                    <input
                        type="text"
                        placeholder={t('instances.edit_placeholder', 'New profile name')}
                        value={editInstanceName}
                        onChange={(e) => setEditInstanceName(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && handleEdit()}
                        className={cn(fieldClass, 'mb-5')}
                        autoFocus
                    />
                    <div className="flex justify-end">
                        <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] overflow-hidden divide-x divide-slate-200 dark:divide-[#40285e]">
                            <button
                                type="button"
                                onClick={close}
                                className="agm-hover-shade px-4 py-1.5 text-xs font-semibold text-slate-600 dark:text-slate-300 cursor-pointer"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleEdit}
                                disabled={!editInstanceName.trim()}
                                className="agm-hover-shade px-4 py-1.5 text-xs font-semibold bg-blue-600 text-white disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                            >
                                {t('common.save', 'Save')}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
