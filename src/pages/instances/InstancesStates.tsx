import { Search, Plus, Play, RotateCw, Laptop, AlertCircle } from 'lucide-react';
import type { InstancePageApi } from './instancePageTypes';

export function InstancesFilterBar({ api }: { api: InstancePageApi }) {
    const { t, searchQuery, setSearchQuery, setNewInstanceName, setIsCreateOpen } = api;

    return (
        <div className="flex items-center justify-between gap-3 flex-wrap">
            <div className="flex items-center gap-2 flex-1 max-w-md bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 rounded-[5px] px-3 py-2 shadow-xs">
                <Search className="w-4 h-4 text-gray-400 shrink-0" />
                <input
                    type="text"
                    placeholder={t('instances.search_placeholder', 'Search by profile name, ID, or bound email...')}
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full bg-transparent border-none outline-hidden text-xs text-gray-900 dark:text-base-content"
                />
            </div>
            <button
                onClick={() => {
                    setNewInstanceName('');
                    setIsCreateOpen(true);
                }}
                className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[4px] transition-all duration-150 ease-out active:scale-[0.98]"
            >
                <Plus className="w-4 h-4" />
                <span>{t('instances.create_btn', 'New Instance')}</span>
            </button>
        </div>
    );
}

export function InstancesAlerts({ api }: { api: InstancePageApi }) {
    const { t, storeError, actionError, setActionError, fetchInstances } = api;

    return (
        <>
            {storeError ? (
                <div className="p-3.5 rounded-[5px] bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{String(storeError)}</span>
                    </div>
                    <button
                        onClick={() => fetchInstances()}
                        className="btn btn-xs btn-outline btn-error gap-1 shrink-0 rounded-[5px]"
                    >
                        <RotateCw className="w-3 h-3" />
                        <span>{t('common.retry', 'Retry')}</span>
                    </button>
                </div>
            ) : null}

            {actionError ? (
                <div className="p-3.5 rounded-[5px] bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{actionError}</span>
                    </div>
                    <button onClick={() => setActionError(null)} className="text-xs font-semibold">
                        ✕
                    </button>
                </div>
            ) : null}
        </>
    );
}

export function InstancesLoadingState({ api }: { api: InstancePageApi }) {
    const { t } = api;
    return (
        <div className="flex flex-col items-center justify-center py-20 bg-white dark:bg-base-200 rounded-[5px] border border-gray-200/80 dark:border-base-100">
            <RotateCw className="w-8 h-8 text-blue-600 dark:text-blue-400 animate-spin mb-3" />
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('instances.loading', 'Loading instances and profiles...')}
            </p>
        </div>
    );
}

export function InstancesEmptyState({ api }: { api: InstancePageApi }) {
    const { t, setNewInstanceName, setIsCreateOpen } = api;

    const { handleInitDefault } = api;

    return (
        <div className="flex flex-col items-center justify-center py-16 px-4 text-center bg-white dark:bg-base-200 rounded-[5px] border border-dashed border-gray-300 dark:border-base-100">
            <div className="p-4 rounded-[5px] bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 mb-4">
                <Laptop className="w-10 h-10" />
            </div>
            <h3 className="text-base font-bold text-gray-900 dark:text-base-content mb-1">
                {t('instances.empty_title', 'No Profiles Found')}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400 max-w-md mb-6 leading-relaxed">
                {t(
                    'instances.empty_desc',
                    'Instances allow you to run isolated Antigravity windows with dedicated account tokens, extensions, and workspaces. Initialize your default profile or create a custom one to get started.'
                )}
            </p>
            <div className="flex items-center gap-3">
                <button onClick={handleInitDefault} className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[5px]">
                    <Play className="w-3.5 h-3.5" />
                    <span>{t('instances.init_default', 'Initialize Default Profile')}</span>
                </button>
                <button
                    onClick={() => {
                        setNewInstanceName('');
                        setIsCreateOpen(true);
                    }}
                    className="btn btn-outline btn-sm gap-1.5 rounded-[5px]"
                >
                    <Plus className="w-3.5 h-3.5" />
                    <span>{t('instances.create_custom', 'Create Custom Profile')}</span>
                </button>
            </div>
        </div>
    );
}

export function InstancesNoResultsState({ api }: { api: InstancePageApi }) {
    const { t } = api;
    return (
        <div className="flex flex-col items-center justify-center py-12 text-center bg-white dark:bg-base-200 rounded-[5px] border border-gray-200/80 dark:border-base-100">
            <Search className="w-8 h-8 text-gray-400 mb-2" />
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('instances.no_search_results', 'No profiles match your search')}
            </p>
            <p className="text-xs text-gray-400 mt-1">
                {t('instances.no_search_results_desc', 'Try searching with a different name, profile ID, or email')}
            </p>
        </div>
    );
}
