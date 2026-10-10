import { useTranslation } from 'react-i18next';
import {
    ShieldCheck,
    Lock,
    FileText,
    AlertTriangle,
    Loader2,
    Eye,
    EyeOff,
    Radio,
    CheckCircle2,
} from 'lucide-react';
import type { FleetDeployResult } from '../../../services/accountService';
import type { ExportScope } from './types';

export interface ExportTabProps {
    exportScope: ExportScope;
    setExportScope: (scope: ExportScope) => void;
    includeAuditHistory: boolean;
    setIncludeAuditHistory: (value: boolean) => void;
    usePassword: boolean;
    setUsePassword: (value: boolean) => void;
    password: string;
    setPassword: (value: string) => void;
    confirmPassword: string;
    setConfirmPassword: (value: string) => void;
    showPassword: boolean;
    setShowPassword: (value: boolean) => void;
    isExporting: boolean;
    includeMainNode: boolean;
    setIncludeMainNode: (value: boolean) => void;
    isDeployingFleet: boolean;
    gitmapAvailable: boolean | null;
    fleetDeployResult: FleetDeployResult | null;
    onDeployToFleet: () => void;
}

export function ExportTab(props: ExportTabProps) {
    const { t } = useTranslation();
    const {
        exportScope,
        setExportScope,
        includeAuditHistory,
        setIncludeAuditHistory,
        usePassword,
        setUsePassword,
        password,
        setPassword,
        confirmPassword,
        setConfirmPassword,
        showPassword,
        setShowPassword,
        includeMainNode,
        setIncludeMainNode,
        isDeployingFleet,
        gitmapAvailable,
        fleetDeployResult,
        onDeployToFleet,
    } = props;

    return (
        <>
            {/* Backup Scope Selection */}
            <div className="space-y-2">
                <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    {t('backup.scope_label', 'Backup Scope')}
                </label>
                <div className="grid grid-cols-2 gap-3">
                    <div
                        onClick={() => setExportScope('accounts')}
                        className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                            exportScope === 'accounts'
                                ? 'border-purple-500 bg-purple-50/50 dark:bg-purple-950/20'
                                : 'border-gray-200 dark:border-slate-800 hover:bg-gray-50 dark:hover:bg-slate-800/50'
                        }`}
                    >
                        <div className="flex items-center gap-2">
                            <FileText className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                            <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                {t('backup.scope_accounts', 'Accounts & Tokens')}
                            </span>
                        </div>
                        <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400">
                            {t(
                                'backup.scope_accounts_desc',
                                'Exports account credentials and refresh tokens only.'
                            )}
                        </p>
                    </div>

                    <div
                        onClick={() => setExportScope('full')}
                        className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                            exportScope === 'full'
                                ? 'border-purple-500 bg-purple-50/50 dark:bg-purple-950/20'
                                : 'border-gray-200 dark:border-slate-800 hover:bg-gray-50 dark:hover:bg-slate-800/50'
                        }`}
                    >
                        <div className="flex items-center gap-2">
                            <ShieldCheck className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                            <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                {t('backup.scope_full', 'Full System')}
                            </span>
                        </div>
                        <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400">
                            {t(
                                'backup.scope_full_desc',
                                'Accounts, configurations, instances, proxies & email vault.'
                            )}
                        </p>
                    </div>
                </div>
            </div>

            {/* Full Scope Options: Audit Records */}
            {exportScope === 'full' && (
                <div className="p-3.5 bg-gray-50 dark:bg-slate-800/60 rounded-xl border border-gray-200 dark:border-slate-800 space-y-2">
                    <label className="flex items-center justify-between cursor-pointer">
                        <div className="flex items-center gap-2">
                            <FileText className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                            <div>
                                <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                    {t('backup.include_audit', 'Include Audit Records & Task History')}
                                </span>
                                <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                    {t('backup.include_audit_desc', 'Export account switches, prompt scheduler runs, and task history (up to 500 records).')}
                                </p>
                            </div>
                        </div>
                        <input
                            type="checkbox"
                            checked={includeAuditHistory}
                            onChange={(e) => setIncludeAuditHistory(e.target.checked)}
                            className="rounded border-gray-300 text-purple-600 focus:ring-purple-500 w-4 h-4"
                        />
                    </label>
                </div>
            )}

            {/* Password Protection Toggle */}
            <div className="p-3.5 bg-gray-50 dark:bg-slate-800/60 rounded-xl border border-gray-200 dark:border-slate-800 space-y-3">
                <label className="flex items-center justify-between cursor-pointer">
                    <div className="flex items-center gap-2">
                        <Lock className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                        <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                            {t('backup.encrypt_toggle', 'Encrypt with Password (AES-256-GCM)')}
                        </span>
                    </div>
                    <input
                        type="checkbox"
                        checked={usePassword}
                        onChange={(e) => setUsePassword(e.target.checked)}
                        className="rounded border-gray-300 text-purple-600 focus:ring-purple-500 w-4 h-4"
                    />
                </label>

                {usePassword && (
                    <div className="pt-2 space-y-2.5 animate-in fade-in duration-150">
                        <div className="relative">
                            <input
                                type={showPassword ? 'text' : 'password'}
                                placeholder={t('backup.password_placeholder', 'Enter strong encryption password')}
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                className="w-full px-3 py-1.5 pr-8 text-xs border border-gray-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100 focus:outline-hidden focus:ring-2 focus:ring-purple-500"
                            />
                            <button
                                type="button"
                                onClick={() => setShowPassword(!showPassword)}
                                className="absolute right-2.5 top-2 text-gray-400 hover:text-gray-600"
                            >
                                {showPassword ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                            </button>
                        </div>

                        <input
                            type="password"
                            placeholder={t('backup.confirm_password_placeholder', 'Confirm encryption password')}
                            value={confirmPassword}
                            onChange={(e) => setConfirmPassword(e.target.value)}
                            className="w-full px-3 py-1.5 text-xs border border-gray-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100 focus:outline-hidden focus:ring-2 focus:ring-purple-500"
                        />

                        <div className="p-2.5 bg-amber-50 dark:bg-amber-950/20 border border-amber-200/50 dark:border-amber-900/30 rounded-lg flex items-start gap-1.5 text-[11px] text-amber-800 dark:text-amber-300">
                            <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                            <span>
                                {t(
                                    'backup.password_warning',
                                    'Encryption uses native PBKDF2 (100k rounds) + AES-GCM. Passwords cannot be recovered if forgotten.'
                                )}
                            </span>
                        </div>
                    </div>
                )}
            </div>

            {/* Fleet Deployment Card via GitMap */}
            <div className="p-4 bg-gradient-to-br from-indigo-50/70 via-purple-50/40 to-blue-50/50 dark:from-indigo-950/25 dark:via-purple-950/20 dark:to-slate-800/60 rounded-xl border border-indigo-200/80 dark:border-indigo-800/50 space-y-3.5 shadow-2xs">
                <div className="flex items-start justify-between gap-3">
                    <div className="flex items-start gap-2.5">
                        <div className="p-2 rounded-lg bg-indigo-100/80 dark:bg-indigo-900/50 text-indigo-600 dark:text-indigo-400 mt-0.5 shrink-0">
                            <Radio className="w-4 h-4 animate-pulse" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2 flex-wrap">
                                <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                    {t('backup.fleet_deploy_title', 'Deploy to Fleet via GitMap')}
                                </span>
                                {gitmapAvailable !== null && (
                                    <span
                                        className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${
                                            gitmapAvailable
                                                ? 'bg-emerald-100 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-300 border border-emerald-300 dark:border-emerald-800'
                                                : 'bg-amber-100 dark:bg-amber-950/60 text-amber-700 dark:text-amber-300 border border-amber-300 dark:border-amber-800'
                                        }`}
                                    >
                                        {gitmapAvailable ? 'GitMap Ready' : 'GitMap CLI Missing'}
                                    </span>
                                )}
                            </div>
                            <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400 leading-relaxed">
                                {t(
                                    'backup.fleet_deploy_subtitle',
                                    'Synchronize local AGM accounts and tokens across all open cluster nodes (u1, w1...) automatically via peer-to-peer encrypted SSH.'
                                )}
                            </p>
                        </div>
                    </div>
                </div>

                {/* Toggle: Include 'main' node */}
                <div className="pt-2 border-t border-indigo-100 dark:border-indigo-900/40">
                    <label className="flex items-start justify-between gap-3 cursor-pointer">
                        <div>
                            <span className="text-xs font-medium text-gray-800 dark:text-gray-200">
                                {t('backup.fleet_include_main', "Include 'main' node")}
                            </span>
                            <p className="text-[11px] text-amber-700 dark:text-amber-400 mt-0.5 leading-tight">
                                {t(
                                    'backup.fleet_main_warning',
                                    "By default, the central orchestrator 'main' node is excluded to prevent accidental override."
                                )}
                            </p>
                        </div>
                        <input
                            type="checkbox"
                            checked={includeMainNode}
                            onChange={(e) => setIncludeMainNode(e.target.checked)}
                            disabled={isDeployingFleet}
                            className="mt-0.5 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500 w-4 h-4 cursor-pointer"
                        />
                    </label>
                </div>

                {/* Deploy Action & Status */}
                <div className="space-y-2 pt-1">
                    <button
                        type="button"
                        onClick={onDeployToFleet}
                        disabled={isDeployingFleet}
                        className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-500 active:bg-indigo-700 disabled:opacity-50 rounded-xl shadow-xs transition-all cursor-pointer"
                    >
                        {isDeployingFleet ? (
                            <>
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                <span>{t('backup.fleet_deploying', 'Deploying to Fleet...')}</span>
                            </>
                        ) : (
                            <span>🚀 Deploy to Fleet</span>
                        )}
                    </button>

                    {fleetDeployResult && (
                        <div
                            className={`p-3 rounded-lg border text-xs animate-in fade-in space-y-1 ${
                                fleetDeployResult.success
                                    ? 'bg-emerald-50 dark:bg-emerald-950/20 border-emerald-200 dark:border-emerald-800/40 text-emerald-900 dark:text-emerald-300'
                                    : 'bg-rose-50 dark:bg-rose-950/20 border-rose-200 dark:border-rose-800/40 text-rose-900 dark:text-rose-300'
                            }`}
                        >
                            <div className="flex items-center gap-1.5 font-semibold">
                                {fleetDeployResult.success ? (
                                    <CheckCircle2 className="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
                                ) : (
                                    <AlertTriangle className="w-4 h-4 text-rose-600 dark:text-rose-400 shrink-0" />
                                )}
                                <span>{fleetDeployResult.message}</span>
                            </div>
                            {fleetDeployResult.raw_output &&
                                fleetDeployResult.raw_output.trim() !== '' &&
                                fleetDeployResult.raw_output !== 'null' && (
                                    <pre className="text-[10px] mt-1 p-2 bg-black/5 dark:bg-black/30 rounded overflow-x-auto font-mono max-h-24 whitespace-pre-wrap">
                                        {fleetDeployResult.raw_output}
                                    </pre>
                                )}
                        </div>
                    )}
                </div>
            </div>
        </>
    );
}
