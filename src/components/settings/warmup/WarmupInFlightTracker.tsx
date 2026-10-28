import React, { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Radio, Cpu, Clock, X } from 'lucide-react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useErrorStore } from '../../../stores/error-store';
import { useWarmupStore } from '../../../stores/useWarmupStore';
import { cn } from '../../../utils/cn';

export interface WarmupProbeEventPayload {
  email?: string;
  model?: string;
  index?: number;
  total?: number;
  duration_ms?: number;
  success?: boolean;
}

export interface WarmupInFlightTrackerProps {
  inFlight?: boolean;
  currentEmail?: string;
  currentModel?: string;
  processedCount?: number;
  totalCount?: number;
  alwaysVisible?: boolean;
  onCancel?: () => void;
  className?: string;
}

export const WarmupInFlightTracker: React.FC<WarmupInFlightTrackerProps> = ({
  inFlight: externalInFlight,
  currentEmail: propEmail,
  currentModel: propModel,
  processedCount: propProcessed,
  totalCount: propTotal,
  alwaysVisible = false,
  onCancel,
  className,
}) => {
  const { t } = useTranslation();

  const storeInFlightProbes = useWarmupStore((state) => state.inFlightProbes);
  const activeProbeList = Object.values(storeInFlightProbes);
  const firstStoreProbe = activeProbeList[0];

  const [internalInFlight, setInternalInFlight] = useState(false);
  const [activeEmail, setActiveEmail] = useState<string>(propEmail || 'dev@example.com');
  const [activeModel, setActiveModel] = useState<string>(propModel || 'gemini-2.5-pro');
  const [processed, setProcessed] = useState<number>(propProcessed || 0);
  const [total, setTotal] = useState<number>(propTotal || 1);
  const [elapsedMs, setElapsedMs] = useState<number>(0);

  const startTimeRef = useRef<number | null>(null);

  const isInFlight = externalInFlight ?? (internalInFlight || activeProbeList.length > 0);
  const displayEmail = propEmail || (firstStoreProbe ? firstStoreProbe.accountId : activeEmail);
  const displayModel = propModel || (firstStoreProbe ? firstStoreProbe.modelId : activeModel);
  const displayProcessed = propProcessed ?? processed;
  const displayTotal = Math.max(1, propTotal ?? total);

  // 100ms duration timer
  useEffect(() => {
    let timer: ReturnType<typeof setInterval> | null = null;
    if (isInFlight) {
      if (!startTimeRef.current) {
        startTimeRef.current = Date.now();
      }
      timer = setInterval(() => {
        if (startTimeRef.current) {
          setElapsedMs(Date.now() - startTimeRef.current);
        }
      }, 100);
    } else {
      startTimeRef.current = null;
      setElapsedMs(0);
    }

    return () => {
      if (timer) clearInterval(timer);
    };
  }, [isInFlight]);

  // Listen for Tauri warmup events
  useEffect(() => {
    let isMounted = true;
    const unlistens: UnlistenFn[] = [];

    const subscribeEvents = async () => {
      try {
        const unlistenStart = await listen<WarmupProbeEventPayload>('warmup://probe_start', (event) => {
          if (!isMounted) return;
          setInternalInFlight(true);
          if (event.payload.email) setActiveEmail(event.payload.email);
          if (event.payload.model) setActiveModel(event.payload.model);
          if (typeof event.payload.index === 'number') setProcessed(event.payload.index);
          if (typeof event.payload.total === 'number') setTotal(event.payload.total);
          startTimeRef.current = Date.now();
          setElapsedMs(0);
        });
        unlistens.push(unlistenStart);

        const unlistenComplete = await listen<WarmupProbeEventPayload>('warmup://probe_complete', (event) => {
          if (!isMounted) return;
          if (typeof event.payload.index === 'number') setProcessed(event.payload.index);
          if (typeof event.payload.total === 'number') setTotal(event.payload.total);
          if (
            event.payload.index !== undefined &&
            event.payload.total !== undefined &&
            event.payload.index >= event.payload.total
          ) {
            setInternalInFlight(false);
          }
        });
        unlistens.push(unlistenComplete);
      } catch (err) {
        useErrorStore.getState().trackWarning(err, {
          source: 'WarmupInFlightTracker.subscribeEvents',
          triggerAction: 'listen warmup://probe_start/complete',
        });
      }
    };

    subscribeEvents();

    return () => {
      isMounted = false;
      for (const unlisten of unlistens) {
        try {
          unlisten();
        } catch (e) {
          useErrorStore.getState().trackWarning(e, {
            source: 'WarmupInFlightTracker.unlisten',
            triggerAction: 'unlisten warmup events',
          });
        }
      }
    };
  }, []);

  if (!isInFlight && !alwaysVisible) {
    return null;
  }

  const progressPercent = Math.min(100, Math.round((displayProcessed / displayTotal) * 100));
  const formattedDuration = elapsedMs >= 1000 ? `${(elapsedMs / 1000).toFixed(1)}s` : `${elapsedMs}ms`;

  return (
    <div
      className={cn(
        'relative overflow-hidden bg-zinc-900/90 border border-orange-500/30 rounded-xl p-3 shadow-lg shadow-orange-500/5 transition-all duration-300',
        className
      )}
    >
      <style>{`
        @keyframes warmupRadarWave {
          0% {
            transform: scale(1);
            opacity: 0.6;
          }
          100% {
            transform: scale(2.4);
            opacity: 0;
          }
        }
        .animate-radar-wave {
          animation: warmupRadarWave 1.8s cubic-bezier(0, 0, 0.2, 1) infinite;
        }
      `}</style>

      <div className="flex items-center justify-between gap-3 mb-2.5">
        <div className="flex items-center gap-3">
          {/* Target Model Icon with Radiating Pulse Wave */}
          <div className="relative flex items-center justify-center w-8 h-8 rounded-lg bg-orange-500/10 border border-orange-500/30 text-orange-400 shrink-0">
            {isInFlight && (
              <>
                <span className="absolute inset-0 rounded-lg border border-orange-500/50 animate-radar-wave pointer-events-none" />
                <span
                  className="absolute inset-0 rounded-lg border border-orange-400/40 animate-radar-wave pointer-events-none"
                  style={{ animationDelay: '600ms' }}
                />
              </>
            )}
            <Cpu className="w-4 h-4 z-10" />
          </div>

          <div>
            <div className="flex items-center gap-2">
              <span className="text-xs font-semibold text-zinc-100 font-mono tracking-tight">
                {displayModel}
              </span>
              <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-orange-500/15 text-orange-300 font-mono font-medium">
                <Radio className="w-2.5 h-2.5 animate-pulse" />
                <span>{isInFlight ? t('settings.warmup.probing', 'PROBING') : t('settings.warmup.idle', 'IDLE')}</span>
              </span>
            </div>
            <p className="text-[11px] text-zinc-400 truncate max-w-[280px] sm:max-w-md">
              {displayEmail}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* 100ms Elapsed Duration Timer */}
          <div className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-zinc-800/80 border border-zinc-700/60 font-mono text-[11px] text-zinc-300">
            <Clock className="w-3 h-3 text-orange-400" />
            <span>{formattedDuration}</span>
          </div>

          {onCancel && (
            <button
              type="button"
              onClick={onCancel}
              className="p-1 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 rounded-md transition-colors"
              title={t('common.cancel', 'Cancel')}
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* Batch Progress Bar */}
      <div className="space-y-1.5 pt-1">
        <div className="flex items-center justify-between text-[10px] text-zinc-400 font-mono">
          <span>
            {t('settings.warmup.batch_progress', 'Processed {{current}} of {{total}} models', {
              current: displayProcessed,
              total: displayTotal,
            })}
          </span>
          <span className="font-semibold text-orange-400">{progressPercent}%</span>
        </div>

        <div className="h-1.5 w-full bg-zinc-800 rounded-full overflow-hidden">
          <div
            className="h-full bg-gradient-to-r from-orange-500 to-amber-400 transition-all duration-300 rounded-full"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      </div>
    </div>
  );
};

export default WarmupInFlightTracker;
