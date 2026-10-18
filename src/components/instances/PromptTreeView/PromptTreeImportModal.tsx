import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import {
    X,
    Upload,
    FileUp,
    FileJson,
    Database,
    FileText,
    CheckCircle2,
    AlertCircle,
    Loader2,
    Image as ImageIcon,
    Check,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../../utils/cn';
import { isTauri } from '../../../utils/env';
import { useErrorStore } from '../../../stores/error-store';

export interface ValidationTelemetry {
    is_valid: boolean;
    format: string;
    scope: string;
    project_count: number;
    conversation_count: number;
    prompt_count: number;
    image_count: number;
    total_image_bytes: number;
    message: string;
}

export interface PromptTreeImportModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId: string;
    instanceName: string;
    onExecuteImport: (params: {
        filePath?: string;
        dataB64?: string;
        conflictMode: 'merge' | 'replace';
        materializeImages: boolean;
        targetInstanceId?: string;
    }) => Promise<number>;
    onImportComplete?: () => void;
}

export function PromptTreeImportModal({
    isOpen,
    onClose,
    instanceId,
    instanceName,
    onExecuteImport,
    onImportComplete,
}: PromptTreeImportModalProps) {
    const [selectedPath, setSelectedPath] = useState<string | null>(null);
    const [selectedFileName, setSelectedFileName] = useState<string | null>(null);
    const [dataB64, setDataB64] = useState<string | null>(null);
    const [isValidating, setIsValidating] = useState<boolean>(false);
    const [validation, setValidation] = useState<ValidationTelemetry | null>(null);
    const [validationError, setValidationError] = useState<string | null>(null);

    const [conflictMode, setConflictMode] = useState<'merge' | 'replace'>('merge');
    const [materializeImages, setMaterializeImages] = useState<boolean>(true);
    const [targetInst, setTargetInst] = useState<string>(instanceId || 'default');

    const [isImporting, setIsImporting] = useState<boolean>(false);
    const [importSuccess, setImportSuccess] = useState<boolean>(false);
    const [importedCount, setImportedCount] = useState<number>(0);

    const fileInputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                e.stopPropagation();
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    const handleValidateFile = async (path?: string, b64?: string) => {
        setIsValidating(true);
        setValidationError(null);
        setValidation(null);
        try {
            const telemetry = await invoke<ValidationTelemetry>('validate_prompt_tree_import', {
                filePath: path || null,
                sourcePath: path || null,
                dataB64: b64 || null,
            });
            setValidation(telemetry);
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : String(err) || 'Validation failed';
            setValidationError(msg);
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptTreeImportModal.handleValidateFile',
                triggerAction: 'validate_prompt_tree_import',
            });
        } finally {
            setIsValidating(false);
        }
    };

    const handleBrowseFile = async () => {
        if (!isTauri()) {
            fileInputRef.current?.click();
            return;
        }

        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({
                multiple: false,
                filters: [
                    {
                        name: 'Prompt Tree Exports (*.json, *.db, *.yaml)',
                        extensions: ['json', 'db', 'sqlite', 'yaml', 'yml'],
                    },
                ],
            });

            if (!selected || typeof selected !== 'string') return;

            setSelectedPath(selected);
            const name = selected.replace(/\\/g, '/').split('/').pop() || selected;
            setSelectedFileName(name);
            setDataB64(null);

            await handleValidateFile(selected, undefined);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeImportModal.handleBrowseFile',
                triggerAction: 'open_file_dialog',
            });
        }
    };

    const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        setSelectedFileName(file.name);
        setSelectedPath(null);

        const reader = new FileReader();
        reader.onload = async (event) => {
            try {
                const buf = event.target?.result;
                if (!buf) return;

                let b64 = '';
                if (typeof buf === 'string') {
                    b64 = btoa(buf);
                } else if (buf instanceof ArrayBuffer) {
                    const bytes = new Uint8Array(buf);
                    let binary = '';
                    const chunk = 0x8000;
                    for (let i = 0; i < bytes.length; i += chunk) {
                        binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + chunk)));
                    }
                    b64 = btoa(binary);
                }

                setDataB64(b64);
                await handleValidateFile(undefined, b64);
            } catch (err: unknown) {
                const msg = err instanceof Error ? err.message : String(err);
                setValidationError(`Failed to read file: ${msg}`);
            }
        };

        if (/\.(db|sqlite)$/i.test(file.name)) {
            reader.readAsArrayBuffer(file);
        } else {
            reader.readAsText(file);
        }
        e.target.value = '';
    };

    const handleImport = async () => {
        if (!selectedPath && !dataB64) return;
        setIsImporting(true);
        try {
            const count = await onExecuteImport({
                filePath: selectedPath || undefined,
                dataB64: dataB64 || undefined,
                conflictMode,
                materializeImages,
                targetInstanceId: targetInst,
            });

            setImportedCount(count);
            setImportSuccess(true);
            if (onImportComplete) {
                onImportComplete();
            }
            setTimeout(() => {
                onClose();
            }, 1500);
        } catch (err: unknown) {
            useErrorStore.getState().captureError(err, {
                source: 'PromptTreeImportModal.handleImport',
                triggerAction: 'import_prompt_tree_section',
            });
        } finally {
            setIsImporting(false);
        }
    };

    const formatBadgeIcon = () => {
        if (!validation) return <FileUp className="w-5 h-5 text-blue-500" />;
        const fmt = validation.format.toLowerCase();
        if (fmt.includes('sqlite') || fmt.includes('db')) return <Database className="w-5 h-5 text-indigo-500" />;
        if (fmt.includes('yaml')) return <FileText className="w-5 h-5 text-emerald-500" />;
        return <FileJson className="w-5 h-5 text-amber-500" />;
    };

    return createPortal(
        <div
            className="fixed inset-0 z-[100050] flex items-center justify-center p-4 bg-black/75 backdrop-blur-md overflow-y-auto animate-in fade-in duration-150"
            onClick={onClose}
        >
            <div
                className="relative w-full max-w-lg rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#120c1d] shadow-2xl p-5 space-y-4 text-slate-800 dark:text-slate-100 animate-in zoom-in-95 duration-150 max-h-[90vh] overflow-y-auto"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Hidden Web File Input */}
                <input
                    ref={fileInputRef}
                    type="file"
                    accept=".json,.db,.sqlite,.yaml,.yml"
                    className="hidden"
                    onChange={handleFileInputChange}
                />

                {/* Header */}
                <div className="flex items-center justify-between border-b border-slate-100 dark:border-[#2b1b42] pb-3">
                    <div className="flex items-center gap-2">
                        <div className="p-1.5 rounded-lg bg-emerald-500/10 dark:bg-emerald-500/20 text-emerald-600 dark:text-emerald-400">
                            <Upload className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold tracking-tight">Import Prompt Tree Section</h2>
                            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                Import JSON, SQLite DB, or YAML into {instanceName || instanceId || 'target profile'} with automatic Base64 image materialization.
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#251b3a] transition-colors cursor-pointer"
                        title="Close (Escape)"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* File Picker Drag & Drop Zone */}
                <div
                    onClick={handleBrowseFile}
                    className={cn(
                        "flex flex-col items-center justify-center p-6 rounded-2xl border-2 border-dashed transition-all cursor-pointer text-center",
                        selectedFileName
                            ? "border-emerald-500/60 bg-emerald-500/5 dark:bg-emerald-500/10"
                            : "border-slate-300 dark:border-[#40285e] hover:border-blue-500 hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                    )}
                >
                    <div className="p-3 rounded-full bg-slate-100 dark:bg-[#1e1430] mb-2">
                        {formatBadgeIcon()}
                    </div>
                    {selectedFileName ? (
                        <>
                            <span className="text-xs font-bold text-slate-800 dark:text-slate-100 break-all max-w-sm">
                                {selectedFileName}
                            </span>
                            <span className="text-[10px] text-emerald-600 dark:text-emerald-400 mt-1 font-medium">
                                Click to select a different export file
                            </span>
                        </>
                    ) : (
                        <>
                            <span className="text-xs font-semibold text-slate-700 dark:text-slate-200">
                                Click to browse export file
                            </span>
                            <span className="text-[10px] text-slate-400 mt-1">
                                Supports .json (envelope v2.0), .db (SQLite portable), .yaml
                            </span>
                        </>
                    )}
                </div>

                {/* Validation Telemetry Banner */}
                {isValidating && (
                    <div className="flex items-center gap-2 px-3 py-2 rounded-xl bg-blue-500/10 border border-blue-500/20 text-xs text-blue-600 dark:text-cyan-400">
                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        <span>Validating export envelope integrity...</span>
                    </div>
                )}

                {validationError && (
                    <div className="flex items-start gap-2 px-3 py-2 rounded-xl bg-rose-500/10 border border-rose-500/20 text-xs text-rose-600 dark:text-rose-400">
                        <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
                        <span className="break-all">{validationError}</span>
                    </div>
                )}

                {validation && !validationError && (
                    <div className="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 space-y-1.5">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-1.5 text-xs font-bold text-emerald-700 dark:text-emerald-300">
                                <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                                <span>Verified AGM Export ({validation.format.toUpperCase()})</span>
                            </div>
                            <span className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 font-bold">
                                {validation.scope}
                            </span>
                        </div>
                        <div className="grid grid-cols-4 gap-2 text-[10px] font-mono text-slate-600 dark:text-slate-300 pt-1 border-t border-emerald-500/20">
                            <div>Projects: <strong>{validation.project_count}</strong></div>
                            <div>Prompts: <strong>{validation.prompt_count}</strong></div>
                            <div>Images: <strong>{validation.image_count}</strong></div>
                            <div>Size: <strong>{(validation.total_image_bytes / 1024).toFixed(1)} KB</strong></div>
                        </div>
                    </div>
                )}

                {/* Conflict Resolution Mode */}
                <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
                        Conflict Mode
                    </label>
                    <div className="grid grid-cols-2 gap-1.5 p-1 rounded-xl bg-slate-100/80 dark:bg-[#1a1228] border border-slate-200/60 dark:border-[#382352]">
                        <button
                            type="button"
                            onClick={() => setConflictMode('merge')}
                            className={cn(
                                "py-1.5 text-xs font-medium rounded-lg transition-all cursor-pointer text-center",
                                conflictMode === 'merge'
                                    ? "bg-blue-600 text-white shadow-xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                            )}
                        >
                            Merge & Upsert (FIFO)
                        </button>
                        <button
                            type="button"
                            onClick={() => setConflictMode('replace')}
                            className={cn(
                                "py-1.5 text-xs font-medium rounded-lg transition-all cursor-pointer text-center",
                                conflictMode === 'replace'
                                    ? "bg-blue-600 text-white shadow-xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                            )}
                        >
                            Clean Scope Replace
                        </button>
                    </div>
                </div>

                {/* Target Instance Selector */}
                <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
                        Target Instance Profile
                    </label>
                    <input
                        type="text"
                        value={targetInst}
                        onChange={(e) => setTargetInst(e.target.value)}
                        placeholder="Instance ID (e.g. default)"
                        className="w-full text-xs font-medium rounded-lg border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#1a1228] px-3 py-1.5 text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                    />
                </div>

                {/* Image Materialization Checkbox */}
                <div className="flex items-center justify-between p-3 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#1a1228]/60">
                    <div className="flex items-start gap-2.5">
                        <ImageIcon className="w-4 h-4 text-purple-500 mt-0.5 shrink-0" />
                        <div>
                            <span className="text-xs font-semibold text-slate-700 dark:text-slate-200">
                                Materialize Images to Disk
                            </span>
                            <p className="text-[10px] text-slate-500 dark:text-slate-400 leading-tight">
                                Decodes embedded Base64 data URIs into physical image files in the instance&apos;s attachments directory and rewrites prompt references to local file links.
                            </p>
                        </div>
                    </div>
                    <input
                        type="checkbox"
                        checked={materializeImages}
                        onChange={(e) => setMaterializeImages(e.target.checked)}
                        className="rounded border-slate-300 text-purple-600 focus:ring-purple-500 w-4 h-4 cursor-pointer"
                    />
                </div>

                {/* Footer Buttons */}
                <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 dark:border-[#2b1b42]">
                    <button
                        type="button"
                        onClick={onClose}
                        disabled={isImporting}
                        className="px-4 py-1.5 text-xs font-semibold rounded-md border border-slate-200 dark:border-[#40285e] hover:bg-slate-100 dark:hover:bg-[#251b3a] text-slate-700 dark:text-slate-300 transition-colors cursor-pointer"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleImport}
                        disabled={isImporting || (!selectedPath && !dataB64) || Boolean(validationError) || importSuccess}
                        className={cn(
                            "flex items-center gap-1.5 px-5 py-1.5 text-xs font-bold rounded-md text-white shadow-md transition-all cursor-pointer",
                            importSuccess
                                ? "bg-emerald-600 hover:bg-emerald-700"
                                : "bg-emerald-600 hover:bg-emerald-700 active:scale-95 disabled:opacity-50"
                        )}
                    >
                        {isImporting ? (
                            <>
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                <span>Importing...</span>
                            </>
                        ) : importSuccess ? (
                            <>
                                <Check className="w-3.5 h-3.5" />
                                <span>Imported {importedCount} Prompt(s)!</span>
                            </>
                        ) : (
                            <>
                                <FileUp className="w-3.5 h-3.5" />
                                <span>Import & Merge Prompts</span>
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>,
        document.body
    );
}
