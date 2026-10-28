import { useErrorStore } from '../stores/error-store';

export const DEFAULT_WARMUP_LEASE_TTL_SECONDS = 300;
const LEASE_STORAGE_PREFIX = 'agm_warmup_lease:';

export interface WarmupFleetLease {
  leaseKey: string;
  holderId: string;
  acquiredAt: number;
  ttlSeconds: number;
}

const inMemoryLeaseMap = new Map<string, WarmupFleetLease>();

export function buildWarmupLeaseKey(accountId: string, modelId: string): string {
  const cleanAccount = accountId.trim().toLowerCase();
  const cleanModel = modelId.trim().toLowerCase();
  return `${LEASE_STORAGE_PREFIX}${cleanAccount}:${cleanModel}`;
}

export function isWarmupLeaseExpired(lease: WarmupFleetLease, currentTime = Date.now()): boolean {
  const elapsedMs = currentTime - lease.acquiredAt;
  const isExpired = elapsedMs >= lease.ttlSeconds * 1000;
  return isExpired;
}

function parseStoredLease(raw: string): WarmupFleetLease | null {
  try {
    const parsed = JSON.parse(raw) as WarmupFleetLease;
    if (parsed && typeof parsed.acquiredAt === 'number' && parsed.holderId) {
      return parsed;
    }
  } catch (err: unknown) {
    useErrorStore.getState().trackWarning(err, {
      source: 'warmupFleetSync.parseStoredLease',
      triggerAction: 'parse_lease_json',
    });
  }
  return null;
}

function readLeaseRecord(leaseKey: string): WarmupFleetLease | null {
  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      const raw = window.localStorage.getItem(leaseKey);
      if (raw) return parseStoredLease(raw);
    } catch (err: unknown) {
      useErrorStore.getState().trackWarning(err, {
        source: 'warmupFleetSync.readLeaseRecord',
        triggerAction: 'read_local_storage',
      });
    }
  }
  return inMemoryLeaseMap.get(leaseKey) ?? null;
}

function writeLeaseRecord(leaseKey: string, lease: WarmupFleetLease): void {
  inMemoryLeaseMap.set(leaseKey, lease);
  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      window.localStorage.setItem(leaseKey, JSON.stringify(lease));
    } catch (err: unknown) {
      useErrorStore.getState().trackWarning(err, {
        source: 'warmupFleetSync.writeLeaseRecord',
        triggerAction: 'write_local_storage',
      });
    }
  }
}

function deleteLeaseRecord(leaseKey: string): void {
  inMemoryLeaseMap.delete(leaseKey);
  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      window.localStorage.removeItem(leaseKey);
    } catch (err: unknown) {
      useErrorStore.getState().trackWarning(err, {
        source: 'warmupFleetSync.deleteLeaseRecord',
        triggerAction: 'remove_local_storage',
      });
    }
  }
}

function canAcquireExistingLease(
  existingLease: WarmupFleetLease | null,
  instanceId: string,
  currentTime: number
): boolean {
  if (!existingLease) {
    return true;
  }
  if (isWarmupLeaseExpired(existingLease, currentTime)) {
    return true;
  }
  return existingLease.holderId === instanceId;
}

function createNewLease(
  leaseKey: string,
  holderId: string,
  now: number,
  ttlSeconds: number
): WarmupFleetLease {
  return { leaseKey, holderId, acquiredAt: now, ttlSeconds };
}

export async function acquireWarmupLease(
  accountId: string,
  modelId: string,
  instanceId: string,
  ttlSeconds = DEFAULT_WARMUP_LEASE_TTL_SECONDS
): Promise<boolean> {
  const leaseKey = buildWarmupLeaseKey(accountId, modelId);
  const now = Date.now();
  const existing = readLeaseRecord(leaseKey);

  if (!canAcquireExistingLease(existing, instanceId, now)) {
    return false;
  }
  writeLeaseRecord(leaseKey, createNewLease(leaseKey, instanceId, now, ttlSeconds));
  return true;
}

export async function releaseWarmupLease(
  accountId: string,
  modelId: string
): Promise<void> {
  const leaseKey = buildWarmupLeaseKey(accountId, modelId);
  deleteLeaseRecord(leaseKey);
}

export async function getWarmupFleetLease(
  accountId: string,
  modelId: string
): Promise<WarmupFleetLease | null> {
  const leaseKey = buildWarmupLeaseKey(accountId, modelId);
  const lease = readLeaseRecord(leaseKey);
  if (lease && isWarmupLeaseExpired(lease)) {
    deleteLeaseRecord(leaseKey);
    return null;
  }
  return lease;
}

function findLeaseStorageKeys(): string[] {
  const keys: string[] = [];
  if (typeof window === 'undefined' || !window.localStorage) return keys;
  for (let i = 0; i < window.localStorage.length; i++) {
    const key = window.localStorage.key(i);
    if (key?.startsWith(LEASE_STORAGE_PREFIX)) {
      keys.push(key);
    }
  }
  return keys;
}

function removeStorageKeys(keys: string[]): void {
  try {
    keys.forEach((k) => window.localStorage.removeItem(k));
  } catch (err: unknown) {
    useErrorStore.getState().trackWarning(err, {
      source: 'warmupFleetSync.removeStorageKeys',
      triggerAction: 'clear_storage_leases',
    });
  }
}

export async function clearAllWarmupLeases(): Promise<void> {
  inMemoryLeaseMap.clear();
  const keys = findLeaseStorageKeys();
  removeStorageKeys(keys);
}
