import { useState } from 'react';
import {
    Sparkles,
    Copy,
    Download,
    Check,
    FileJson,
    FileText,
    Bot,
    HelpCircle,
    Terminal,
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

const SAMPLE_INBOUND_COMMANDS = [
    {
        title: 'PowerShell Service Inspection',
        description: 'Inspect running system, WSL, and Docker services on target node.',
        subject: 'Run Diagnostics [Node: <node_name>]',
        body: `exec: <node_name>\npowershell: Get-Service -Name '*wsl*', '*docker*', '*hyper*' -ErrorAction SilentlyContinue | Select-Object Name, Status, StartType`,
        command: `powershell: Get-Service -Name '*wsl*', '*docker*', '*hyper*' -ErrorAction SilentlyContinue | Select-Object Name, Status, StartType`,
    },
    {
        title: 'PowerShell Top Memory Processes',
        description: 'Identify the top memory-consuming processes on target node.',
        subject: 'Process Check [Node: <node_name>]',
        body: `exec: <node_name>\npowershell: Get-Process | Sort-Object -Property WorkingSet64 -Descending | Select-Object -First 10 -Property Name, Id, @{Name="MB";Expression={[math]::round($_.WorkingSet64/1MB,2)}}`,
        command: `powershell: Get-Process | Sort-Object -Property WorkingSet64 -Descending | Select-Object -First 10 -Property Name, Id, @{Name="MB";Expression={[math]::round($_.WorkingSet64/1MB,2)}}`,
    },
    {
        title: 'GitMap Repository Health Scan',
        description: 'Trigger autonomous repository health and branch scanning via GitMap CLI.',
        subject: 'GitMap Health [Node: <node_name>]',
        body: `exec: <node_name>\ngitmap scan`,
        command: `gitmap scan`,
    },
    {
        title: 'GitMap Pipeline Status Check',
        description: 'Verify current pipeline runs and pending tasks via GitMap CLI.',
        subject: 'Pipeline Check [Node: <node_name>]',
        body: `exec: <node_name>\ngitmap status`,
        command: `gitmap status`,
    },
    {
        title: 'Antigravity AI Prompt Injection',
        description: 'Queue an AI instructions prompt directly into the active Antigravity workspace.',
        subject: 'AI Task: Fix CI/CD [Node: <node_name>]',
        body: `Project: Antigravity-Manager\nPlease analyze recent build failures in .github/workflows/ci.yml and optimize test run times.`,
        command: `Project: Antigravity-Manager\nPlease analyze recent build failures in .github/workflows/ci.yml and optimize test run times.`,
    },
    {
        title: 'Instance Rotation Trigger',
        description: 'Force Antigravity Manager to launch a fresh clean browser instance.',
        subject: 'Rotate Instance [Node: <node_name>]',
        body: `instance: new`,
        command: `instance: new`,
    },
];

export default function AiSampleTemplatesModal({ isOpen, onClose }: Props) {
    const [activeTab, setActiveTab] = useState<'json' | 'csv' | 'prompt' | 'commands'>('json');
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
            onConfirm={onClose}
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
                    <button
                        onClick={() => setActiveTab('commands')}
                        className={`px-3 py-1.5 rounded-lg font-medium transition-all flex items-center gap-1.5 ${
                            activeTab === 'commands'
                                ? 'bg-emerald-600 text-white shadow-xs'
                                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-base-200'
                        }`}
                    >
                        <Terminal className="w-3.5 h-3.5" />
                        <span>Inbound Commands</span>
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

                    {activeTab === 'commands' && (
                        <div className="space-y-3 max-h-96 overflow-y-auto pr-1">
                            <div className="p-2.5 rounded-lg bg-emerald-50/70 dark:bg-emerald-950/30 border border-emerald-100 dark:border-emerald-900/40 text-[11px] text-emerald-800 dark:text-emerald-300">
                                <div className="font-semibold mb-0.5">Email Routing &amp; Syntax Guide</div>
                                <div>• Include <code className="font-mono font-bold text-emerald-700 dark:text-emerald-300">[Node: &lt;machine_name&gt;]</code> or <code className="font-mono font-bold text-emerald-700 dark:text-emerald-300">[IP: &lt;ip&gt;]</code> in the email subject.</div>
                                <div>• Body must specify target with <code className="font-mono font-bold">exec: &lt;node&gt;</code> or <code className="font-mono font-bold">Project: &lt;project&gt;</code>.</div>
                                <div>• Command outputs and exit codes are emailed back automatically.</div>
                            </div>

                            <div className="space-y-2.5">
                                {SAMPLE_INBOUND_COMMANDS.map((cmd, idx) => (
                                    <div
                                        key={idx}
                                        className="p-3 rounded-xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-xs space-y-2"
                                    >
                                        <div className="flex items-start justify-between gap-2">
                                            <div>
                                                <h4 className="font-semibold text-gray-900 dark:text-slate-100 text-xs">
                                                    {cmd.title}
                                                </h4>
                                                <p className="text-[11px] text-gray-500 dark:text-slate-400 mt-0.5">
                                                    {cmd.description}
                                                </p>
                                            </div>
                                            <div className="flex items-center gap-1 shrink-0">
                                                <button
                                                    type="button"
                                                    onClick={() => handleCopy(cmd.command, `cmd-${idx}`)}
                                                    className="px-2 py-1 text-[10px] font-medium border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-700 text-gray-700 dark:text-slate-200 rounded flex items-center gap-1 transition-colors cursor-pointer"
                                                    title="Copy Raw Command"
                                                >
                                                    {copiedTab === `cmd-${idx}` ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                                                    <span>Command</span>
                                                </button>
                                                <button
                                                    type="button"
                                                    onClick={() => handleCopy(`Subject: ${cmd.subject}\n\n${cmd.body}`, `email-${idx}`)}
                                                    className="px-2 py-1 text-[10px] font-medium bg-emerald-600 hover:bg-emerald-700 text-white rounded flex items-center gap-1 transition-colors cursor-pointer"
                                                    title="Copy Full Email Message"
                                                >
                                                    {copiedTab === `email-${idx}` ? <Check className="w-3 h-3 text-white" /> : <Copy className="w-3 h-3" />}
                                                    <span>Email Body</span>
                                                </button>
                                            </div>
                                        </div>

                                        <div className="bg-gray-900 rounded-lg p-2 font-mono text-[10px] text-gray-200 space-y-1 overflow-x-auto">
                                            <div className="text-gray-400">
                                                <span className="text-gray-500 select-none">Subject: </span>{cmd.subject}
                                            </div>
                                            <div className="text-emerald-400 whitespace-pre-wrap">
                                                {cmd.body}
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </div>
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
