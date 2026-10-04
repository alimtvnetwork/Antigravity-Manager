import { useConfigStore } from '../../stores/useConfigStore';
import { THEME_PALETTES } from '../common/themePalettes';

export default function ThemePicker() {
    const { config, updateTheme } = useConfigStore();
    const current = config?.theme || 'system';

    return (
        <div className="space-y-3">
            <div>
                <h3 className="text-sm font-semibold text-gray-900 dark:text-base-content">Themes</h3>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Pick a catalogue theme. The choice is saved with the rest of the app settings.
                </p>
            </div>
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2">
                {THEME_PALETTES.map((palette) => {
                    const selected = current === palette.id;
                    return (
                        <button
                            key={palette.id}
                            type="button"
                            onClick={() => updateTheme(palette.id)}
                            className={`flex items-center gap-2 rounded-[5px] border px-2.5 py-2 text-left transition-colors ${selected
                                ? 'border-[var(--primary)] ring-1 ring-[var(--primary)]'
                                : 'border-gray-200 dark:border-base-300 hover:border-gray-400'
                                }`}
                            style={{ backgroundColor: palette.surface, color: palette.fg }}
                        >
                            <span
                                className="h-7 w-7 shrink-0 rounded-[5px] border border-white/10"
                                style={{ background: `linear-gradient(135deg, ${palette.bg} 50%, ${palette.primary} 50%)` }}
                            />
                            <span className="min-w-0">
                                <span className="block truncate text-xs font-semibold">{palette.label}</span>
                                <span className="block truncate text-[10px] opacity-70">{palette.primary}</span>
                            </span>
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
