# Subtask: Eliminate Scale Shifts & Standardize CSS3 Transitions

**Target Files:**
- `src/components/navbar/InstanceSelector.tsx`
- `src/components/navbar/NavMenu.tsx`
- `src/components/navbar/NavSettings.tsx`
- `src/components/settings/EmailNotificationSettings.tsx`
- `src/components/accounts/AccountRow.tsx`

**Action:**
1. In `src/components/navbar/InstanceSelector.tsx`:
   - Remove `hover:scale-105`, `active:scale-95`, and `hover:scale-[1.01]` from Play, Stop, Double Play, and profile buttons.
   - Replace with smooth CSS3 transitions: `transition-colors duration-150`, brightness adjustments, and subtle shadow illumination.
2. In `src/components/navbar/NavMenu.tsx` and `NavSettings.tsx`:
   - Remove `active:scale-95`.
3. In `src/components/settings/EmailNotificationSettings.tsx` and `AccountRow.tsx`:
   - Remove `hover:scale-[1.02]`, `hover:scale-105`, and `transition-transform`.
   - Maintain fixed bounding boxes to eliminate layout shifts, font distortion, and subpixel jumping.

**Constraints:**
- TOTAL BAN on `transform: scale()` on buttons and interactive elements.
- Rely purely on CSS3 color, border, and background transitions.
