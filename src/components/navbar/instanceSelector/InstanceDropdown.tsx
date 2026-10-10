import {
    ChevronDown,
    Copy,
    Play,
    FastForward,
    Square,
    Trash2,
    Search,
    Download,
    Upload,
    Star,
    Plus,
    X,
    Pencil,
} from 'lucide-react';
import { useAccountStore } from '../../../stores/useAccountStore';
import { cn } from '../../../utils/cn';
import { SELECTED_ROW_CLASSES } from '../../common/selectedState';
import { showToast } from '../../common/ToastContainer';
import type { InstanceSelectorApi } from './useInstanceSelector';

export function InstanceDropdown({ api }: { api: InstanceSelectorApi }) {
    const {
        t,
        instances,
        activeInstance,
        activeInstanceId,
        isOpen,
        setIsOpen,
        searchQuery,
        setSearchQuery,
        isIoOpen,
        setIsIoOpen,
        activeItemRef,
        filteredInstances,
        launchingId,
        isRotating,
        handleToggleLaunch,
        handleSmartRotate,
        handleSmartPlay,
        handleSetDefault,
        handleExportProfiles,
        handleImportProfiles,
        openCreate,
        openCopy,
        openEdit,
        openDelete,
    } = api;

    const accounts = useAccountStore((state) => state.accounts);
    const currentAccount = useAccountStore((state) => state.currentAccount);

    if (!isOpen) return null;

    return (
        <div
            className="absolute top-full right-0 mt-1.5 w-96 max-w-[calc(100vw-24px)] md:w-[420px] rounded-xl shadow-2xl bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] py-2 z-[9999] animate-in fade-in zoom-in-95"
            style={{ isolation: 'isolate' }}
        >
            {/* Dropdown Header Bar with Duplicate and Combined Import / Export Dropdown */}
            <div className="flex items-center justify-between px-3 py-1 border-b border-gray-100 dark:border-[#15334d] pb-1.5">
                <span className="text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
                    {t('instances.header_title', 'INSTANCES / PROFILES')}
                </span>
                <div className="flex items-center gap-1 relative">
                    <button
                        type="button"
                        onClick={openCreate}
                        className="p-1 rounded-md text-gray-500 hover:text-blue-600 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
                        title={t('instances.create_button', 'Create New Profile')}
                    >
                        <Plus className="w-3.5 h-3.5" />
                    </button>

                    <button
                        type="button"
                        onClick={() => {
                            const target = activeInstance || instances[0];
                            if (target) openCopy(target.config.id, target.config.name);
                        }}
                        className="p-1 rounded-md text-gray-500 hover:text-indigo-600 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
                        title={t('instances.duplicate_active', 'Duplicate Active Profile')}
                    >
                        <Copy className="w-3.5 h-3.5" />
                    </button>

                    {/* Combined Import / Export Icon Dropdown */}
                    <div className="relative">
                        <button
                            type="button"
                            onClick={() => setIsIoOpen(!isIoOpen)}
                            className="px-1.5 py-1 rounded-md text-gray-500 hover:text-blue-600 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer flex items-center gap-0.5"
                            title="Import / Export Profiles (JSON)"
                        >
                            <Download className="w-3.5 h-3.5" />
                            <ChevronDown className="w-2.5 h-2.5" />
                        </button>
                        {isIoOpen && (
                            <div className="absolute right-0 mt-1 w-44 rounded-lg bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 shadow-xl py-1 z-[9999] text-xs">
                                <button
                                    type="button"
                                    onClick={() => {
                                        setIsIoOpen(false);
                                        handleImportProfiles();
                                    }}
                                    className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
                                >
                                    <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                    <span>{t('instances.import_json', 'Import Profiles (JSON)')}</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => {
                                        setIsIoOpen(false);
                                        handleExportProfiles();
                                    }}
                                    className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
                                >
                                    <Download className="w-3.5 h-3.5 text-blue-500" />
                                    <span>{t('instances.export_json', 'Export Profiles (JSON)')}</span>
                                </button>
                            </div>
                        )}
                    </div>

                    <button
                        type="button"
                        onClick={() => {
                            setIsOpen(false);
                            setIsIoOpen(false);
                        }}
                        className="p-1 rounded-md text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer ml-0.5"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Profile Search Input Box */}
            <div className="px-2.5 pt-2 pb-1">
                <div className="relative">
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder={t('instances.search_placeholder', 'Search profiles...')}
                        className="w-full pl-8 pr-2.5 py-1 text-xs bg-gray-50 dark:bg-[#071a27] border border-gray-200 dark:border-[#15334d] rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500 text-gray-900 dark:text-slate-100"
                    />
                </div>
            </div>

            {/* Filtered Profiles List */}
            <div className="max-h-60 overflow-y-auto py-1">
                {filteredInstances.length === 0 ? (
                    <div className="px-3 py-4 text-center text-xs text-gray-400">
                        {t('instances.no_profiles_found', 'No profiles found')}
                    </div>
                ) : (
                    filteredInstances.map((inst) => (
                        <InstanceDropdownItem key={inst.config.id} api={api} instId={inst.config.id} />
                    ))
                )}
            </div>

            {/* Footer Bar: Smart Play / Run Selected Profile */}
            <div className="p-2 border-t border-gray-100 dark:border-[#15334d]">
                <button
                    type="button"
                    disabled={launchingId === activeInstance?.config.id}
                    onClick={() => {
                        if (!activeInstance) return;
                        if (activeInstance.is_running) {
                            handleToggleLaunch(activeInstance.config.id, true);
                        } else {
                            handleSmartPlay(activeInstance.config.id);
                        }
                    }}
                    className={`w-full py-1.5 px-3 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors cursor-pointer ${
                        activeInstance?.is_running
                            ? 'bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300 hover:bg-rose-100 dark:hover:bg-rose-900/60 border border-rose-200 dark:border-rose-800/50 shadow-xs'
                            : 'bg-[#16a97a] hover:bg-[#13946a] active:bg-[#0f7a56] hover:brightness-105 text-white shadow-xs'
                    }`}
                >
                    {activeInstance?.is_running ? (
                        <>
                            <Square className="w-3.5 h-3.5 fill-current" />
                            <span>{t('instances.close_active', 'Close Active Instance')}</span>
                        </>
                    ) : (
                        <>
                            <Play className="w-3.5 h-3.5 fill-current" />
                            <span>{t('instances.smart_play_active', 'Smart Play Selected Profile')}</span>
                        </>
                    )}
                </button>
            </div>
        </div>
    );
}

