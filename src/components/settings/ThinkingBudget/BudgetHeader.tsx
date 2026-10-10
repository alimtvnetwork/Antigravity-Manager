import { BudgetSectionProps } from "./BudgetSectionProps";

export function BudgetHeader(props: BudgetSectionProps) {
    const { t } = props;

    return (
        <>
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-gray-200 dark:border-base-200 pb-2.5">
                    <div>
                        <h4 className="text-xs font-bold text-gray-900 dark:text-white flex items-center gap-1.5">
                            <svg
                                className="w-4 h-4 text-purple-500 dark:text-purple-400"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth="2"
                                    d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
                                />
                            </svg>
                            {t("proxy.config.thinking_budget.title", {
                                defaultValue: "思考链预算控制 (Thinking Budget)",
                            })}
                        </h4>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            {t("proxy.config.thinking_budget.description", {
                                defaultValue:
                                    "跨协议归一化思考参数，统一调控 Gemini 与 Claude 模型的深度思考 Token 预算及自适应行为。",
                            })}
                        </p>
                    </div>
                </div>

        </>
    );
}
