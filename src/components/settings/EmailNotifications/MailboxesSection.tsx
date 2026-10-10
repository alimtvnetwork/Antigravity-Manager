import { EmailNotificationsApi } from './useEmailNotifications';
import { ChevronDown, Database, FileCode, FileJson, FileSpreadsheet, FileText, Mail, Plus, RefreshCw, Server, SlidersHorizontal, Sparkles, Trash2, Upload } from 'lucide-react';

export function MailboxesSection(props: EmailNotificationsApi) {
    const { accounts, setIsImportModalOpen, testingAccountId, isActionsOpen, setIsActionsOpen, setIsSampleTemplatesOpen, actionsDropdownRef, handleOpenAddAccount, handleOpenEditAccount, handleDeleteAccount, handleSetDefault, handleTestSmtp, handleTestImap, handleExport, handleExportSingleAccount, handleLoadSampleMailboxes, handleBackupDb, handleRestoreDb } = props;

    return (
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
                    <div>
                        <h3 className="text-sm sm:text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Server className="w-4 h-4 text-blue-500" />
                            Email Accounts & Mailbox Pool
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400">
                            Mailbox pool with default sender prioritization and automatic failover swapping.
                        </p>
                    </div>

                    <div className="flex items-center gap-2 self-end sm:self-auto">
                        <button
                            type="button"
                            onClick={() => setIsSampleTemplatesOpen(true)}
                            className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-purple-200 dark:border-purple-900/50 bg-purple-50/50 dark:bg-purple-950/20 text-purple-700 dark:text-purple-300 hover:bg-purple-100 dark:hover:bg-purple-900/40 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                            title="AI Sample Instructions & Template Formats"
                        >
                            <Sparkles className="w-3.5 h-3.5 text-purple-600 dark:text-purple-400" />
                            <span>AI Templates</span>
                        </button>

                        <div className="relative" ref={actionsDropdownRef}>
                            <button
                                type="button"
                                onClick={() => setIsActionsOpen(!isActionsOpen)}
                                className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 text-gray-800 dark:text-slate-100 hover:bg-gray-50 dark:hover:bg-slate-700 hover:border-gray-400 dark:hover:border-slate-600 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                                title="Import / Export Mailboxes & Database Actions"
                            >
                                <SlidersHorizontal className="w-3.5 h-3.5 text-gray-600 dark:text-slate-300" />
                                <span>Export / Import Actions</span>
                                <ChevronDown className={`w-3.5 h-3.5 text-gray-500 dark:text-slate-400 transition-transform duration-150 ${isActionsOpen ? 'rotate-180' : ''}`} />
                            </button>

                            {isActionsOpen && (
                                <div className="absolute right-0 mt-1.5 w-52 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-lg z-50 py-1 divide-y divide-gray-100 dark:divide-slate-800 animate-in fade-in slide-in-from-top-1">
                                    <div className="py-1">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('json');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes as structured JSON"
                                        >
                                            <FileJson className="w-3.5 h-3.5 text-amber-500" />
                                            <span>Export JSON</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('yaml');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes as clean YAML"
                                        >
                                            <FileCode className="w-3.5 h-3.5 text-emerald-500" />
                                            <span>Export YAML</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('csv');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes in CSV format"
                                        >
                                            <FileText className="w-3.5 h-3.5 text-blue-500" />
                                            <span>Export CSV</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('xlsx');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes to Microsoft Excel format"
                                        >
                                            <FileSpreadsheet className="w-3.5 h-3.5 text-emerald-500" />
                                            <span>Export Excel</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                setIsImportModalOpen(true);
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Import mailboxes from JSON, YAML, or CSV files"
                                        >
                                            <Upload className="w-3.5 h-3.5 text-indigo-500" />
                                            <span>Import Accounts</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleLoadSampleMailboxes();
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-purple-700 dark:text-purple-300 hover:bg-purple-50 dark:hover:bg-purple-950/30 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Preload sample mailbox accounts for testing"
                                        >
                                            <Sparkles className="w-3.5 h-3.5 text-purple-500" />
                                            <span>Load Sample Mailboxes</span>
                                        </button>
                                    </div>
                                    <div className="py-1">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleBackupDb();
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <Database className="w-3.5 h-3.5 text-purple-500" />
                                            <span>Backup Vault DB</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleRestoreDb();
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <RefreshCw className="w-3.5 h-3.5 text-cyan-500" />
                                            <span>Restore Vault DB</span>
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>

                        <button
                            type="button"
                            onClick={handleOpenAddAccount}
                            className="px-3 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white rounded-lg transition-all flex items-center gap-1 shadow-sm hover:shadow hover:scale-[1.02] cursor-pointer"
                            title="Add New Mailbox Account"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>Add Mailbox</span>
                        </button>
                    </div>
                </div>

                {accounts.length === 0 ? (
                    <div className="py-6 px-4 text-center border border-dashed border-gray-200 dark:border-slate-800 rounded-xl flex flex-col items-center justify-center bg-gray-50/50 dark:bg-slate-900/40">
                        <Mail className="w-6 h-6 text-gray-400 dark:text-slate-500 mx-auto mb-1.5" />
                        <p className="text-xs font-medium text-gray-600 dark:text-slate-300">No mailboxes configured in vault</p>
                        <p className="text-[11px] text-gray-400 dark:text-slate-500 mt-0.5 mb-3">Add an SMTP/IMAP account to enable dispatch and remote command execution</p>
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={handleOpenAddAccount}
                                className="px-3.5 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center gap-1.5 shadow-sm cursor-pointer"
                            >
                                <Plus className="w-3.5 h-3.5" />
                                <span>Add Mailbox</span>
                            </button>
                            <button
                                type="button"
                                onClick={handleLoadSampleMailboxes}
                                className="px-3.5 py-1.5 text-xs font-semibold bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors flex items-center gap-1.5 shadow-sm cursor-pointer"
                                title="Preload sample mailbox accounts for testing"
                            >
                                <Sparkles className="w-3.5 h-3.5" />
                                <span>Load Sample Mailboxes</span>
                            </button>
                        </div>
                    </div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full text-left text-xs border-collapse">
                            <thead>
                                <tr className="border-b border-gray-200 dark:border-slate-800 text-gray-500 dark:text-slate-400">
                                    <th className="py-2 px-2.5 font-semibold">Alias & Email</th>
                                    <th className="py-2 px-2.5 font-semibold">SMTP Host</th>
                                    <th className="py-2 px-2.5 font-semibold">IMAP Host</th>
                                    <th className="py-2 px-2.5 font-semibold">Enc</th>
                                    <th className="py-2 px-2.5 font-semibold">Role</th>
                                    <th className="py-2 px-2.5 font-semibold text-right whitespace-nowrap min-w-[290px]">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-gray-100 dark:divide-slate-800">
                                {accounts.map((acc) => (
                                    <tr key={acc.id} className="hover:bg-gray-100/70 dark:hover:bg-slate-800/80 transition-colors group">
                                        <td className="py-2 px-2.5">
                                            <div className="font-semibold text-gray-800 dark:text-slate-100 group-hover:text-blue-600 dark:group-hover:text-white leading-tight transition-colors">{acc.alias}</div>
                                            <div className="text-[11px] text-gray-500 dark:text-slate-400 group-hover:text-gray-700 dark:group-hover:text-slate-200 leading-tight transition-colors">{acc.email}</div>
                                        </td>
                                        <td className="py-2 px-2.5 font-mono text-gray-600 dark:text-slate-300 group-hover:text-gray-900 dark:group-hover:text-slate-100 transition-colors">
                                            {acc.smtp_host}:{acc.smtp_port}
                                        </td>
                                        <td className="py-2 px-2.5 font-mono text-gray-600 dark:text-slate-300 group-hover:text-gray-900 dark:group-hover:text-slate-100 transition-colors">
                                            {acc.imap_host}:{acc.imap_port}
                                        </td>
                                        <td className="py-2 px-2.5">
                                            <span className="px-1.5 py-0.5 rounded bg-gray-100 dark:bg-slate-800 border border-gray-200/80 dark:border-slate-700 text-gray-700 dark:text-slate-300 text-[10px] font-mono">
                                                {acc.encryption_type}
                                            </span>
                                        </td>
                                        <td className="py-2 px-2.5">
                                            {acc.is_default ? (
                                                <span className="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300 border border-blue-200/60 dark:border-blue-800/50 text-[10px] font-semibold">
                                                    Default Sender
                                                </span>
                                            ) : (
                                                <button
                                                    onClick={() => handleSetDefault(acc.id)}
                                                    className="text-[10px] text-gray-400 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-400 underline cursor-pointer"
                                                >
                                                    Set Default
                                                </button>
                                            )}
                                        </td>
                                        <td className="py-2 px-2.5 text-right space-x-1 whitespace-nowrap min-w-[290px]">
                                            <button
                                                onClick={() => handleTestSmtp(acc.id)}
                                                disabled={testingAccountId === acc.id}
                                                className="px-2 py-0.5 rounded bg-sky-50 dark:bg-sky-950/40 text-sky-600 dark:text-sky-400 hover:bg-sky-100 dark:hover:bg-sky-900/50 border border-sky-200/60 dark:border-sky-800/50 text-[10px] font-medium cursor-pointer transition-colors"
                                                title="Test Outbound SMTP"
                                            >
                                                Test SMTP
                                            </button>
                                            <button
                                                onClick={() => handleTestImap(acc.id)}
                                                disabled={testingAccountId === acc.id}
                                                className="px-2 py-0.5 rounded bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 border border-indigo-200/60 dark:border-indigo-800/50 text-[10px] font-medium cursor-pointer transition-colors"
                                                title="Test Inbound IMAP"
                                            >
                                                Test IMAP
                                            </button>
                                            <button
                                                onClick={() => handleOpenEditAccount(acc)}
                                                className="px-2 py-0.5 rounded hover:bg-gray-100 dark:hover:bg-slate-700 text-gray-600 dark:text-slate-300 text-[10px] cursor-pointer transition-colors"
                                            >
                                                Edit
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => handleExportSingleAccount(acc)}
                                                className="px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/40 text-amber-600 dark:text-amber-400 hover:bg-amber-100 dark:hover:bg-amber-900/50 border border-amber-200/60 dark:border-amber-800/50 text-[10px] font-medium cursor-pointer transition-colors"
                                                title="Export this account (JSON / YAML / CSV)"
                                            >
                                                Export
                                            </button>
                                            <button
                                                onClick={() => handleDeleteAccount(acc.id)}
                                                className="px-1.5 py-0.5 rounded hover:bg-red-50 dark:hover:bg-red-950/40 text-red-600 dark:text-red-400 text-[10px] cursor-pointer transition-colors"
                                            >
                                                <Trash2 className="w-3 h-3 inline" />
                                            </button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

    );
}
