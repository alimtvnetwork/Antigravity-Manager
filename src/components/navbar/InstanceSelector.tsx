import { useState, useEffect, useRef } from 'react';
import { ChevronDown, Copy, Plus, Check, Laptop } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { isTauri } from '../../utils/env';

export function InstanceSelector() {
    const { t } = useTranslation();
    const {
        instances,
        activeInstanceId,
        fetchInstances,
        setActiveInstance,
        createInstance,
        copyInstance,
    } = useInstanceStore();

    const [isOpen, setIsOpen] = useState(false);
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [isCopyOpen, setIsCopyOpen] = useState(false);
    const [newInstanceName, setNewInstanceName] = useState('');
    const [copyInstanceName, setCopyInstanceName] = useState('');
    const dropdownRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!isTauri()) return;
        fetchInstances();
        const interval = setInterval(fetchInstances, 4000);
        return () => clearInterval(interval);
    }, [fetchInstances]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const activeInstance = instances.find(i => i.config.id === activeInstanceId) || instances[0];

    const handleCreate = async () => {
        if (!newInstanceName.trim()) return;
        try {
            const created = await createInstance(newInstanceName.trim());
            await setActiveInstance(created.id);
            setNewInstanceName('');
            setIsCreateOpen(false);
        } catch (e) {
            console.error('Failed to create instance:', e);
        }
    };

    const handleCopy = async () => {
        if (!copyInstanceName.trim() || !activeInstance) return;
        try {
            const copied = await copyInstance(activeInstance.config.id, copyInstanceName.trim());
            await setActiveInstance(copied.id);
            setCopyInstanceName('');
            setIsCopyOpen(false);
        } catch (e) {
            console.error('Failed to copy instance:', e);
        }
    };

    if (!isTauri()) return null;

    return (
        <div className="relative flex items-center gap-1 shrink-0" ref={dropdownRef}>
            {/* Instance Dropdown Button */}
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="flex items-center gap-1.5 md:gap-2 px-2.5 md:px-3 py-1.5 rounded-lg text-xs font-medium bg-gray-100 dark:bg-base-200 hover:bg-gray-200 dark:hover:bg-base-100 transition-colors border border-gray-200/60 dark:border-base-100 shrink-0"
                title={t('instances.selector_tooltip', 'Select active Antigravity instance')}
            >
                <span
                    className={`w-2 h-2 rounded-full shrink-0 ${activeInstance?.is_running ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'}`}
                />
                <span className="truncate max-w-[90px] md:max-w-[120px] text-gray-800 dark:text-gray-200">
                    {activeInstance?.config.name || 'Default'}
                </span>
                <ChevronDown className="w-3.5 h-3.5 text-gray-500 shrink-0" />
            </button>

            {/* Quick Copy Button */}
            <button
                onClick={() => {
                    setCopyInstanceName(`${activeInstance?.config.name || 'Instance'} Copy`);
                    setIsCopyOpen(true);
                }}
                className="p-1.5 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-base-100 transition-colors shrink-0"
                title={t('instances.copy_current', 'Clone current instance profile')}
            >
                <Copy className="w-3.5 h-3.5" />
            </button>

            {/* Quick New Button */}
            <button
                onClick={() => {
                    setNewInstanceName('');
                    setIsCreateOpen(true);
                }}
                className="p-1.5 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-base-100 transition-colors shrink-0"
                title={t('instances.create_new', 'Create new isolated instance')}
            >
                <Plus className="w-3.5 h-3.5" />
            </button>

            {/* Dropdown Menu */}
            {isOpen && (
                <div className="absolute top-full right-0 mt-1.5 w-64 rounded-xl shadow-xl bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 py-2 z-50 animate-in fade-in zoom-in-95">
                    <div className="px-3 py-1 text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
                        {t('instances.title', 'Instances / Profiles')}
                    </div>
                    <div className="max-h-56 overflow-y-auto py-1">
                        {instances.map((inst) => {
                            const isSelected = inst.config.id === activeInstanceId;
                            return (
                                <button
                                    key={inst.config.id}
                                    onClick={() => {
                                        setActiveInstance(inst.config.id);
                                        setIsOpen(false);
                                    }}
                                    className={`w-full flex items-center justify-between px-3 py-2 text-xs text-left transition-colors hover:bg-gray-50 dark:hover:bg-base-100 ${isSelected ? 'bg-blue-50/60 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-medium' : 'text-gray-700 dark:text-gray-300'}`}
                                >
                                    <div className="flex items-center gap-2 truncate">
                                        <span
                                            className={`w-2 h-2 rounded-full shrink-0 ${inst.is_running ? 'bg-emerald-500' : 'bg-gray-400'}`}
                                        />
                                        <div className="flex flex-col truncate">
                                            <span className="truncate">{inst.config.name}</span>
                                            {inst.config.bound_email && (
                                                <span className="text-[10px] text-gray-400 truncate">
                                                    {inst.config.bound_email}
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                    {isSelected && <Check className="w-3.5 h-3.5 text-blue-600 shrink-0" />}
                                </button>
                            );
                        })}
                    </div>
                </div>
            )}

            {/* Create Instance Modal */}
            {isCreateOpen && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2 mb-3">
                            <Laptop className="w-5 h-5 text-blue-600" />
                            <h3 className="font-bold text-sm text-gray-900 dark:text-base-content">
                                {t('instances.create_modal_title', 'Create New Profile')}
                            </h3>
                        </div>
                        <input
                            type="text"
                            placeholder={t('instances.name_placeholder', 'Profile name (e.g., Work, Client B)')}
                            value={newInstanceName}
                            onChange={(e) => setNewInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                            className="input input-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-lg mb-4 text-xs"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => setIsCreateOpen(false)}
                                className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCreate}
                                disabled={!newInstanceName.trim()}
                                className="btn btn-primary btn-xs"
                            >
                                {t('common.create', 'Create')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Copy Instance Modal */}
            {isCopyOpen && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2 mb-3">
                            <Copy className="w-5 h-5 text-indigo-600" />
                            <h3 className="font-bold text-sm text-gray-900 dark:text-base-content">
                                {t('instances.copy_modal_title', 'Duplicate Profile')}
                            </h3>
                        </div>
                        <input
                            type="text"
                            placeholder={t('instances.copy_placeholder', 'New profile name')}
                            value={copyInstanceName}
                            onChange={(e) => setCopyInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                            className="input input-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-lg mb-4 text-xs"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => setIsCopyOpen(false)}
                                className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCopy}
                                disabled={!copyInstanceName.trim()}
                                className="btn btn-primary btn-xs"
                            >
                                {t('instances.duplicate', 'Duplicate')}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
