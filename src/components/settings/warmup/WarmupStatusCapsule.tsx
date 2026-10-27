import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Activity, Clock, Users, RefreshCw } from 'lucide-react';
import { useAccountStore } from '../../../stores/useAccountStore';
import { useWarmupStore } from '../../../stores/useWarmupStore';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';

export interface WarmupStatusCapsuleProps {
  status?: 'Active' | 'Idle' | 'Paused';
  nextScanSeconds?: number;
  accountsDueCount?: number;
  onTriggerScan?: () => Promise<void> | void;
  isScanning?: boolean;
  className?: string;
}

export const WarmupStatusCapsule: React.FC<WarmupStatusCapsuleProps> = ({
  status: propStatus,
  nextScanSeconds: propNextScan,
  accountsDueCount: propDueCount,
  onTriggerScan,
  isScanning: externalIsScanning,
  className,
}) => {
  const { t } = useTranslation();
  const accounts = useAccountStore((state) => state.accounts);
  const warmUpAccounts = useAccountStore((state) => state.warmUpAccounts);

  const storeIsActive = useWarmupStore((state) => state.isActive);
  const storeNextScanSeconds = useWarmupStore((state) => state.nextScanSeconds);
  const storeEligibleCount = useWarmupStore((state) => state.eligibleCount);

  // Derive status: props take precedence, then store, default to Active
  const status = propStatus ?? (storeIsActive ? 'Active' : 'Paused');
  const initialSeconds = propNextScan ?? (storeNextScanSeconds > 0 ? storeNextScanSeconds : 222);

  const [secondsRemaining, setSecondsRemaining] = useState<number>(initialSeconds);
  const [internalScanning, setInternalScanning] = useState(false);

  const isScanning = externalIsScanning ?? internalScanning;

  // Compute due accounts count from props, store, or account list
  const calculatedDueCount = React.useMemo(() => {
    if (typeof propDueCount === 'number') {
      return propDueCount;
    }
    if (typeof storeEligibleCount === 'number' && storeEligibleCount > 0) {
      return storeEligibleCount;
    }
    const eligible = accounts.filter((a) => !a.disabled && !a.proxy_disabled);
    const dueAccounts = eligible.filter((acc) => {
      const models = acc.quota?.models || [];
      return models.some((m) => m.percentage >= 99 || m.percentage <= 0);
    });
    return dueAccounts.length > 0 ? dueAccounts.length : Math.min(3, eligible.length);
  }, [accounts, propDueCount, storeEligibleCount]);

  // Sync with store countdown when updated
  useEffect(() => {
    if (propNextScan !== undefined) {
      setSecondsRemaining(propNextScan);
    } else if (storeNextScanSeconds > 0) {
      setSecondsRemaining(storeNextScanSeconds);
    }
  }, [propNextScan, storeNextScanSeconds]);

  // Synchronized countdown ticker
  useEffect(() => {
    if (status !== 'Active') return;
    const interval = setInterval(() => {
      setSecondsRemaining((prev) => (prev > 0 ? prev - 1 : 300));
    }, 1000);
    return () => clearInterval(interval);
  }, [status]);

  const formatCountdown = (totalSecs: number): string => {
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    return `${mins}m ${secs < 10 ? '0' : ''}${secs}s`;
  };

  const handleManualScan = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isScanning) return;

    setInternalScanning(true);
    try {
      if (onTriggerScan) {
        await onTriggerScan();
      } else {
        await warmUpAccounts();
      }
      setSecondsRemaining(300);
    } catch (err) {
      useErrorStore.getState().trackWarning(err, {
        source: 'WarmupStatusCapsule.handleManualScan',
        triggerAction: 'warm_up_accounts',
      });
    } finally {
      setInternalScanning(false);
    }
  };

  const getStatusColor = () => {
    if (isScanning) return 'bg-orange-500 animate-spin';
    switch (status) {
      case 'Active':
        return 'bg-emerald-400 animate-pulse';
      case 'Paused':
        return 'bg-amber-400';
      case 'Idle':
      default:
        return 'bg-zinc-500';
    }
  };

  return (
    <div
      className={cn(
        'inline-flex items-center rounded-full bg-zinc-900/80 backdrop-blur-md border border-zinc-800 p-0.5 text-xs shadow-inner divide-x divide-zinc-800/80 select-none transition-all duration-200',
        className
      )}
      role="status"
      aria-label={t('settings.warmup.status_capsule_label', 'Warmup Scheduler Status')}
    >
      {/* Segment 1: Scheduler State */}
      <div
        className="px-2.5 py-1 text-zinc-300 font-medium flex items-center gap-1.5"
        title={t('settings.warmup.scheduler_state_tooltip', 'Scheduler Status: {{status}}', { status })}
      >
        <span className={cn('w-2 h-2 rounded-full shrink-0', getStatusColor())} />
        <span className="font-semibold text-zinc-200">
          {isScanning
            ? t('settings.warmup.state_scanning', 'Scanning')
            : t(`settings.warmup.state_${status.toLowerCase()}`, status)}
        </span>
      </div>

      {/* Segment 2: Synchronized Ticker Countdown */}
      <div
        className="px-2.5 py-1 font-mono text-zinc-400 text-[11px] flex items-center gap-1.5"
        title={t('settings.warmup.next_scan_tooltip', 'Next scheduled quota reset check in {{time}}', {
          time: formatCountdown(secondsRemaining),
        })}
      >
        <Clock className="w-3 h-3 text-zinc-500 shrink-0" />
        <span>
          {t('settings.warmup.next_scan_prefix', 'Next Scan')} {formatCountdown(secondsRemaining)}
        </span>
      </div>

      {/* Segment 3: Due Accounts Badge & Manual Trigger */}
      <button
        type="button"
        onClick={handleManualScan}
        disabled={isScanning}
        className={cn(
          'px-2.5 py-1 text-orange-400 font-semibold text-[11px] hover:bg-zinc-800/60 rounded-r-full transition-colors cursor-pointer flex items-center gap-1.5 focus:outline-none focus:ring-1 focus:ring-orange-500/50',
          isScanning && 'opacity-70 cursor-not-allowed'
        )}
        title={t('settings.warmup.trigger_scan_tooltip', '{{count}} accounts eligible. Click to trigger scan now.', {
          count: calculatedDueCount,
        })}
      >
        {isScanning ? (
          <RefreshCw className="w-3 h-3 text-orange-400 animate-spin" />
        ) : (
          <Users className="w-3 h-3 text-orange-400/80" />
        )}
        <span>
          {t('settings.warmup.accounts_due_count', '{{count}} Accounts Due', {
            count: calculatedDueCount,
          })}
        </span>
      </button>
    </div>
  );
};

export default WarmupStatusCapsule;
