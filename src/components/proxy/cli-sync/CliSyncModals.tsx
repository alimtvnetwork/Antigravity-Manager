import type { TFunction } from 'i18next';
import { CodeXml, Copy, X } from 'lucide-react';
import { cn } from '../../../utils/cn';
import ModalDialog from '../../common/ModalDialog';
import { DroidSyncModal } from '../DroidSyncModal';
import { OpenCodeSyncModal } from '../OpenCodeSyncModal';
import { HermesSyncModal } from '../HermesSyncModal';
import { OpenClawSyncModal } from '../OpenClawSyncModal';
import type { CliAppType, CliStatus, ViewingConfig } from './types';
import { copyViewingConfigContent } from './CliAppCard';

export interface CliSyncModalsProps {
    t: TFunction;
    proxyUrl: string;
    apiKey: string;
    statuses: Record<CliAppType, CliStatus | null>;
    viewingConfig: ViewingConfig | null;
    onCloseViewingConfig: () => void;
    onViewConfigFile: (app: CliAppType, fileName: string) => void;
    restoreConfirmApp: CliAppType | null;
    onCancelRestore: () => void;
    onConfirmRestore: () => void;
    syncConfirmApp: CliAppType | null;
    onCancelSync: () => void;
    onConfirmSync: () => void;
    clearConfirmApp: CliAppType | null;
    onCancelClear: () => void;
    onConfirmClear: () => void;
    droidSyncModal: boolean;
    onCloseDroidModal: () => void;
    onDroidSyncDone: () => void;
    hermesSyncModal: boolean;
    onCloseHermesModal: () => void;
    onHermesSyncDone: () => void;
    openClawSyncModal: boolean;
    onCloseOpenClawModal: () => void;
    onOpenClawSyncDone: () => void;
    openCodeSyncModal: boolean;
    onCloseOpenCodeModal: () => void;
    onOpenCodeSyncDone: () => void;
    syncAccounts: boolean;
    getFormattedProxyUrl: (app: CliAppType) => string;
}

