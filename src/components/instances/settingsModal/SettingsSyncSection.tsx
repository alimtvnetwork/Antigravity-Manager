import { Copy, ClipboardPaste, Sliders, FolderSync, ArrowRight } from 'lucide-react';
import type { InstanceStatus } from '../../../services/instanceService';
import type { InstanceSettingsApi } from './instanceSettingsTypes';

function idSuffix(id: string): string {
    return id.startsWith('antigravity-') ? id : `antigravity-${id}`;
}

function SourceTargetSelect({
    value,
    onChange,
    options,
    borderClass,
    focusClass,
}: {
    value: string;
    onChange: (v: string) => void;
    options: InstanceStatus[];
    borderClass: string;
    focusClass: string;
}) {
    return (
        <select
            value={value}
            onChange={(e) => onChange(e.target.value)}
            className={`w-full truncate bg-white dark:bg-[#0c2438] border ${borderClass} rounded-lg px-2.5 py-1.5 text-xs font-semibold text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 ${focusClass} cursor-pointer`}
        >
            {options.map((inst, idx) => {
                const seq = inst.config.seq_num ?? idx + 1;
                const isDefault = inst.config.is_default || inst.config.id === 'default';
                return (
                    <option key={inst.config.id} value={inst.config.id}>
                        #{seq} {inst.config.name} ({idSuffix(inst.config.id)}){isDefault ? ' [Default]' : ''}
                    </option>
                );
            })}
        </select>
    );
}

