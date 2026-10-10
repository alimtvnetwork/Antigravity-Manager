import { AutoSwitcherApi } from './useAutoSwitcher';

export function ManualRotation(props: AutoSwitcherApi) {
    const { rotationFeedback, isRotating, handleManualRotate } = props;

    return (
        <>
                        <div className="pt-2 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-blue-50/50 dark:bg-blue-950/20 p-3.5 rounded-xl border border-blue-200/60 dark:border-blue-900/40 shadow-2xs">
                            <div className="flex items-center gap-2.5 text-xs text-slate-600 dark:text-slate-400">
                                <div className="p-1 rounded-md bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400">
                                    <ArrowRightLeft className="w-3.5 h-3.5" />
                                </div>
                                <div>
                                    <span className="font-semibold text-slate-900 dark:text-slate-100">Test Failover:</span>{' '}
                                    Manually rotate the IDE to the next best profile candidate right now.
                                </div>
                            </div>
                            <button
                                type="button"
                                onClick={handleManualRotate}
                                disabled={isRotating}
                                className="btn btn-xs btn-primary gap-1.5 shadow-xs shrink-0 cursor-pointer"
                            >
                                <Play size={12} className={isRotating ? 'animate-spin' : ''} />
                                <span>{isRotating ? 'Rotating...' : 'Rotate Now'}</span>
                            </button>
                        </div>

                        {rotationFeedback ? (
                            <div className="p-2.5 rounded-lg bg-slate-100 dark:bg-slate-900 border border-slate-200 dark:border-slate-700/80 text-xs flex items-center gap-2 text-slate-800 dark:text-slate-200">
                                {rotationFeedback.startsWith('Error') ? (
                                    <AlertCircle size={14} className="text-rose-500 shrink-0" />
                                ) : (
                                    <CheckCircle2 size={14} className="text-emerald-500 shrink-0" />
                                )}
                                <span className="truncate">{rotationFeedback}</span>
                            </div>
                    ) : null}
        </>
    );
}