export function CliSyncModals(props: CliSyncModalsProps) {
    const {
        t,
        proxyUrl,
        apiKey,
        statuses,
        viewingConfig,
        onCloseViewingConfig,
        onViewConfigFile,
        restoreConfirmApp,
        onCancelRestore,
        onConfirmRestore,
        syncConfirmApp,
        onCancelSync,
        onConfirmSync,
        clearConfirmApp,
        onCancelClear,
        onConfirmClear,
        droidSyncModal,
        onCloseDroidModal,
        onDroidSyncDone,
        hermesSyncModal,
        onCloseHermesModal,
        onHermesSyncDone,
        openClawSyncModal,
        onCloseOpenClawModal,
        onOpenClawSyncDone,
        openCodeSyncModal,
        onCloseOpenCodeModal,
        onOpenCodeSyncDone,
        syncAccounts,
        getFormattedProxyUrl,
    } = props;

    return (
        <>
            {/* Config Viewer Modal */}
            {viewingConfig && (
                <div className="fixed inset-0 z-[300] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="bg-white dark:bg-base-100 rounded-2xl shadow-2xl border border-gray-200 dark:border-base-300 w-full max-w-2xl overflow-hidden animate-in zoom-in-95 duration-200">
                        <div className="px-6 py-4 border-b border-gray-100 dark:border-base-200 flex items-center justify-between bg-gray-50/50 dark:bg-base-200/50">
                            <div>
                                <h3 className="font-bold text-gray-900 dark:text-base-content flex items-center gap-2">
                                    <CodeXml size={18} className="text-blue-500" />
                                    {t('proxy.cli_sync.modal.view_title', { name: viewingConfig.app })}
                                </h3>
                                <div className="mt-2 flex gap-2">
                                    {viewingConfig.allFiles.map(file => (
                                        <button
                                            key={file}
                                            onClick={() => onViewConfigFile(viewingConfig.app, file)}
                                            className={cn(
                                                "px-3 py-1 text-[10px] font-bold rounded-lg transition-all border",
                                                viewingConfig.fileName === file
                                                    ? "bg-blue-500 text-white border-blue-500"
                                                    : "bg-white dark:bg-base-300 text-gray-400 border-gray-100 dark:border-base-400 hover:border-blue-200"
                                            )}
                                        >
                                            {file}
                                        </button>
                                    ))}
                                </div>
                            </div>
                            <div className="flex items-center gap-2">
                                <button
                                    onClick={() => void copyViewingConfigContent(t, viewingConfig)}
                                    className="btn btn-ghost btn-sm hover:bg-blue-50 hover:text-blue-600 dark:hover:bg-blue-900/20"
                                >
                                    <Copy size={16} />
                                </button>
                                <button
                                    onClick={onCloseViewingConfig}
                                    className="btn btn-ghost btn-sm hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20"
                                >
                                    <X size={18} />
                                </button>
                            </div>
                        </div>
                        <div className="p-6">
                            <div className="bg-gray-900 rounded-xl p-4 overflow-auto max-h-[50vh] border border-gray-800 shadow-inner">
                                <pre className="text-xs font-mono text-gray-300 leading-relaxed">
                                    {viewingConfig.content}
                                </pre>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {/* Restore default/backup confirmation modal */}
            <ModalDialog
                isOpen={!!restoreConfirmApp}
                title={restoreConfirmApp && statuses[restoreConfirmApp]?.has_backup
                    ? t('proxy.cli_sync.btn_restore_backup')
                    : t('proxy.cli_sync.btn_restore') || t('proxy.cli_sync.title')}
                message={restoreConfirmApp
                    ? (statuses[restoreConfirmApp]?.has_backup
                        ? t('proxy.cli_sync.restore_backup_confirm')
                        : t('proxy.cli_sync.restore_confirm', { name: restoreConfirmApp }))
                    : ''}
                onConfirm={onConfirmRestore}
                onCancel={onCancelRestore}
                isDestructive={true}
            />

            {/* Sync config confirmation modal (Issue #756) */}
            <ModalDialog
                isOpen={!!syncConfirmApp}
                title={t('proxy.cli_sync.sync_confirm_title')}
                message={syncConfirmApp ? t('proxy.cli_sync.sync_confirm_message', { name: syncConfirmApp }) : ''}
                onConfirm={onConfirmSync}
                onCancel={onCancelSync}
                isDestructive={true}
            />

            {/* Clear confirmation modal - OpenCode / Hermes / OpenClaw */}
            <ModalDialog
                isOpen={!!clearConfirmApp}
                title={clearConfirmApp === 'Hermes'
                    ? t('proxy.hermes_sync.clear_confirm_title', { defaultValue: 'Clear Hermes Configuration' })
                    : clearConfirmApp === 'OpenClaw'
                        ? t('proxy.openclaw_sync.clear_confirm_title', { defaultValue: 'Clear OpenClaw Configuration' })
                        : t('proxy.opencode_sync.clear_confirm_title', { defaultValue: 'Clear OpenCode Configuration' })}
                message={clearConfirmApp === 'Hermes'
                    ? t('proxy.hermes_sync.clear_confirm_message', { defaultValue: 'This will remove the Antigravity Manager provider from Hermes. Are you sure?' })
                    : clearConfirmApp === 'OpenClaw'
                        ? t('proxy.openclaw_sync.clear_confirm_message', { defaultValue: 'This will remove the Antigravity Manager provider from OpenClaw. Are you sure?' })
                        : t('proxy.opencode_sync.clear_confirm_message', { defaultValue: 'This will clear all OpenCode configurations including legacy settings. Are you sure?' })}
                onConfirm={onConfirmClear}
                onCancel={onCancelClear}
                isDestructive={true}
            />

            {/* OpenClaw modal */}
            {openClawSyncModal && (
                <OpenClawSyncModal
                    apiKey={apiKey}
                    getFormattedProxyUrl={getFormattedProxyUrl}
                    onClose={onCloseOpenClawModal}
                    onSyncDone={onOpenClawSyncDone}
                />
            )}

            {/* Hermes modal */}
            {hermesSyncModal && (
                <HermesSyncModal
                    apiKey={apiKey}
                    getFormattedProxyUrl={getFormattedProxyUrl}
                    onClose={onCloseHermesModal}
                    onSyncDone={onHermesSyncDone}
                />
            )}

            {/* Droid modal */}
            {droidSyncModal && (
                <DroidSyncModal
                    proxyUrl={proxyUrl}
                    apiKey={apiKey}
                    getFormattedProxyUrl={getFormattedProxyUrl}
                    onClose={onCloseDroidModal}
                    onSyncDone={onDroidSyncDone}
                />
            )}

            {/* OpenCode model selection modal */}
            {openCodeSyncModal && (
                <OpenCodeSyncModal
                    proxyUrl={proxyUrl}
                    apiKey={apiKey}
                    getFormattedProxyUrl={getFormattedProxyUrl}
                    syncAccounts={syncAccounts}
                    onClose={onCloseOpenCodeModal}
                    onSyncDone={onOpenCodeSyncDone}
                />
            )}
        </>
    );
}
