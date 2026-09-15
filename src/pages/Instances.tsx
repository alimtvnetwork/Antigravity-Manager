import { useState, useEffect } from 'react';
import {
    Laptop,
    Play,
    Square,
    Copy,
    Trash2,
    RotateCcw,
    Plus,
    Folder,
    Search,
    AlertCircle,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../stores/useInstanceStore';
import { isTauri } from '../utils/env';

export default function Instances() {
    const { t } = useTranslation();
    const {
        instances,
        activeInstanceId,
        switcherStatus,
        fetchInstances,
        fetchSwitcherStatus,
        triggerManualRotation,
        createInstance,
        copyInstance,
        deleteInstance,
        wipeSession,
        launchInstance,
        closeInstance,
        setActiveInstance,
    } = useInstanceStore();

    const [searchQuery, setSearchQuery] = useState('');
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [newInstanceName, setNewInstanceName] = useState('');
    const [copyTargetId, setCopyTargetId] = useState<string | null>(null);
    const [copyInstanceName, setCopyInstanceName] = useState('');
    const [actionError, setActionError] = useState<string | null>(null);

    useEffect(() => {
        if (!isTauri()) return;
        fetchInstances();
        fetchSwitcherStatus();
        const timer = setInterval(() => {
            fetchInstances();
            fetchSwitcherStatus();
        }, 3000);
        return () => clearInterval(timer);
    }, [fetchInstances, fetchSwitcherStatus]);

    const filteredInstances = instances.filter((inst) => {
        const query = searchQuery.toLowerCase();
        return (
            inst.config.name.toLowerCase().includes(query) ||
            inst.config.id.toLowerCase().includes(query) ||
            (inst.config.bound_email && inst.config.bound_email.toLowerCase().includes(query))
        );
    });

    const runningCount = instances.filter((i) => i.is_running).length;

    const handleCreate = async () => {
        if (!newInstanceName.trim()) return;
        setActionError(null);
        try {
            const created = await createInstance(newInstanceName.trim());
            await setActiveInstance(created.id);
            setNewInstanceName('');
            setIsCreateOpen(false);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to create instance');
        }
    };

    const handleCopy = async () => {
        if (!copyTargetId || !copyInstanceName.trim()) return;
        setActionError(null);
        try {
            const copied = await copyInstance(copyTargetId, copyInstanceName.trim());
            await setActiveInstance(copied.id);
            setCopyInstanceName('');
            setCopyTargetId(null);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to copy instance');
        }
    };

    const handleDelete = async (id: string) => {
        setActionError(null);
        if (window.confirm(t('instances.confirm_delete', 'Are you sure you want to delete this profile?'))) {
            try {
                await deleteInstance(id);
            } catch (e: any) {
                setActionError(e?.toString() || 'Failed to delete instance');
            }
        }
    };

    const handleWipeSession = async (id: string) => {
        setActionError(null);
        if (window.confirm(t('instances.confirm_wipe', 'Wipe session tokens for this profile? User preferences will be kept.'))) {
            try {
                await wipeSession(id);
            } catch (e: any) {
                setActionError(e?.toString() || 'Failed to wipe session');
            }
        }
    };

    return (
        <div className="max-w-7xl mx-auto px-8 py-8 space-y-6">
            {/* Header */}
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                <div>
                    <div className="flex items-center gap-3">
                        <div className="p-2.5 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                            <Laptop className="w-6 h-6" />
                        </div>
                        <div>
                            <h1 className="text-2xl font-bold text-gray-900 dark:text-base-content">
                                {t('instances.page_title', 'Instances & Profiles')}
                            </h1>
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                {t('instances.page_desc', 'Run multiple Antigravity windows in parallel with isolated credentials and extensions')}
                            </p>
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-3 w-full sm:w-auto">
                    <div className="px-3 py-1.5 rounded-lg bg-gray-100 dark:bg-base-200 text-xs text-gray-600 dark:text-gray-300 font-medium">
                        {t('instances.running_summary', 'Running: {{running}} / {{total}}', {
                            running: runningCount,
                            total: instances.length,
                        })}
                    </div>
                    <button
                        onClick={() => {
                            setNewInstanceName('');
                            setIsCreateOpen(true);
                        }}
                        className="btn btn-primary btn-sm gap-1.5 shadow-sm"
                    >
                        <Plus className="w-4 h-4" />
                        <span>{t('instances.create_btn', 'New Instance')}</span>
                    </button>
                </div>
            </div>

            {/* Error Alert */}
            {actionError && (
                <div className="p-3.5 rounded-xl bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{actionError}</span>
                    </div>
                    <button onClick={() => setActionError(null)} className="text-xs font-semibold">✕</button>
                </div>
            )}

            {/* Auto-Switcher Status Banner */}
            {switcherStatus && switcherStatus.is_running && (
                <div className="p-3.5 rounded-xl bg-gradient-to-r from-blue-50/80 to-indigo-50/50 dark:from-blue-900/20 dark:to-indigo-900/10 border border-blue-200/60 dark:border-blue-800/40 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 text-xs">
                    <div className="flex items-center gap-2.5">
                        <span className="relative flex h-2.5 w-2.5">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-blue-500"></span>
                        </span>
                        <div>
                            <span className="font-semibold text-gray-900 dark:text-gray-100">
                                {t('instances.auto_switcher_active', 'Auto Profile Switcher Active')}
                            </span>
                            <span className="text-gray-500 dark:text-gray-400 ml-2">
                                {switcherStatus.current_quota_percent !== undefined && switcherStatus.current_quota_percent !== null
                                    ? `Current Active Quota: ${switcherStatus.current_quota_percent.toFixed(0)}%`
                                    : 'Monitoring active profile quota'}
                                {switcherStatus.last_switch_reason && ` • Last switch: ${switcherStatus.last_switch_reason}`}
                            </span>
                        </div>
                    </div>
                    <button
                        onClick={async () => {
                            try {
                                const msg = await triggerManualRotation();
                                alert(msg);
                            } catch (e: any) {
                                setActionError(e?.toString() || 'Rotation failed');
                            }
                        }}
                        className="btn btn-xs btn-outline btn-primary gap-1 shrink-0"
                    >
                        <RotateCcw className="w-3 h-3" />
                        <span>{t('instances.rotate_next_best', 'Rotate to Next Best')}</span>
                    </button>
                </div>
            )}

            {/* Search Filter */}
            <div className="flex items-center gap-2 max-w-md bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 rounded-xl px-3 py-2 shadow-xs">
                <Search className="w-4 h-4 text-gray-400 shrink-0" />
                <input
                    type="text"
                    placeholder={t('instances.search_placeholder', 'Search by profile name, ID, or bound email...')}
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full bg-transparent border-none outline-hidden text-xs text-gray-900 dark:text-base-content"
                />
            </div>

            {/* Instance Cards Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                {filteredInstances.map((inst) => {
                    const isActive = inst.config.id === activeInstanceId;
                    return (
                        <div
                            key={inst.config.id}
                            className={`rounded-2xl border p-5 transition-all flex flex-col justify-between bg-white dark:bg-base-200 ${isActive ? 'border-blue-500 shadow-md ring-2 ring-blue-500/20' : 'border-gray-200/80 dark:border-base-100 hover:border-gray-300 dark:hover:border-base-content/20 shadow-xs'}`}
                        >
                            {/* Card Top */}
                            <div>
                                <div className="flex items-start justify-between gap-2 mb-3">
                                    <div className="flex items-center gap-2 min-w-0">
                                        <span
                                            className={`w-2.5 h-2.5 rounded-full shrink-0 ${inst.is_running ? 'bg-emerald-500 shadow-xs shadow-emerald-500/50 animate-pulse' : 'bg-gray-300 dark:bg-gray-600'}`}
                                        />
                                        <h3 className="font-bold text-sm text-gray-900 dark:text-base-content truncate" title={inst.config.name}>
                                            {inst.config.name}
                                        </h3>
                                        {inst.config.is_default && (
                                            <span className="px-1.5 py-0.5 rounded text-[10px] font-bold bg-gray-100 dark:bg-base-100 text-gray-600 dark:text-gray-400 shrink-0">
                                                DEFAULT
                                            </span>
                                        )}
                                    </div>
                                    <div className="shrink-0">
                                        {isActive ? (
                                            <span className="px-2 py-0.5 rounded-md text-[10px] font-semibold bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300">
                                                Active Target
                                            </span>
                                        ) : (
                                            <button
                                                onClick={() => setActiveInstance(inst.config.id)}
                                                className="text-[10px] text-gray-500 hover:text-blue-600 transition-colors font-medium"
                                                title="Set as active instance for account switches"
                                            >
                                                Set Active
                                            </button>
                                        )}
                                    </div>
                                </div>

                                {/* Status details */}
                                <div className="space-y-2 text-xs py-2 border-y border-gray-100 dark:border-base-100 text-gray-600 dark:text-gray-400">
                                    <div className="flex justify-between items-center">
                                        <span className="text-gray-400">Status:</span>
                                        <span className={`font-medium ${inst.is_running ? 'text-emerald-600 dark:text-emerald-400' : 'text-gray-500'}`}>
                                            {inst.is_running ? `Running (PID: ${inst.pid})` : 'Idle'}
                                        </span>
                                    </div>
                                    <div className="flex justify-between items-center">
                                        <span className="text-gray-400">Bound Account:</span>
                                        <span className="font-medium text-gray-900 dark:text-gray-200 truncate max-w-[170px]" title={inst.config.bound_email || 'None'}>
                                            {inst.config.bound_email || 'Unassigned'}
                                        </span>
                                    </div>
                                    <div className="flex justify-between items-center">
                                        <span className="text-gray-400">Profile ID:</span>
                                        <span className="font-mono text-[11px] text-gray-500 truncate max-w-[170px]">
                                            {inst.config.id}
                                        </span>
                                    </div>
                                    <div className="flex items-center gap-1.5 text-[11px] text-gray-400 truncate pt-1" title={inst.config.data_dir}>
                                        <Folder className="w-3.5 h-3.5 shrink-0" />
                                        <span className="truncate">{inst.config.data_dir}</span>
                                    </div>
                                </div>
                            </div>

                            {/* Card Actions */}
                            <div className="pt-4 flex items-center justify-between gap-1.5">
                                <div className="flex items-center gap-1">
                                    {inst.is_running ? (
                                        <button
                                            onClick={() => closeInstance(inst.config.id)}
                                            className="btn btn-xs btn-error btn-outline gap-1"
                                            title="Gracefully close this instance window"
                                        >
                                            <Square className="w-3 h-3" />
                                            <span>Close</span>
                                        </button>
                                    ) : (
                                        <button
                                            onClick={() => launchInstance(inst.config.id)}
                                            className="btn btn-xs btn-primary gap-1"
                                            title="Launch instance window"
                                        >
                                            <Play className="w-3 h-3" />
                                            <span>Launch</span>
                                        </button>
                                    )}
                                    <button
                                        onClick={() => {
                                            setCopyTargetId(inst.config.id);
                                            setCopyInstanceName(`${inst.config.name} Copy`);
                                        }}
                                        className="btn btn-xs btn-ghost text-gray-600 dark:text-gray-300"
                                        title="Clone profile settings and extensions"
                                    >
                                        <Copy className="w-3.5 h-3.5" />
                                    </button>
                                    <button
                                        onClick={() => handleWipeSession(inst.config.id)}
                                        disabled={inst.is_running}
                                        className="btn btn-xs btn-ghost text-amber-600 dark:text-amber-400"
                                        title="Wipe auth credentials (keep settings)"
                                    >
                                        <RotateCcw className="w-3.5 h-3.5" />
                                    </button>
                                </div>

                                {!inst.config.is_default && (
                                    <button
                                        onClick={() => handleDelete(inst.config.id)}
                                        disabled={inst.is_running}
                                        className="btn btn-xs btn-ghost text-rose-600 dark:text-rose-400"
                                        title="Delete profile"
                                    >
                                        <Trash2 className="w-3.5 h-3.5" />
                                    </button>
                                )}
                            </div>
                        </div>
                    );
                })}
            </div>

            {/* Create Instance Modal */}
            {isCreateOpen && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2.5 mb-4">
                            <Laptop className="w-5 h-5 text-blue-600" />
                            <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                {t('instances.create_modal_title', 'Create New Profile')}
                            </h3>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                            Creates an isolated Antigravity profile folder with its own SQLite token store, extensions, and configuration.
                        </p>
                        <input
                            type="text"
                            placeholder={t('instances.name_placeholder', 'Profile name (e.g., Personal, Client Work)')}
                            value={newInstanceName}
                            onChange={(e) => setNewInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-5 text-sm"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => setIsCreateOpen(false)}
                                className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCreate}
                                disabled={!newInstanceName.trim()}
                                className="btn btn-primary btn-sm"
                            >
                                {t('common.create', 'Create Profile')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Copy Modal */}
            {copyTargetId && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2.5 mb-4">
                            <Copy className="w-5 h-5 text-indigo-600" />
                            <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                {t('instances.copy_modal_title', 'Duplicate Profile')}
                            </h3>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                            Copies extensions, preferences, and editor settings into a new isolated profile.
                        </p>
                        <input
                            type="text"
                            placeholder={t('instances.copy_placeholder', 'New profile name')}
                            value={copyInstanceName}
                            onChange={(e) => setCopyInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-5 text-sm"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => setCopyTargetId(null)}
                                className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCopy}
                                disabled={!copyInstanceName.trim()}
                                className="btn btn-primary btn-sm"
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
