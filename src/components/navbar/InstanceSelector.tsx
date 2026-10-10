import { ChevronDown, FastForward } from 'lucide-react';
import { isTauri } from '../../utils/env';
import { useInstanceSelector } from './instanceSelector/useInstanceSelector';
import { InstanceDropdown } from './instanceSelector/InstanceDropdown';
import { CreateInstanceModal } from './instanceSelector/CreateInstanceModal';
import { CopyInstanceModal } from './instanceSelector/CopyInstanceModal';
import { EditInstanceModal, DeleteInstanceModal } from './instanceSelector/EditDeleteModals';

export function InstanceSelector() {
    const api = useInstanceSelector();
    const {
        t,
        instances,
        activeInstance,
        isOpen,
        setIsOpen,
        isRotating,
        fastForwardShortcut,
        dropdownRef,
        fileInputRef,
        handleFileInput,
        handleSmartRotate,
    } = api;

    const isAvailable = isTauri();
    if (!isAvailable) return null;

    const isActiveRunning = Boolean(activeInstance?.is_running);

    return (
        <div className="relative flex items-center gap-1 shrink-0" ref={dropdownRef}>
            <input
                ref={fileInputRef}
                type="file"
                accept=".json,application/json"
                style={{ display: 'none' }}
                onChange={handleFileInput}
            />

            {/* 1. Profile Dropdown Trigger */}
            <button
                type="button"
                onClick={() => {
                    const next = !isOpen;
                    setIsOpen(next);
                    if (next) {
                        window.dispatchEvent(
                            new CustomEvent('agm:dropdown-open', { detail: { source: 'instance-selector' } })
                        );
                    }
                }}
                className="flex items-center gap-1.5 md:gap-2 px-2.5 md:px-3 py-1.5 rounded-lg text-xs font-medium bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors border border-gray-200/60 dark:border-slate-700 shrink-0 cursor-pointer"
                title={t('instances.selector_tooltip', 'Select active Antigravity instance')}
            >
                <span
                    className={`w-2 h-2 rounded-full shrink-0 ${isActiveRunning ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'}`}
                />
                <span className="truncate max-w-[95px] md:max-w-[130px] text-gray-800 dark:text-gray-200 font-medium">
                    {activeInstance
                        ? `#${instances.findIndex((i) => i.config.id === activeInstance.config.id) + 1} ${activeInstance.config.name}`
                        : 'Default'}
                </span>
                <ChevronDown className="w-3.5 h-3.5 text-gray-500 shrink-0" />
            </button>

            {/* 2. Top Action Bar: Fast-Forward only (Smart Switch) */}
            <button
                type="button"
                disabled={isRotating}
                onClick={(e) => {
                    e.stopPropagation();
                    handleSmartRotate(activeInstance?.config.id);
                }}
                className={`flex p-1.5 rounded-lg text-white shadow-xs transition-colors duration-150 shrink-0 items-center justify-center cursor-pointer ${
                    isRotating
                        ? 'bg-blue-400 dark:bg-blue-800 cursor-not-allowed opacity-80'
                        : 'bg-blue-600 hover:bg-blue-700 active:bg-blue-800'
                }`}
                title={t(
                    'instances.smart_switch_shortcut_tooltip',
                    `Fast-Forward (${fastForwardShortcut}): Close running process, pick account with longest refill runway, and switch profile`
                )}
            >
                <FastForward className={`w-3.5 h-3.5 fill-current ${isRotating ? 'animate-spin' : ''}`} />
            </button>

            {/* Dropdown Menu Popup */}
            <InstanceDropdown api={api} />

            {/* Modals */}
            <CreateInstanceModal api={api} />
            <CopyInstanceModal api={api} />
            <EditInstanceModal api={api} />
            <DeleteInstanceModal api={api} />
        </div>
    );
}
