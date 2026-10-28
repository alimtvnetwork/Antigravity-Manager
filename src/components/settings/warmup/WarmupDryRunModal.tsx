import React, { useEffect } from 'react';
import { FlaskConical, ShieldCheck, Users, Sparkles, Clock, AlertTriangle, X, RefreshCw } from 'lucide-react';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';

export interface DryRunSimulationReport {
  totalEligible: number;
  wouldProbeModels: number;
  authIssues: number;
  cooldownSkipped: number;
}

export interface WarmupDryRunModalProps {
  isOpen: boolean;
  onClose: () => void;
  report?: DryRunSimulationReport;
  onRunSimulation?: () => Promise<void>;
  isLoading?: boolean;
}

interface MetricCardProps {
  label: string;
  value: number;
  icon: React.ComponentType<{ className?: string }>;
  accentColorClass: string;
  badgeText?: string;
}

function MetricBadge({ badgeText }: { badgeText?: string }) {
  if (!badgeText) return null;
  return (
    <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-slate-700/40 text-slate-300">
      {badgeText}
    </span>
  );
}

function MetricCard({ label, value, icon: Icon, accentColorClass, badgeText }: MetricCardProps) {
  return (
    <div className="flex flex-col p-3 rounded-xl bg-slate-800/60 dark:bg-slate-900/60 border border-slate-700/40">
      <div className="flex items-center justify-between text-xs text-slate-400 mb-1.5">
        <span className="flex items-center gap-1.5">
          <Icon className={cn('w-4 h-4', accentColorClass)} />
          {label}
        </span>
        <MetricBadge badgeText={badgeText} />
      </div>
      <div className="text-xl font-bold text-slate-100 pl-5">{value}</div>
    </div>
  );
}

function SimulationBanner() {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-xl bg-amber-500/15 border border-amber-500/30 text-amber-300 text-xs font-medium">
      <FlaskConical className="w-4 h-4 text-amber-400 shrink-0" />
      <span>[DRY-RUN SIMULATION] No quota consumed</span>
    </div>
  );
}

function MetricFirstRow({ report }: { report: DryRunSimulationReport }) {
  return (
    <>
      <MetricCard
        label="Eligible Accounts"
        value={report.totalEligible}
        icon={Users}
        accentColorClass="text-sky-400"
      />
      <MetricCard
        label="Probed Models"
        value={report.wouldProbeModels}
        icon={Sparkles}
        accentColorClass="text-purple-400"
      />
    </>
  );
}

function MetricSecondRow({ report }: { report: DryRunSimulationReport }) {
  return (
    <>
      <MetricCard
        label="Cooldown Skipped"
        value={report.cooldownSkipped}
        icon={Clock}
        accentColorClass="text-amber-400"
        badgeText="< 6 days"
      />
      <MetricCard
        label="Auth Anomalies"
        value={report.authIssues}
        icon={AlertTriangle}
        accentColorClass="text-rose-400"
      />
    </>
  );
}

function SimulationMetricGrid({ report }: { report: DryRunSimulationReport }) {
  return (
    <div className="grid grid-cols-2 gap-2.5 my-4">
      <MetricFirstRow report={report} />
      <MetricSecondRow report={report} />
    </div>
  );
}

function SimulationExplanation() {
  return (
    <div className="rounded-xl p-3 bg-slate-800/40 border border-slate-700/30 text-xs text-slate-400 space-y-1.5">
      <div className="flex items-center gap-1.5 text-slate-300 font-medium">
        <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
        Zero Quota Simulation Guarantee
      </div>
      <p className="leading-relaxed">
        Dry-run scans account health tokens and cooldown windows without initiating completion calls or consuming Google Gemini token quotas.
      </p>
    </div>
  );
}

export function WarmupDryRunModal({
  isOpen,
  onClose,
  report = { totalEligible: 0, wouldProbeModels: 0, authIssues: 0, cooldownSkipped: 0 },
  onRunSimulation,
  isLoading = false,
}: WarmupDryRunModalProps) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen && !isLoading) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
    }
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isOpen, isLoading, onClose]);

  if (!isOpen) return null;

  const handleSimulate = async () => {
    if (!onRunSimulation || isLoading) return;
    try {
      await onRunSimulation();
    } catch (err: unknown) {
      useErrorStore.getState().trackWarning(err, {
        source: 'WarmupDryRunModal.handleSimulate',
        triggerAction: 'run_warmup_simulation',
      });
    }
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4">
      <div
        onClick={() => !isLoading && onClose()}
        className="fixed inset-0 bg-black/60 backdrop-blur-sm animate-in fade-in duration-150"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="dry-run-modal-title"
        className={cn(
          'relative w-full max-w-md rounded-2xl border shadow-2xl p-5',
          'bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-xl',
          'border-slate-700/70 dark:border-slate-800/80 text-slate-100',
          'animate-in zoom-in-95 duration-150'
        )}
      >
        <div className="flex items-center justify-between pb-3 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <FlaskConical className="w-5 h-5 text-amber-400" />
            <h3 id="dry-run-modal-title" className="text-base font-semibold text-slate-100">
              Warmup Fleet Simulation
            </h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            disabled={isLoading}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="mt-4 space-y-3">
          <SimulationBanner />
          <SimulationMetricGrid report={report} />
          <SimulationExplanation />
        </div>

        <div className="flex items-center justify-end gap-2.5 mt-5 pt-3 border-t border-slate-800">
          <button
            type="button"
            onClick={onClose}
            disabled={isLoading}
            className="px-3.5 py-1.5 rounded-xl text-xs font-medium text-slate-300 hover:bg-slate-800 transition-colors"
          >
            Close
          </button>
          {onRunSimulation && (
            <button
              type="button"
              onClick={handleSimulate}
              disabled={isLoading}
              className={cn(
                'flex items-center gap-1.5 px-4 py-1.5 rounded-xl text-xs font-semibold',
                'bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500',
                'text-slate-950 shadow-md shadow-amber-500/20 transition-all disabled:opacity-50'
              )}
            >
              <RefreshCw className={cn('w-3.5 h-3.5', isLoading && 'animate-spin')} />
              {isLoading ? 'Simulating...' : 'Run Simulation'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export default WarmupDryRunModal;
