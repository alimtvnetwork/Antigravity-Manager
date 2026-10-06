export interface ThemePalette {
    id: string;
    label: string;
    bg: string;
    surface: string;
    primary: string;
    fg: string;
    dark: boolean;
    surfaceHover?: string;
    primaryHover?: string;
    borderHover?: string;
}

/** Catalogue colors from 02-spec/07-design-system/16-theme-catalogue-and-palettes.md. */
export const THEME_PALETTES: ThemePalette[] = [
    { id: 'light', label: 'Light Clean', bg: '#F8FAFC', surface: '#FFFFFF', primary: '#2563EB', fg: '#0F172A', dark: false, surfaceHover: '#F1F5F9', primaryHover: '#1D4ED8', borderHover: '#3B82F6' },
    { id: 'dark', label: 'Antigravity Dark', bg: '#071A27', surface: '#0E2433', primary: '#38BDF8', fg: '#E5E7EB', dark: true, surfaceHover: '#15334D', primaryHover: '#0EA5E9', borderHover: '#38BDF8' },
    { id: 'system', label: 'System Automatic', bg: '#071A27', surface: '#0E2433', primary: '#38BDF8', fg: '#E5E7EB', dark: true, surfaceHover: '#15334D', primaryHover: '#0EA5E9', borderHover: '#38BDF8' },
    { id: 'vscode-dark', label: 'VS Code Dark+', bg: '#1E1E1E', surface: '#252526', primary: '#007ACC', fg: '#D4D4D4', dark: true, surfaceHover: '#2A2D2E', primaryHover: '#0062A3', borderHover: '#007ACC' },
    { id: 'tokyo-night', label: 'Tokyo Night Slate', bg: '#1A1B26', surface: '#24283B', primary: '#7AA2F7', fg: '#C0CAF5', dark: true, surfaceHover: '#2F354D', primaryHover: '#628CF0', borderHover: '#7AA2F7' },
    { id: 'one-dark', label: 'One Dark Pro', bg: '#21252B', surface: '#282C34', primary: '#61AFEF', fg: '#ABB2BF', dark: true, surfaceHover: '#353B45', primaryHover: '#4D9CE6', borderHover: '#61AFEF' },
    { id: 'github-dark', label: 'GitHub Dark HC', bg: '#0D1117', surface: '#161B22', primary: '#58A6FF', fg: '#F0F6FC', dark: true, surfaceHover: '#21262D', primaryHover: '#388BFD', borderHover: '#58A6FF' },
    { id: 'obsidian', label: 'Obsidian Cyan', bg: '#0B0F19', surface: '#111827', primary: '#06B6D4', fg: '#F3F4F6', dark: true, surfaceHover: '#1F2937', primaryHover: '#0891B2', borderHover: '#06B6D4' },
    { id: 'mint-terminal', label: 'Nordic Mint', bg: '#0B1614', surface: '#122320', primary: '#10B981', fg: '#ECFDF5', dark: true, surfaceHover: '#1B332F', primaryHover: '#059669', borderHover: '#10B981' },
    { id: 'green-choice', label: 'Green Choice Light', bg: '#F4F8F5', surface: '#FFFFFF', primary: '#16A34A', fg: '#13201B', dark: false, surfaceHover: '#E8F5E9', primaryHover: '#15803D', borderHover: '#16A34A' },
    { id: 'green-choice-dark', label: 'Green Choice Dark', bg: '#0D1713', surface: '#14231E', primary: '#22C55E', fg: '#F0FDF4', dark: true, surfaceHover: '#1E332B', primaryHover: '#16A34A', borderHover: '#22C55E' },
    { id: 'clean-wide', label: 'Clean Studio Light', bg: '#FFFFFF', surface: '#F8FAFC', primary: '#0284C7', fg: '#0F172A', dark: false, surfaceHover: '#F1F5F9', primaryHover: '#0369A1', borderHover: '#0284C7' },
    { id: 'cobalt', label: 'Cobalt Blue', bg: '#0B1528', surface: '#13223E', primary: '#3B82F6', fg: '#F1F5F9', dark: true, surfaceHover: '#1E3258', primaryHover: '#2563EB', borderHover: '#3B82F6' },
    { id: 'vscode-navy-gold', label: 'Navy Amber', bg: '#0A121E', surface: '#111F33', primary: '#F59E0B', fg: '#F8FAFC', dark: true, surfaceHover: '#1A2C47', primaryHover: '#D97706', borderHover: '#F59E0B' },
];

export function findPalette(id: string): ThemePalette {
    return THEME_PALETTES.find((item) => item.id === id) ?? THEME_PALETTES[1];
}
