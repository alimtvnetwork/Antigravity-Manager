import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export interface UiTraceEvent {
  step: string;
  action: string;
  target: string;
  metadata?: Record<string, unknown>;
}

/**
 * Dispatches UI step and click telemetry events to the Tauri backend for audit logging
 * and console debug traces.
 */
export function logUiTrace(event: UiTraceEvent): void {
  if (typeof console !== 'undefined' && console.debug) {
    console.debug(`⚡ [UI_TRACE] [${event.step}] ${event.action} -> ${event.target}`, event.metadata ?? '');
  }

  const isTauri =
    typeof window !== 'undefined' &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

  if (!isTauri) {
    return;
  }

  invoke('record_ui_trace', {
    step: event.step,
    action: event.action,
    target: event.target,
    metadata: event.metadata,
  }).catch((e) => {
    useErrorStore.getState().trackWarning(e, {
      source: 'uiLogger.logUiTrace',
      triggerAction: event.action,
    });
  });
}
