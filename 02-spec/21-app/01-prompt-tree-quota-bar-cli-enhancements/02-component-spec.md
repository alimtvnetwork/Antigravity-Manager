# Specification 02: Component Specification — Quota Progress Bar, Instance Card UI Polish, AGM CLI Roadmap & Secure REST Gateway

## User Request (Verbatim)

```markdown
Author two comprehensive, production-grade specification files:
1. 02-spec/21-app/01-prompt-tree-quota-bar-cli-enhancements/02-component-spec.md
2. .ai-memory/plans/subtasks/01-prompt-tree-quota-bar-cli-enhancements/02-quota-bar-cards-and-roadmap.md

Invariants & Non-Negotiable Specifications to Include in 02-component-spec.md:
- Quota Progress Bar Modernization:
  - Strictly limit round milestone balls/dots to maximum 5 (e.g. [100, 75, 50, 25] or 4-5 dots).
  - Dot colors must blend seamlessly with the track/background at that position.
  - When percentage < 25%, the left-hand side must transition to dark red with white text/highlights.
  - Below 25%, checkpoint nodes must display the actual percentage number (e.g. 18%, 12%) rather than a checkmark icon.
  - Over >= 75%, maintain the accepted #1af18d neon green gradient (from-emerald-400 to-[#1af18d]) with glow.
- Instance Card UI Polish & Whitespace Reduction:
  - Tighten card vertical and horizontal padding, reducing wasted whitespace.
  - Compact recent projects list rows with sleek folder icons and turn counters.
  - Provide a dedicated thin custom scrollbar (.payload-viewer-scroll / .thin-scrollbar) to eliminate bulky scrollbars while preserving accessibility.
  - Professional hover effects: dark-glass glow (dark:hover:border-blue-500/40 dark:hover:bg-[#061421]) with subtle transition curves.
- AGM CLI 30-40 Command Roadmap:
  - Exhaustive catalog of 30 to 40 CLI verbs, subcommands, and flags following GitMap patterns (agm <noun> <verb> or agm <verb>).
  - 7 functional categories:
    1. Instance & Profile Lifecycle (create, list, switch, launch, stop, delete, duplicate)
    2. Quota & Fast-Forward Auto-Switching (status, ff, switch-if-low-credit, auto-switch status/enable/config)
    3. Prompt Tree & Conversation Management (prompts ls, tree, query, resend, backup, restore, queue-scheduler)
    4. Local Proxy & Protocol Routing (proxy status, test, restart, models, sessions, logs)
    5. REST Endpoint Security & Token Management (security status, token create/list/revoke, ip whitelist/blacklist)
    6. Supabase Fleet & SSH Delegation (supabase status, sync, leases, ssh nodes, ssh delegate)
    7. System Diagnostics, Hygiene & Self-Healing (doctor, clean, agy cache-clear, undo, update)
  - Every command must support --json and human table output.
- Secure REST Endpoints Gateway Specification:
  - Axum server listening on 127.0.0.1:8045 by default (allow_lan_access = false).
  - Bearer token authentication (Authorization: Bearer <agm_token>), admin token verification for write/lifecycle endpoints.
  - IP whitelist and rate-limiting middleware.
- End-to-End Verification Protocol:
  - 5-step validation harness: (1) Create new test instance, (2) dispatch prompt, (3) verify running indicator in card & tree view, (4) switch account and verify state preservation, (5) verify repeated prompt grouping and quota progress bar styling.

Coding Guidelines & Strict Boundaries:
- Use positive booleans exclusively (hasCompleted, hasGlow, isHealthy, isCritical, isCompact).
- All file paths MUST be relative Git paths (no absolute paths or file:/// URIs).
- DO NOT execute any git commands.
- DO NOT touch files owned by Spec Author 01 (01-architecture-spec.md and 01-prompt-tree-and-cli-architecture.md).
```

---

## 1. Quota Progress Bar Modernization Specification

### 1.1 Architectural Motivation & Visual Principles
The legacy quota progress bar in `src/components/accounts/QuotaProgressBar.tsx` previously rendered 11 equidistant milestone bubbles (`[100, 90, 80, ..., 0]`). In compact card layouts and dense tables, this produced excessive visual clutter, occluded progress fill nuances, and generated rendering artifacts at small pixel widths.

