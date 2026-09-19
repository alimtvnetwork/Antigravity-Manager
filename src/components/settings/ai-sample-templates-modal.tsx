import React, { useState } from 'react';
import {
    Sparkles,
    Copy,
    Download,
    Check,
    FileJson,
    FileText,
    Bot,
    HelpCircle,
} from 'lucide-react';
import ModalDialog from '../common/ModalDialog';
import { showToast } from '../common/ToastContainer';

interface Props {
    isOpen: boolean;
    onClose: () => void;
}

const SAMPLE_JSON_DATA = [
    {
        alias: "Primary Google Workspace",
        email: "alerts@yourcompany.com",
        password: "app-password-without-spaces",
        smtp_host: "smtp.gmail.com",
        smtp_port: 587,
        imap_host: "imap.gmail.com",
        imap_port: 993,
        encryption_type: "TLS",
        is_default: true,
        is_active: true
    },
    {
        alias: "Backup Microsoft 365",
        email: "backup-agent@outlook.com",
        password: "app-specific-token",
        smtp_host: "smtp.office365.com",
        smtp_port: 587,
        imap_host: "outlook.office365.com",
        imap_port: 993,
        encryption_type: "STARTTLS",
        is_default: false,
        is_active: true
    }
];

const SAMPLE_CSV_DATA = `alias,email,password,smtp_host,smtp_port,imap_host,imap_port,encryption_type,is_default,is_active
Primary Gmail,agent@gmail.com,app-password-here,smtp.gmail.com,587,imap.gmail.com,993,TLS,true,true
Corporate Outlook,alerts@corp.com,app-token-here,smtp.office365.com,587,outlook.office365.com,993,STARTTLS,false,true`;

const SAMPLE_AI_PROMPT = `Generate a JSON array of email mailboxes for AGM (Antigravity Manager) with the following schema for each object:
- alias (string, human-readable name)
- email (string, email address)
- password (string, application password or token)
- smtp_host (string, e.g. "smtp.gmail.com")
- smtp_port (number, e.g. 587 or 465)
- imap_host (string, e.g. "imap.gmail.com")
- imap_port (number, e.g. 993)
- encryption_type (string: "TLS", "STARTTLS", or "SSL")
- is_default (boolean: true for first, false for others)
- is_active (boolean: true)

Output only valid JSON array.`;

