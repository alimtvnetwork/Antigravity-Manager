/**
 * Real-time Quota Burn Velocity Meter
 * Computes hourly consumption rate (%/hr) and projected exhaustion time based on recent quota samples.
 */

export interface QuotaBurnPoint {
  timestamp: number; // Unix seconds
  quotaPercent: number; // 0 - 100
}

export type QuotaTrend = 'steep' | 'moderate' | 'stable' | 'recovering';

export interface QuotaVelocityResult {
  burnRatePerHour: number;
  estimatedHoursRemaining: number | null;
  isDepletingFast: boolean;
  trend: QuotaTrend;
}

export function calculateQuotaVelocity(samples: QuotaBurnPoint[]): QuotaVelocityResult {
  if (samples.length < 2) {
    return {
      burnRatePerHour: 0,
      estimatedHoursRemaining: null,
      isDepletingFast: false,
      trend: 'stable',
    };
  }

  // Sort chronologically
  const sorted = [...samples].sort((a, b) => a.timestamp - b.timestamp);
  const oldest = sorted[0];
  const newest = sorted[sorted.length - 1];

  const timeDeltaHours = (newest.timestamp - oldest.timestamp) / 3600;
  if (timeDeltaHours <= 0) {
    return {
      burnRatePerHour: 0,
      estimatedHoursRemaining: null,
      isDepletingFast: false,
      trend: 'stable',
    };
  }

  const quotaDelta = oldest.quotaPercent - newest.quotaPercent; // Positive if consuming
  const burnRatePerHour = Math.max(0, quotaDelta / timeDeltaHours);

  let estimatedHoursRemaining: number | null = null;
  if (burnRatePerHour > 0.1 && newest.quotaPercent > 0) {
    estimatedHoursRemaining = Math.max(0, newest.quotaPercent / burnRatePerHour);
  }

  let trend: QuotaTrend = 'stable';
  if (quotaDelta < -1) {
    trend = 'recovering'; // Quota increased (reset cycle)
  } else if (burnRatePerHour > 25) {
    trend = 'steep';
  } else if (burnRatePerHour > 5) {
    trend = 'moderate';
  }

  const isDepletingFast = burnRatePerHour >= 20;

  return {
    burnRatePerHour: Math.round(burnRatePerHour * 10) / 10,
    estimatedHoursRemaining: estimatedHoursRemaining ? Math.round(estimatedHoursRemaining * 10) / 10 : null,
    isDepletingFast,
    trend,
  };
}
