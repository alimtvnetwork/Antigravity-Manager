import type { ReactNode } from 'react';
import { WifiOff, Wifi } from 'lucide-react';
import { useNetworkConnectivity } from '../../../hooks/useNetworkConnectivity';

export interface NetworkConnectivityGateProps {
  children?: ReactNode;
  showWhenOnline?: boolean;
  className?: string;
}

/**
 * NetworkConnectivityGate guards warmup interactions against offline execution.
 * When offline, it displays an offline status banner or pill warning that warmup is suspended.
 */
export function NetworkConnectivityGate({
  children,
  showWhenOnline = false,
  className = '',
}: NetworkConnectivityGateProps) {
  const isOnline = useNetworkConnectivity();

  if (isOnline && !showWhenOnline) {
    return <>{children}</>;
  }

  if (isOnline && showWhenOnline) {
    return (
      <div className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 ${className}`}>
        <Wifi className="w-3.5 h-3.5" />
        <span>Network Connected</span>
      </div>
    );
  }

  return (
    <div
      className={`rounded-xl border border-zinc-800 bg-zinc-900/80 p-3.5 flex items-center justify-between gap-3 text-zinc-300 ${className}`}
      role="alert"
    >
      <div className="flex items-center gap-2.5">
        <div className="w-8 h-8 rounded-full bg-zinc-800 flex items-center justify-center text-zinc-400 border border-zinc-700">
          <WifiOff className="w-4 h-4" />
        </div>
        <div>
          <div className="text-xs font-semibold text-zinc-200">
            Offline Mode Active
          </div>
          <p className="text-[11px] text-zinc-400">
            Network probes and warmup scans are automatically suspended to prevent socket timeouts.
          </p>
        </div>
      </div>

      <span className="shrink-0 bg-zinc-800 text-zinc-400 border border-zinc-700 text-[10px] px-2 py-0.5 rounded-full flex items-center gap-1 font-mono">
        <WifiOff className="w-3 h-3 text-zinc-400" />
        Warmup Suspended
      </span>
    </div>
  );
}