export default function AiSampleTemplatesModal({ isOpen, onClose }: Props) {
    const [activeTab, setActiveTab] = useState<'json' | 'csv' | 'prompt'>('json');
    const [copiedTab, setCopiedTab] = useState<string | null>(null);

    const handleCopy = (text: string, tabName: string) => {
        navigator.clipboard.writeText(text);
        setCopiedTab(tabName);
        showToast('Template copied to clipboard', 'success');
        setTimeout(() => setCopiedTab(null), 2000);
    };

    const handleDownload = (filename: string, content: string, mimeType: string) => {
        const blob = new Blob([content], { type: mimeType });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        showToast(`Downloaded ${filename}`, 'success');
    };

    const jsonString = JSON.stringify(SAMPLE_JSON_DATA, null, 2);

    return (
        <ModalDialog
            isOpen={isOpen}
            title="AI Ingestion Templates & Schemas"
            type="info"
            onClose={onClose}
        >
            <div className="space-y-4 text-xs">
                <div className="p-3 bg-blue-50/70 dark:bg-blue-950/30 rounded-xl border border-blue-100 dark:border-blue-900/40 text-blue-800 dark:text-blue-300 flex items-start gap-2.5">
                    <Sparkles className="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
                    <div>
                        <p className="font-semibold text-xs">Feed structured mailbox data to AI models</p>
                        <p className="text-[11px] text-blue-600/80 dark:text-blue-400/80 mt-0.5">
                            Copy or download these sample templates to prompt ChatGPT, Claude, or Gemini to generate bulk mailbox lists formatted for AGM.
                        </p>
                    </div>
                </div>

                {/* Tabs */}
                <div className="flex items-center gap-1.5 border-b border-gray-200 dark:border-base-300 pb-2">
                    <button
                        onClick={() => setActiveTab('json')}
                        className={`px-3 py-1.5 rounded-lg font-medium transition-all flex items-center gap-1.5 ${
                            activeTab === 'json'
                                ? 'bg-amber-500 text-white shadow-xs'
                                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-base-200'
                        }`}
                    >
                        <FileJson className="w-3.5 h-3.5" />
                        <span>JSON Schema</span>
                    </button>
                    <button
                        onClick={() => setActiveTab('csv')}
                        className={`px-3 py-1.5 rounded-lg font-medium transition-all flex items-center gap-1.5 ${
                            activeTab === 'csv'
                                ? 'bg-blue-600 text-white shadow-xs'
                                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-base-200'
                        }`}
                    >
                        <FileText className="w-3.5 h-3.5" />
                        <span>CSV Format</span>
                    </button>
                    <button
                        onClick={() => setActiveTab('prompt')}
                        className={`px-3 py-1.5 rounded-lg font-medium transition-all flex items-center gap-1.5 ${
                            activeTab === 'prompt'
                                ? 'bg-purple-600 text-white shadow-xs'
                                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-base-200'
                        }`}
                    >
                        <Bot className="w-3.5 h-3.5" />
                        <span>AI Prompt Template</span>
                    </button>
                </div>

                {/* Code Box */}
                <div className="relative">
                    {activeTab === 'json' && (
                        <div>
                            <div className="flex items-center justify-between mb-2">
                                <span className="text-[11px] text-gray-500 dark:text-gray-400">
                                    Sample Mailboxes JSON (`mailboxes_sample.json`)
                                </span>
                                <div className="flex items-center gap-1.5">
                                    <button
                                        onClick={() => handleCopy(jsonString, 'json')}
                                        className="btn btn-xs btn-ghost gap-1 hover:bg-gray-200 dark:hover:bg-base-300"
                                    >
                                        {copiedTab === 'json' ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                                        <span>Copy</span>
                                    </button>
                                    <button
                                        onClick={() => handleDownload('mailboxes_sample.json', jsonString, 'application/json')}
                                        className="btn btn-xs btn-primary gap-1"
                                    >
                                        <Download className="w-3 h-3" />
                                        <span>Download JSON</span>
                                    </button>
                                </div>
                            </div>
                            <pre className="p-3 bg-gray-900 text-gray-100 rounded-xl font-mono text-[11px] overflow-x-auto max-h-56 leading-relaxed">
                                {jsonString}
                            </pre>
                        </div>
                    )}

                    {activeTab === 'csv' && (
                        <div>
                            <div className="flex items-center justify-between mb-2">
                                <span className="text-[11px] text-gray-500 dark:text-gray-400">
                                    Sample Mailboxes CSV (`mailboxes_sample.csv`)
                                </span>
                                <div className="flex items-center gap-1.5">
                                    <button
                                        onClick={() => handleCopy(SAMPLE_CSV_DATA, 'csv')}
                                        className="btn btn-xs btn-ghost gap-1 hover:bg-gray-200 dark:hover:bg-base-300"
                                    >
                                        {copiedTab === 'csv' ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                                        <span>Copy</span>
                                    </button>
                                    <button
                                        onClick={() => handleDownload('mailboxes_sample.csv', SAMPLE_CSV_DATA, 'text/csv')}
                                        className="btn btn-xs btn-primary gap-1"
                                    >
                                        <Download className="w-3 h-3" />
                                        <span>Download CSV</span>
                                    </button>
                                </div>
                            </div>
                            <pre className="p-3 bg-gray-900 text-gray-100 rounded-xl font-mono text-[11px] overflow-x-auto max-h-56 leading-relaxed">
                                {SAMPLE_CSV_DATA}
                            </pre>
                        </div>
                    )}

                    {activeTab === 'prompt' && (
                        <div>
                            <div className="flex items-center justify-between mb-2">
                                <span className="text-[11px] text-gray-500 dark:text-gray-400">
                                    AI Prompt Instruction Template (Copy into ChatGPT, Claude, Gemini)
                                </span>
                                <div className="flex items-center gap-1.5">
                                    <button
                                        onClick={() => handleCopy(SAMPLE_AI_PROMPT, 'prompt')}
                                        className="btn btn-xs btn-primary gap-1"
                                    >
                                        {copiedTab === 'prompt' ? <Check className="w-3 h-3 text-white" /> : <Copy className="w-3 h-3" />}
                                        <span>Copy AI Prompt</span>
                                    </button>
                                </div>
                            </div>
                            <pre className="p-3 bg-gray-900 text-gray-100 rounded-xl font-mono text-[11px] overflow-x-auto max-h-56 leading-relaxed whitespace-pre-wrap">
                                {SAMPLE_AI_PROMPT}
                            </pre>
                        </div>
                    )}
                </div>

                <div className="pt-2 border-t border-gray-100 dark:border-base-300 flex items-center justify-between text-[11px] text-gray-500 dark:text-gray-400">
                    <span className="flex items-center gap-1">
                        <HelpCircle className="w-3.5 h-3.5 text-gray-400" />
                        Import generated files using the Actions dropdown in AGM.
                    </span>
                    <button
                        onClick={onClose}
                        className="btn btn-sm btn-ghost"
                    >
                        Close
                    </button>
                </div>
            </div>
        </ModalDialog>
    );
}
