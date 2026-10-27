import React, { useMemo } from 'react';
import { Sparkles, Gauge, Layers } from 'lucide-react';
import { useWarmupStore, LatencyTelemetryRecord } from '../../../stores/useWarmupStore';

interface LatencyStats {
  p50: number;
  p95: number;
  min: number;
  max: number;
  count: number;
}

interface SparklinePoint {
  x: number;
  y: number;
  durationMs: number;
}

const SPARK_WIDTH = 96; // w-24
const SPARK_HEIGHT = 24; // h-6
const PAD_X = 4;
const PAD_Y = 3;

/**
 * Calculates rolling p50 and p95 latency distributions in milliseconds.
 */
function computeLatencyStats(records: LatencyTelemetryRecord[]): LatencyStats {
  if (records.length === 0) {
    return { p50: 0, p95: 0, min: 0, max: 0, count: 0 };
  }

  const sorted = records.map((r) => r.durationMs).sort((a, b) => a - b);
  const p50 = sorted[Math.floor(sorted.length * 0.5)];
  const p95 = sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * 0.95))];
  const min = sorted[0];
  const max = sorted[sorted.length - 1];

  return { p50, p95, min, max, count: sorted.length };
}

interface FamilySparklineRowProps {
  label: string;
  family: 'gemini' | 'claude';
  records: LatencyTelemetryRecord[];
  strokeClass: string;
  fillClass: string;
  badgeClass: string;
}

/**
 * Renders an inline dual polyline SVG sparkline with min/max indicator dots and metric pills.
 */
