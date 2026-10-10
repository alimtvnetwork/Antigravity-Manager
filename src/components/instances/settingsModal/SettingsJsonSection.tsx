import { Copy, ClipboardPaste, Download, Upload, RotateCcw, RotateCw, FileCode } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { InstanceSettingsApi } from './instanceSettingsTypes';

export function SettingsJsonSection({ api }: { api: InstanceSettingsApi }) {
    const {
        canUndo,
        canRedo,
        isOperating,
        jsonContent,
        setJsonContent,
        isJsonExpanded,
        setIsJsonExpanded,
        fileInputRef,
        handleUndo,
        handleRedo,
        handleCopyToClipboard,
        handlePasteFromClipboard,
        handleExportJsonFile,
        handleApplyJsonText,
    } = api;

    return (
        <div className="border border-gray-200 dark:border-[#15334d] rounded-xl p-3.5 bg-gray-50/50 dark:bg-[#071a27]/50 space-y-3">
            <div className="flex items-center justify-between flex-wrap gap-2">
                <span className="text-[11px] font-bold text-gray-400 uppercase tracking-wider flex items-center gap-1.5">
                    <FileCode className="w-3.5 h-3.5" />
                    <span>Clipboard & JSON Editor</span>
                </span>

                <div className="flex items-center gap-1">
                    <button
                        type="button"
                        disabled={!canUndo || isOperating}
                        onClick={handleUndo}
                        className="p-1 px-2 rounded-lg text-xs font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d] disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1 transition-colors cursor-pointer"
                        title="Undo last applied JSON settings change"
                    >
                        <RotateCcw className="w-3 h-3" />
                        <span>Undo</span>
                    </button>
                    <button
                        type="button"
                        disabled={!canRedo || isOperating}
                        onClick={handleRedo}
                        className="p-1 px-2 rounded-lg text-xs font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d] disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1 transition-colors cursor-pointer"
                        title="Redo previously undone change"
                    >
                        <RotateCw className="w-3 h-3" />
                        <span>Redo</span>
                    </button>
                </div>
            </div>

            <div className="flex items-center justify-between flex-wrap gap-2">
                <div className="inline-flex items-center rounded-[4px] bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs">
                    <button
                        type="button"
                        onClick={handleCopyToClipboard}
                        className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-l-[4px] rounded-r-none flex items-center gap-1.5 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Copy settings JSON to clipboard"
                    >
                        <Copy className="w-3.5 h-3.5 text-blue-500" />
                        <span>Copy</span>
                    </button>

                    <button
                        type="button"
                        disabled={isOperating}
                        onClick={handlePasteFromClipboard}
                        className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-none flex items-center gap-1.5 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
                        title="Paste settings JSON from clipboard and apply"
                    >
                        <ClipboardPaste className="w-3.5 h-3.5 text-emerald-500" />
                        <span>Paste</span>
                    </button>

                    <button
                        type="button"
                        onClick={handleExportJsonFile}
                        className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-none flex items-center gap-1.5 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Export settings to JSON file"
                    >
                        <Download className="w-3.5 h-3.5 text-indigo-500" />
                        <span>Export</span>
                    </button>

                    <button
                        type="button"
                        onClick={() => fileInputRef.current?.click()}
                        className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-none flex items-center gap-1.5 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Import settings from JSON file"
                    >
                        <Upload className="w-3.5 h-3.5 text-purple-500" />
                        <span>Import</span>
                    </button>

                    <button
                        type="button"
                        onClick={() => setIsJsonExpanded(!isJsonExpanded)}
                        className={cn(
                            'px-2.5 py-1.5 text-xs font-medium rounded-r-[4px] rounded-l-none flex items-center gap-1.5 transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            isJsonExpanded
                                ? 'bg-blue-50 dark:bg-blue-900/40 text-blue-600 dark:text-cyan-300 font-semibold'
                                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d]'
                        )}
                        title={isJsonExpanded ? 'Hide Raw JSON Editor' : 'Show Raw JSON Editor'}
                    >
                        <FileCode className="w-3.5 h-3.5 text-amber-500" />
                        <span>Raw Editor</span>
                    </button>
                </div>
            </div>

            {isJsonExpanded && (
                <div className="space-y-2 pt-2 animate-in fade-in">
                    <textarea
                        value={jsonContent}
                        onChange={(e) => setJsonContent(e.target.value)}
                        rows={8}
                        className="w-full p-2.5 rounded-xl font-mono text-[11px] bg-white dark:bg-[#051421] border border-gray-200 dark:border-[#15334d] text-gray-900 dark:text-emerald-300 focus:outline-none focus:ring-1 focus:ring-blue-500"
                        placeholder="{\n  // settings.json contents\n}"
                    />
                    <div className="flex justify-end gap-2">
                        <button
                            type="button"
                            disabled={isOperating}
                            onClick={handleApplyJsonText}
                            className="px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-700 text-white font-semibold text-xs shadow-xs transition-colors cursor-pointer"
                        >
                            Apply JSON to Target
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