export function SettingsSyncSection({ api, instances }: { api: InstanceSettingsApi; instances: InstanceStatus[] }) {
    const {
        clipboardBuffer,
        replicationSourceId,
        setReplicationSourceId,
        setCopySettingsSourceId,
        setCopyProjectsSourceId,
        copySettingsSourceId,
        copyProjectsSourceId,
        selectedTargetId,
        isOperating,
        currentSourceObj,
        currentTargetObj,
        candidateSources,
        handleTargetChange,
        handleCopyBothDirect,
        handleMoveBothDirect,
        handlePasteFromBuffer,
        handleCopySettings,
        handleMoveSettings,
        handleCopyProjects,
        handleMoveProjects,
    } = api;

    return (
        <div className="space-y-3">
            <div className="flex items-center justify-between gap-2 flex-wrap">
                <span className="block text-[11px] font-bold text-gray-400 uppercase tracking-wider">
                    Cross-Instance Replication & Migration
                </span>

                {clipboardBuffer ? (
                    <div className="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/25 text-[10px] font-mono text-emerald-600 dark:text-emerald-400">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                        <span>
                            Buffer: {clipboardBuffer.sourceName} (
                            {clipboardBuffer.hasSettings && clipboardBuffer.hasWorkspaces
                                ? 'Both'
                                : clipboardBuffer.hasSettings
                                  ? 'Settings'
                                  : 'Workspaces'}
                            )
                        </span>
                    </div>
                ) : (
                    <span className="text-[10px] text-gray-400 italic font-mono">Buffer empty</span>
                )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 items-stretch">
                {/* Source Instance Card */}
                <div className="rounded-xl bg-amber-500/5 dark:bg-amber-950/20 border-2 border-amber-500/30 dark:border-amber-500/40 p-3.5 space-y-2.5 flex flex-col justify-between">
                    <div className="space-y-1.5">
                        <div className="flex items-center justify-between gap-2">
                            <div className="flex items-center gap-1.5">
                                <span className="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-amber-500 text-white shadow-xs">
                                    Source
                                </span>
                                <span className="font-bold text-xs text-gray-900 dark:text-gray-100">Origin Profile</span>
                            </div>
                            {currentSourceObj?.is_running && (
                                <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
                                    Running
                                </span>
                            )}
                        </div>
                        <SourceTargetSelect
                            value={replicationSourceId}
                            onChange={(v) => {
                                setReplicationSourceId(v);
                                setCopySettingsSourceId(v);
                                setCopyProjectsSourceId(v);
                            }}
                            options={candidateSources}
                            borderClass="border-amber-300 dark:border-amber-600/40"
                            focusClass="focus:ring-amber-500"
                        />
                        <div className="text-[10px] text-gray-500 dark:text-gray-400 flex items-center justify-between">
                            <span>Data to replicate or transfer</span>
                            <span className="font-mono">ID: {replicationSourceId || 'none'}</span>
                        </div>
                    </div>
                    <div className="pt-1 flex items-center gap-2">
                        <button
                            type="button"
                            disabled={isOperating || !replicationSourceId}
                            onClick={() => handleCopyBothDirect(replicationSourceId)}
                            className="flex-1 py-1.5 px-2.5 rounded-[4px] text-xs font-semibold bg-amber-600 hover:bg-amber-700 text-white shadow-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                            title="Copy both settings and workspaces directly into target"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            <span>Copy Both</span>
                        </button>
                        <button
                            type="button"
                            disabled={isOperating || !replicationSourceId}
                            onClick={() => handleMoveBothDirect(replicationSourceId)}
                            className="flex-1 py-1.5 px-2.5 rounded-[4px] text-xs font-semibold bg-rose-600 hover:bg-rose-700 text-white shadow-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                            title="Move settings and workspaces to target (reset source to baseline)"
                        >
                            <ArrowRight className="w-3.5 h-3.5" />
                            <span>Move Both</span>
                        </button>
                    </div>
                </div>

                {/* Target Instance Card */}
                <div className="rounded-xl bg-cyan-500/5 dark:bg-cyan-950/20 border-2 border-cyan-500/30 dark:border-cyan-500/40 p-3.5 space-y-2.5 flex flex-col justify-between">
                    <div className="space-y-1.5">
                        <div className="flex items-center justify-between gap-2">
                            <div className="flex items-center gap-1.5">
                                <span className="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-cyan-600 text-white shadow-xs">
                                    Target
                                </span>
                                <span className="font-bold text-xs text-gray-900 dark:text-gray-100">Destination Profile</span>
                            </div>
                            {currentTargetObj?.is_running && (
                                <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
                                    Running
                                </span>
                            )}
                        </div>
                        <SourceTargetSelect
                            value={selectedTargetId}
                            onChange={handleTargetChange}
                            options={instances}
                            borderClass="border-cyan-300 dark:border-cyan-600/40"
                            focusClass="focus:ring-cyan-500"
                        />
                        <div className="text-[10px] text-gray-500 dark:text-gray-400 flex items-center justify-between">
                            <span>Recipient of applied configurations</span>
                            <span className="font-mono">ID: {selectedTargetId}</span>
                        </div>
                    </div>
                    <div className="pt-1">
                        <div className="flex items-center rounded-[4px] border border-emerald-500/40 bg-emerald-500/10 p-0.5 divide-x divide-emerald-500/30 shadow-2xs">
                            <button
                                type="button"
                                disabled={isOperating || !clipboardBuffer}
                                onClick={() => handlePasteFromBuffer('both')}
                                className="flex-1 py-1 px-2 text-xs font-bold text-emerald-700 dark:text-emerald-300 hover:bg-emerald-600 hover:text-white transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-1 rounded-l-[4px] rounded-r-none"
                                title="Paste both settings & workspaces from buffer into target"
                            >
                                <ClipboardPaste className="w-3.5 h-3.5" />
                                <span>Paste Both</span>
                            </button>
                            <button
                                type="button"
                                disabled={isOperating || !clipboardBuffer}
                                onClick={() => handlePasteFromBuffer('settings')}
                                className="py-1 px-2 text-xs font-medium text-emerald-700 dark:text-emerald-300 hover:bg-emerald-600 hover:text-white transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-1 rounded-none"
                                title="Paste settings only from buffer into target"
                            >
                                <Sliders className="w-3 h-3" />
                                <span>Settings</span>
                            </button>
                            <button
                                type="button"
                                disabled={isOperating || !clipboardBuffer}
                                onClick={() => handlePasteFromBuffer('workspaces')}
                                className="py-1 px-2 text-xs font-medium text-emerald-700 dark:text-emerald-300 hover:bg-emerald-600 hover:text-white transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-1 rounded-r-[4px] rounded-l-none"
                                title="Paste workspaces only from buffer into target"
                            >
                                <FolderSync className="w-3 h-3" />
                                <span>Folders</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {/* Copy & Move Settings Card */}
                <div className="rounded-xl bg-slate-50 dark:bg-[#071a27]/80 border border-gray-200 dark:border-[#15334d] p-3 space-y-2.5 flex flex-col justify-between">
                    <div>
                        <div className="flex items-center justify-between gap-2 mb-1.5">
                            <div className="flex items-center gap-2 min-w-0">
                                <Sliders className="w-4 h-4 text-blue-500 shrink-0" />
                                <span className="font-bold text-xs text-gray-900 dark:text-gray-100 truncate">
                                    Settings & Themes
                                </span>
                            </div>
                            <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20 shrink-0">
                                settings.json
                            </span>
                        </div>
                        <p className="text-[10px] text-gray-500 dark:text-gray-400 line-clamp-1" title="Copies color themes, policies, and Antigravity preferences into target">
                            Replicate or transfer UI themes, policies, and Antigravity config.
                        </p>
                    </div>
                    <div className="space-y-2 pt-1">
                        <SourceTargetSelect
                            value={copySettingsSourceId}
                            onChange={setCopySettingsSourceId}
                            options={candidateSources}
                            borderClass="border-gray-200 dark:border-[#15334d]"
                            focusClass="focus:ring-blue-500"
                        />
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                disabled={isOperating || !copySettingsSourceId}
                                onClick={handleCopySettings}
                                className="flex-1 px-3 py-1.5 rounded-[4px] text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white shadow-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                                title="Copy settings and themes from source to target profile"
                            >
                                <Sliders className="w-3 h-3" />
                                <span>Copy Settings</span>
                            </button>
                            <button
                                type="button"
                                disabled={isOperating || !copySettingsSourceId}
                                onClick={handleMoveSettings}
                                className="flex-1 px-3 py-1.5 rounded-[4px] text-xs font-semibold bg-amber-600 hover:bg-amber-700 text-white shadow-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                                title="Move settings from source to target profile (resets source to baseline)"
                            >
                                <ArrowRight className="w-3 h-3" />
                                <span>Move Settings</span>
                            </button>
                        </div>
                    </div>
                </div>

                {/* Copy & Move Projects/Workspaces Card */}
                <div className="rounded-xl bg-slate-50 dark:bg-[#071a27]/80 border border-gray-200 dark:border-[#15334d] p-3 space-y-2.5 flex flex-col justify-between">
                    <div>
                        <div className="flex items-center justify-between gap-2 mb-1.5">
                            <div className="flex items-center gap-2 min-w-0">
                                <FolderSync className="w-4 h-4 text-teal-500 shrink-0" />
                                <span className="font-bold text-xs text-gray-900 dark:text-gray-100 truncate">
                                    Workspaces & Folders
                                </span>
                            </div>
                            <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-teal-500/10 text-teal-600 dark:text-teal-400 border border-teal-500/20 shrink-0">
                                workspaceStorage
                            </span>
                        </div>
                        <p className="text-[10px] text-gray-500 dark:text-gray-400 line-clamp-1" title="Copies open projects, workspace storage, and recent folder state">
                            Replicate or transfer open projects, workspace storage, and recent folders.
                        </p>
                    </div>
                    <div className="space-y-2 pt-1">
                        <SourceTargetSelect
                            value={copyProjectsSourceId}
                            onChange={setCopyProjectsSourceId}
                            options={candidateSources}
                            borderClass="border-gray-200 dark:border-[#15334d]"
                            focusClass="focus:ring-blue-500"
                        />
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                disabled={isOperating || !copyProjectsSourceId}
                                onClick={handleCopyProjects}
                                className="flex-1 px-3 py-1.5 rounded-[4px] text-xs font-semibold bg-teal-600 hover:bg-teal-700 text-white shadow-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                                title="Copy workspaces and project folders from source to target profile"
                            >
                                <FolderSync className="w-3 h-3" />
                                <span>Copy Folders</span>
                            </button>
                            <button
                                type="button"
                                disabled={isOperating || !copyProjectsSourceId}
                                onClick={handleMoveProjects}
                                className="flex-1 px-3 py-1.5 rounded-[4px] text-xs font-semibold bg-amber-600 hover:bg-amber-700 text-white shadow-xs transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                                title="Move workspaces and project folders to target profile (source wiped)"
                            >
                                <ArrowRight className="w-3 h-3" />
                                <span>Move Folders</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
