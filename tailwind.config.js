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
                    blue: '#7C3AED',     // Royal Violet (Primary Brand Anchor)
                    green: '#31C9A4',    // Success
                    cyan: '#A855F7',     // Aurora (Bridge / Glow)
                    mint: '#E879F9',     // Highlight (Luminosity Highlight)
                    dark: '#090611',     // Canvas (Dark Background)
                    card: '#171025',     // Panel Surface
                    border: '#40285E',   // Violet Border
                    slate: '#B8ACCB',    // Muted (Supporting Neutral)
                    light: '#F5F0FF',    // Text (Light)
                },
            },
        },
    },
    plugins: [daisyui, containerQueries],
    daisyui: {
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
                    "primary": "#7C3AED",   // Royal Violet primary
                    "secondary": "#A855F7", // Aurora
                    "accent": "#E879F9",    // Highlight
                    "neutral": "#10091D",   // Navigation
                    "base-100": "#090611",  // Canvas
                    "base-200": "#171025",  // Panel
                    "base-300": "#211631",  // Surface 2
                    "info": "#A855F7",      // Aurora
                    "success": "#31C9A4",
                    "warning": "#F5BF58",
                    "error": "#EF6A8A",
                },
            },
        ],
        darkTheme: "dark",
    },
}
