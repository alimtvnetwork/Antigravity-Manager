import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ShieldCheck, Download, Upload, Mail, Loader2, X } from 'lucide-react';
import { useBackupExport } from './unified-backup/useBackupExport';
import { useBackupImport } from './unified-backup/useBackupImport';
import { ExportTab } from './unified-backup/ExportTab';
import { ImportTab } from './unified-backup/ImportTab';
import type { BackupTab, UnifiedBackupModalProps } from './unified-backup/types';

export function UnifiedBackupModal({ isOpen, onClose, initialTab = 'export' }: UnifiedBackupModalProps) {
    const { t } = useTranslation();
    const [activeTab, setActiveTab] = useState<BackupTab>(initialTab);

    const backupExport = useBackupExport({ isOpen, onClose });
    const backupImport = useBackupImport({ onClose });

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs animate-in fade-in duration-200">
            <div className="relative w-full max-w-xl bg-white dark:bg-slate-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-slate-800 overflow-hidden">
                {/* Modal Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-slate-800 bg-gray-50/50 dark:bg-slate-800/50">
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-xl bg-purple-50 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400">
                            <ShieldCheck className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-base font-semibold text-gray-900 dark:text-gray-100">
                                {t('backup.title', 'Unified Backup & Encrypted Vault')}
                            </h2>
                            <p className="text-xs text-gray-500 dark:text-gray-400">
                                {t('backup.subtitle', 'Export accounts, configurations, or restore encrypted archives')}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Tab Switcher */}
                <div className="flex border-b border-gray-100 dark:border-slate-800 px-6 pt-3 bg-gray-50/30 dark:bg-slate-800/30">
                    <button
                        type="button"
                        onClick={() => setActiveTab('export')}
                        className={`flex items-center gap-2 pb-2.5 px-3 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                            activeTab === 'export'
                                ? 'border-purple-600 text-purple-600 dark:text-purple-400'
                                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        }`}
                    >
                        <Download className="w-3.5 h-3.5" />
                        <span>{t('backup.export_tab', 'Export Backup')}</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => setActiveTab('import')}
                        className={`flex items-center gap-2 pb-2.5 px-3 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                            activeTab === 'import'
                                ? 'border-purple-600 text-purple-600 dark:text-purple-400'
                                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        }`}
                    >
                        <Upload className="w-3.5 h-3.5" />
                        <span>{t('backup.import_tab', 'Restore Backup')}</span>
                    </button>
                </div>

                {/* Modal Body */}
                <div className="p-6 space-y-5 max-h-[75vh] overflow-y-auto">
                    {activeTab === 'export' ? (
                        <ExportTab
                            exportScope={backupExport.exportScope}
                            setExportScope={backupExport.setExportScope}
                            includeAuditHistory={backupExport.includeAuditHistory}
                            setIncludeAuditHistory={backupExport.setIncludeAuditHistory}
                            usePassword={backupExport.usePassword}
                            setUsePassword={backupExport.setUsePassword}
                            password={backupExport.password}
                            setPassword={backupExport.setPassword}
                            confirmPassword={backupExport.confirmPassword}
                            setConfirmPassword={backupExport.setConfirmPassword}
                            showPassword={backupExport.showPassword}
                            setShowPassword={backupExport.setShowPassword}
                            isExporting={backupExport.isExporting}
                            includeMainNode={backupExport.includeMainNode}
                            setIncludeMainNode={backupExport.setIncludeMainNode}
                            isDeployingFleet={backupExport.isDeployingFleet}
                            gitmapAvailable={backupExport.gitmapAvailable}
                            fleetDeployResult={backupExport.fleetDeployResult}
                            onDeployToFleet={backupExport.handleDeployToFleet}
                        />
                    ) : (
                        <ImportTab
                            fileInputRef={backupImport.fileInputRef}
                            importFileName={backupImport.importFileName}
                            isImportEncrypted={backupImport.isImportEncrypted}
                            decryptPassword={backupImport.decryptPassword}
                            setDecryptPassword={backupImport.setDecryptPassword}
                            parsedDataPreview={backupImport.parsedDataPreview}
                            onSelectImportFile={backupImport.handleSelectImportFile}
                            onFileInputChange={backupImport.handleFileInputChange}
                            onDecryptPreview={backupImport.handleDecryptPreview}
                        />
                    )}
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-between px-6 py-4 border-t border-gray-100 dark:border-slate-800 bg-gray-50/50 dark:bg-slate-800/50">
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-2 text-xs font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 rounded-xl transition-colors cursor-pointer"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>

                    {activeTab === 'export' ? (
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={backupExport.handleSendToEmail}
                                disabled={backupExport.isExporting}
                                className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-900/30 border border-purple-200 dark:border-purple-800 hover:bg-purple-100 dark:hover:bg-purple-900/50 rounded-xl disabled:opacity-50 transition-all cursor-pointer"
                                title="Send backup directly to your outbound mailbox"
                            >
                                <Mail className="w-3.5 h-3.5" />
                                <span>{t('backup.send_to_email', 'Send to Mailbox')}</span>
                            </button>
                            <button
                                type="button"
                                onClick={backupExport.handleSaveToFile}
                                disabled={backupExport.isExporting}
                                className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 rounded-xl shadow-xs disabled:opacity-50 transition-all cursor-pointer"
                            >
                                {backupExport.isExporting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Download className="w-3.5 h-3.5" />}
                                <span>{t('backup.save_file_btn', 'Save Backup File')}</span>
                            </button>
                        </div>
                    ) : (
                        <button
                            type="button"
                            onClick={backupImport.handleExecuteRestore}
                            disabled={!backupImport.parsedDataPreview || backupImport.isImporting}
                            className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 rounded-xl shadow-xs disabled:opacity-50 transition-all cursor-pointer"
                        >
                            {backupImport.isImporting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Upload className="w-3.5 h-3.5" />}
                            <span>{t('backup.restore_now_btn', 'Execute Restore')}</span>
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
}