function InstanceDropdownItem({ api, instId }: { api: InstanceSelectorApi; instId: string }) {
    const {
        t,
        instances,
        activeInstanceId,
        setIsOpen,
        activeItemRef,
        launchingId,
        isRotating,
        handleToggleLaunch,
        handleSmartRotate,
        handleSetDefault,
        openEdit,
        openCopy,
        openDelete,
    } = api;

    const accounts = useAccountStore((state) => state.accounts);
    const currentAccount = useAccountStore((state) => state.currentAccount);

    const inst = instances.find((i) => i.config.id === instId);
    if (!inst) return null;

    const isSelected = inst.config.id === activeInstanceId;
    const isDefault = Boolean(inst.config.is_default);
    const isRunning = Boolean(inst.is_running);
    const linkedAccount = accounts.find(
        (a) => a.id === inst.config.bound_account_id || (inst.config.bound_email && a.email === inst.config.bound_email)
    );
    const displayEmail =
        inst.config.bound_email || linkedAccount?.email || (isSelected ? currentAccount?.email : undefined);
    const seqNum = instances.findIndex((i) => i.config.id === inst.config.id) + 1;

    return (
        <div
            ref={isSelected ? activeItemRef : undefined}
            className={cn(
                'w-full group flex items-center justify-between px-3 py-2 text-xs text-left transition-all duration-150',
                isSelected
                    ? cn(SELECTED_ROW_CLASSES, 'my-1 rounded-r-md ring-1 ring-amber-400/40')
                    : 'border-l-4 border-l-transparent text-gray-700 dark:text-gray-200 hover:bg-slate-100/70 hover:text-slate-900 dark:hover:bg-[#15334d]/60 dark:hover:text-white hover:border-l-blue-400 dark:hover:border-l-[#19b7c9]'
            )}
        >
            <SelectInstanceButton api={api} instId={instId} seqNum={seqNum} />

            {/* Action Buttons */}
            <div className="flex items-center gap-1 shrink-0 ml-auto">
                <button
                    type="button"
                    disabled={launchingId === inst.config.id}
                    onClick={(e) => {
                        e.stopPropagation();
                        handleToggleLaunch(inst.config.id, isRunning);
                    }}
                    className={cn(
                        'w-6 h-6 p-1 rounded transition-colors cursor-pointer flex items-center justify-center',
                        isRunning
                            ? 'text-red-500 hover:bg-red-100 dark:hover:bg-red-900/40'
                            : 'text-emerald-600 hover:bg-emerald-100 dark:hover:bg-emerald-900/40'
                    )}
                    title={isRunning ? t('instances.close_title', 'Close window') : t('instances.launch_title', 'Run profile')}
                >
                    {isRunning ? <Square className="w-3 h-3 fill-current" /> : <Play className="w-3 h-3 fill-current" />}
                </button>

                <button
                    type="button"
                    disabled={isRotating}
                    onClick={(e) => {
                        e.stopPropagation();
                        handleSmartRotate(inst.config.id);
                    }}
                    className="w-6 h-6 p-1 rounded text-blue-600 hover:text-blue-700 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors cursor-pointer flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                    title={t('instances.smart_switch_tooltip', 'Smart Switch: Close process, pick account with longest refill runway, and switch')}
                >
                    <FastForward className={`w-3 h-3 fill-current ${isRotating ? 'animate-spin' : ''}`} />
                </button>

                <button
                    type="button"
                    onClick={(e) => {
                        e.stopPropagation();
                        openEdit(inst.config.id, inst.config.name);
                        setIsOpen(false);
                    }}
                    className="w-6 h-6 p-1 rounded text-gray-400 hover:text-blue-600 hover:bg-gray-200/60 dark:hover:bg-slate-800 transition-colors opacity-70 group-hover:opacity-100 cursor-pointer flex items-center justify-center"
                    title={t('instances.edit_title', 'Rename profile')}
                >
                    <Pencil className="w-3 h-3" />
                </button>

                <button
                    type="button"
                    onClick={(e) => {
                        e.stopPropagation();
                        openCopy(inst.config.id, inst.config.name);
                        setIsOpen(false);
                    }}
                    className="w-6 h-6 p-1 rounded text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 transition-colors opacity-70 group-hover:opacity-100 cursor-pointer flex items-center justify-center"
                    title={t('instances.duplicate_title', 'Duplicate profile')}
                >
                    <Copy className="w-3 h-3" />
                </button>

                <button
                    type="button"
                    disabled={isDefault}
                    onClick={(e) => {
                        e.stopPropagation();
                        if (!isDefault) handleSetDefault(inst.config.id);
                    }}
                    className={cn(
                        'w-6 h-6 p-1 rounded transition-colors flex items-center justify-center',
                        isDefault
                            ? 'text-amber-500 cursor-default'
                            : 'text-gray-400 hover:text-amber-500 hover:bg-amber-50 dark:hover:bg-amber-950/30 cursor-pointer opacity-70 group-hover:opacity-100'
                    )}
                    title={isDefault ? t('instances.is_default_title', 'Default profile') : t('instances.set_default_title', 'Set as default profile')}
                >
                    <Star className={cn('w-3 h-3', isDefault ? 'fill-amber-400 text-amber-500' : '')} />
                </button>

                {isDefault ? (
                    <span className="w-6 h-6" aria-hidden="true" />
                ) : (
                    <button
                        type="button"
                        onClick={(e) => {
                            e.stopPropagation();
                            openDelete(inst);
                            setIsOpen(false);
                        }}
                        className="w-6 h-6 p-1 rounded text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 transition-colors opacity-70 group-hover:opacity-100 cursor-pointer flex items-center justify-center"
                        title={t('instances.delete_title', 'Delete profile')}
                    >
                        <Trash2 className="w-3 h-3" />
                    </button>
                )}
            </div>
        </div>
    );
}

