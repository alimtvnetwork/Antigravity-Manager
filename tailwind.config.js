import daisyui from "daisyui";
import containerQueries from "@tailwindcss/container-queries";

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    darkMode: 'class',
    theme: {
        extend: {
            fontFamily: {
                sans: ['Ubuntu', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
                mono: ['Ubuntu Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
                ubuntu: ['Ubuntu', 'sans-serif'],
                'ubuntu-mono': ['Ubuntu Mono', 'monospace'],
            },
            colors: {
                awan: {
                    blue: '#2878f0',     // Lift Blue (Main Brand Anchor)
                    green: '#16a97a',    // Vector Green (Automation Success)
                    cyan: '#19b7c9',     // Zero-G Cyan (Bridge / Glow)
                    mint: '#43d6a2',     // Plasma Mint (Luminosity Highlight)
                    dark: '#071a27',     // Deep Orbit (Dark Background)
                    card: '#0c2438',     // Deep Orbit Card Surface
                    border: '#15334d',   // Deep Orbit Border
                    slate: '#5d6b76',    // Slate Vector (Supporting Neutral)
                    light: '#f5faf9',    // Cloud Field (Light Background)
                },
                // RiseUp Asia brand twins (hex only; WebView2 has no oklch path).
                // READ-ONLY cite: riseup-asia-website-project/spec/05-brand-system.md §1–§1.2.
                rise: {
                    amber: '#FFAD01',           // Brand amber — the one accent
                    'amber-hover': '#FFC24A',   // Amber hover lift (dark)
                    'amber-ink': '#8A5A00',     // Amber as text on paper (AA)
                    transition: '#FF9346',      // Amber→orange mark transition
                    violet: '#791AB0',          // Depth accent (gradients/glows only)
                    'violet-soft': '#9B3FD1',   // Gradient stop
                    red: '#C72D2D',             // Errors / comparison negatives
                    success: '#33A96B',         // Confirmations only
                    bg: '#0B0A09',              // Warm-black page base
                    raised: '#151312',          // Section alternation
                    surface: '#1D1A18',         // Cards
                    'surface-hover': '#262220', // Card hover
                    fg: '#F7F5F2',              // Warm white text
                    'fg-muted': '#ADA8A2',      // Body secondary
                    'fg-subtle': '#7A746F',     // Labels, captions
                    paper: '#FBF9F6',           // Warm paper (light base)
                    'paper-raised': '#F4F0EA',  // Light alternate bands
                    'paper-surface': '#FFFFFF', // Light cards
                    'paper-fg': '#141210',      // Light primary text
                    'paper-muted': '#57514B',   // Light body secondary
                },
                // Opacity-capable daisyUI aliases (see ui-toolchain-alpha.md).
                // daisyUI v5 under Tailwind v3 registers theme colors as bare
                // `var(--color-*)`, so `/NN` opacity modifiers silently emit
                // nothing. These user-config overrides win over the plugin and
                // re-express each slot as color-mix with <alpha-value>, which
                // Tailwind substitutes per utility (1 for solids). WebView2
                // (Chromium 111+) supports color-mix; values still resolve
                // from the live --color-* vars per data-theme (incl. RiseUp).
                primary: 'color-mix(in srgb, var(--color-primary) calc(<alpha-value> * 100%), transparent)',
                'primary-content': 'color-mix(in srgb, var(--color-primary-content) calc(<alpha-value> * 100%), transparent)',
                secondary: 'color-mix(in srgb, var(--color-secondary) calc(<alpha-value> * 100%), transparent)',
                'secondary-content': 'color-mix(in srgb, var(--color-secondary-content) calc(<alpha-value> * 100%), transparent)',
                accent: 'color-mix(in srgb, var(--color-accent) calc(<alpha-value> * 100%), transparent)',
                'accent-content': 'color-mix(in srgb, var(--color-accent-content) calc(<alpha-value> * 100%), transparent)',
                neutral: 'color-mix(in srgb, var(--color-neutral) calc(<alpha-value> * 100%), transparent)',
                'neutral-content': 'color-mix(in srgb, var(--color-neutral-content) calc(<alpha-value> * 100%), transparent)',
                info: 'color-mix(in srgb, var(--color-info) calc(<alpha-value> * 100%), transparent)',
                'info-content': 'color-mix(in srgb, var(--color-info-content) calc(<alpha-value> * 100%), transparent)',
                success: 'color-mix(in srgb, var(--color-success) calc(<alpha-value> * 100%), transparent)',
                'success-content': 'color-mix(in srgb, var(--color-success-content) calc(<alpha-value> * 100%), transparent)',
                warning: 'color-mix(in srgb, var(--color-warning) calc(<alpha-value> * 100%), transparent)',
                'warning-content': 'color-mix(in srgb, var(--color-warning-content) calc(<alpha-value> * 100%), transparent)',
                error: 'color-mix(in srgb, var(--color-error) calc(<alpha-value> * 100%), transparent)',
                'error-content': 'color-mix(in srgb, var(--color-error-content) calc(<alpha-value> * 100%), transparent)',
                'base-100': 'color-mix(in srgb, var(--color-base-100) calc(<alpha-value> * 100%), transparent)',
                'base-200': 'color-mix(in srgb, var(--color-base-200) calc(<alpha-value> * 100%), transparent)',
                'base-300': 'color-mix(in srgb, var(--color-base-300) calc(<alpha-value> * 100%), transparent)',
                'base-content': 'color-mix(in srgb, var(--color-base-content) calc(<alpha-value> * 100%), transparent)',
            },
        },
    },
    plugins: [daisyui, containerQueries],
    daisyui: {
        // NOTE (daisyUI v5): the plugin reads themes ONLY from its built-in
        // theme/object.js — objects in `themes` below are dead config (verified
        // against daisyui@5.7.47 index.js/pluginOptionsHandler.js). Live custom
        // themes ([data-theme="riseup-dark"/"riseup-paper"]) are defined as
        // --color-* var blocks in src/App.css. Kept untouched for history.
        themes: [
            {
                light: {
                    "primary": "#2878f0",
                    "secondary": "#5d6b76",
                    "accent": "#16a97a",
                    "neutral": "#071a27",
                    "base-100": "#ffffff",
                    "base-200": "#f5faf9",
                    "base-300": "#e2e8f0",
                    "info": "#19b7c9",
                    "success": "#16a97a",
                    "warning": "#f59e0b",
                    "error": "#ef4444",
                },
            },
            {
                dark: {
                    "primary": "#2878f0",
                    "secondary": "#19b7c9",
                    "accent": "#16a97a",
                    "neutral": "#071a27",
                    "base-100": "#071a27", // Deep Orbit Dark
                    "base-200": "#0c2438", // Deep Orbit Card
                    "base-300": "#15334d", // Deep Orbit Border / Component
                    "info": "#19b7c9",     // Zero-G Cyan
                    "success": "#16a97a",  // Vector Green
                    "warning": "#f59e0b",
                    "error": "#ef4444",
                },
            },
        ],
        darkTheme: "dark",
    },
}
