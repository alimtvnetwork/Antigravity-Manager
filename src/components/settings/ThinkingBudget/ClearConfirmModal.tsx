import { AlertTriangle, Trash2 } from "lucide-react";
import { BudgetSectionProps } from "./BudgetSectionProps";

export function ClearConfirmModal(props: BudgetSectionProps) {
    const { t, isClearingThinking, showClearThinkingConfirm, setShowClearThinkingConfirm, handleClearThinkingStore } = props;

    return (
        <>
                {showClearThinkingConfirm && (
                    <div className="modal modal-open">
                        <div className="modal-box max-w-md bg-white dark:bg-base-100 border border-base-300 shadow-2xl p-5">
                            <div className="flex items-start gap-3">
                                <div className="p-2 rounded-full bg-error/10 text-error shrink-0 mt-0.5">
                                    <AlertTriangle size={20} />
                                </div>
                                <div className="space-y-2 min-w-0">
                                    <h3 className="text-sm font-bold text-gray-900 dark:text-white">
                                        {t("proxy.config.thinking_budget.clear_modal_title", {
                                            defaultValue: "Confirm to clear thinking block storage?",
                                        })}
                                    </h3>
                                    <p className="text-xs text-gray-600 dark:text-gray-300 leading-relaxed whitespace-pre-line">
                                        {t("proxy.config.thinking_budget.clear_modal_desc", {
                                            defaultValue:
                                                "This operation will completely clear all historical thought chains and tool signature records in the resident memory (RAM) and local SQLite database. \n\n⚠️ Note: This operation only cleans the thinking block data and will never delete any reverse proxy request logs.",
                                        })}
                                    </p>
                                    <div className="p-2.5 rounded-lg bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 text-[11px] text-amber-800 dark:text-amber-300 leading-relaxed">
                                        {t("proxy.config.thinking_budget.clear_modal_warning", {
                                            defaultValue:
                                                "It is recommended that this be executed only when a cache hit exception occurs, when a version upgrade occurs, or when explicitly requested by the developer. New requests after clearing will re-establish a clean prefix index.",
                                        })}
                                    </div>
                                </div>
                            </div>

                            <div className="modal-action mt-5 flex justify-end gap-2">
                                <button
                                    type="button"
                                    className="btn btn-sm btn-ghost"
                                    onClick={() => setShowClearThinkingConfirm(false)}
                                    disabled={isClearingThinking}
                                >
                                    {t("common.cancel", { defaultValue: "Cancel" })}
                                </button>
                                <button
                                    type="button"
                                    className="btn btn-sm btn-error text-white gap-1.5"
                                    onClick={handleClearThinkingStore}
                                    disabled={isClearingThinking}
                                >
                                    {isClearingThinking ? (
                                        <span className="loading loading-spinner loading-xs" />
                                    ) : (
                                        <Trash2 size={13} />
                                    )}
                                    {t("proxy.config.thinking_budget.clear_confirm_btn", {
                                        defaultValue: "Confirm clearing",
                                    })}
                                </button>
                            </div>
                        </div>
                        <div
                            className="modal-backdrop bg-black/40"
                            onClick={() => !isClearingThinking && setShowClearThinkingConfirm(false)}
                        />
                    </div>
                )}
        </>
    );
}
