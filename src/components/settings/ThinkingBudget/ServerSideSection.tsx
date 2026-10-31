import { BudgetSectionProps } from "./BudgetSectionProps";

export function ServerSideSection(props: BudgetSectionProps) {
    const { t, thinkingStoreEnabled, onThinkingStoreChange } = props;

    return (
        <>
                {onThinkingStoreChange && (
                    <div className="p-3.5 bg-purple-50/70 dark:bg-purple-900/20 border border-purple-200/80 dark:border-purple-800/40 rounded-xl flex items-center justify-between gap-4 shadow-2xs">
                        <div className="space-y-0.5">
                            <div className="flex items-center gap-2">
                                <span className="text-xs font-bold text-gray-900 dark:text-white">
                                    {t("proxy.config.thinking_budget.store_enabled", { defaultValue: "Server-side think blocks and signature backfilling" })}
                                </span>
                                <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300 border border-purple-200 dark:border-purple-800/60">
                                    {t("proxy.config.thinking_budget.default_on_tag", { defaultValue: "Enabled by default" })}
                                </span>
                            </div>
                            <p className="text-xs text-gray-600 dark:text-gray-300 max-w-xl leading-relaxed">
                                {t("proxy.config.thinking_budget.store_enabled_desc", {
                                    defaultValue: "Automatically intercept and persist the thinking process and encrypted signature; when the business agent (such as Cline, Roo, Claude Code, etc.) fails to bring back the thinking after multiple rounds of dialogue, the server automatically completes it, eliminating upstream 400 errors or context loss."
                                })}
                            </p>
                        </div>
                        <input
                            type="checkbox"
                            className="toggle toggle-sm toggle-primary shrink-0"
                            checked={thinkingStoreEnabled}
                            onChange={(e) => onThinkingStoreChange(e.target.checked)}
                        />
                    </div>
                )}

        </>
    );
}
