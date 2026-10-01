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