The modernized Quota Progress Bar establishes the following non-negotiable principles:
1. **Strict 4-5 Milestone Cap**: Checkpoint nodes are capped at a maximum of 5 milestones (`[100, 75, 50, 25]` by default, with optional `0` for empty state).
2. **Seamless Track Color Blending**: Rather than generic static fills, each checkpoint bubble takes on the exact tonal gradient stop of the track at its horizontal position.
3. **Critical State (<25%) Transition**: When remaining quota drops below 25%, the entire progress bar transitions to a deep, high-urgency dark red gradient (`from-[#7f1d1d] via-[#991b1b] to-[#dc2626]`), illuminated by high-contrast crisp white typography and highlights.
4. **Dynamic Numeric Checkpoint Micro-Typography**: In the critical state (<25%), filled checkpoint nodes discard the decorative checkmark SVG and dynamically render the exact numeric percentage (e.g., `18%`, `12%`, `5%`) in bold micro-monospace typography (`text-[8px] font-black font-mono leading-none text-white`).
5. **High-Tier Neon Glow (>=75%)**: Accounts with abundant capacity maintain the accepted `#1af18d` neon emerald gradient (`from-emerald-400 to-[#1af18d]`) paired with a soft diffuse neon glow (`shadow-[0_0_10px_rgba(26,241,141,0.75)]`).

### 1.2 Mathematical Model & Gradient Stops

Let $P \in [0, 100]$ be the clamped quota percentage. The active track gradient $G(P)$ is defined piecewise:

