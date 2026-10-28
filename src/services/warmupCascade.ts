import { useErrorStore } from '../stores/error-store';

export const DEFAULT_WARMUP_CASCADE_LADDER: readonly string[] = [
  'gemini-2.5-pro',
  'gemini-2.5-flash',
  'gemini-2.0-flash',
] as const;

export interface CascadeStepResult {
  model: string;
  success: boolean;
  status: number;
  durationMs: number;
  error?: string;
}

export interface CascadeExecutionReport {
  accountId: string;
  initialModel: string;
  finalModel: string;
  isSuccess: boolean;
  totalAttempts: number;
  steps: CascadeStepResult[];
}

export function isProbeSuccess(status: number): boolean {
  const isOkStatus = status >= 200 && status < 300;
  return isOkStatus;
}

export function getNextFallbackModel(
  currentModel: string,
  ladder: readonly string[] = DEFAULT_WARMUP_CASCADE_LADDER
): string | null {
  const currentIndex = ladder.indexOf(currentModel);
  if (currentIndex >= 0 && currentIndex < ladder.length - 1) {
    return ladder[currentIndex + 1];
  }
  if (currentIndex === -1 && ladder.length > 0) {
    const fallbackTarget = ladder[0] === currentModel ? (ladder[1] ?? null) : ladder[0];
    return fallbackTarget;
  }
  return null;
}

function recordCascadeWarning(
  accountId: string,
  model: string,
  nextModel: string | null,
  error: unknown
): void {
  const actionName = nextModel ? `fallback_to_${nextModel}` : 'cascade_exhausted';
  useErrorStore.getState().trackWarning(error, {
    source: 'warmupCascade.executeModelCascadeWithRetry',
    triggerAction: actionName,
    metadata: { accountId, failedModel: model, nextModel },
  });
}

function buildStepResult(
  model: string,
  success: boolean,
  status: number,
  durationMs: number,
  error?: string
): CascadeStepResult {
  return {
    model,
    success,
    status,
    durationMs,
    ...(error ? { error } : {}),
  };
}

async function runSingleProbeStep(
  model: string,
  probeFn: (model: string) => Promise<{ status: number; durationMs: number }>
): Promise<CascadeStepResult> {
  const startTime = Date.now();
  try {
    const res = await probeFn(model);
    const durationMs = res.durationMs || Date.now() - startTime;
    const success = isProbeSuccess(res.status);
    return buildStepResult(model, success, res.status, durationMs);
  } catch (err: unknown) {
    const durationMs = Date.now() - startTime;
    const errText = err instanceof Error ? err.message : String(err);
    return buildStepResult(model, false, 500, durationMs, errText);
  }
}

function createExecutionReport(
  accountId: string,
  initialModel: string,
  steps: CascadeStepResult[]
): CascadeExecutionReport {
  const lastStep = steps[steps.length - 1];
  const finalModel = lastStep ? lastStep.model : initialModel;
  const isSuccess = lastStep ? lastStep.success : false;
  return {
    accountId,
    initialModel,
    finalModel,
    isSuccess,
    totalAttempts: steps.length,
    steps,
  };
}

export async function executeModelCascadeWithRetry(
  accountId: string,
  primaryModel: string,
  probeFn: (model: string) => Promise<{ status: number; durationMs: number }>
): Promise<CascadeExecutionReport> {
  const steps: CascadeStepResult[] = [];
  let currentModel: string | null = primaryModel;

  while (currentModel) {
    const step = await runSingleProbeStep(currentModel, probeFn);
    steps.push(step);
    if (step.success) {
      break;
    }
    const nextModel = getNextFallbackModel(currentModel);
    recordCascadeWarning(accountId, currentModel, nextModel, step.error || `HTTP ${step.status}`);
    currentModel = nextModel;
  }

  return createExecutionReport(accountId, primaryModel, steps);
}
