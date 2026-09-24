import { useState, useMemo } from 'react';
import {
    Copy,
    Download,
    Check,
    FileJson,
    FileText,
    FileCode,
    Sparkles,
} from 'lucide-react';
import ModalDialog from '../common/ModalDialog';
import { showToast } from '../common/ToastContainer';
import type { EmailAccount } from '../../services/emailService';
import {
    accountToJson,
    accountsToJson,
    accountToYaml,
    accountsToYaml,
    accountToCsv,
    accountsToCsv,
    downloadOrSaveFile,
} from '../../utils/emailFormatters';

interface Props {
    isOpen: boolean;
    onClose: () => void;
    singleAccount?: Partial<EmailAccount> | null;
    allAccounts?: EmailAccount[];
    initialFormat?: 'json' | 'yaml' | 'csv';
}

export default function MailboxExportModal({
    isOpen,
    onClose,
    singleAccount,
    allAccounts,
    initialFormat = 'json',
}: Props) {
    const [format, setFormat] = useState<'json' | 'yaml' | 'csv'>(initialFormat);
    const [isCopied, setIsCopied] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    const isSingle = Boolean(singleAccount);

    const exportedContent = useMemo(() => {
        if (isSingle && singleAccount) {
            switch (format) {
                case 'json':
                    return accountToJson(singleAccount, true);
                case 'yaml':
                    return accountToYaml(singleAccount);
                case 'csv':
                    return accountToCsv(singleAccount);
                default:
                    return accountToJson(singleAccount, true);
            }
        } else if (allAccounts) {
            switch (format) {
                case 'json':
                    return accountsToJson(allAccounts, true);
                case 'yaml':
                    return accountsToYaml(allAccounts);
                case 'csv':
                    return accountsToCsv(allAccounts);
                default:
                    return accountsToJson(allAccounts, true);
            }
        }
        return '';
    }, [isSingle, singleAccount, allAccounts, format]);

    const title = isSingle
        ? `Export Mailbox: ${singleAccount?.alias || singleAccount?.email || 'Single Account'}`
        : `Export Mailboxes (${allAccounts?.length || 0} Accounts)`;

    const defaultFilename = useMemo(() => {
        const timestamp = Date.now();
        if (isSingle) {
            const safeName = (singleAccount?.alias || singleAccount?.email || 'mailbox')
                .replace(/[^a-zA-Z0-9_-]/g, '_')
                .toLowerCase();
            return `${safeName}-${timestamp}.${format}`;
        }
        return `antigravity-mailboxes-${timestamp}.${format}`;
    }, [isSingle, singleAccount, format]);

    const handleCopy = () => {
        if (!exportedContent) return;
        navigator.clipboard.writeText(exportedContent);
        setIsCopied(true);
        showToast(`Copied ${format.toUpperCase()} to clipboard`, 'success');
        setTimeout(() => setIsCopied(false), 2000);
    };

    const handleSave = async () => {
        if (!exportedContent) return;
        setIsSaving(true);
        try {
            await downloadOrSaveFile(defaultFilename, exportedContent, format);
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <ModalDialog
            isOpen={isOpen}
            title={title}
            type="info"
            maxWidth="max-w-2xl"
            onClose={onClose}
            onConfirm={onClose}
        >
            <div className="space-y-3.5 text-xs">
                {/* Format selection and action toolbar */}
                <div className="flex flex-wrap items-center justify-between gap-2 border-b border-gray-200 dark:border-slate-800 pb-2.5">
                    <div className="flex items-center gap-1 bg-gray-100 dark:bg-slate-800 p-1 rounded-lg">
                        <button
                            type="button"
                            onClick={() => setFormat('json')}
                            className={`px-3 py-1 rounded-md text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer ${
                                format === 'json'
                                    ? 'bg-white dark:bg-slate-700 text-amber-600 dark:text-amber-400 shadow-xs'
                                    : 'text-gray-600 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-100'
                            }`}
                        >
                            <FileJson className="w-3.5 h-3.5" />
                            <span>JSON</span>
                        </button>

                        <button
                            type="button"
                            onClick={() => setFormat('yaml')}
                            className={`px-3 py-1 rounded-md text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer ${
                                format === 'yaml'
                                    ? 'bg-white dark:bg-slate-700 text-emerald-600 dark:text-emerald-400 shadow-xs'
                                    : 'text-gray-600 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-100'
                            }`}
                        >
                            <FileCode className="w-3.5 h-3.5" />
                            <span>YAML</span>
                        </button>

                        <button
                            type="button"
                            onClick={() => setFormat('csv')}
                            className={`px-3 py-1 rounded-md text-xs font-semibold flex items-center gap-1.5 transition-all cursor-pointer ${
                                format === 'csv'
                                    ? 'bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-xs'
                                    : 'text-gray-600 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-100'
                            }`}
                        >
                            <FileText className="w-3.5 h-3.5" />
                            <span>CSV</span>
                        </button>
                    </div>

                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={handleCopy}
                            className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-800 hover:bg-gray-50 dark:hover:bg-slate-700 text-gray-700 dark:text-slate-200 rounded-lg flex items-center gap-1.5 transition-all shadow-xs cursor-pointer"
                            title="Copy formatted text to clipboard"
                        >
                            {isCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-gray-500" />}
                            <span>{isCopied ? 'Copied!' : 'Copy to Clipboard'}</span>
                        </button>

                        <button
                            type="button"
                            onClick={handleSave}
                            disabled={isSaving}
                            className="px-3 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center gap-1.5 transition-all shadow-xs cursor-pointer disabled:opacity-50"
                            title="Save as file to local disk"
                        >
                            <Download className="w-3.5 h-3.5" />
                            <span>Save to File</span>
                        </button>
                    </div>
                </div>

                {/* Subtitle / summary */}
                <div className="flex items-center justify-between text-[11px] text-gray-500 dark:text-slate-400 px-0.5">
                    <span>
                        Format: <span className="font-semibold text-gray-700 dark:text-slate-200 uppercase">{format}</span> • {exportedContent.length} characters
                    </span>
                    <span className="font-mono text-gray-400 dark:text-slate-500">{defaultFilename}</span>
                </div>

                {/* Code block */}
                <div className="relative rounded-xl border border-gray-200 dark:border-slate-800 bg-slate-950 overflow-hidden shadow-inner">
                    <pre className="p-3.5 text-slate-100 font-mono text-[11px] overflow-auto max-h-72 leading-relaxed select-all">
                        {exportedContent}
                    </pre>
                </div>

                {/* Footer advice */}
                <div className="pt-2 flex items-center justify-between text-[11px] text-gray-500 dark:text-slate-400">
                    <span className="flex items-center gap-1">
                        <Sparkles className="w-3.5 h-3.5 text-amber-500" />
                        This formatted data can be directly imported into any Antigravity Manager Tools instance or provided to AI models.
                    </span>
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-3 py-1 text-xs rounded-lg hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-600 dark:text-slate-300 cursor-pointer"
                    >
                        Close
                    </button>
                </div>
            </div>
        </ModalDialog>
    );
}
