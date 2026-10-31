import { useState, useCallback } from 'react';

export interface MetricSample {
  value: number;
  timestamp: number;
}

export interface SlidingWindowStats {
  samples: MetricSample[];
  average: number;
  min: number;
  max: number;
  latest: number;
}

export function useSlidingWindowMetrics(windowCapacity: number = 20) {
  const [samples, setSamples] = useState<MetricSample[]>([]);

  const addSample = useCallback(
    (value: number) => {
      setSamples((prev) => {
        const next = [...prev, { value, timestamp: Date.now() }];
        if (next.length > windowCapacity) {
          return next.slice(next.length - windowCapacity);
        }
        return next;
      });
    },
    [windowCapacity]
  );

  const clearSamples = useCallback(() => {
    setSamples([]);
  }, []);

  const computeStats = (): SlidingWindowStats => {
    if (samples.length === 0) {
      return {
        samples: [],
        average: 0,
        min: 0,
        max: 0,
        latest: 0,
      };
    }

    const values = samples.map((s) => s.value);
    const sum = values.reduce((a, b) => a + b, 0);
    const average = sum / values.length;
    const min = Math.min(...values);
    const max = Math.max(...values);
    const latest = values[values.length - 1];

    return {
      samples,
      average,
      min,
      max,
      latest,
    };
  };

  return {
    samples,
    addSample,
    clearSamples,
    stats: computeStats(),
  };
}
