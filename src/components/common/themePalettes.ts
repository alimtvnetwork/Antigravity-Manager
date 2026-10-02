export interface ThemePalette {
    id: string;
    label: string;
    bg: string;
    surface: string;
    primary: string;
    fg: string;
    dark: boolean;
}

/** Catalogue colors from 02-spec/07-design-system/16-theme-catalogue-and-palettes.md. */
export const THEME_PALETTES: ThemePalette[] = [
    { id: 'light', label: 'Light', bg: '#F5FAF9', surface: '#FFFFFF', primary: '#2563EB', fg: '#111827', dark: false },
    { id: 'dark', label: 'Dark', bg: '#071A27', surface: '#0E2433', primary: '#60A5FA', fg: '#E5E7EB', dark: true },
    { id: 'system', label: 'System', bg: '#071A27', surface: '#0E2433', primary: '#60A5FA', fg: '#E5E7EB', dark: true },
    { id: 'midnight', label: 'Midnight Cyber', bg: '#0A0F1D', surface: '#172036', primary: '#A855F7', fg: '#F8FAFC', dark: true },
    { id: 'vscode-dark', label: 'VS Code Dark+', bg: '#1E1E1E', surface: '#2D2D2D', primary: '#007ACC', fg: '#D4D4D4', dark: true },
    { id: 'tokyo-night', label: 'Tokyo Night', bg: '#1F2335', surface: '#292E42', primary: '#7AA2F7', fg: '#C0CAF5', dark: true },
    { id: 'one-dark', label: 'One Dark', bg: '#282C34', surface: '#2E3440', primary: '#61AFEF', fg: '#ABB2BF', dark: true },
    { id: 'github-dark', label: 'GitHub Dark HC', bg: '#0A0C10', surface: '#1C212C', primary: '#79C0FF', fg: '#FFFFFF', dark: true },
    { id: 'monokai', label: 'Monokai Pro', bg: '#2D2A2E', surface: '#3A363B', primary: '#FF6188', fg: '#FCFCFA', dark: true },
    { id: 'editorial', label: 'Warm Editorial', bg: '#0B0A09', surface: '#1D1A18', primary: '#FFAD01', fg: '#F7F5F2', dark: true },
];

export function findPalette(id: string): ThemePalette {
    return THEME_PALETTES.find((item) => item.id === id) ?? THEME_PALETTES[1];
}
