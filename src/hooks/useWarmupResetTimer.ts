import { useSyncExternalStore } from 'react';

export type TimerUrgency = 'normal' | 'approaching' | 'imminent' | 'expired';

export interface TimerResult {
  formatted: string;
  rawSeconds: number;
  urgency: TimerUrgency;
  isExpired: boolean;
}

let nowEpochSec = Math.floor(Date.now() / 1000);
const subscribers = new Set<() => void>();
let masterIntervalId: ReturnType<typeof setInterval> | null = null;
let isThrottled = false;

function notifySubscribers(): void {
  nowEpochSec = Math.floor(Date.now() / 1000);
  subscribers.forEach((callback) => callback());
}

function updateMasterInterval(): void {
  if (masterIntervalId !== null) {
    clearInterval(masterIntervalId);
  }
  const delayMs = isThrottled ? 5000 : 1000;
  masterIntervalId = setInterval(notifySubscribers, delayMs);
}

function handleVisibilityChange(): void {
  const isHidden = typeof document !== 'undefined' && document.hidden;
  if (isHidden === isThrottled) return;
  isThrottled = isHidden;
  notifySubscribers();
  updateMasterInterval();
}

function startMasterTicker(): void {
  isThrottled = typeof document !== 'undefined' && document.hidden;
  notifySubscribers();
  updateMasterInterval();
  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', handleVisibilityChange);
  }
}

function stopMasterTicker(): void {
  if (masterIntervalId !== null) {
    clearInterval(masterIntervalId);
    masterIntervalId = null;
  }
  if (typeof document !== 'undefined') {
    document.removeEventListener('visibilitychange', handleVisibilityChange);
  }
}

function subscribe(callback: () => void): () => void {
  subscribers.add(callback);
  if (subscribers.size === 1) {
    startMasterTicker();
  }
  return () => {
    subscribers.delete(callback);
    if (subscribers.size === 0) {
      stopMasterTicker();
    }
  };
}

function getSnapshot(): number {
  return nowEpochSec;
}

function parseTargetSeconds(target: number | string | null | undefined): number | null {
  if (target === null || target === undefined || target === '') {
    return null;
  }
  if (typeof target === 'number') {
    if (!Number.isFinite(target) || target <= 0) return null;
    return target > 10000000000 ? Math.floor(target / 1000) : Math.floor(target);
  }
  const trimmed = target.trim();
  const numeric = Number(trimmed);
  if (!Number.isNaN(numeric) && numeric > 0) {
    return numeric > 10000000000 ? Math.floor(numeric / 1000) : Math.floor(numeric);
  }
  const parsedMs = Date.parse(trimmed);
  return Number.isNaN(parsedMs) || parsedMs <= 0 ? null : Math.floor(parsedMs / 1000);
}

function determineUrgency(remainingSeconds: number): TimerUrgency {
  if (remainingSeconds <= 0) return 'expired';
  if (remainingSeconds <= 1800) return 'imminent';
  if (remainingSeconds <= 7200) return 'approaching';
  return 'normal';
}

function formatCountdown(rawSeconds: number): string {
  if (rawSeconds <= 0) return 'RESET DUE';
  if (rawSeconds > 86400) {
    const days = Math.floor(rawSeconds / 86400);
    const rem = rawSeconds % 86400;
    const dd = String(days).padStart(2, '0');
    const hh = String(Math.floor(rem / 3600)).padStart(2, '0');
    const mm = String(Math.floor((rem % 3600) / 60)).padStart(2, '0');
    const ss = String(rem % 60).padStart(2, '0');
    return `T-${dd}d:${hh}h:${mm}m:${ss}s`;
  }
  const hh = String(Math.floor(rawSeconds / 3600)).padStart(2, '0');
  const mm = String(Math.floor((rawSeconds % 3600) / 60)).padStart(2, '0');
  const ss = String(rawSeconds % 60).padStart(2, '0');
  return `T-${hh}h:${mm}m:${ss}s`;
}

function calculateTimerResult(
  target: number | string | null | undefined,
  nowSec: number
): TimerResult {
  const targetSec = parseTargetSeconds(target);
  if (targetSec === null) {
    return { formatted: '--:--:--', rawSeconds: 0, urgency: 'normal', isExpired: false };
  }
  const remainingSeconds = targetSec - nowSec;
  return {
    formatted: formatCountdown(remainingSeconds),
    rawSeconds: Math.max(0, remainingSeconds),
    urgency: determineUrgency(remainingSeconds),
    isExpired: remainingSeconds <= 0,
  };
}

export function useWarmupResetTimer(
  targetEpochSecs: number | string | null | undefined
): TimerResult {
  const currentEpochSec = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  return calculateTimerResult(targetEpochSecs, currentEpochSec);
}
