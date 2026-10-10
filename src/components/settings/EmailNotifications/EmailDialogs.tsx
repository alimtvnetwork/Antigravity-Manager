import { EmailNotificationsApi } from './useEmailNotifications';
import AiSampleTemplatesModal from '../ai-sample-templates-modal';
import MailboxExportModal from '../MailboxExportModal';
import ModalDialog from '../../common/ModalDialog';
import { Upload } from 'lucide-react';

export function EmailDialogs(props: EmailNotificationsApi) {
    const { isImportModalOpen, setIsImportModalOpen, importFormat, setImportFormat, importPayload, setImportPayload, fileInputRef, isSampleTemplatesOpen, setIsSampleTemplatesOpen, exportModalState, setExportModalState, handleImportSubmit, handleFileUpload } = props;

    return (
        <>
                {/* Import Modal */}
                <ModalDialog
                    isOpen={isImportModalOpen}
                    title="Import Email Accounts & Settings"
                    type="confirm"
                    confirmText="Execute Import"
                    cancelText="Cancel"
                    onConfirm={handleImportSubmit}
                    onCancel={() => setIsImportModalOpen(false)}
                >
                    <div className="space-y-4 text-xs">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <span className="font-semibold text-gray-700 dark:text-gray-300">Format:</span>
                                <label className="flex items-center gap-1.5 cursor-pointer">
                                    <input
                                        type="radio"
                                        name="import_fmt"
                                        value="json"
                                        checked={importFormat === 'json'}
                                        onChange={() => setImportFormat('json')}
                                    />
                                    <span>JSON</span>
                                </label>
                                <label className="flex items-center gap-1.5 cursor-pointer">
                                    <input
                                        type="radio"
                                        name="import_fmt"
                                        value="yaml"
                                        checked={importFormat === 'yaml'}
                                        onChange={() => setImportFormat('yaml')}
                                    />
                                    <span>YAML</span>
                                </label>
                                <label className="flex items-center gap-1.5 cursor-pointer">
                                    <input
                                        type="radio"
                                        name="import_fmt"
                                        value="csv"
                                        checked={importFormat === 'csv'}
                                        onChange={() => setImportFormat('csv')}
                                    />
                                    <span>CSV</span>
                                </label>
                                <label className="flex items-center gap-1.5 cursor-pointer">
                                    <input
                                        type="radio"
                                        name="import_fmt"
                                        value="xlsx"
                                        checked={importFormat === 'xlsx'}
                                        onChange={() => setImportFormat('xlsx')}
                                    />
                                    <span>Excel (XML / XLSX)</span>
                                </label>
                            </div>

                            <div>
                                <input
                                    type="file"
                                    ref={fileInputRef}
                                    className="hidden"
                                    accept=".json,.yaml,.yml,.csv,.xml,.xlsx,.xls"
                                    onChange={handleFileUpload}
                                />
                                <button
                                    type="button"
                                    onClick={() => fileInputRef.current?.click()}
                                    className="px-2.5 py-1 text-[11px] font-medium bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 rounded text-gray-700 dark:text-slate-200 border border-gray-200 dark:border-slate-700 flex items-center gap-1.5 transition-colors cursor-pointer"
                                >
                                    <Upload className="w-3 h-3 text-blue-500" />
                                    Browse File
                                </button>
                            </div>
                        </div>

                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">
                                Paste or Load {importFormat.toUpperCase()} Content
                            </label>
                            <textarea
                                rows={8}
                                value={importPayload}
                                onChange={(e) => setImportPayload(e.target.value)}
                                placeholder={`Paste your ${importFormat.toUpperCase()} here or click Browse File...`}
                                className="w-full px-3 py-2 font-mono text-[11px] border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                    </div>
                </ModalDialog>

                {/* AI Sample Instructions & Template Modal */}
                <AiSampleTemplatesModal
                    isOpen={isSampleTemplatesOpen}
                    onClose={() => setIsSampleTemplatesOpen(false)}
                />

                {/* Mailbox Export Preview & Save Modal */}
                <MailboxExportModal
                    isOpen={exportModalState.isOpen}
                    onClose={() => setExportModalState({ isOpen: false })}
                    singleAccount={exportModalState.singleAccount}
                    allAccounts={exportModalState.allAccounts}
                    initialFormat={exportModalState.initialFormat}
                />
        </>
    );
}
