import React, { useEffect, useMemo, useState } from 'react';
import { Activity, Radio } from 'lucide-react';
import { useWarmupStore, HeartbeatPoint } from '../../../stores/useWarmupStore';
import { isTauri } from '../../../utils/env';
import { useErrorStore } from '../../../stores/error-store';

interface Point2D {
  x: number;
  y: number;
}

interface WarmupHeartbeatPayload {
  timestamp: number;
  scan_active: boolean;
  queue_depth: number;
  next_scan_secs: number;
}

const SVG_WIDTH = 176; // w-44
const SVG_HEIGHT = 32; // h-8
const BASELINE_Y = 16;

/**
 * Constructs a smooth cubic Bezier path from a sequence of 2D coordinates.
 */
function buildCubicBezierPath(pts: Point2D[]): string {
  if (pts.length === 0) return '';
  if (pts.length === 1) return `M ${pts[0].x.toFixed(1)},${pts[0].y.toFixed(1)}`;

  let path = `M ${pts[0].x.toFixed(1)},${pts[0].y.toFixed(1)}`;
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)];
    const p1 = pts[i];
    const p2 = pts[i + 1];
    const p3 = pts[Math.min(pts.length - 1, i + 2)];

    const cp1x = p1.x + (p2.x - p0.x) / 6;
    const cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6;
    const cp2y = p2.y - (p3.y - p1.y) / 6;

    path += ` C ${cp1x.toFixed(1)},${cp1y.toFixed(1)} ${cp2x.toFixed(1)},${cp2y.toFixed(1)} ${p2.x.toFixed(1)},${p2.y.toFixed(1)}`;
  }
  return path;
}

/**
 * Improvement 02: SVG Heartbeat Activity Waveform & Loop Ticker
 * Renders 32-point ECG-style line with cubic Bezier smoothing, orange glow,
 * and background scan countdown ticker.
 */
export const WarmupHeartbeatWaveform: React.FC = () => {
  const heartbeatPoints = useWarmupStore((state) => state.heartbeatPoints);
  const isActive = useWarmupStore((state) => state.isActive);
  const nextScanSeconds = useWarmupStore((state) => state.nextScanSeconds);
  const setScanCountdown = useWarmupStore((state) => state.setScanCountdown);
  const setIsActive = useWarmupStore((state) => state.setIsActive);
  const tickHeartbeat = useWarmupStore((state) => state.tickHeartbeat);

  const [localTick, setLocalTick] = useState(0);

  // Subscribe to Tauri heartbeat events if running inside desktop app
  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<WarmupHeartbeatPayload>('warmup://heartbeat', (event) => {
          if (!event.payload) return;
          setIsActive(event.payload.scan_active);
          setScanCountdown(event.payload.next_scan_secs);
          tickHeartbeat({
            amplitude: event.payload.queue_depth > 0 ? 1.0 : 0.25,
            isProbe: event.payload.queue_depth > 0,
          });
        });
      } catch (err) {
        useErrorStore.getState().trackWarning(err instanceof Error ? err : new Error(String(err)), {
          source: 'WarmupHeartbeatWaveform.listen',
          triggerAction: 'warmup://heartbeat',
        });
      }
    })();

    return () => {
      if (unlisten) unlisten();
    };
  }, [setIsActive, setScanCountdown, tickHeartbeat]);

  // Local fallback countdown ticker for responsive UI animation
  useEffect(() => {
    const timer = setInterval(() => {
      setLocalTick((prev) => prev + 1);
      if (isActive && nextScanSeconds > 0) {
        setScanCountdown(nextScanSeconds - 1);
      } else if (isActive && nextScanSeconds <= 0) {
        setScanCountdown(300);
        tickHeartbeat({ amplitude: 0.35, isProbe: false });
      }
    }, 1000);

    return () => clearInterval(timer);
  }, [isActive, nextScanSeconds, setScanCountdown, tickHeartbeat]);

  // Transform 32 points into 2D SVG coordinates
  const svgPath = useMemo(() => {
    const count = Math.max(1, heartbeatPoints.length);
    const coords: Point2D[] = heartbeatPoints.map((pt, idx) => {
      const x = (idx / (count - 1)) * SVG_WIDTH;

      if (!isActive) {
        return { x, y: BASELINE_Y };
      }

      if (pt.isProbe) {
        // High amplitude ECG spike for probe burst
        const peakY = BASELINE_Y - Math.min(1.0, pt.amplitude) * 13;
        return { x, y: peakY };
      }

      // Gentle ambient oscillation for idle heartbeat
      const waveOffset = Math.sin((idx + localTick * 0.2) * 0.8) * 3 * pt.amplitude;
      return { x, y: BASELINE_Y - waveOffset };
    });

    return buildCubicBezierPath(coords);
  }, [heartbeatPoints, isActive, localTick]);

  const strokeColorClass = isActive
    ? 'stroke-orange-500/80 dark:stroke-orange-400'
    : 'stroke-zinc-700';

  return (
    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-zinc-900/60 backdrop-blur-md border border-zinc-800/80 rounded-xl p-3 shadow-lg">
      {/* Title & Loop Ticker Info */}
      <div className="flex items-center gap-2.5">
        <div
          className={`w-7 h-7 rounded-lg border flex items-center justify-center transition-colors ${
            isActive
              ? 'bg-orange-500/10 border-orange-500/20 text-orange-400'
              : 'bg-zinc-800/50 border-zinc-700/50 text-zinc-500'
          }`}
        >
          {isActive ? <Radio size={14} className="animate-pulse" /> : <Activity size={14} />}
        </div>
        <div>
          <div className="flex items-center gap-1.5">
            <span className="text-[11px] font-bold text-zinc-200 uppercase tracking-wider">
              Scheduler Loop
            </span>
            <span
              className={`text-[9px] px-1.5 py-0.2 rounded-full font-mono font-semibold ${
                isActive
                  ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30'
                  : 'bg-zinc-800 text-zinc-400 border border-zinc-700'
              }`}
            >
              {isActive ? 'LIVE' : 'HALTED'}
            </span>
          </div>
          <div className="text-[10px] font-mono text-zinc-400 font-medium tracking-wider mt-0.5">
            {isActive ? `Next scan: in ${nextScanSeconds}s` : 'Background scheduler paused'}
          </div>
        </div>
      </div>

      {/* ECG SVG Waveform Container */}
      <div className="h-8 w-44 bg-zinc-950/60 rounded-lg border border-zinc-800/60 overflow-hidden flex items-center px-1">
        <svg
          viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}
          className="w-full h-full overflow-visible"
        >
          <defs>
            <filter id="heartbeat-glow" x="-20%" y="-20%" width="140%" height="140%">
              <feDropShadow
                dx="0"
                dy="0"
                stdDeviation="2"
                floodColor="rgba(249, 115, 22, 0.4)"
              />
            </filter>
          </defs>

          {/* Baseline guide line */}
          <line
            x1="0"
            y1={BASELINE_Y}
            x2={SVG_WIDTH}
            y2={BASELINE_Y}
            className="stroke-zinc-800/60 stroke-[1]"
            strokeDasharray="2,2"
          />

          {/* Dynamic 32-point ECG wave path */}
          <path
            d={svgPath}
            fill="none"
            className={`${strokeColorClass} stroke-2 transition-colors duration-200`}
            style={{
              filter: isActive ? 'drop-shadow(0 0 4px rgba(249, 115, 22, 0.4))' : undefined,
            }}
          />
        </svg>
      </div>
    </div>
  );
};

export default WarmupHeartbeatWaveform;
