/**
 * Dark-Glass Segmented Button Capsule Helper
 * Standardized Tailwind classes for contiguous pill capsules, eliminating disjointed white pill borders.
 */

export const CAPSULE_WRAPPER_CLASS =
  'inline-flex items-center rounded-full bg-slate-900/90 border border-slate-700/60 p-0.5 shadow-xs divide-x divide-slate-800/80 backdrop-blur-md';

export const CAPSULE_SEGMENT_ACTIVE_CLASS =
  'inline-flex items-center justify-center rounded-full bg-slate-800 text-slate-100 shadow-xs px-3 py-1 text-xs font-medium transition-colors';

export const CAPSULE_SEGMENT_INACTIVE_CLASS =
  'inline-flex items-center justify-center rounded-full text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 px-3 py-1 text-xs font-medium transition-colors';

export const CAPSULE_BORDERLESS_SELECT_CLASS =
  'bg-transparent border-0 focus:ring-0 focus:outline-none text-xs text-slate-300 pr-2 cursor-pointer appearance-none';

export function getCapsuleSegmentClass(isActive: boolean): string {
  return isActive ? CAPSULE_SEGMENT_ACTIVE_CLASS : CAPSULE_SEGMENT_INACTIVE_CLASS;
}
