# Subtask 01: Warmup Dashboard & Telemetry Visualizers

**Subtask ID:** `232-01`  
**Parent Task:** `232-warm-up-ui-and-emailing-ui-improvements`  
**Status:** `DONE` (Cluster 1: Improvements 01–10 IMPLEMENTED; Cluster 2: Improvements 11–20 IMPLEMENTED)  
**Improvements Covered:** Improvements 01–20 (Clusters 1 & 2)  
**Target Specs:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/02-warmup-ui-improvements-spec.md`  

---

## Technical Scope & Deliverables
1. **Live Token Expenditure HUD (Improvement 01)**: Real-time counter in `SmartWarmup.tsx` tracking tokens consumed by warmup probes today, this week, and all-time.
2. **SVG Heartbeat Waveform (Improvement 02)**: Animated SVG pulse monitor rendering active 5-minute scheduler loop ticks.
3. **Model Latency Sparklines (Improvement 03)**: Micro sparklines displaying p50/p95 latency (ms) per model family.
4. **4-Tier Origin Traffic Badge (Improvement 04)**: Tag warmup requests with `WARMUP_PROBE` zinc badge in network inspector.
5. **Real-Time Execution Drawer (Improvement 05)**: Slide-over drawer displaying live HTTP status, headers, and duration.
6. **Weekly Reset Convergence Radar (Improvement 06)**: Visual dial showing percentage of accounts completing weekly reset cycles.
7. **Zero-Usage Token Shield (Improvement 07)**: Dark-glass verification callout proving warmups consume <= 2 output tokens.
8. **Dark-Glass Header Status Capsule (Improvement 08)**: Header pill capsule (`Active | Next Scan 3m 42s`).
9. **Live In-Flight Request Pulse Waves (Improvement 09)**: Progress tracker with radar pulse waves.
10. **Warmup Error Diagnostic Banner (Improvement 10)**: Error matching banner with one-click health fixes.
11. **Real-Time 7-Day Countdown Clocks (Improvement 11)**: Dynamic ticker (`T-04h:22m:15s`) until weekly quota reset.
12. **Synchronized Ticker Hook (Improvement 12)**: `useWarmupResetTimer` hook updating once per second without table re-renders.
13. **Cold-Start Indicator (Improvement 13)**: Cyan badge (`Cold-Start - Uninitialized`) for accounts with uninitialized weekly reset timers.
14. **Reset Window Glow (Improvement 14)**: Imminence orange border at T-30m (`WarmupImminenceGlow`).
15. **Dual-Layer Rolling vs. Weekly Reset Progress Bar (Improvement 15)**: Stacked 4h and 7d progress bars (`DualLayerQuotaProgressBar`).
16. **Time-Zone Normalized Clocks & UTC Tooltips (Improvement 16)**: Local and UTC timezone formatting (`timeFormat.ts`, `TimeZoneNormalizedClock`).
17. **Humanized Relative Reset Labels (Improvement 17)**: Human-friendly reset countdown labels (`HumanizedResetLabel`).
18. **Reset Completion Audio & Notifications (Improvement 18)**: Native desktop and Web Audio chime (`warmupNotificationService.ts`).
19. **Animated Quota Replenishment Wave Canvas (Improvement 19)**: 1200ms HTML5 Canvas wave effect (`QuotaReplenishmentWave`).
20. **Stale Quota Visual Warning & Auto-Sync Trigger (Improvement 20)**: Amber warning badge with auto-sync on focus (`StaleQuotaWarning`).

---

## Acceptance Criteria
- [x] Live token expenditure HUD displays verified probe tokens consumed.
- [x] 7-day countdown timers render on all account cards with tabular mono numbers.
- [x] `useWarmupResetTimer` maintains 1-second cadence without CPU thrash.
- [x] Design system tokens strictly follow 4-plane depth hierarchy and segmented pill capsules.
- [x] Fast-Forwarding Protocol maintained with zero intermediate compile/test overhead.
