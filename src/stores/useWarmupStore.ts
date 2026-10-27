import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { useErrorStore } from './error-store';

/**
 * Telemetry summary representing aggregate warmup token consumption
 * and probe execution metrics.
 */
export interface WarmupTelemetryState {
  totalTokensConsumed: number;
  sessionTokensConsumed: number;
  avgTokensPerProbe: number;
  totalProbesExecuted: number;
  estimatedCostUsd: number;
  lastResetEpochMs: number;
}

/**
 * Individual probe latency execution record for tracking response distributions.
 */
export interface LatencyTelemetryRecord {
  modelId: string;
  family: 'gemini' | 'claude';
  durationMs: number;
  timestamp: number;
  httpStatus: number;
}

/**
 * Heartbeat waveform sample point for the SVG ECG display.
 */
export interface HeartbeatPoint {
  timestamp: number;
  amplitude: number;
  isProbe: boolean;
}

/**
 * In-flight probe descriptor during execution.
 */
export interface InFlightProbeInfo {
  id: string;
  accountId: string;
  modelId: string;
  startTime: number;
  status: 'in-flight' | 'success' | 'failed';
}

export interface ProbeOutcomePayload {
  modelId: string;
  family?: 'gemini' | 'claude';
  durationMs: number;
  tokensConsumed?: number;
  httpStatus?: number;
}

export interface WarmupStoreState {
  telemetry: WarmupTelemetryState;
  latencyHistory: LatencyTelemetryRecord[];
  heartbeatPoints: HeartbeatPoint[];
  isActive: boolean;
  nextScanSeconds: number;
  eligibleCount: number;
  inFlightProbes: Record<string, InFlightProbeInfo>;

  // Actions
  recordProbeOutcome: (payload: ProbeOutcomePayload) => void;
  tickHeartbeat: (point?: { amplitude?: number; isProbe?: boolean }) => void;
  setScanCountdown: (seconds: number) => void;
  setIsActive: (active: boolean) => void;
  setEligibleCount: (count: number) => void;
  resetSessionTokens: () => void;
  resetAllMetrics: () => void;
  addInFlightProbe: (id: string, info: InFlightProbeInfo) => void;
  removeInFlightProbe: (id: string) => void;
}

const DEFAULT_HEARTBEAT_POINT_COUNT = 32;
const PRO_TOKEN_COST_USD = 0.00000015;

/**
 * Generates initial flat baseline points for the ECG waveform.
 */
function createInitialHeartbeatPoints(): HeartbeatPoint[] {
  const now = Date.now();
  const points: HeartbeatPoint[] = [];
  for (let i = DEFAULT_HEARTBEAT_POINT_COUNT - 1; i >= 0; i--) {
    points.push({
      timestamp: now - i * 3000,
      amplitude: 0.1,
      isProbe: false,
    });
  }
  return points;
}

const INITIAL_TELEMETRY: WarmupTelemetryState = {
  totalTokensConsumed: 0,
  sessionTokensConsumed: 0,
  avgTokensPerProbe: 0,
  totalProbesExecuted: 0,
  estimatedCostUsd: 0,
  lastResetEpochMs: Date.now(),
};

/**
 * Zustand store for Smart Warmup HUD, telemetry ring buffers, and heartbeat status.
 */
export const useWarmupStore = create<WarmupStoreState>()(
  persist(
    (set, get) => ({
      telemetry: INITIAL_TELEMETRY,
      latencyHistory: [],
      heartbeatPoints: createInitialHeartbeatPoints(),
      isActive: true,
      nextScanSeconds: 300,
      eligibleCount: 0,
      inFlightProbes: {},

      recordProbeOutcome: ({
        modelId,
        family,
        durationMs,
        tokensConsumed = 1,
        httpStatus = 200,
      }: ProbeOutcomePayload) => {
        try {
          const resolvedFamily: 'gemini' | 'claude' =
            family ?? (modelId.toLowerCase().includes('claude') ? 'claude' : 'gemini');

          set((state) => {
            const totalTokens = state.telemetry.totalTokensConsumed + tokensConsumed;
            const sessionTokens = state.telemetry.sessionTokensConsumed + tokensConsumed;
            const totalProbes = state.telemetry.totalProbesExecuted + 1;
            const avgTokens = totalProbes > 0 ? Number((totalTokens / totalProbes).toFixed(2)) : 0;
            const estimatedCostUsd = Number((totalTokens * PRO_TOKEN_COST_USD).toFixed(6));

            const newLatencyRecord: LatencyTelemetryRecord = {
              modelId,
              family: resolvedFamily,
              durationMs: Math.max(0, durationMs),
              timestamp: Date.now(),
              httpStatus,
            };

            const updatedLatencyHistory = [...state.latencyHistory, newLatencyRecord].slice(-50);

            const probePoint: HeartbeatPoint = {
              timestamp: Date.now(),
              amplitude: 1.0,
              isProbe: true,
            };
            const updatedHeartbeat = [...state.heartbeatPoints.slice(1), probePoint];

            return {
              telemetry: {
                ...state.telemetry,
                totalTokensConsumed: totalTokens,
                sessionTokensConsumed: sessionTokens,
                avgTokensPerProbe: avgTokens,
                totalProbesExecuted: totalProbes,
                estimatedCostUsd,
              },
              latencyHistory: updatedLatencyHistory,
              heartbeatPoints: updatedHeartbeat,
            };
          });
        } catch (err) {
          useErrorStore.getState().trackWarning(err instanceof Error ? err : new Error(String(err)), {
            source: 'useWarmupStore.recordProbeOutcome',
            triggerAction: 'recordProbeOutcome',
          });
        }
      },

      tickHeartbeat: (point) => {
        set((state) => {
          const isProbe = point?.isProbe ?? false;
          const defaultAmp = state.isActive ? 0.2 : 0.05;
          const amplitude = point?.amplitude ?? (isProbe ? 1.0 : defaultAmp);

          const newPoint: HeartbeatPoint = {
            timestamp: Date.now(),
            amplitude,
            isProbe,
          };

          return {
            heartbeatPoints: [...state.heartbeatPoints.slice(1), newPoint],
          };
        });
      },

      setScanCountdown: (seconds: number) => {
        set({ nextScanSeconds: Math.max(0, Math.floor(seconds)) });
      },

      setIsActive: (active: boolean) => {
        set({ isActive: active });
      },

      setEligibleCount: (count: number) => {
        set({ eligibleCount: Math.max(0, count) });
      },

      resetSessionTokens: () => {
        set((state) => ({
          telemetry: {
            ...state.telemetry,
            sessionTokensConsumed: 0,
          },
        }));
      },

      resetAllMetrics: () => {
        set({
          telemetry: {
            ...INITIAL_TELEMETRY,
            lastResetEpochMs: Date.now(),
          },
          latencyHistory: [],
          heartbeatPoints: createInitialHeartbeatPoints(),
        });
      },

      addInFlightProbe: (id: string, info: InFlightProbeInfo) => {
        set((state) => ({
          inFlightProbes: { ...state.inFlightProbes, [id]: info },
        }));
      },

      removeInFlightProbe: (id: string) => {
        set((state) => {
          const next = { ...state.inFlightProbes };
          delete next[id];
          return { inFlightProbes: next };
        });
      },
    }),
    {
      name: 'agm_warmup_telemetry_store',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        telemetry: {
          ...state.telemetry,
          sessionTokensConsumed: 0,
        },
        latencyHistory: state.latencyHistory.slice(-50),
      }),
    }
  )
);
