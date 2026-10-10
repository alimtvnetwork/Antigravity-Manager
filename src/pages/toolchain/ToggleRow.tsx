export function ToggleRow(props: {
    label: string;
    hint: string;
    checked: boolean;
    onChange: (v: boolean) => void;
}) {
    return (
        <label className="flex items-center gap-3 rounded-xl border border-gray-200 dark:border-white/10 px-3 py-2.5 cursor-pointer">
            <button
                type="button"
                role="switch"
                aria-checked={props.checked}
                onClick={(e) => {
                    e.preventDefault();
                    props.onChange(!props.checked);
                }}
                className={`relative w-10 h-6 rounded-full transition-colors shrink-0 cursor-pointer ${
                    props.checked ? 'bg-indigo-600' : 'bg-gray-200 dark:bg-white/15'
                }`}
            >
                <span
                    className={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-all ${
                        props.checked ? 'left-[18px]' : 'left-0.5'
                    }`}
                />
            </button>
            <span className="flex-1">
                <span className="block text-sm font-medium text-gray-900 dark:text-gray-100">
                    {props.label}
                </span>
                <span className="block text-xs text-gray-500 dark:text-gray-400">
                    {props.hint}
                </span>
            </span>
        </label>
    );
}

