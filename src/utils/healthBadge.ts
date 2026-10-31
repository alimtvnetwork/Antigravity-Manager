/**
 * System Diagnostic Health Badge Resolver
 * Maps doctor and connectivity diagnostic states to high-contrast badge styles and status labels.
 */

export interface HealthBadgeInfo {
  status: 'ok' | 'warn' | 'error' | 'unknown';
  label: string;
  badgeClass: string;
  dotColor: string;
}

export function resolveHealthBadge(status: string): HealthBadgeInfo {
  const s = status.trim().toLowerCase();

  if (s === 'ok' || s === 'healthy' || s === 'passed' || s === 'ready') {
    return {
      status: 'ok',
      label: 'Operational',
      badgeClass: 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20',
      dotColor: '#10b981',
    };
  }

  if (s === 'warn' || s === 'warning' || s === 'degraded' || s === 'caution') {
    return {
      status: 'warn',
      label: 'Warning',
      badgeClass: 'bg-amber-500/10 text-amber-400 border border-amber-500/20',
      dotColor: '#f59e0b',
    };
  }

  if (s === 'error' || s === 'failed' || s === 'critical' || s === 'down') {
    return {
      status: 'error',
      label: 'Critical',
      badgeClass: 'bg-rose-500/10 text-rose-400 border border-rose-500/20',
      dotColor: '#f43f5e',
    };
  }

  return {
    status: 'unknown',
    label: 'Unknown',
    badgeClass: 'bg-slate-500/10 text-slate-400 border border-slate-500/20',
    dotColor: '#94a3b8',
  };
}
