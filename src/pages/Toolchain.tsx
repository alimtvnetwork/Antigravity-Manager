import { Wrench, Loader2, Play, RefreshCw, CheckCircle2, XCircle, TerminalSquare, ChevronDown, MonitorDown } from "lucide-react";
import { useToolchain } from "./toolchain/useToolchain";
import { PROFILES, TROUBLESHOOTING } from "./toolchain/data";
import type { Profile, CheckEntry } from "./toolchain/types";
import { ToggleRow } from "./toolchain/ToggleRow";

export default function Toolchain() {
  const {
    t, items, loadingItems, selectedItems, force, setForce,
    quiet, setQuiet, skipNativeDeps, setSkipNativeDeps, sshTarget, setSshTarget,
    sshError, installing, log, setLog, logRef, checkReport, checking, openTrouble,
    setOpenTrouble, itemById, toggleItem, toggleProfile,
    profileActive, validateSsh, runCheck, runInstall,
  } = useToolchain();

  return (
    <div className="p-4 md:p-6 max-w-5xl mx-auto space-y-5">
        {/* Header */}
        <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-2xl bg-indigo-600/10 dark:bg-indigo-500/15">
                <Wrench className="w-6 h-6 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
                <h1 className="text-xl font-bold text-gray-900 dark:text-gray-100">
                    {t('toolchain.title', 'Toolchain Installer')}
                </h1>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                    {t(
                        'toolchain.subtitle',
                        'Install the Rust + frontend build toolchain — locally or over SSH.',
                    )}
                </p>
            </div>
        </div>

        {/* 1. Profile picker */}
        <section className="rounded-2xl border border-gray-100 dark:border-white/10 bg-white/70 dark:bg-white/5 backdrop-blur-xl p-4 md:p-5">
            <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-1">
                {t('toolchain.profiles', 'Install profiles')}
            </h2>
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                {t(
                    'toolchain.profiles_hint',
                    'Profiles stack — pick several, their items merge. Toggle cards on/off.',
                )}
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                {PROFILES.map((profile: Profile) => {
                    const active = profileActive(profile);
                    const Icon = profile.icon;
                    const previewNames = profile.items
                        .map((id: string) => itemById(id)?.name ?? id);
                    return (
                        <button
                            key={profile.id}
                            type="button"
                            onClick={() => toggleProfile(profile)}
                            className={`text-left rounded-2xl border p-4 transition-all cursor-pointer ${
                                active
                                    ? 'border-indigo-500 dark:border-indigo-400 bg-indigo-50 dark:bg-indigo-500/10 shadow-sm'
                                    : 'border-gray-200 dark:border-white/10 bg-white dark:bg-white/5 hover:border-indigo-300 dark:hover:border-indigo-500/40'
                            }`}
                        >
                            <div className="flex items-center justify-between mb-2">
                                <Icon
                                    className={`w-5 h-5 ${
                                        active
                                            ? 'text-indigo-600 dark:text-indigo-400'
                                            : 'text-gray-400 dark:text-gray-500'
                                    }`}
                                />
                                <span
                                    className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                                        active
                                            ? 'bg-indigo-600 text-white'
                                            : 'bg-gray-100 dark:bg-white/10 text-gray-500 dark:text-gray-400'
                                    }`}
                                >
                                    {active
                                        ? t('toolchain.selected', 'SELECTED')
                                        : t('toolchain.select', 'SELECT')}
                                </span>
                            </div>
                            <div className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                                {profile.label}
                            </div>
                            <div className="text-xs text-gray-500 dark:text-gray-400 mt-1 min-h-8">
                                {profile.desc}
                            </div>
                            <div className="mt-2 flex flex-wrap gap-1">
                                {previewNames.map((name: string) => (
                                    <span
                                        key={name}
                                        className="text-[10px] px-1.5 py-0.5 rounded-md bg-gray-100 dark:bg-white/10 text-gray-600 dark:text-gray-300"
                                    >
                                        {name}
                                    </span>
                                ))}
                            </div>
                        </button>
                    );
                })}
            </div>
        </section>

        {/* 2. Item checklist */}
        <section className="rounded-2xl border border-gray-100 dark:border-white/10 bg-white/70 dark:bg-white/5 backdrop-blur-xl p-4 md:p-5">
            <div className="flex items-center justify-between mb-3">
                <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                    {t('toolchain.items', 'Individual items')}
                </h2>
                {loadingItems && (
                    <Loader2 className="w-4 h-4 animate-spin text-gray-400" />
                )}
            </div>
            <div className="space-y-2">
                {items.map((item) => {
                    const checked = selectedItems.includes(item.id);
                    return (
                        <label
                            key={item.id}
                            className={`flex items-center gap-3 rounded-xl border px-3 py-2.5 cursor-pointer transition-colors ${
                                checked
                                    ? 'border-indigo-300 dark:border-indigo-500/50 bg-indigo-50/60 dark:bg-indigo-500/5'
                                    : 'border-gray-200 dark:border-white/10 hover:border-gray-300 dark:hover:border-white/20'
                            }`}
                        >
                            <input
                                type="checkbox"
                                checked={checked}
                                onChange={() => toggleItem(item.id)}
                                className="w-4 h-4 rounded accent-indigo-600 shrink-0"
                            />
                            <div className="flex-1 min-w-0">
                                <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                                    {item.name}
                                </div>
                                <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                                    {item.desc}
                                </div>
                            </div>
                            <span
                                className={`text-[10px] font-semibold px-2 py-0.5 rounded-full shrink-0 ${
                                    item.installed
                                        ? 'bg-emerald-100 dark:bg-emerald-500/15 text-emerald-700 dark:text-emerald-400'
                                        : 'bg-gray-100 dark:bg-white/10 text-gray-500 dark:text-gray-400'
                                }`}
                            >
                                {item.installed
                                    ? t('toolchain.installed', 'INSTALLED')
                                    : t('toolchain.missing', 'MISSING')}
                            </span>
                        </label>
                    );
                })}
            </div>
        </section>

        {/* 3. Options */}
        <section className="rounded-2xl border border-gray-100 dark:border-white/10 bg-white/70 dark:bg-white/5 backdrop-blur-xl p-4 md:p-5">
            <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">
                {t('toolchain.options', 'Options')}
            </h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <ToggleRow
                    label={t('toolchain.force', 'Force reinstall')}
                    hint={t('toolchain.force_hint', 'Reinstall even if already present')}
                    checked={force}
                    onChange={setForce}
                />
                <ToggleRow
                    label={t('toolchain.skip_native', 'Skip native deps')}
                    hint={t(
                        'toolchain.skip_native_hint',
                        'Skip OS packages (GTK/WebKit) — Rust toolchain only',
                    )}
                    checked={skipNativeDeps}
                    onChange={setSkipNativeDeps}
                />
                <ToggleRow
                    label={t('toolchain.quiet', 'Quiet mode')}
                    hint={t('toolchain.quiet_hint', 'Minimal output, non-interactive')}
                    checked={quiet}
                    onChange={setQuiet}
                />
                <div className="rounded-xl border border-gray-200 dark:border-white/10 px-3 py-2.5">
                    <label className="text-sm font-medium text-gray-900 dark:text-gray-100">
                        {t('toolchain.ssh_target', 'SSH target (optional)')}
                    </label>
                    <input
                        type="text"
                        value={sshTarget}
                        onChange={(e) => {
                            setSshTarget(e.target.value);
                            if (sshError) validateSsh(e.target.value);
                        }}
                        onBlur={() => validateSsh(sshTarget)}
                        placeholder="user@host"
                        spellCheck={false}
                        className="mt-1 w-full bg-transparent text-sm font-mono text-gray-900 dark:text-gray-100 placeholder-gray-400 outline-none"
                    />
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {t(
                            'toolchain.ssh_hint',
                            'Run the install on a remote machine. Key auth only — no passwords.',
                        )}
                    </p>
                    {sshError !== '' && (
                        <p className="text-xs text-red-500 mt-1">{sshError}</p>
                    )}
                </div>
            </div>
        </section>

        {/* 4. Actions + terminal */}
        <section className="rounded-2xl border border-gray-100 dark:border-white/10 bg-white/70 dark:bg-white/5 backdrop-blur-xl p-4 md:p-5 space-y-4">
            <div className="flex flex-wrap items-center gap-3">
                <button
                    type="button"
                    onClick={() => void runInstall()}
                    disabled={installing || selectedItems.length === 0}
                    className="flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-semibold bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer"
                >
                    {installing ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                    ) : (
                        <Play className="w-4 h-4" />
                    )}
                    {installing
                        ? t('toolchain.installing', 'Installing…')
                        : t('toolchain.install', 'Install')}
                    {selectedItems.length > 0 && ` (${selectedItems.length})`}
                </button>
                <button
                    type="button"
                    onClick={() => void runCheck()}
                    disabled={checking || installing}
                    className="flex items-center gap-2 px-4 py-2.5 rounded-full text-sm font-medium border border-gray-200 dark:border-white/15 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/10 disabled:opacity-50 transition-colors cursor-pointer"
                >
                    {checking ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                    ) : (
                        <RefreshCw className="w-4 h-4" />
                    )}
                    {t('toolchain.check', 'Check readiness')}
                </button>
                {log.length > 0 && !installing && (
                    <button
                        type="button"
                        onClick={() => setLog([])}
                        className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 underline underline-offset-2 cursor-pointer"
                    >
                        {t('toolchain.clear_log', 'Clear log')}
                    </button>
                )}
            </div>

            {checkReport !== null && (
                <div className="rounded-xl border border-gray-200 dark:border-white/10 divide-y divide-gray-100 dark:divide-white/5">
                    {checkReport.map((entry: CheckEntry) => (
                        <div
                            key={entry.id}
                            className="flex items-center gap-3 px-3 py-2"
                        >
                            {entry.ok ? (
                                <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
                            ) : (
                                <XCircle className="w-4 h-4 text-red-500 shrink-0" />
                            )}
                            <div className="flex-1 min-w-0">
                                <span className="text-sm text-gray-800 dark:text-gray-200">
                                    {entry.label}
                                </span>
                                {entry.hint && (
                                    <span className="block text-xs text-gray-500 dark:text-gray-400 truncate">
                                        {entry.hint}
                                    </span>
                                )}
                            </div>
                            <span
                                className={`text-[10px] font-semibold ${
                                    entry.ok
                                        ? 'text-emerald-600 dark:text-emerald-400'
                                        : 'text-red-600 dark:text-red-400'
                                }`}
                            >
                                {entry.ok ? 'OK' : 'MISSING'}
                            </span>
                        </div>
                    ))}
                </div>
            )}

            <div>
                <div className="flex items-center gap-2 mb-2">
                    <TerminalSquare className="w-4 h-4 text-gray-500 dark:text-gray-400" />
                    <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
                        {t('toolchain.log', 'Install log')}
                    </span>
                </div>
                <div
                    ref={logRef}
                    className="h-64 overflow-y-auto rounded-xl bg-gray-950 dark:bg-black/60 border border-gray-800 p-3 font-mono text-xs leading-relaxed text-gray-200"
                >
                    {log.length === 0 ? (
                        <span className="text-gray-600">
                            {t(
                                'toolchain.log_empty',
                                'No output yet — run an install to stream progress here.',
                            )}
                        </span>
                    ) : (
                        log.map((line, i) => (
                            <div key={i} className="whitespace-pre-wrap break-words">
                                {line}
                            </div>
                        ))
                    )}
                </div>
            </div>
        </section>

        {/* 5. Troubleshooting */}
        <section className="rounded-2xl border border-gray-100 dark:border-white/10 bg-white/70 dark:bg-white/5 backdrop-blur-xl p-4 md:p-5">
            <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">
                {t('toolchain.troubleshooting', 'Troubleshooting')}
            </h2>
            <div className="divide-y divide-gray-100 dark:divide-white/5 rounded-xl border border-gray-200 dark:border-white/10">
                {TROUBLESHOOTING.map((entry, i) => {
                    const open = openTrouble === i;
                    return (
                        <div key={i}>
                            <button
                                type="button"
                                onClick={() => setOpenTrouble(open ? null : i)}
                                className="w-full flex items-center justify-between gap-3 px-3 py-2.5 text-left cursor-pointer"
                            >
                                <span className="text-sm font-mono text-gray-800 dark:text-gray-200">
                                    {entry.issue}
                                </span>
                                <ChevronDown
                                    className={`w-4 h-4 text-gray-400 shrink-0 transition-transform ${
                                        open ? 'rotate-180' : ''
                                    }`}
                                />
                            </button>
                            {open && (
                                <div className="px-3 pb-3 text-sm text-gray-600 dark:text-gray-300">
                                    <span className="text-emerald-600 dark:text-emerald-400 font-medium">
                                        Fix:{' '}
                                    </span>
                                    {entry.fix}
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
            <p className="mt-3 text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1.5">
                <MonitorDown className="w-3.5 h-3.5" />
                {t(
                    'toolchain.ssh_note',
                    'Tip: use the SSH target field above to fix a remote machine without leaving this page.',
                )}
            </p>
        </section>
    </div>
  );
}
