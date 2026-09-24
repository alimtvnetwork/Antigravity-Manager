# Subtask 01: Settings UX Tab Consolidation & Header Debug Integration

## Objective
Implement clean, concise, non-intrusive Settings menu tabs and buttons strictly adhering to user design annotations:
1. Shorten tab names: "Proxy Settings" -> "Proxy", "Email & Alerts" -> "Email-Alerts", "Supabase Sync" -> "Supabase".
2. Shorten action button labels: "Save Settings" -> "Save", "Backup & Restore" -> "Backup".
3. Remove redundant "Debug" tab from the main settings tab list, eliminating clutter and directly directing all debug operations to the top-level Bug icon in the application title bar.
4. Align English (`en.json`) and Chinese (`zh.json`) dictionary files with these concise labels.
5. Provide a convenient Debug Console quick-access card inside the Advanced tab for power users.

## Status
- [x] Verified `src/pages/Settings.tsx` tab list renders: General, Account, Proxy, Email-Alerts, Supabase, Advanced, About.
- [x] Verified `src/locales/en.json` and `src/locales/zh.json` labels match.
- [x] Verified header Bug icon in `Navbar.tsx` seamlessly toggles debug overlay across all application routes.
