import { cn } from '../utils/cn';
import InstanceTable from '../components/instances/InstanceTable';
import FleetMachinesTable from '../components/instances/FleetMachinesTable';
import { useInstancePage } from './instances/useInstancePage';
import { INSTANCE_THEMES } from './instances/instancePageUtils';
import { InstancesHeader } from './instances/InstancesHeader';
import {
    InstancesFilterBar,
    InstancesAlerts,
    InstancesLoadingState,
    InstancesEmptyState,
    InstancesNoResultsState,
} from './instances/InstancesStates';
import { InstanceCard } from './instances/InstanceCard';
import { CreateInstanceDialog, EditInstanceDialog } from './instances/CreateEditDialogs';
import { SwitchAccountDialog } from './instances/SwitchAccountDialog';
import { CopyInstanceDialog } from './instances/CopyInstanceDialog';
import { InstanceConfirmDialogs, InstanceAttachedModals } from './instances/InstanceModals';

export { isNodeOwnedByInstance } from './instances/instancePageUtils';

export default function Instances() {
    const api = useInstancePage();
    const {
        instances,
        activeInstanceId,
        isLoading,
        searchQuery,
        viewMode,
        cardDensity,
        filteredInstances,
        actionState,
        syncingInstanceIds,
        handleLaunch,
        handleStop,
        handleRestart,
        handleFastForward,
        handleSync,
        handleDelete,
        openSwitchDialog,
        openAuditDialog,
        openSettingsDialog,
        openCopyDialog,
        openPromptTree,
        setActiveInstance,
        setDefaultInstance,
    } = api;

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="max-w-[1920px] mx-auto px-4 sm:px-6 lg:px-8 pt-2 pb-4 space-y-3">
                <InstancesHeader api={api} />

                <InstancesAlerts api={api} />

                <InstancesFilterBar api={api} />

                {isLoading && instances.length === 0 ? (
                    <InstancesLoadingState api={api} />
                ) : instances.length === 0 ? (
                    <InstancesEmptyState api={api} />
                ) : filteredInstances.length === 0 ? (
                    <InstancesNoResultsState api={api} />
                ) : viewMode === 'list' ? (
                    <InstanceTable
                        instances={filteredInstances}
                        activeInstanceId={activeInstanceId}
                        searchQuery={searchQuery}
                        actionState={actionState}
                        onLaunch={handleLaunch}
                        onStop={handleStop}
                        onRestart={handleRestart}
                        onSwitch={openSwitchDialog}
                        onFastForward={handleFastForward}
                        onAudit={openAuditDialog}
                        onSync={handleSync}
                        syncingInstanceIds={syncingInstanceIds}
                        onSettings={openSettingsDialog}
                        onClone={openCopyDialog}
                        onDelete={handleDelete}
                        onOpenPromptTree={openPromptTree}
                        onSetActive={setActiveInstance}
                        onSetDefault={setDefaultInstance}
                    />
                ) : (
                    <div
                        className={cn(
                            cardDensity === 'compact'
                                ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2.5'
                                : 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3'
                        )}
                    >
                        {filteredInstances.map((inst, index) => {
                            const originalIndex = instances.findIndex((i) => i.config.id === inst.config.id);
                            const seqNumber = originalIndex !== -1 ? originalIndex + 1 : index + 1;
                            const theme = INSTANCE_THEMES[index % INSTANCE_THEMES.length];
                            return (
                                <InstanceCard
                                    key={inst.config.id}
                                    api={api}
                                    inst={inst}
                                    index={index}
                                    seqNumber={seqNumber}
                                    theme={theme}
                                />
                            );
                        })}
                    </div>
                )}

                <div className="mt-8 pt-6 border-t border-slate-200/60 dark:border-slate-800/80">
                    <FleetMachinesTable />
                </div>

                <CreateInstanceDialog api={api} />
                <SwitchAccountDialog api={api} />
                <CopyInstanceDialog api={api} />
                <EditInstanceDialog api={api} />
                <InstanceAttachedModals api={api} />
                <InstanceConfirmDialogs api={api} />
            </div>
        </div>
    );
}
