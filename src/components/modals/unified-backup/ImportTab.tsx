import type { ChangeEvent, RefObject } from 'react';
import { useTranslation } from 'react-i18next';
import { Upload, KeyRound, CheckCircle2 } from 'lucide-react';
import type { ImportPreviewData } from './types';

export interface ImportTabProps {
    fileInputRef: RefObject<HTMLInputElement | null>;
    importFileName: string | null;
    isImportEncrypted: boolean;
    decryptPassword: string;
    setDecryptPassword: (value: string) => void;
    parsedDataPreview: ImportPreviewData | null;
    onSelectImportFile: () => void;
    onFileInputChange: (e: ChangeEvent<HTMLInputElement>) => void;
    onDecryptPreview: () => void;
}

export function ImportTab(props: ImportTabProps) {
    const { t } = useTranslation();
    const {
        fileInputRef,
        importFileName,
        isImportEncrypted,
        decryptPassword,
        setDecryptPassword,
        parsedDataPreview,
        onSelectImportFile,
        onFileInputChange,
        onDecryptPreview,
    } = props;

    return (
        <>
            {/* Hidden browser input */}
            <input
                ref={fileInputRef}
                type="file"
                accept=".json,.agmbackup,application/json"
                style={{ display: 'none' }}
                onChange={onFileInputChange}
            />

            {/* Import File Selection */}
            <div className="space-y-3">
                <div
                    onClick={onSelectImportFile}
                    className="p-6 border-2 border-dashed border-gray-300 dark:border-slate-700 rounded-2xl text-center hover:border-purple-500 dark:hover:border-purple-400 cursor-pointer transition-all bg-gray-50/50 dark:bg-slate-800/40 group"
                >
                    <div className="mx-auto w-10 h-10 rounded-xl bg-purple-50 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 flex items-center justify-center group-hover:scale-110 transition-transform">
                        <Upload className="w-5 h-5" />
                    </div>
                    <div className="mt-2 text-xs font-semibold text-gray-800 dark:text-gray-200">
                        {importFileName || t('backup.click_to_select', 'Click to choose backup file (.json or .agmbackup)')}
                    </div>
                    <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400">
                        {t('backup.drop_support', 'Supports plain JSON account lists and password-encrypted AGM backups.')}
                    </p>
                </div>

                {/* Password prompt if encrypted */}
                {isImportEncrypted && !parsedDataPreview && (
                    <div className="p-4 bg-purple-50/50 dark:bg-purple-950/20 border border-purple-200 dark:border-purple-800/40 rounded-xl space-y-3 animate-in fade-in">
                        <div className="flex items-center gap-2 text-xs font-semibold text-purple-900 dark:text-purple-300">
                            <KeyRound className="w-4 h-4" />
                            <span>{t('backup.enter_decrypt_password', 'Encrypted Archive: Enter Password')}</span>
                        </div>
                        <div className="flex gap-2">
                            <input
                                type="password"
                                placeholder={t('backup.password', 'Password')}
                                value={decryptPassword}
                                onChange={(e) => setDecryptPassword(e.target.value)}
                                className="grow px-3 py-1.5 text-xs border border-purple-300 dark:border-purple-800 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100"
                            />
                            <button
                                type="button"
                                onClick={onDecryptPreview}
                                disabled={!decryptPassword}
                                className="px-3 py-1.5 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 disabled:opacity-50 rounded-lg cursor-pointer"
                            >
                                {t('backup.decrypt_btn', 'Decrypt')}
                            </button>
                        </div>
                    </div>
                )}

                {/* Decrypted Preview Summary */}
                {parsedDataPreview && (
                    <div className="p-4 bg-emerald-50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-800/40 rounded-xl space-y-2 text-xs text-emerald-900 dark:text-emerald-300 animate-in fade-in">
                        <div className="flex items-center gap-2 font-semibold">
                            <CheckCircle2 className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                            <span>{t('backup.ready_to_restore', 'Backup Verified & Ready to Restore')}</span>
                        </div>
                        <div className="grid grid-cols-2 gap-2 pt-1 text-[11px] text-gray-700 dark:text-gray-300">
                            <div>
                                • Accounts detected: <strong>{parsedDataPreview.accountsCount}</strong>
                            </div>
                            <div>
                                • Scope: <strong>{parsedDataPreview.backupType.toUpperCase()}</strong>
                            </div>
                            <div>
                                • Config present: <strong>{parsedDataPreview.hasConfig ? 'Yes' : 'No'}</strong>
                            </div>
                            <div>
                                • Instances: <strong>{parsedDataPreview.instancesCount}</strong>
                            </div>
                            {parsedDataPreview.backupType === 'full' && (
                                <div>
                                    • Audit Records: <strong>{parsedDataPreview.auditHistoryCount ?? 0}</strong>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </div>
        </>
    );
}
