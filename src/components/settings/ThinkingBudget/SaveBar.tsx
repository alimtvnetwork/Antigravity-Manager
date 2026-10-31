import { BudgetSectionProps } from "./BudgetSectionProps";
import { Check, Save } from "lucide-react";

export function SaveBar(props: BudgetSectionProps) {
    const { t, isSaving, isSaved, handleSave, onSave } = props;

    return (
        <>
                {onSave && (
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3.5 border-t border-gray-200 dark:border-base-200">
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                            {t("proxy.config.thinking_budget.save_hint", {
                                defaultValue: "After modifying the thinking budget settings, click Save to take effect immediately, and the configuration will be hot-updated to the currently running anti-generation service.",
                            })}
                        </span>
                        <button
                            type="button"
                            disabled={isSaving}
                            onClick={handleSave}
                            className={`px-4 py-2 rounded-xl text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-all active:scale-95 shrink-0 cursor-pointer ${
                                isSaved
                                    ? "bg-emerald-600 hover:bg-emerald-500 text-white"
                                    : "bg-blue-600 hover:bg-blue-500 text-white"
                            } ${isSaving ? "opacity-75 cursor-wait" : ""}`}
                        >
                            {isSaved ? (
                                <>
                                    <Check size={14} className="text-white" />
                                    <span>{t("common.saved", { defaultValue: "Saved and effective" })}</span>
                                </>
                            ) : (
                                <>
                                    <Save size={14} className={isSaving ? "animate-spin" : ""} />
                                    <span>{isSaving ? t("common.saving", { defaultValue: "Saving..." }) : t("proxy.config.thinking_budget.save_btn", { defaultValue: "Save thinking settings and hot take effect" })}</span>
                                </>
                            )}
                        </button>
                    </div>
                )}

        </>
    );
}