$$
G(P) = 
\begin{cases}
\text{from-emerald-400 to-[\#1af18d]} & \text{if } P \ge 75 \\
\text{from-emerald-500 via-teal-400 to-[\#1af18d]} & \text{if } 50 \le P < 75 \\
\text{from-amber-400 via-amber-500 to-orange-500} & \text{if } 25 \le P < 50 \\
\text{from-[\#7f1d1d] via-[\#991b1b] to-[\#dc2626]} & \text{if } P < 25
\end{cases}
$$

#### Milestone Nodes Distribution & Coordinate Positioning
For checkpoints $C = [100, 75, 50, 25]$:
- Position $x(c) = c\%$ along the horizontal track.
- Boundary transform clamping prevents overflow outside container bounds:
  - $c = 100 \implies \text{transform: } \text{translate}(-100\%, -50\%)$
  - $c = 0 \implies \text{transform: } \text{translate}(0\%, -50\%)$
  - $0 < c < 100 \implies \text{transform: } \text{translate}(-50\%, -50\%)$

#### Checkpoint Color Blending Matrix
| Checkpoint | Position | Filled Style (Normal $P \ge 25\%$) | Filled Style (Critical $P < 25\%$) | Unfilled Style |
| :--- | :--- | :--- | :--- | :--- |
| **100%** | $100\%$ | `bg-[#1af18d] border-[1.5px] border-emerald-300 shadow-[0_0_8px_rgba(26,241,141,0.85)]` | `bg-rose-900 border-[1.5px] border-rose-700/60 shadow-none` | `bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60` |
| **75%** | $75\%$ | `bg-emerald-400 border-[1.5px] border-emerald-300 shadow-[0_0_6px_rgba(26,241,141,0.6)]` | `bg-rose-800 border-[1.5px] border-rose-600/60 shadow-none` | `bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60` |
| **50%** | $50\%$ | `bg-amber-400 dark:bg-amber-500 border-[1.5px] border-amber-300 shadow-none` | `bg-rose-700 border-[1.5px] border-rose-500/60 shadow-none` | `bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60` |
| **25%** | $25\%$ | `bg-orange-500 border-[1.5px] border-orange-400 shadow-none` | `bg-red-600 border-[1.5px] border-red-400 shadow-[0_0_6px_rgba(220,38,38,0.7)]` | `bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60` |

### 1.3 TypeScript Component Interface (`QuotaProgressBarProps`)

All boolean properties strictly follow positive naming conventions (`hasCompleted`, `hasGlow`, `isHealthy`, `isCritical`, `isCompact`):

```typescript
export interface QuotaProgressBarProps {
    percentage: number;
    resetTime?: string;
    label?: string;
    isProtected?: boolean;
    isWeekly?: boolean;
    isWeeklyConstrained?: boolean;
    isCompact?: boolean;
    showCheckpoints?: boolean;
    hasGlow?: boolean;
    checkpoints?: number[]; // Default: [100, 75, 50, 25] (Strictly <= 5 items)
    className?: string;
    heightClassName?: string;
    Icon?: React.ComponentType<{ size?: number; className?: string }>;
}
```

### 1.4 Production Reference Implementation (`QuotaProgressBar.tsx`)

```tsx
import React from 'react';
import { Clock, Lock } from 'lucide-react';
import { cn } from '../../utils/cn';
import { formatTimeRemaining, getTimeRemainingColor } from '../../utils/format';

export interface QuotaProgressBarProps {
    percentage: number;
    resetTime?: string;
    label?: string;
    isProtected?: boolean;
    isWeekly?: boolean;
    isWeeklyConstrained?: boolean;
    isCompact?: boolean;
    showCheckpoints?: boolean;
    hasGlow?: boolean;
    checkpoints?: number[];
    className?: string;
    heightClassName?: string;
    Icon?: React.ComponentType<{ size?: number; className?: string }>;
}

export function QuotaProgressBar({
    percentage,
    resetTime,
    label,
    isProtected = false,
    isWeekly = false,
    isWeeklyConstrained = false,
    isCompact = false,
    showCheckpoints = true,
    hasGlow = true,
    checkpoints = [100, 75, 50, 25],
    className,
    heightClassName = "h-3.5",
    Icon,
}: QuotaProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));
    const isCritical = clamped < 25;
    const isHealthy = clamped >= 75;

    // Track gradient selection
    const getTrackGradient = (pct: number) => {
        if (pct >= 75) return 'bg-gradient-to-r from-emerald-400 to-[#1af18d]';
        if (pct >= 50) return 'bg-gradient-to-r from-emerald-500 via-teal-400 to-[#1af18d]';
        if (pct >= 25) return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
        return 'bg-gradient-to-r from-[#7f1d1d] via-[#991b1b] to-[#dc2626]';
    };

    // Track glow: neon green for >= 75%, subtle crimson glow for < 25%
    const getTrackGlow = (pct: number) => {
        if (!hasGlow) return '';
        if (pct >= 75) return 'shadow-[0_0_10px_rgba(26,241,141,0.75)]';
        if (pct >= 50) return 'shadow-[0_0_6px_rgba(16,185,129,0.4)]';
        if (pct < 25) return 'shadow-[0_0_8px_rgba(220,38,38,0.55)]';
        return '';
    };

    // Text color classes
    const getPercentColorClass = (pct: number) => {
        if (pct >= 75) return 'text-emerald-700 dark:text-[#1af18d]';
        if (pct >= 50) return 'text-emerald-600 dark:text-emerald-400';
        if (pct >= 25) return 'text-amber-700 dark:text-amber-400';
        return 'text-rose-600 dark:text-rose-300 font-extrabold';
    };

    const getTimeColorClass = (time?: string) => {
        if (!time) return 'text-gray-400 dark:text-gray-500';
        const color = getTimeRemainingColor(time);
        switch (color) {
            case 'success': return 'text-emerald-700 dark:text-[#1af18d]';
            case 'warning': return 'text-amber-700 dark:text-amber-400';
            default: return 'text-blue-700 dark:text-blue-400';
        }
    };

    // Milestone bubble styling strictly limited to 4-5 checkpoints
    const getNodeStyle = (checkpoint: number, isFilled: boolean) => {
        if (!isFilled) {
            return 'bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none';
        }
        if (isCritical) {
            return 'bg-[#991b1b] border-[1.5px] border-red-400/80 shadow-[0_0_6px_rgba(220,38,38,0.7)] text-white';
        }
        if (checkpoint >= 100) {
            return 'bg-[#1af18d] border-[1.5px] border-emerald-300 shadow-[0_0_8px_rgba(26,241,141,0.85)]';
        }
        if (checkpoint >= 75) {
            return 'bg-emerald-400 border-[1.5px] border-emerald-300 shadow-[0_0_6px_rgba(26,241,141,0.6)]';
        }
        if (checkpoint >= 50) {
            return 'bg-amber-400 dark:bg-amber-500 border-[1.5px] border-amber-300 dark:border-amber-400 shadow-none';
        }
        return 'bg-orange-500 border-[1.5px] border-orange-400 shadow-none';
    };

    // Ensure maximum 5 checkpoints sorted descending
    const sanitizedCheckpoints = checkpoints.slice(0, 5).sort((a, b) => b - a);

    return (
        <div className={cn("w-[82%] max-w-[82%] flex items-center gap-2", className)}>
            <style>{`
                @keyframes agm-water-shimmer {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(200%); }
                }
            `}</style>

            {/* Optional Left Label */}
            {(Icon || label) && (
                <div className="flex items-center gap-1 shrink-0 max-w-[18%] min-w-0 text-slate-700 dark:text-slate-300">
                    {Icon && <Icon size={13} className="shrink-0" />}
                    {label && (
                        <span className="text-[11px] font-semibold truncate" title={label}>
                            {label}
                        </span>
                    )}
                </div>
            )}

            {/* Progress Track */}
            <div className="relative flex-1 min-w-[60px] flex items-center">
                <div className={cn(
                    "relative w-full rounded-full overflow-hidden border transition-all duration-300",
                    isCritical
                        ? "bg-rose-950/40 border-rose-800/80"
                        : "bg-slate-200 dark:bg-[#071a27] border-slate-300 dark:border-[#15334d]",
                    heightClassName
                )}>
                    {/* Active Fill */}
                    <div
                        className={cn(
                            "h-full rounded-full relative overflow-hidden transition-all duration-500 ease-out",
                            getTrackGradient(clamped),
                            getTrackGlow(clamped)
                        )}
                        style={{ width: `${clamped}%` }}
                    >
                        <div
                            className="absolute inset-0 bg-gradient-to-r from-transparent via-white/35 to-transparent pointer-events-none"
                            style={{ animation: 'agm-water-shimmer 2s infinite linear' }}
                        />
                    </div>
                </div>

                {/* Milestone Checkpoint Nodes (Max 5) */}
                {showCheckpoints && sanitizedCheckpoints.map((cp) => {
                    const isFilled = clamped >= cp;
                    const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                    const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';

                    return (
                        <div
                            key={cp}
                            className={cn(
                                "absolute top-1/2 w-3.5 h-3.5 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none select-none",
                                getNodeStyle(cp, isFilled)
                            )}
                            style={{ left: leftPos, transform }}
                            title={`Checkpoint ${cp}%`}
                        >
                            {/* In critical state (<25%), render exact percentage string rather than checkmark */}
                            {isCritical && isFilled ? (
                                <span className="text-[7.5px] font-black font-mono leading-none text-white">
                                    {clamped}
                                </span>
                            ) : (
                                <svg
                                    className={cn(
                                        "w-2 h-2 fill-none stroke-current transition-colors",
                                        isFilled ? "text-white stroke-[2.5]" : "text-gray-400/50 dark:text-white/30 stroke-[2.2]"
                                    )}
                                    viewBox="0 0 12 12"
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                >
                                    <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                                </svg>
                            )}
                        </div>
                    );
                })}
            </div>

            {/* Trailing Info: Time Remaining & Percentage */}
            <div className="flex flex-col items-end shrink-0 leading-tight w-[18%] max-w-[18%]">
                {resetTime ? (
                    <span
                        className={cn(
                            "text-[10px] font-bold flex items-center gap-0.5 font-mono",
                            getTimeColorClass(resetTime)
                        )}
                        title={`Resets in ${formatTimeRemaining(resetTime)}`}
                    >
                        <Clock className="w-2.5 h-2.5 shrink-0" />
                        {formatTimeRemaining(resetTime)}
                    </span>
                ) : null}

                <div className="flex items-center gap-1 justify-end">
                    {isProtected && (
                        <span title="Quota protected">
                            <Lock className="w-2.5 h-2.5 text-amber-500" />
                        </span>
                    )}

                    {isWeeklyConstrained && (
                        <span className="px-1 py-[0.5px] rounded bg-rose-500/15 text-rose-700 dark:text-rose-300 text-[9px] font-bold">
                            Weekly
                        </span>
                    )}

                    <span className={cn("text-[10px] font-black font-mono", getPercentColorClass(clamped))}>
                        {clamped}%
                    </span>
                </div>
            </div>
        </div>
    );
}

export default QuotaProgressBar;
```

---

## 2. Instance Card UI Polish & Whitespace Reduction Specification

### 2.1 Whitespace Optimization Rationale & Grid Metrics
In `src/pages/Instances.tsx`, cards occupy significant screen real estate due to oversized default paddings (`p-3.5`), loose flex margins (`mb-2.5`), and unconstrained project lists. 

#### Structural Metrics Refactoring Table
| Element | Current Style | Modernized Target Style | Delta / Improvement |
| :--- | :--- | :--- | :--- |
| **Card Container Padding (Normal)** | `p-3.5` (14px) | `p-2.5` (10px) | 28.5% reduction in perimeter waste |
| **Card Container Padding (Compact)** | `p-2.5` (10px) | `p-2` (8px) | 20% reduction |
| **Grid Gap Between Cards** | `gap-3` (12px) | `gap-2.5` (10px) | Better screen density for multi-monitor |
| **Header Row Spacing** | `mb-2.5 h-6` | `mb-1.5 h-5` | Eliminates vertical dead zones |
| **Account Email Section Margin** | `mb-2.5 py-1 px-2.5` | `mb-1.5 py-0.5 px-2` | Sleek compact badge style |
| **Quota Box Padding** | `p-2.5 space-y-2` | `p-2 space-y-1.5` | Preserves hierarchy while tightening line height |
| **Details Row Padding** | `py-2 text-xs` | `py-1 text-[11px]` | Compact metrics layout |
| **Action Toolbar Top Spacing** | `pt-2.5 mt-2` | `pt-1.5 mt-1.5` | Direct proximity to action targets |

### 2.2 Compact Recent Projects List Rows
The recent projects sub-panel renders up to 3 recent workspace repositories owned by the instance. Under the modernized spec:
1. **Vertical Row Height**: Tightened from `py-1` to `py-0.5 px-2` with `rounded-[4px]`.
2. **Icons & Visual Language**: Lucide `Folder` icon (`w-3 h-3 text-blue-500/80 shrink-0`).
3. **Turn Counters**: Monospace badge (`px-1.5 py-0.2 rounded-[4px] text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-gray-100 dark:bg-[#0c2438] border border-gray-200/50 dark:border-[#15334d]`).
4. **Running Indicator**: If an active conversation is executing, displays a compact cyan pulse badge (`<span className="w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse" /> RUNNING`).
5. **Interactive Double-Click**: Double-clicking any project row immediately launches the Prompt Tree modal filtered to that specific workspace.

### 2.3 Dedicated Thin Custom Scrollbar Specification
When recent projects exceed 3 items, or in projects with extended multi-line descriptions, bulky default browser scrollbars distort the card layout. We mandate the unified `.thin-scrollbar` / `.payload-viewer-scroll` design token:

```css
/* Custom Thin Scrollbar Token (src/App.css) */
.thin-scrollbar,
.payload-viewer-scroll {
  scroll-behavior: auto !important;
  overflow-anchor: none;
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.4) transparent;
}

.thin-scrollbar::-webkit-scrollbar,
.payload-viewer-scroll::-webkit-scrollbar {
  width: 5px;
  height: 5px;
}

.thin-scrollbar::-webkit-scrollbar-track,
.payload-viewer-scroll::-webkit-scrollbar-track {
  background-color: transparent;
}

.thin-scrollbar::-webkit-scrollbar-thumb,
.payload-viewer-scroll::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.35);
  border-radius: 9999px;
  border: 1px solid transparent;
  background-clip: content-box;
  transition: background-color 0.15s ease-out;
}

.thin-scrollbar::-webkit-scrollbar-thumb:hover,
.payload-viewer-scroll::-webkit-scrollbar-thumb:hover {
  background-color: rgba(107, 114, 128, 0.7);
}

.dark .thin-scrollbar,
.dark .payload-viewer-scroll {
  scrollbar-color: rgba(148, 163, 184, 0.3) transparent;
}

.dark .thin-scrollbar::-webkit-scrollbar-thumb,
.dark .payload-viewer-scroll::-webkit-scrollbar-thumb {
  background-color: rgba(148, 163, 184, 0.3);
}

.dark .thin-scrollbar::-webkit-scrollbar-thumb:hover,
.dark .payload-viewer-scroll::-webkit-scrollbar-thumb:hover {
  background-color: rgba(148, 163, 184, 0.65);
}
```

### 2.4 Professional Dark-Glass Glow & Micro-Interactions
Card states must satisfy:
- **Base State (Inactive)**: `bg-white dark:bg-[#0a1e30] border-gray-200/50 dark:border-[#15334d]/60 shadow-xs`.
- **Hover State (Dark-Glass)**: `hover:border-gray-300/80 dark:hover:border-blue-500/40 hover:bg-slate-50/90 dark:hover:bg-[#061421] transition-all duration-200 ease-out shadow-sm`.
- **Active Instance State**: `border-amber-400/50 dark:border-amber-400/60 bg-sky-50/30 dark:bg-[#0d263d] shadow-md ring-1 ring-amber-400/30`.
- **Executing Mutex Overlay**: When `isBusy = true`, render an absolute glass curtain (`backdrop-blur-[2px] bg-white/75 dark:bg-[#071a27]/85`) with an animated spinning loader (`RotateCw`) and human-readable action label (`Launching...`, `Stopping...`, `Rotating...`).

---

## 3. AGM CLI 30-40 Command Roadmap Specification

### 3.1 Design Philosophy & Syntax Standards
The `agm` binary (`src-tauri/src/bin/agm.rs`) acts as an autonomous developer companion and headless CLI for Antigravity-Manager. It follows GitMap command conventions:
- **Standard Syntax**: `agm <noun> <verb> [options]` with ergonomic direct verb shortcuts: `agm <verb> [options]`.
- **Dual Output Channels**:
  - Human-friendly tabular output with ANSI colors and Unicode box-drawing characters.
  - Machine-readable `--json` flag output strictly conforming to the `JsonEnvelope<T>` envelope.
- **Fail-Safe Exit Codes**: Exit `0` on success; exit `1` on invalid arguments; exit `2` on connectivity failure; exit `3` on quota exhaustion.

### 3.2 Complete 35-Command Catalog Across 7 Functional Categories

#### Category 1: Instance & Profile Lifecycle
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm instance list` | `agm ls`, `agm instances` | List all configured IDE instances with status, PID, and bound account | `--json`, `--all`, `--compact` |
| `agm instance create <name>` | `agm create` | Provision a new isolated instance workspace directory | `--path <dir>`, `--exe <ide_bin>`, `--json` |
| `agm instance switch <id>` | `agm switch` | Set target instance as the active profile | `--force`, `--json` |
| `agm instance launch <id>` | `agm launch`, `agm start` | Launch the Antigravity IDE process for the instance | `--background`, `--dry-run`, `--json` |
| `agm instance stop <id>` | `agm stop` | Gracefully terminate IDE and background watcher processes | `--force`, `--timeout <secs>`, `--json` |
| `agm instance delete <id>` | `agm delete` | Delete an instance configuration and unregister profile | `--purge-data`, `--yes`, `--json` |
| `agm instance duplicate <src> <dest>` | `agm duplicate`, `agm clone` | Clone instance configuration, extensions, and settings | `--copy-projects`, `--json` |

#### Category 2: Quota & Fast-Forward Auto-Switching
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm quota status` | `agm status`, `agm credits` | Display real-time 4H and weekly quotas across all accounts | `--all`, `--watch <secs>`, `--json` |
| `agm quota ff` | `agm ff`, `agm fast-forward` | Instantly rotate active instance to highest-quota healthy account | `--dry-run`, `--min-quota <pct>`, `--json` |
| `agm switch-if-low-credit` | `agm swlc` | Trigger automated rotation only if active account quota < 25% | `--threshold <pct>`, `--json` |
| `agm auto-switch status` | `agm as status` | Query background auto-switcher daemon state and next tick | `--json` |
| `agm auto-switch enable` | `agm as on` | Enable background periodic quota monitoring daemon | `--interval <mins>`, `--json` |
| `agm auto-switch disable` | `agm as off` | Pause background auto-switcher daemon | `--json` |
| `agm auto-switch config` | `agm as cfg` | Set auto-switch parameters (thresholds, cool-down, priority models) | `--threshold <pct>`, `--cooldown <secs>`, `--json` |

#### Category 3: Prompt Tree & Conversation Management
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm prompts ls` | `agm prompts` | List active and recent prompt turns across all instances | `--limit <n>`, `--status <running\|done>`, `--json` |
| `agm prompts tree [project-id]` | `agm tree` | Render interactive tree hierarchy of projects, sessions, and prompts | `--depth <n>`, `--all`, `--json` |
| `agm prompts query <pattern>` | `agm query`, `agm search` | Full-text search prompts history and user instruction payloads | `--regex`, `--case-sensitive`, `--json` |
| `agm prompts resend <prompt-id>` | `agm resend` | Re-dispatch an interrupted prompt to the active instance | `--target-instance <id>`, `--json` |
| `agm prompts backup` | `agm backup`, `agm brp` | Export running prompts state and history to durable SQLite backup | `--output <file>`, `--compress`, `--json` |
| `agm prompts restore <file>` | `agm restore`, `agm rrp` | Restore prompts and active conversation state from backup | `--dry-run`, `--merge`, `--json` |
| `agm queue-scheduler status` | `agm qs` | Inspect queued prompts waiting for execution or quota clearance | `--pause`, `--resume`, `--json` |

#### Category 4: Local Proxy & Protocol Routing
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm proxy status` | `agm proxy` | Check local HTTP/HTTPS proxy health, port, and uptime | `--json` |
| `agm proxy test [model]` | `agm test-proxy` | Send ping prompt through proxy adapter pipeline to verify routing | `--stream`, `--timeout <secs>`, `--json` |
| `agm proxy restart` | `agm proxy reload` | Reload proxy configuration and cycle upstream connection pools | `--force`, `--json` |
| `agm proxy models` | `agm models` | List supported models, thinking budget rules, and rate limits | `--json` |
| `agm proxy sessions` | `agm sessions` | List active HTTP streaming client connections and token rates | `--json` |
| `agm proxy logs` | `agm logs` | Stream live proxy traffic, token consumption, and sanitized payloads | `--tail <n>`, `--filter <str>`, `--json` |

#### Category 5: REST Endpoint Security & Token Management
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm security status` | `agm sec` | Report REST API gateway port (8045), auth mode, and IP rules | `--json` |
| `agm token create <name>` | `agm token new` | Generate a new Bearer authentication token with scoped permissions | `--admin`, `--expires-in <days>`, `--json` |
| `agm token list` | `agm tokens` | Enumerate active REST Bearer tokens with masked hash and last used | `--json` |
| `agm token revoke <token-id>` | `agm token rm` | Revoke and invalidate a Bearer token immediately | `--json` |
| `agm ip whitelist <add\|rm\|ls>` | `agm ip wl` | Manage authorized IP addresses permitted to access REST API | `--ip <cidr>`, `--json` |
| `agm ip blacklist <add\|rm\|ls>` | `agm ip bl` | Manage blocked IP addresses rejected by IP filter middleware | `--ip <cidr>`, `--json` |

#### Category 6: Supabase Fleet & SSH Delegation
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm supabase status` | `agm supa` | Probe remote Supabase fleet synchronization health and ping | `--json` |
| `agm supabase sync` | `agm supa sync` | Trigger immediate bi-directional database sync for accounts & quota | `--full`, `--json` |
| `agm supabase leases` | `agm leases` | Query cross-machine workspace leases and lease expirations | `--release <id>`, `--json` |
| `agm ssh nodes` | `agm nodes` | Enumerate remote cluster worker machines configured in AGM | `--json` |
| `agm ssh delegate <node> <cmd>` | `agm delegate` | Dispatch remote AGM command execution to a cluster node over SSH | `--timeout <secs>`, `--json` |

#### Category 7: System Diagnostics, Hygiene & Self-Healing
| Command | Primary Aliases | Description | Key Options / Flags |
| :--- | :--- | :--- | :--- |
| `agm doctor` | `agm check` | Verify system requirements, ports, SQLite databases, and permissions | `--fix`, `--json` |
| `agm clean` | `agm purge` | Purge stale IDE lockfiles, dangling PIDs, and temporary scratch files | `--all`, `--dry-run`, `--json` |
| `agm agy cache-clear` | `agm cache-clear` | Clear Antigravity tools cache, thinking store, and schema caches | `--yes`, `--json` |
| `agm undo` | `agm rollback` | Revert the last automatic account rotation or configuration change | `--json` |
| `agm update` | `agm upgrade` | Check and apply latest Antigravity-Manager binary release | `--channel <stable\|beta>`, `--check-only`, `--json` |

---

## 4. Secure REST Endpoints Gateway Specification

### 4.1 Server Architecture & Network Binding
The AGM Secure REST Endpoints Gateway is powered by an embedded `axum` microservice within the Tauri core process:
- **Default Listening Socket**: `127.0.0.1:8045`.
- **LAN Access Policy**: Loopback only (`127.0.0.1`). The configuration flag `allow_lan_access` defaults strictly to `false`. Binding to `0.0.0.0:8045` requires explicit administrative opt-in via settings or `agm security allow-lan --enable`.
- **Architecture**:
  ```
  Client Request (Local IDE / agm CLI / VSCode Ext)
             │
             ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Axum HTTP Gateway (Port: 8045, Default: 127.0.0.1)     │
  ├─────────────────────────────────────────────────────────┤
  │  1. CORS Middleware (Restricted to Local Webview/Origin)│
  │  2. IP Filter Middleware (Whitelist / Blacklist)       │
  │  3. Rate Limiting Middleware (Leaky Bucket)            │
  │  4. Auth Middleware (Bearer Token Verification)         │
  └────────────────────────────┬────────────────────────────┘
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
  Read-Only / Metrics APIs             Admin / Mutation APIs
  (User Token or Admin Token)           (Admin Token Strictly Required)
  - /healthz                            - /api/instance/create
  - /api/status                         - /api/instance/switch
  - /api/quota                          - /api/token/create
  - /api/prompts/tree                   - /api/prompts/resend
  ```

### 4.2 Middleware Pipeline & Security Contracts

#### 1. IP Filter Middleware (`ip_filter_middleware`)
- Extracts client IP address from socket connection or `X-Forwarded-For` (only if behind trusted proxy).
- Rejects requests from blacklisted IPs immediately with `HTTP 403 Forbidden` (`{"error": "Access denied by IP blacklist"}`).
- If whitelist mode is active, drops any IP not in `security_db::whitelist`.

#### 2. Rate-Limiting Middleware (`rate_limiter_middleware`)
- Implements a token bucket algorithm:
  - Burst allowance: 30 requests.
  - Replenishment rate: 5 requests per second per IP/token.
- Returns `HTTP 429 Too Many Requests` with `Retry-After: <seconds>` header.

#### 3. Authentication Middleware (`auth_middleware` & `admin_auth_middleware`)
- Expects header: `Authorization: Bearer <agm_token>`.
- Token lookup queries `security_db::get_token_by_hash(sha256(raw_token))`.
- Validates:
  - Token expiration (`expires_at > Utc::now()`).
  - Active state (`is_active = true`).
  - Required role (`is_admin = true` for mutation/lifecycle routes).

### 4.3 REST API Endpoints Registry Table

| Method | Endpoint Path | Auth Required | Description | Request Payload | Response Schema |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `GET` | `/healthz` | None (Public) | Liveness probe and health status | None | `{"status": "ok", "uptime_secs": 1240}` |
| `GET` | `/api/status` | User / Admin | Overview of instances and quota | None | `{"instances": [...], "active_instance": "default"}` |
| `GET` | `/api/quota` | User / Admin | Real-time quota metrics by account | None | `{"accounts": [{"id": "...", "percentage": 82}]}` |
| `POST` | `/api/instance/switch` | Admin Only | Switch active instance target | `{"instance_id": "profile-2"}` | `{"success": true, "previous": "default", "active": "profile-2"}` |
| `POST` | `/api/instance/launch` | Admin Only | Launch instance IDE binary | `{"instance_id": "profile-2"}` | `{"success": true, "pid": 48210}` |
| `POST` | `/api/instance/stop` | Admin Only | Stop instance IDE binary | `{"instance_id": "profile-2"}` | `{"success": true}` |
| `GET` | `/api/prompts/tree` | User / Admin | Get prompt projects and hierarchy | Query: `?instance_id=...` | `{"projects": [{"project_id": "...", "turns": 14}]}` |
| `POST` | `/api/prompts/resend` | Admin Only | Re-dispatch prompt | `{"prompt_id": "p-104", "target_instance": "..."}` | `{"success": true, "task_id": "task-882"}` |
| `POST` | `/api/security/token` | Admin Only | Issue new Bearer auth token | `{"name": "cli-remote", "is_admin": false}` | `{"token": "agm_live_...", "id": "t-12"}` |
| `DELETE` | `/api/security/token/:id` | Admin Only | Revoke a Bearer auth token | None | `{"success": true}` |

---

## 5. End-to-End Verification Protocol

To validate all modernized UI components, CLI commands, and secure REST endpoints, the following 5-step test protocol is executed:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   5-Step E2E Validation Harness                        │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │
  Step 1: Provision Isolated Instance
  └─► Command: agm instance create "E2E-Verification-Test"
      Check: Config created, seq_num assigned, zero collisions.
                                   │
  Step 2: Dispatch Prompt & Check Live Indicators
  └─► Command: agm prompt --instance "E2E-Verification-Test" "Hello AGY"
      Check: Card renders pulsating RUNNING badge, Prompt Tree shows cyan node.
                                   │
  Step 3: Verify Real-Time Animation & Turn Counter
  └─► Action: Inspect UI card in browser / webview.
      Check: Turn counter increments, .thin-scrollbar scrolls smoothly,
             no layout shifts or scrollbar flickering.
                                   │
  Step 4: Rotate Account & Verify State Preservation
  └─► Command: agm switch-if-low-credit --threshold 90
      Check: Target switches smoothly, workspace storage preserved,
             no prompt loss during rotation.
                                   │
  Step 5: Verify Quota Progress Bar Visual Styles
  └─► Action: Simulate quota states (18%, 52%, 88%).
      Check: 88% -> Neon emerald (#1af18d) with glow;
             18% -> Dark red (#991b1b) with numeric checkpoint text "18%".
```

### Detailed Validation Checkpoints

1. **Step 1: Create Test Instance**
   - Action: Run `agm instance create "E2E-Test-Alpha"`.
   - Verification: Instance registered in `instances.json` and SQLite database; appears in `InstanceTable.tsx` and Card Grid with assigned sequence number.
2. **Step 2: Dispatch Prompt**
   - Action: Dispatch a sample prompt via CLI or REST API.
   - Verification: Running indicator activates immediately across Card Header (`hasActiveTask = true`), Project Row (`RUNNING` badge), and Prompt Tree modal.
3. **Step 3: Card UI & Scrollbar Audit**
   - Action: Populate recent projects list with 5 workspaces.
   - Verification: Container applies `.thin-scrollbar` (5px width, rounded thumb, hover darkening); card padding respects `p-2.5` compact bounds.
4. **Step 4: Account Switch & State Continuity**
   - Action: Trigger account switch via `agm instance switch`.
   - Verification: IDE window credentials update seamlessly; active conversation queue remains intact without dropped messages.
5. **Step 5: Quota Bar Visual Polish Audit**
   - Action: Render quota states at 85%, 45%, and 18%.
   - Verification:
     - 85%: Rendered with `#1af18d` neon emerald gradient and glowing shadow.
     - 18%: Rendered with `#991b1b` crimson gradient; milestone node displays numeric `18%` text instead of checkmark icon.

---

## 6. Coding Guidelines & Invariants Alignment

All implementations must conform to the following non-negotiable standards:
- **Positive Booleans Only**: Strict ban on negative flags (`disabled`, `notReady`, `unhealthy`). Use `isReady`, `isHealthy`, `isCritical`, `isProtected`, `hasGlow`, `isCompact`, `allowLanAccess`.
- **Relative Path Hygiene**: All documentation, code references, and script paths must use relative repository paths (e.g. `src/components/accounts/QuotaProgressBar.tsx`). Never use absolute paths or `file:///` URLs.
- **Clean Construction & Branch Immutability**: All component transformations and CLI response mappings must construct new immutable data structures rather than mutating parameters in place.
