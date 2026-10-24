import { ChevronDown, FastForward } from 'lucide-react';
import { cn } from '../../utils/cn';
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
        <div className="relative flex items-center rounded-full bg-gray-100 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs shrink-0" ref={dropdownRef}>
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
                className="flex items-center gap-1.5 md:gap-2 h-7 px-2.5 rounded-l-full text-xs font-medium hover:bg-gray-200 dark:hover:bg-[#40285e] transition-colors shrink-0 cursor-pointer text-gray-700 dark:text-gray-300"
                title={t('instances.selector_tooltip', 'Select active Antigravity instance')}
            >
                <span
                    className={`w-2 h-2 rounded-full shrink-0 ${isActiveRunning ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'}`}
                />
                <span className="truncate max-w-[70px] sm:max-w-[90px] md:max-w-[125px] font-medium">
                    {activeInstance
                        ? `#${instances.findIndex((i) => i.config.id === activeInstance.config.id) + 1} ${activeInstance.config.name}`
                        : 'Default'}
                </span>
                <ChevronDown className="w-3.5 h-3.5 text-gray-500 shrink-0" />
            </button>

            {/* 2. Top Action Bar: Fast-Forward only (Smart Switch) with 3D tactile elevation */}
            <button
                type="button"
                disabled={isRotating}
                onClick={(e) => {
                    e.stopPropagation();
                    handleSmartRotate(activeInstance?.config.id);
                }}
                className={cn(
                    "flex items-center justify-center w-7 h-7 rounded-r-full text-white transition-all duration-150 shrink-0 cursor-pointer",
                    isRotating
                        ? "bg-blue-400 dark:bg-blue-800 cursor-not-allowed opacity-80"
                        : "bg-blue-600 hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:!text-black hover:font-bold hover:shadow-[0_2px_0_0_#a16207] active:translate-y-0.5 active:shadow-none"
                )}
                title={t(
                    'instances.smart_switch_shortcut_tooltip',
                    `Fast-Forward (${fastForwardShortcut}): Close running process, pick account with longest refill runway, and switch profile`
                )}
            >
                <FastForward className={cn("w-3.5 h-3.5 fill-current", isRotating && "animate-spin")} />
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
