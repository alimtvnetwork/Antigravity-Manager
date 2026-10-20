import { useState } from 'react';
import { X, Send, Server, Loader2 } from 'lucide-react';
import { sshService } from '../../../services/sshService';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';
import type { FleetMachineInfo } from './fleetTypes';

interface SendFleetPromptModalProps {
    machine: FleetMachineInfo | null;
    onClose: () => void;
}

export function SendFleetPromptModal({ machine, onClose }: SendFleetPromptModalProps) {
    const instances = machine?.active_instances || [];
    const defaultInst = instances[0]?.profile_name || 'default';

    const [selectedInstance, setSelectedInstance] = useState<string>(defaultInst);
    const [promptText, setPromptText] = useState<string>('');
    const [isSending, setIsSending] = useState<boolean>(false);

    if (!machine) return null;

    const handleSend = async () => {
        const text = promptText.trim();
        if (!text) {
            showToast('Please enter prompt instruction to dispatch', 'warning');
            return;
        }

        setIsSending(true);
        try {
            const alias = machine.node_alias || (machine as any).alias || 'local';
            const res = await sshService.sendRemoteSshPrompt({
                node_alias: alias,
                instance_target: selectedInstance || 'default',
                prompt_text: text,
            });

            if (res.success) {
                showToast(`Dispatched prompt to ${alias}:${selectedInstance}`, 'success');
                onClose();
            } else {
                showToast(`Prompt dispatch to ${alias} reported warning: ${res.message || res.stderr || 'Check remote logs'}`, 'warning');
            }
        } catch (err: any) {
            useErrorStore.getState().captureError(err, {
                source: 'SendFleetPromptModal.handleSend',
                triggerAction: 'send_remote_ssh_prompt',
            });
            showToast(`Failed to dispatch prompt: ${err?.message || err}`, 'error');
        } finally {
            setIsSending(false);
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/70 backdrop-blur-xs">
            <div className="w-full max-w-lg rounded-2xl bg-slate-900 border border-slate-700/60 dark:border-purple-900/50 shadow-2xl overflow-hidden flex flex-col">
                {/* Header */}
                <div className="px-5 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/50">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 rounded-xl bg-purple-500/15 text-purple-400 border border-purple-500/30">
                            <Server className="w-4 h-4" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2">
                                <h3 className="font-semibold text-sm text-slate-100">
                                    Send Prompt to Cluster Node
                                </h3>
                                <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-cyan-500/15 text-cyan-300 border border-cyan-500/30">
                                    {machine.node_alias}
                                </span>
                            </div>
                            <p className="text-[11px] text-slate-400">
                                Target Host: {machine.ip_address || '127.0.0.1'} · Dispatch via SSH &amp; CDP
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors cursor-pointer"
                        title="Close modal"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Body */}
                <div className="p-5 space-y-4">
                    {/* Target Instance */}
                    <div>
                        <label className="block text-xs font-medium text-slate-300 mb-1.5">
                            Target IDE Instance / Profile:
                        </label>
                        {instances.length > 0 ? (
                            <select
                                value={selectedInstance}
                                onChange={(e) => setSelectedInstance(e.target.value)}
                                className="w-full px-3 py-2 text-xs rounded-xl bg-slate-800/80 border border-slate-700 text-slate-200 focus:outline-none focus:border-purple-500"
                            >
                                {instances.map((inst, i) => (
                                    <option key={inst.profile_id || i} value={inst.profile_name}>
                                        {inst.profile_name} {inst.is_running ? '(Active/Running)' : '(Idle)'}
                                    </option>
                                ))}
                            </select>
                        ) : (
                            <input
                                type="text"
                                value={selectedInstance}
                                onChange={(e) => setSelectedInstance(e.target.value)}
                                placeholder="default"
                                className="w-full px-3 py-2 text-xs rounded-xl bg-slate-800/80 border border-slate-700 text-slate-200 focus:outline-none focus:border-purple-500"
                            />
                        )}
                    </div>

                    {/* Prompt Text Input */}
                    <div>
                        <label className="block text-xs font-medium text-slate-300 mb-1.5 flex items-center justify-between">
                            <span>Prompt Instruction:</span>
                            <span className="text-[10px] text-slate-500 font-normal">
                                Direct injection into Antigravity session
                            </span>
                        </label>
                        <textarea
                            rows={5}
                            value={promptText}
                            onChange={(e) => setPromptText(e.target.value)}
                            placeholder="Type prompt or task instructions to send to this remote instance..."
                            className="w-full px-3 py-2.5 text-xs rounded-xl bg-slate-950/70 border border-slate-700/80 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-purple-500 font-mono resize-none"
                        />
                    </div>

                    {/* Quick Suggestions */}
                    <div className="flex items-center gap-1.5 flex-wrap">
                        <span className="text-[10px] text-slate-500">Quick:</span>
                        <button
                            type="button"
                            onClick={() => setPromptText("agm status")}
                            className="px-2 py-0.5 rounded-md text-[10px] bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/50 cursor-pointer"
                        >
                            agm status
                        </button>
                        <button
                            type="button"
                            onClick={() => setPromptText("Review and resume all pending tasks in the workspace.")}
                            className="px-2 py-0.5 rounded-md text-[10px] bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/50 cursor-pointer"
                        >
                            Resume Pending Tasks
                        </button>
                    </div>
                </div>

                {/* Footer */}
                <div className="px-5 py-3.5 border-t border-slate-800 bg-slate-950/40 flex items-center justify-end gap-2.5">
                    <button
                        type="button"
                        onClick={onClose}
                        disabled={isSending}
                        className="px-3.5 py-1.5 rounded-xl text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors cursor-pointer"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleSend}
                        disabled={isSending || !promptText.trim()}
                        className="flex items-center gap-2 px-4 py-1.5 rounded-xl text-xs font-semibold bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white shadow-md transition-all cursor-pointer disabled:opacity-50"
                    >
                        {isSending ? (
                            <>
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                <span>Dispatching...</span>
                            </>
                        ) : (
                            <>
                                <Send className="w-3.5 h-3.5" />
                                <span>Send Prompt</span>
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>
    );
}

export default SendFleetPromptModal;
