/**
 * Virtualized Row Index Range Calculator
 * Computes visible slice boundaries and top padding offsets for virtual scrolling containers.
 */

export interface VirtualRangeOptions {
  totalCount: number;
  scrollTop: number;
  viewportHeight: number;
  rowHeight: number;
  overscan?: number;
}

export interface VirtualRangeResult {
  startIndex: number;
  endIndex: number;
  offsetY: number;
  totalHeight: number;
}

export function calculateVirtualRange({
  totalCount,
  scrollTop,
  viewportHeight,
  rowHeight,
  overscan = 3,
}: VirtualRangeOptions): VirtualRangeResult {
  const totalHeight = totalCount * rowHeight;
  if (totalCount === 0 || rowHeight <= 0) {
    return { startIndex: 0, endIndex: 0, offsetY: 0, totalHeight: 0 };
  }

  const rawStartIndex = Math.floor(scrollTop / rowHeight);
  const visibleCount = Math.ceil(viewportHeight / rowHeight);

  const startIndex = Math.max(0, rawStartIndex - overscan);
  const endIndex = Math.min(totalCount, rawStartIndex + visibleCount + overscan);

  const offsetY = startIndex * rowHeight;

  return {
    startIndex,
    endIndex,
    offsetY,
    totalHeight,
  };
}
