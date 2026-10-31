/**
 * Quota Health Score Categorizer
 * Maps numerical quota percentages (0-100%) to standardized UI health bands, colors, and badge classes.
 */

export type QuotaHealthBand = 'pristine' | 'healthy' | 'caution' | 'depleted';

export interface QuotaHealthInfo {
  band: QuotaHealthBand;
  label: string;
  badgeClass: string;
  dotColor: string;
}

export function categorizeQuotaHealth(quotaPercent: number): QuotaHealthInfo {
  const q = Math.max(0, Math.min(100, quotaPercent));

  if (q >= 100) {
    return {
      band: 'pristine',
      label: 'Pristine (100%)',
      badgeClass: 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20',
      dotColor: '#10b981',
    };
  }

  if (q >= 50) {
    return {
      band: 'healthy',
      label: `Healthy (${Math.round(q)}%)`,
      badgeClass: 'bg-sky-500/10 text-sky-400 border border-sky-500/20',
      dotColor: '#0ea5e9',
    };
  }

  if (q >= 20) {
    return {
      band: 'caution',
      label: `Caution (${Math.round(q)}%)`,
      badgeClass: 'bg-amber-500/10 text-amber-400 border border-amber-500/20',
      dotColor: '#f59e0b',
    };
  }

  return {
    band: 'depleted',
    label: `Depleted (${Math.round(q)}%)`,
    badgeClass: 'bg-rose-500/10 text-rose-400 border border-rose-500/20',
    dotColor: '#f43f5e',
  };
}