const FamilySparklineRow: React.FC<FamilySparklineRowProps> = ({
  label,
  records,
  strokeClass,
  fillClass,
  badgeClass,
}) => {
  const stats = useMemo(() => computeLatencyStats(records), [records]);

  // Map latency records to SVG coordinates
  const { polylinePoints, areaPoints, minPoint, maxPoint } = useMemo(() => {
    if (records.length === 0) {
      return { polylinePoints: '', areaPoints: '', minPoint: null, maxPoint: null };
    }

    const range = Math.max(1, stats.max - stats.min);
    const usableWidth = SPARK_WIDTH - 2 * PAD_X;
    const usableHeight = SPARK_HEIGHT - 2 * PAD_Y;

    const points: SparklinePoint[] = records.map((r, i) => {
      const x = PAD_X + (records.length === 1 ? usableWidth / 2 : (i / (records.length - 1)) * usableWidth);
      const normalized = (r.durationMs - stats.min) / range;
      const y = SPARK_HEIGHT - PAD_Y - normalized * usableHeight;
      return { x, y, durationMs: r.durationMs };
    });

    const poly = points.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');
    const area = `${points[0].x.toFixed(1)},${SPARK_HEIGHT} ${poly} ${points[points.length - 1].x.toFixed(1)},${SPARK_HEIGHT}`;

    let minP = points[0];
    let maxP = points[0];
    for (const p of points) {
      if (p.durationMs < minP.durationMs) minP = p;
      if (p.durationMs > maxP.durationMs) maxP = p;
    }

    return {
      polylinePoints: poly,
      areaPoints: area,
      minPoint: minP,
      maxPoint: maxP,
    };
  }, [records, stats]);

  return (
    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 p-2.5 rounded-xl bg-zinc-950/40 border border-zinc-800/50">
      {/* Model Family Label Badge */}
      <div className="flex items-center gap-2 min-w-[100px]">
        <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full border ${badgeClass}`}>
          {label}
        </span>
      </div>

      {/* Sparkline Canvas w-24 h-6 */}
      <div className="flex items-center gap-3">
        <div className="w-24 h-6 inline-block align-middle bg-zinc-900/60 rounded border border-zinc-800/60 overflow-hidden relative">
          <svg viewBox={`0 0 ${SPARK_WIDTH} ${SPARK_HEIGHT}`} className="w-full h-full">
            {records.length === 0 ? (
              // Graceful empty state: dashed placeholder baseline
              <line
                x1="2"
                y1={SPARK_HEIGHT / 2}
                x2={SPARK_WIDTH - 2}
                y2={SPARK_HEIGHT / 2}
                className="stroke-zinc-700 stroke-[1.5]"
                strokeDasharray="3,3"
              />
            ) : (
              <>
                {/* Area under sparkline */}
                <polygon points={areaPoints} className={fillClass} />

                {/* Polyline */}
                <polyline
                  points={polylinePoints}
                  fill="none"
                  className={`${strokeClass} stroke-[1.8]`}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />

                {/* Min indicator dot (green/cyan) */}
                {minPoint && (
                  <circle
                    cx={minPoint.x}
                    cy={minPoint.y}
                    r="2"
                    className="fill-emerald-400 stroke-zinc-900 stroke-[0.5]"
                  />
                )}

                {/* Max indicator dot (amber/red) */}
                {maxPoint && (
                  <circle
                    cx={maxPoint.x}
                    cy={maxPoint.y}
                    r="2"
                    className="fill-rose-400 stroke-zinc-900 stroke-[0.5]"
                  />
                )}
              </>
            )}
          </svg>
        </div>

        {/* Metric Pills */}
        <div className="flex items-center gap-1.5 flex-wrap">
          <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-[10px] font-mono text-zinc-300">
            <span className="text-zinc-500">p50:</span>
            <strong className="text-zinc-100">{stats.p50}ms</strong>
          </div>

          <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-[10px] font-mono text-zinc-300">
            <span className="text-zinc-500">p95:</span>
            <strong className="text-amber-300">{stats.p95}ms</strong>
          </div>

          <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-[10px] font-mono text-zinc-400">
            <span>{stats.count} probes</span>
          </div>
        </div>
      </div>
    </div>
  );
};

/**
 * Improvement 03: Model Latency Sparklines (Gemini vs. Claude p50/p95)
 * Ingests rolling 50 records and visualizes latency trends with dual inline SVG sparklines.
 */
export const ModelLatencySparklines: React.FC = () => {
  const latencyHistory = useWarmupStore((state) => state.latencyHistory);

  // Group the last 50 records by model family
  const { geminiRecords, claudeRecords } = useMemo(() => {
    const gemini: LatencyTelemetryRecord[] = [];
    const claude: LatencyTelemetryRecord[] = [];

    const recent = latencyHistory.slice(-50);
    for (const rec of recent) {
      if (rec.family === 'claude') {
        claude.push(rec);
      } else {
        gemini.push(rec);
      }
    }

    return { geminiRecords: gemini, claudeRecords: claude };
  }, [latencyHistory]);

  return (
    <div className="bg-zinc-900/70 backdrop-blur-md border border-zinc-800/80 rounded-2xl p-4 shadow-xl text-zinc-100 transition-all duration-300">
      {/* Header */}
      <div className="flex items-center justify-between pb-3 mb-3 border-b border-zinc-800/60">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400">
            <Gauge size={16} />
          </div>
          <div>
            <div className="text-xs font-bold text-zinc-200 tracking-wide flex items-center gap-2">
              Model Latency Sparklines
              <span className="bg-zinc-800/80 text-zinc-400 border border-zinc-700/60 text-[9px] px-2 py-0.5 rounded-full font-mono">
                p50 / p95 (Last 50)
              </span>
            </div>
            <p className="text-[10px] text-zinc-400 mt-0.5">
              Probe roundtrip duration comparison between Google Gemini and Anthropic Claude endpoints.
            </p>
          </div>
        </div>
      </div>

      {/* Dual Sparkline Rows */}
      <div className="space-y-2.5">
        {/* Gemini Sparkline */}
        <FamilySparklineRow
          label="Google Gemini"
          family="gemini"
          records={geminiRecords}
          strokeClass="stroke-sky-400"
          fillClass="fill-sky-400/10"
          badgeClass="bg-sky-500/10 text-sky-400 border-sky-500/20"
        />

        {/* Claude Sparkline */}
        <FamilySparklineRow
          label="Anthropic Claude"
          family="claude"
          records={claudeRecords}
          strokeClass="stroke-amber-400"
          fillClass="fill-amber-400/10"
          badgeClass="bg-amber-500/10 text-amber-400 border-amber-500/20"
        />
      </div>
    </div>
  );
};

export default ModelLatencySparklines;