function SelectInstanceButton({
    api,
    instId,
    seqNum,
}: {
    api: InstanceSelectorApi;
    instId: string;
    seqNum: number;
}) {
    const { t, instances, activeInstanceId } = api;
    const accounts = useAccountStore((state) => state.accounts);
    const currentAccount = useAccountStore((state) => state.currentAccount);

    const inst = instances.find((i) => i.config.id === instId);
    if (!inst) return null;

    const isSelected = inst.config.id === activeInstanceId;
    const isDefault = Boolean(inst.config.is_default);
    const isRunning = Boolean(inst.is_running);
    const linkedAccount = accounts.find(
        (a) => a.id === inst.config.bound_account_id || (inst.config.bound_email && a.email === inst.config.bound_email)
    );
    const displayEmail =
        inst.config.bound_email || linkedAccount?.email || (isSelected ? currentAccount?.email : undefined);

    return (
        <button
            type="button"
            onClick={async () => {
                await api.handleSelectInstance(instId, seqNum, inst.config.name);
            }}
            className="flex items-center gap-2 truncate flex-1 text-left cursor-pointer min-w-0 pr-2"
        >
            <span
                className={`w-2 h-2 rounded-full shrink-0 ${isRunning ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'}`}
            />
            <div className="flex flex-col truncate min-w-0">
                <div className="flex items-center gap-1.5 truncate">
                    <span
                        className={cn(
                            'px-1.5 py-0.5 rounded text-[10px] font-black shrink-0',
                            isSelected
                                ? 'bg-blue-600 text-white'
                                : 'bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25'
                        )}
                    >
                        #{seqNum}
                    </span>
                    <span className={cn('truncate font-bold', isSelected ? 'text-blue-950 dark:text-white' : 'text-gray-900 dark:text-gray-100')}>
                        {inst.config.name}
                    </span>
                    {isDefault && (
                        <span className="px-1 py-0.2 rounded text-[8px] font-bold bg-[#16a97a]/20 text-[#16a97a] dark:text-[#43d6a2] border border-[#16a97a]/40 shrink-0">
                            DEFAULT
                        </span>
                    )}
                    {isSelected && (
                        <span className="px-1.5 py-0.5 rounded text-[8px] font-black bg-blue-600 text-white shadow-xs shrink-0 tracking-wider">
                            ACTIVE
                        </span>
                    )}
                </div>
                {displayEmail ? (
                    <span className={cn('text-[10px] truncate font-mono', isSelected ? 'text-slate-700 dark:text-cyan-200 font-semibold' : 'text-gray-500 dark:text-slate-300 font-medium')}>
                        {displayEmail}
                    </span>
                ) : (
                    <span className={cn('text-[10px] italic truncate', isSelected ? 'text-slate-500 dark:text-cyan-300/70' : 'text-gray-400 dark:text-slate-400')}>
                        {t('instances.unlinked', 'No account linked')}
                    </span>
                )}
            </div>
        </button>
    );
}
