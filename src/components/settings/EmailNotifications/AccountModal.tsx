import { EmailNotificationsApi } from './useEmailNotifications';
import ModalDialog from '../../common/ModalDialog';
import { AlertCircle, CheckCircle2, Copy, FileCode, FileJson, FileText, Key, Loader2, RefreshCw, Send, Sparkles, Upload } from 'lucide-react';
import { showToast } from '../../common/ToastContainer';

export function AccountModal(props: EmailNotificationsApi) {
    const { isAccountModalOpen, setIsAccountModalOpen, editingAccount, setEditingAccount, isModalQuickImportOpen, setIsModalQuickImportOpen, modalQuickImportText, setModalQuickImportText, showAiJsonSyntax, setShowAiJsonSyntax, singleAccountFileInputRef, isTestingDirect, testResult, handleEmailChange, handleTestDirectConnection, handleSaveAccount, handleExportSingleAccount, handleQuickImportSingle, handleSingleFileUpload, emailFormatStatus, testResultStatus } = props;

    return (
        <>
                {/* Account Add/Edit Modal */}
                <ModalDialog
                    isOpen={isAccountModalOpen}
                    title={editingAccount.id ? 'Edit Mailbox Configuration' : 'Add Mailbox to Secure Split Vault'}
                    type="confirm"
                    maxWidth="max-w-xl"
                    confirmText={editingAccount.id ? 'Save Mailbox' : 'Add Mailbox to Vault'}
                    cancelText="Cancel"
                    onConfirm={handleSaveAccount}
                    onCancel={() => setIsAccountModalOpen(false)}
                >
                    <form
                        onSubmit={(e) => {
                            e.preventDefault();
                            handleSaveAccount();
                        }}
                        className="space-y-3.5 text-xs"
                    >
                        {/* Single Account Import / Export Action Bar */}
                        <div className="flex flex-wrap items-center justify-between gap-2 p-2 rounded-lg bg-gray-50 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700">
                            <div className="flex items-center gap-1.5">
                                <span className="text-[11px] font-semibold text-gray-500 dark:text-slate-400">Account IO:</span>
                                <button
                                    type="button"
                                    onClick={() => setIsModalQuickImportOpen(!isModalQuickImportOpen)}
                                    className={`px-2 py-1 text-[11px] font-medium rounded-md border flex items-center gap-1 transition-colors cursor-pointer ${
                                        isModalQuickImportOpen
                                            ? 'bg-blue-600 text-white border-blue-600'
                                            : 'bg-white dark:bg-slate-800 border-gray-200 dark:border-slate-700 text-gray-700 dark:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-700'
                                    }`}
                                    title="Import single account from JSON, YAML, or CSV"
                                >
                                    <Upload className="w-3 h-3 text-indigo-500" />
                                    <span>Import (JSON / YAML / CSV)</span>
                                </button>
                            </div>

                            <div className="flex items-center gap-1">
                                <span className="text-[10px] text-gray-400 dark:text-slate-500 mr-0.5">Export:</span>
                                <button
                                    type="button"
                                    onClick={() => handleExportSingleAccount(editingAccount, 'json')}
                                    className="px-1.5 py-0.5 text-[10px] font-medium rounded border border-amber-200 dark:border-amber-800/60 bg-amber-50/50 dark:bg-amber-950/20 text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-900/40 flex items-center gap-1 cursor-pointer transition-colors"
                                    title="Export single account as JSON"
                                >
                                    <FileJson className="w-3 h-3 text-amber-500" />
                                    <span>JSON</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => handleExportSingleAccount(editingAccount, 'yaml')}
                                    className="px-1.5 py-0.5 text-[10px] font-medium rounded border border-emerald-200 dark:border-emerald-800/60 bg-emerald-50/50 dark:bg-emerald-950/20 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 flex items-center gap-1 cursor-pointer transition-colors"
                                    title="Export single account as YAML"
                                >
                                    <FileCode className="w-3 h-3 text-emerald-500" />
                                    <span>YAML</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => handleExportSingleAccount(editingAccount, 'csv')}
                                    className="px-1.5 py-0.5 text-[10px] font-medium rounded border border-blue-200 dark:border-blue-800/60 bg-blue-50/50 dark:bg-blue-950/20 text-blue-700 dark:text-blue-300 hover:bg-blue-100 dark:hover:bg-blue-900/40 flex items-center gap-1 cursor-pointer transition-colors"
                                    title="Export single account as CSV"
                                >
                                    <FileText className="w-3 h-3 text-blue-500" />
                                    <span>CSV</span>
                                </button>
                            </div>
                        </div>

                        {/* Expandable Quick Import Panel */}
                        {isModalQuickImportOpen && (
                            <div className="p-3 rounded-lg border border-indigo-200 dark:border-indigo-900/50 bg-indigo-50/40 dark:bg-indigo-950/20 space-y-2 animate-in fade-in slide-in-from-top-1">
                                <div className="flex items-center justify-between">
                                    <span className="text-[11px] font-semibold text-indigo-900 dark:text-indigo-200 flex items-center gap-1.5">
                                        <Upload className="w-3.5 h-3.5 text-indigo-600 dark:text-indigo-400" />
                                        Import Account Config (Paste or Browse File)
                                    </span>
                                    <div>
                                        <input
                                            type="file"
                                            ref={singleAccountFileInputRef}
                                            className="hidden"
                                            accept=".json,.yaml,.yml,.csv,.txt"
                                            onChange={handleSingleFileUpload}
                                        />
                                        <button
                                            type="button"
                                            onClick={() => singleAccountFileInputRef.current?.click()}
                                            className="px-2 py-0.5 text-[10px] font-medium bg-white dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-700 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 rounded flex items-center gap-1 cursor-pointer"
                                        >
                                            <Upload className="w-3 h-3" />
                                            Browse File
                                        </button>
                                    </div>
                                </div>
                                <textarea
                                    rows={3}
                                    value={modalQuickImportText}
                                    onChange={(e) => setModalQuickImportText(e.target.value)}
                                    placeholder="Paste single account JSON, YAML, or CSV snippet here to auto-fill form..."
                                    className="w-full px-2.5 py-1.5 font-mono text-[11px] border border-indigo-200 dark:border-indigo-800 rounded bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 focus:outline-none focus:ring-1 focus:ring-indigo-500"
                                />
                                <div className="flex items-center justify-end gap-2">
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setIsModalQuickImportOpen(false);
                                            setModalQuickImportText('');
                                        }}
                                        className="px-2 py-1 text-[11px] text-gray-500 hover:text-gray-700 dark:text-slate-400 cursor-pointer"
                                    >
                                        Cancel
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => handleQuickImportSingle(modalQuickImportText)}
                                        className="px-3 py-1 text-[11px] font-semibold bg-indigo-600 hover:bg-indigo-700 text-white rounded transition-colors cursor-pointer shadow-xs"
                                    >
                                        Apply to Form
                                    </button>
                                </div>
                            </div>
                        )}

                        {/* One-Click AI Instructions with Embedded JSON Syntax Segment */}
                        <div className="rounded-lg bg-blue-50/70 dark:bg-blue-950/30 border border-blue-100 dark:border-blue-900/40 p-2.5 space-y-2">
                            <div className="flex items-center justify-between gap-2">
                                <div className="flex items-center gap-1.5 text-blue-700 dark:text-blue-300">
                                    <Sparkles className="w-3.5 h-3.5 shrink-0 text-blue-600 dark:text-blue-400" />
                                    <span className="font-semibold text-[11px]">AI Automated Mailbox Configuration</span>
                                </div>
                                <div className="flex items-center gap-1.5">
                                    <button
                                        type="button"
                                        onClick={() => setShowAiJsonSyntax(!showAiJsonSyntax)}
                                        className="px-2 py-0.5 text-[10px] font-medium bg-blue-100/70 dark:bg-blue-900/50 hover:bg-blue-200/70 dark:hover:bg-blue-900/80 text-blue-800 dark:text-blue-200 rounded flex items-center gap-1 transition-colors cursor-pointer"
                                    >
                                        <FileJson className="w-3 h-3" />
                                        <span>{showAiJsonSyntax ? 'Hide JSON Format' : 'View JSON Format'}</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            const instructions = `Generate or configure a mailbox for Antigravity-Manager matching this exact JSON specification segment:

        \`\`\`json
        {
          "alias": "${editingAccount.alias || 'Primary Mailbox'}",
          "email": "${editingAccount.email || 'your-email@gmail.com'}",
          "password": "<generated-app-password>",
          "smtp_host": "${editingAccount.smtp_host || 'smtp.gmail.com'}",
          "smtp_port": ${editingAccount.smtp_port || 587},
          "imap_host": "${editingAccount.imap_host || 'imap.gmail.com'}",
          "imap_port": ${editingAccount.imap_port || 993},
          "encryption_type": "${editingAccount.encryption_type || 'TLS'}",
          "is_default": ${editingAccount.is_default},
          "is_active": ${editingAccount.is_active}
        }
        \`\`\`

        Instructions: Provide the app-password and verified SMTP/IMAP settings inside the JSON segment so it can be pasted directly into Antigravity-Manager's mailbox import.`;
                                            navigator.clipboard.writeText(instructions);
                                            showToast('AI instructions with embedded JSON syntax copied to clipboard', 'success');
                                        }}
                                        className="px-2.5 py-1 text-[11px] font-semibold bg-white dark:bg-slate-800 hover:bg-blue-50 dark:hover:bg-blue-900/40 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-800 rounded-md flex items-center gap-1.5 transition-all shadow-xs cursor-pointer"
                                        title="Copy AI Instructions with Embedded JSON Syntax to Clipboard"
                                    >
                                        <Copy className="w-3 h-3" />
                                        <span>Copy AI Instructions</span>
                                    </button>
                                </div>
                            </div>

                            {/* Embedded JSON Syntax Segment Preview */}
                            {showAiJsonSyntax && (
                                <div className="pt-1.5 border-t border-blue-200/50 dark:border-blue-900/40">
                                    <div className="text-[10px] text-blue-700 dark:text-blue-300 font-medium mb-1">
                                        Embedded JSON Syntax Segment:
                                    </div>
                                    <pre className="p-2 bg-slate-950 text-amber-300 font-mono text-[10px] rounded-md overflow-x-auto leading-tight select-all">
        {JSON.stringify(
        {
            alias: editingAccount.alias || 'Primary Mailbox',
            email: editingAccount.email || 'your-email@gmail.com',
            password: editingAccount.password || '<app-password>',
            smtp_host: editingAccount.smtp_host || 'smtp.gmail.com',
            smtp_port: editingAccount.smtp_port || 587,
            imap_host: editingAccount.imap_host || 'imap.gmail.com',
            imap_port: editingAccount.imap_port || 993,
            encryption_type: editingAccount.encryption_type || 'TLS',
            is_default: editingAccount.is_default,
            is_active: editingAccount.is_active,
        },
        null,
        2
        )}
                                    </pre>
                                </div>
                            )}
                        </div>

                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Email Address</label>
                            <input
                                type="email"
                                placeholder="e.g. ai-agm-tool-v2@hire-seoexperts.com"
                                value={editingAccount.email}
                                onChange={(e) => handleEmailChange(e.target.value)}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                            {emailFormatStatus === 'valid' && (
                                <div className="flex items-center gap-1 text-[11px] text-emerald-600 dark:text-emerald-400 mt-1">
                                    <CheckCircle2 className="w-3.5 h-3.5 shrink-0" />
                                    <span>Valid email detected. Host and port settings auto-configured.</span>
                                </div>
                            )}
                            {emailFormatStatus === 'invalid' && (
                                <div className="flex items-center gap-1 text-[11px] text-amber-600 dark:text-amber-400 mt-1">
                                    <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                                    <span>Please enter a complete email address (e.g. user@domain.com)</span>
                                </div>
                            )}
                        </div>

                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Account Alias</label>
                            <input
                                type="text"
                                placeholder="e.g. Primary Gmail, Alerts Mailer"
                                value={editingAccount.alias}
                                onChange={(e) => setEditingAccount({ ...editingAccount, alias: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>

                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">
                                Password / App Password
                                {editingAccount.id && <span className="text-gray-400 dark:text-slate-500 ml-1 font-normal">(Leave blank to keep existing)</span>}
                            </label>
                            <div className="relative">
                                <input
                                    type="password"
                                    placeholder={editingAccount.id ? '••••••••' : 'Application password / secret'}
                                    value={editingAccount.password || ''}
                                    onChange={(e) => setEditingAccount({ ...editingAccount, password: e.target.value })}
                                    className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                />
                                <Key className="w-3.5 h-3.5 text-gray-400 absolute right-3 top-2.5" />
                            </div>
                            <p className="text-[11px] text-gray-400 dark:text-slate-400 mt-1">
                                Stored in isolated split database <code>email_passwords.db</code> with salted SSH RSA identity and machine-bound encryption.
                            </p>
                        </div>

                        <div className="grid grid-cols-3 gap-3">
                            <div className="col-span-2">
                                <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Outgoing Server (SMTP Host)</label>
                                <input
                                    type="text"
                                    value={editingAccount.smtp_host}
                                    onChange={(e) => setEditingAccount({ ...editingAccount, smtp_host: e.target.value })}
                                    className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                                />
                            </div>
                            <div>
                                <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">SMTP Port</label>
                                <input
                                    type="number"
                                    value={editingAccount.smtp_port}
                                    onChange={(e) =>
                                        setEditingAccount({ ...editingAccount, smtp_port: parseInt(e.target.value) || 465 })
                                    }
                                    className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                                />
                            </div>
                        </div>
                        {/* SMTP Port Preset Pills */}
                        <div className="flex items-center gap-1.5 -mt-1.5">
                            <span className="text-[10px] text-gray-400 dark:text-slate-400">Presets:</span>
                            <button
                                type="button"
                                onClick={() => setEditingAccount({ ...editingAccount, smtp_port: 465, encryption_type: 'SSL' })}
                                className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.smtp_port === 465 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                            >
                                465 (SSL)
                            </button>
                            <button
                                type="button"
                                onClick={() => setEditingAccount({ ...editingAccount, smtp_port: 587, encryption_type: 'TLS' })}
                                className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.smtp_port === 587 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                            >
                                587 (TLS)
                            </button>
                            <button
                                type="button"
                                onClick={() => setEditingAccount({ ...editingAccount, smtp_port: 25, encryption_type: 'NONE' })}
                                className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.smtp_port === 25 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                            >
                                25 (Plain)
                            </button>
                        </div>

                        <div className="grid grid-cols-3 gap-3">
                            <div className="col-span-2">
                                <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Incoming Server (IMAP Host)</label>
                                <input
                                    type="text"
                                    value={editingAccount.imap_host}
                                    onChange={(e) => setEditingAccount({ ...editingAccount, imap_host: e.target.value })}
                                    className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                                />
                            </div>
                            <div>
                                <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">IMAP Port</label>
                                <input
                                    type="number"
                                    value={editingAccount.imap_port}
                                    onChange={(e) =>
                                        setEditingAccount({ ...editingAccount, imap_port: parseInt(e.target.value) || 993 })
                                    }
                                    className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                                />
                            </div>
                        </div>
                        {/* IMAP Port Preset Pills */}
                        <div className="flex items-center gap-1.5 -mt-1.5">
                            <span className="text-[10px] text-gray-400 dark:text-slate-400">Presets:</span>
                            <button
                                type="button"
                                onClick={() => setEditingAccount({ ...editingAccount, imap_port: 993, encryption_type: 'SSL' })}
                                className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.imap_port === 993 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                            >
                                993 (IMAP SSL)
                            </button>
                            <button
                                type="button"
                                onClick={() => setEditingAccount({ ...editingAccount, imap_port: 143, encryption_type: 'TLS' })}
                                className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.imap_port === 143 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                            >
                                143 (IMAP)
                            </button>
                            <button
                                type="button"
                                onClick={() => setEditingAccount({ ...editingAccount, imap_port: 995, encryption_type: 'SSL' })}
                                className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.imap_port === 995 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                            >
                                995 (POP3)
                            </button>
                        </div>

                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Encryption Type</label>
                            <select
                                value={editingAccount.encryption_type}
                                onChange={(e) => setEditingAccount({ ...editingAccount, encryption_type: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                            >
                                <option value="SSL">SSL / TLS (Recommended for Custom Domain)</option>
                                <option value="TLS">TLS (Recommended for Gmail / Outlook)</option>
                                <option value="STARTTLS">STARTTLS</option>
                                <option value="NONE">None / Plain</option>
                            </select>
                        </div>

                        <div className="flex items-center gap-4 pt-1">
                            <label className="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={editingAccount.is_default}
                                    onChange={(e) => setEditingAccount({ ...editingAccount, is_default: e.target.checked })}
                                    className="rounded text-blue-600"
                                />
                                <span>Set as Default Sender</span>
                            </label>
                            <label className="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={editingAccount.is_active}
                                    onChange={(e) => setEditingAccount({ ...editingAccount, is_active: e.target.checked })}
                                    className="rounded text-blue-600"
                                />
                                <span>Active</span>
                            </label>
                        </div>

                        {/* Dedicated Test Section */}
                        <div className="p-3 rounded-xl border border-blue-100 dark:border-blue-900/40 bg-blue-50/40 dark:bg-blue-950/20 space-y-2 mt-2">
                            <div className="flex items-center justify-between gap-2">
                                <div>
                                    <div className="font-semibold text-xs text-gray-800 dark:text-gray-200 flex items-center gap-1.5">
                                        <Send className="w-3.5 h-3.5 text-blue-500" />
                                        <span>Test Connection & Self-Test Email</span>
                                    </div>
                                    <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                        Sends a self-test email to verify outgoing SMTP & auth before saving.
                                    </p>
                                </div>
                                <button
                                    type="button"
                                    onClick={handleTestDirectConnection}
                                    disabled={isTestingDirect}
                                    className="px-3 py-1.5 text-xs font-semibold rounded-lg bg-blue-600 hover:bg-blue-700 text-white flex items-center gap-1.5 shadow-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer shrink-0"
                                >
                                    {isTestingDirect ? (
                                        <>
                                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                            <span>Testing...</span>
                                        </>
                                    ) : (
                                        <>
                                            <RefreshCw className="w-3.5 h-3.5" />
                                            <span>Send Test Email</span>
                                        </>
                                    )}
                                </button>
                            </div>

                            {testResultStatus === 'success' && (
                                <div className="p-2.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/60 flex items-start gap-2 text-emerald-800 dark:text-emerald-200 animate-in fade-in">
                                    <span className="relative flex h-3 w-3 mt-0.5 shrink-0">
                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                        <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
                                    </span>
                                    <div className="flex-1 text-[11px]">
                                        <div className="font-bold flex items-center gap-1">
                                            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                                            <span>Connection & Delivery Successful (Green Signal)</span>
                                        </div>
                                        <div className="text-emerald-700 dark:text-emerald-300 mt-0.5">
                                            {testResult?.message}
                                        </div>
                                    </div>
                                </div>
                            )}

                            {testResultStatus === 'failed' && (
                                <div className="p-2.5 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800/60 flex items-start gap-2 text-rose-800 dark:text-rose-200 animate-in fade-in">
                                    <AlertCircle className="w-3.5 h-3.5 text-rose-600 dark:text-rose-400 mt-0.5 shrink-0" />
                                    <div className="flex-1 text-[11px]">
                                        <div className="font-bold">Connection Verification Failed</div>
                                        <div className="text-rose-700 dark:text-rose-300 mt-0.5 break-all">
                                            {testResult?.message}
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>
                    </form>
                </ModalDialog>

        </>
    );
}
