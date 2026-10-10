import { SupabaseSyncApi } from './useSupabaseSync';
import { UnifiedBackupModal } from '../../modals/UnifiedBackupModal';
import ModalDialog from '../common/ModalDialog';
import { showToast } from '../common/ToastContainer';

export function SyncModalsB(props: SupabaseSyncApi) {
    const { config, isSchemaModalOpen, setIsSchemaModalOpen, schemaRole, schemaSql, isExportModalOpen, setIsExportModalOpen, exportContent, exportFormat, exportRounds, setExportRounds, isImportModalOpen, setIsImportModalOpen, importText, setImportText, isAiPromptModalOpen, setIsAiPromptModalOpen, isTgGuideOpen, setIsTgGuideOpen, isBackupModalOpen, setIsBackupModalOpen, username, handleOpenExport, text, handleImportSubmit, aiInstructionTemplate } = props;

    return (
        <>
                {/* Schema SQL Modal */}
                <ModalDialog
                    isOpen={isSchemaModalOpen}
                    onClose={() => setIsSchemaModalOpen(false)}
                    title={`PostgreSQL Schema Migration (${schemaRole.toUpperCase()} DB)`}
                >
                    <div className="space-y-3">
                        <p className="text-xs text-gray-400">
                            Copy and execute this idempotent DDL script in your Supabase Project SQL Editor to provision required tables and stored functions.
                        </p>
                        <pre className="p-3 bg-slate-950 border border-slate-800 rounded-lg text-[11px] font-mono text-gray-300 overflow-x-auto max-h-72 select-all">
                            {schemaSql}
                        </pre>
                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => {
                                    navigator.clipboard.writeText(schemaSql);
                                    showToast('SQL copied to clipboard', 'success');
                                }}
                                className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs flex items-center gap-1.5 transition font-medium"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                Copy SQL
                            </button>
                        </div>
                    </div>
                </ModalDialog>

                {/* Export Modal with Iterative Base64 Rounds */}
                <ModalDialog
                    isOpen={isExportModalOpen}
                    onClose={() => setIsExportModalOpen(false)}
                    title={`Export Supabase Configuration (${exportFormat.toUpperCase()})`}
                >
                    <div className="space-y-4">
                        <div className="flex items-center justify-between p-3 rounded-lg bg-slate-950 border border-slate-800">
                            <div className="flex items-center gap-2">
                                <Lock className="w-4 h-4 text-emerald-400" />
                                <span className="text-xs text-gray-300 font-medium">
                                    Multi-Pass Base64 Encryption Rounds:
                                </span>
                            </div>
                            <select
                                value={exportRounds}
                                onChange={(e) => {
                                    const r = Number(e.target.value);
                                    setExportRounds(r);
                                    handleOpenExport(exportFormat);
                                }}
                                className="px-2 py-1 bg-slate-900 border border-slate-700 rounded text-xs text-white"
                            >
                                <option value={1}>1 Round (Standard)</option>
                                <option value={2}>2 Rounds</option>
                                <option value={3}>3 Rounds</option>
                                <option value={4}>4 Rounds (Recommended: 4$)</option>
                                <option value={6}>6 Rounds</option>
                            </select>
                        </div>

                        <textarea
                            readOnly
                            rows={10}
                            value={exportContent}
                            className="w-full p-3 bg-slate-950 border border-slate-800 rounded-lg text-xs font-mono text-gray-300 focus:outline-none select-all"
                        />

                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => {
                                    navigator.clipboard.writeText(exportContent);
                                    showToast('Export configuration copied', 'success');
                                }}
                                className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs flex items-center gap-1.5 transition font-medium"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                Copy to Clipboard
                            </button>
                        </div>
                    </div>
                </ModalDialog>

                {/* Import Modal */}
                <ModalDialog
                    isOpen={isImportModalOpen}
                    onClose={() => setIsImportModalOpen(false)}
                    title="Import Supabase Configuration"
                >
                    <div className="space-y-3">
                        <p className="text-xs text-gray-400">
                            Paste exported JSON/YAML configuration. Multi-pass Base64 keys (e.g. <code>4$payload</code>) are automatically decrypted.
                        </p>
                        <textarea
                            rows={10}
                            placeholder="Paste JSON / YAML config bundle here..."
                            value={importText}
                            onChange={(e) => setImportText(e.target.value)}
                            className="w-full p-3 bg-slate-950 border border-slate-800 rounded-lg text-xs font-mono text-white focus:outline-none focus:border-emerald-500"
                        />
                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => setIsImportModalOpen(false)}
                                className="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 text-xs hover:bg-slate-700 transition"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleImportSubmit}
                                className="px-4 py-2 rounded-lg bg-emerald-600 text-white text-xs hover:bg-emerald-500 font-medium transition"
                            >
                                Import & Decrypt
                            </button>
                        </div>
                    </div>
                </ModalDialog>

                {/* AI Prompt Template Modal */}
                <ModalDialog
                    isOpen={isAiPromptModalOpen}
                    onClose={() => setIsAiPromptModalOpen(false)}
                    title="AI Prompt Instruction Generator"
                >
                    <div className="space-y-3">
                        <p className="text-xs text-gray-400">
                            Copy this instruction prompt and send it to ChatGPT, Claude, or Gemini alongside your Supabase project keys to instantly generate an AGM-compatible configuration bundle.
                        </p>
                        <pre className="p-3 bg-slate-950 border border-slate-800 rounded-lg text-[11px] font-mono text-gray-300 overflow-x-auto max-h-72 select-all">
                            {aiInstructionTemplate}
                        </pre>
                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => {
                                    navigator.clipboard.writeText(aiInstructionTemplate);
                                    showToast('AI Prompt copied to clipboard', 'success');
                                }}
                                className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs flex items-center gap-1.5 transition font-medium"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                Copy AI Prompt
                            </button>
                        </div>
                    </div>
                </ModalDialog>

                {/* Telegram Bot Setup Guide Modal */}
                <ModalDialog
                    isOpen={isTgGuideOpen}
                    title="Telegram Bot Setup Guide"
                    type="info"
                    maxWidth="max-w-xl"
                    onClose={() => setIsTgGuideOpen(false)}
                    onConfirm={() => setIsTgGuideOpen(false)}
                >
                    <div className="space-y-4 text-xs text-gray-300">
                        <p className="text-gray-400">
                            Follow these simple steps to configure your personal Telegram remote bot:
                        </p>

                        <div className="space-y-3">
                            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
                                <div className="font-semibold text-sky-400 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-400 flex items-center justify-center text-xs">1</span>
                                    Create Bot via @BotFather
                                </div>
                                <p className="text-gray-400 pl-7 leading-relaxed">
                                    Open Telegram and search for <strong className="text-white">@BotFather</strong> (verified with blue checkmark).
                                    Send <code className="text-sky-300">/newbot</code>, choose a display name and a username ending in <code className="text-sky-300">bot</code>.
                                </p>
                            </div>

                            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
                                <div className="font-semibold text-sky-400 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-400 flex items-center justify-center text-xs">2</span>
                                    Copy HTTP API Token
                                </div>
                                <p className="text-gray-400 pl-7 leading-relaxed">
                                    BotFather will provide an API token (e.g. <code className="text-sky-300">123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ</code>).
                                    Paste this token into the <strong>Telegram Bot Token</strong> field.
                                </p>
                            </div>

                            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
                                <div className="font-semibold text-sky-400 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-400 flex items-center justify-center text-xs">3</span>
                                    Get your Telegram Chat ID (Security Filter)
                                </div>
                                <p className="text-gray-400 pl-7 leading-relaxed">
                                    Search for <strong className="text-white">@userinfobot</strong> or <strong className="text-white">@RawDataBot</strong> in Telegram and send <code className="text-sky-300">/start</code>.
                                    Copy the numeric <code className="text-sky-300">Id</code> and paste it into the <strong>Allowed Chat ID</strong> field to restrict bot access exclusively to your account.
                                </p>
                            </div>

                            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
                                <div className="font-semibold text-sky-400 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-400 flex items-center justify-center text-xs">4</span>
                                    Start and Test
                                </div>
                                <p className="text-gray-400 pl-7 leading-relaxed">
                                    Click <strong>Test Bot</strong> to verify token connectivity, then click <strong>Save Telegram</strong> and send <code className="text-sky-300">/status</code> to your bot!
                                </p>
                            </div>
                        </div>
                    </div>
                </ModalDialog>

                {/* Unified Encrypted System Backup Modal */}
                <UnifiedBackupModal
                    isOpen={isBackupModalOpen}
                    onClose={() => setIsBackupModalOpen(false)}
                />
        </>
    );
}
