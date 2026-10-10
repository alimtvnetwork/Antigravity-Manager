import ModalDialog from '../../components/common/ModalDialog';
import { InstanceSettingsModal } from '../../components/instances/InstanceSettingsModal';
import PromptTreeViewModal from '../../components/instances/PromptTreeViewModal';
import InstanceAuditTrailModal from '../../components/instances/InstanceAuditTrailModal';
import type { InstancePageApi } from './instancePageTypes';

export function InstanceConfirmDialogs({ api }: { api: InstancePageApi }) {
    const {
        t,
        deleteModalTarget,
        setDeleteModalTarget,
        wipeModalTarget,
        setWipeModalTarget,
        deletingId,
        actionState,
        handleConfirmDelete,
        handleConfirmWipe,
    } = api;

    return (
        <>
            <ModalDialog
                isOpen={Boolean(deleteModalTarget)}
                title={t('instances.delete_title', 'Delete Profile')}
                type="confirm"
                isDestructive={true}
                isLoading={deletingId === deleteModalTarget?.config.id}
                onConfirm={handleConfirmDelete}
                onCancel={() => setDeleteModalTarget(null)}
                confirmText={t('common.delete', 'Delete')}
                cancelText={t('common.cancel', 'Cancel')}
            >
                {deleteModalTarget && (
                    <div className="space-y-3">
                        <div className="p-3 rounded-xl bg-rose-50/80 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900/50 text-xs text-rose-700 dark:text-rose-300">
                            {t(
                                'instances.delete_warning',
                                'Are you sure you want to delete this profile? This will permanently delete the isolated profile data directory and all settings.'
                            )}
                        </div>
                        <div className="bg-gray-50 dark:bg-[#0c2438] rounded-xl p-3 border border-gray-200/70 dark:border-[#15334d] space-y-1.5 text-xs">
                            <div className="flex items-center justify-between">
                                <span className="text-gray-500 dark:text-slate-400 font-medium">
                                    {t('instances.name', 'Profile Name')}:
                                </span>
                                <span className="font-bold text-gray-900 dark:text-white flex items-center gap-1.5">
                                    <span className="px-1.5 py-0.5 rounded-[5px] text-[10px] font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25">
                                        #{deleteModalTarget.config.seq_num ?? 0}
                                    </span>
                                    {deleteModalTarget.config.name}
                                </span>
                            </div>
                            <div className="flex flex-col gap-1 pt-1.5 border-t border-gray-200/50 dark:border-[#15334d]/60">
                                <span className="text-gray-500 dark:text-slate-400 font-medium">
                                    {t('instances.data_dir', 'Data Directory')}:
                                </span>
                                <span className="font-mono text-[11px] text-gray-700 dark:text-slate-300 break-all bg-white dark:bg-[#071a27] p-1.5 rounded-lg border border-gray-200 dark:border-[#15334d]">
                                    {deleteModalTarget.config.data_dir}
                                </span>
                            </div>
                        </div>
                    </div>
                )}
            </ModalDialog>

            <ModalDialog
                isOpen={Boolean(wipeModalTarget)}
                title={t('instances.wipe_title', 'Wipe Credentials')}
                type="confirm"
                isDestructive={true}
                isLoading={actionState[wipeModalTarget?.config.id || ''] === 'wipe'}
                onConfirm={handleConfirmWipe}
                onCancel={() => setWipeModalTarget(null)}
                confirmText={t('instances.wipe_confirm', 'Wipe Credentials')}
                cancelText={t('common.cancel', 'Cancel')}
            >
                {wipeModalTarget && (
                    <div className="space-y-3">
                        <div className="p-3 rounded-xl bg-amber-50/80 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900/50 text-xs text-amber-700 dark:text-amber-300">
                            {t(
                                'instances.wipe_warning',
                                'This will clear all saved account credentials and session tokens from this profile without deleting its workspace or configurations.'
                            )}
                        </div>
                        <div className="bg-gray-50 dark:bg-[#0c2438] rounded-xl p-3 border border-gray-200/70 dark:border-[#15334d] space-y-1.5 text-xs">
                            <div className="flex items-center justify-between">
                                <span className="text-gray-500 dark:text-slate-400 font-medium">
                                    {t('instances.name', 'Profile Name')}:
                                </span>
                                <span className="font-bold text-gray-900 dark:text-white flex items-center gap-1.5">
                                    <span className="px-1.5 py-0.5 rounded-[5px] text-[10px] font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25">
                                        #{wipeModalTarget.config.seq_num ?? 0}
                                    </span>
                                    {wipeModalTarget.config.name}
                                </span>
                            </div>
                            {wipeModalTarget.config.bound_email && (
                                <div className="flex items-center justify-between pt-1.5 border-t border-gray-200/50 dark:border-[#15334d]/60">
                                    <span className="text-gray-500 dark:text-slate-400 font-medium">
                                        {t('instances.bound_account', 'Bound Account')}:
                                    </span>
                                    <span className="font-mono text-[11px] text-amber-600 dark:text-amber-400 font-semibold">
                                        {wipeModalTarget.config.bound_email}
                                    </span>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </ModalDialog>
        </>
    );
}

export function InstanceAttachedModals({ api }: { api: InstancePageApi }) {
    const {
        isSettingsModalOpen,
        setIsSettingsModalOpen,
        settingsModalTarget,
        setSettingsModalTarget,
        instances,
        fetchInstances,
        promptTreeInstance,
        setPromptTreeInstance,
        auditModalInstance,
        setAuditModalInstance,
        fetchRunningTasks,
    } = api;

    return (
        <>
            <InstanceSettingsModal
                isOpen={isSettingsModalOpen}
                onClose={() => {
                    setIsSettingsModalOpen(false);
                    setSettingsModalTarget(null);
                }}
                targetInstance={settingsModalTarget}
                instances={instances}
                onInstancesUpdated={() => fetchInstances(true)}
            />

            <PromptTreeViewModal
                isOpen={Boolean(promptTreeInstance)}
                onClose={() => {
                    setPromptTreeInstance(null);
                    fetchRunningTasks();
                }}
                instanceId={promptTreeInstance?.id || ''}
                instanceName={promptTreeInstance?.name || ''}
                sequenceNumber={promptTreeInstance?.seqNum}
                executablePath={promptTreeInstance?.executablePath}
                initialSelectedProjectId={promptTreeInstance?.projectId}
            />

            <InstanceAuditTrailModal
                isOpen={Boolean(auditModalInstance)}
                onClose={() => setAuditModalInstance(null)}
                instance={auditModalInstance}
            />
        </>
    );
}
