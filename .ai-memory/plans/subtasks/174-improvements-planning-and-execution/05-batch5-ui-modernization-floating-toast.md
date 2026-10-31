# Subtask 05: Batch 5 (Imp 535–540: Desktop React UI Modernization, Accessibility & Feedback Primitives)

- **Parent Plan**: `.ai-memory/plans/pending/174-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/174-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-535: Accessible Collapsible Detail Row** (`src/components/common/CollapsibleDetailRow.tsx`)
   - Expandable table/card detail row with smooth height transition and accessible keyboard controls.
2. **Imp-536: Dark-Glass Floating Toast Card** (`src/components/common/FloatingNotificationCard.tsx`)
   - Dark-glass toast card with progress bar, dismiss button, and icon-coded severity styling.
3. **Imp-537: Sliding Window Telemetry Metrics Hook** (`src/hooks/useSlidingWindowMetrics.ts`)
   - React hook maintaining a fixed-size time-window buffer of latency/throughput metrics for dashboard sparklines.
4. **Imp-538: Segmented Number Stepper Input** (`src/components/common/CompactNumberStepper.tsx`)
   - Accessible segmented number stepper (`[-] [value] [+]`) with keyboard arrow increments and bounds clamping.
5. **Imp-539: Human-Readable Duration Formatter** (`src/utils/timeDurationFormatter.ts`)
   - Formats milliseconds into human-readable compact time units (`12ms`, `1.4s`, `3m 22s`, `4h 10m`).
6. **Imp-540: Glowing Health Pulse Status Indicator** (`src/components/common/HealthPulseIndicator.tsx`)
   - Animated heartbeat/pulse status dot with glowing ring effects and WCAG-compliant status labels.
