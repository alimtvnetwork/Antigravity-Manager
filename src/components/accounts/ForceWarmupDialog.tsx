import { useEffect } from 'react';
import { AlertTriangle, Flame, RefreshCw, X, ShieldAlert } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';
import { cn } from '../../utils/cn';

export interface ForceWarmupDialogProps {
  isOpen: boolean;
  onClose: () => void;
  accountEmail: string;
  onConfirmForce: () => Promise<void> | void;
  isForcing?: boolean;
}

function WarningHeader() {
  return (
    <div className="flex items-center gap-1.5 font-semibold text-xs text-rose-200">
      <AlertTriangle className="w-4 h-4 text-rose-400 shrink-0" />
      Force Warmup: This account was recently warmed. Forcing a probe will bypass the 6-day cooldown and consume quota.
    </div>
  );
}

function WarningCallout({ accountEmail }: { accountEmail: string }) {
  return (
    <div className="rounded-xl p-3 bg-rose-500/10 border border-rose-500/25 text-rose-300 space-y-2">
      <WarningHeader />
      <div className="text-[11px] text-rose-300/80 leading-relaxed">
        Target Account:{' '}
        <span className="font-mono font-medium text-rose-200 underline decoration-rose-500/40">
          {accountEmail}
        </span>
      </div>
    </div>
  );
}

function CooldownNotice() {
  return (
    <div className="rounded-xl p-3 bg-slate-800/40 border border-slate-700/30 text-xs text-slate-400 flex items-start gap-2">
      <ShieldAlert className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
      <p className="leading-relaxed">
        The 6-day cooldown preserves upstream Google quota and prevents rate-limit lockouts. Safe forcing should only be used when diagnosing quota recovery or account readiness.
      </p>
    </div>
  );
}

function CancelButton({ onClose, isForcing }: { onClose: () => void; isForcing: boolean }) {
  return (
    <button
      type="button"
      onClick={onClose}
      disabled={isForcing}
      className="px-3.5 py-1.5 rounded-xl text-xs font-medium text-slate-300 hover:bg-slate-800 transition-colors"
    >
      Cancel
    </button>
  );
}

function ConfirmButton({ onConfirm, isForcing }: { onConfirm: () => void; isForcing: boolean }) {
  const buttonStyles = cn(
    'flex items-center gap-1.5 px-4 py-1.5 rounded-xl text-xs font-semibold',
    'bg-gradient-to-r from-rose-600 to-rose-700 hover:from-rose-500 hover:to-rose-600',
    'text-white shadow-md shadow-rose-600/25 transition-all disabled:opacity-50'
  );
  return (
    <button type="button" onClick={onConfirm} disabled={isForcing} className={buttonStyles}>
      <RefreshCw className={cn('w-3.5 h-3.5', isForcing && 'animate-spin')} />
      {isForcing ? 'Bypassing Cooldown...' : 'Force Warmup Probe'}
    </button>
  );
}

interface ActionButtonsProps {
  onClose: () => void;
  onConfirm: () => void;
  isForcing: boolean;
}

function ActionButtons({ onClose, onConfirm, isForcing }: ActionButtonsProps) {
  return (
    <div className="flex items-center justify-end gap-2.5 mt-5 pt-3 border-t border-slate-800">
      <CancelButton onClose={onClose} isForcing={isForcing} />
      <ConfirmButton onConfirm={onConfirm} isForcing={isForcing} />
    </div>
  );
}

export function ForceWarmupDialog({
  isOpen,
  onClose,
  accountEmail,
  onConfirmForce,
  isForcing = false,
}: ForceWarmupDialogProps) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen && !isForcing) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
    }
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isOpen, isForcing, onClose]);

  if (!isOpen) return null;

  const handleConfirm = async () => {
    if (isForcing) return;
    try {
      await onConfirmForce();
    } catch (err: unknown) {
      useErrorStore.getState().trackWarning(err, {
        source: 'ForceWarmupDialog.handleConfirm',
        triggerAction: 'force_warmup_probe',
        metadata: { accountEmail },
      });
    }
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4">
      <div
        onClick={() => !isForcing && onClose()}
        className="fixed inset-0 bg-black/65 backdrop-blur-sm animate-in fade-in duration-150"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="force-warmup-title"
        className={cn(
          'relative w-full max-w-md rounded-2xl border shadow-2xl p-5',
          'bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-xl',
          'border-rose-500/30 text-slate-100',
          'animate-in zoom-in-95 duration-150'
        )}
      >
        <div className="flex items-center justify-between pb-3 border-b border-slate-800">
          <div className="flex items-center gap-2">
            <Flame className="w-5 h-5 text-rose-500" />
            <h3 id="force-warmup-title" className="text-base font-semibold text-slate-100">
              Override Warmup Cooldown
            </h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            disabled={isForcing}
            className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="mt-4 space-y-3">
          <WarningCallout accountEmail={accountEmail} />
          <CooldownNotice />
        </div>

        <ActionButtons onClose={onClose} onConfirm={handleConfirm} isForcing={isForcing} />
      </div>
    </div>
  );
}

export default ForceWarmupDialog;
