# Progress Bar Specification: Slim Track, Protruding Milestone Nodes, Checkmark Glyphs & Hover Glow

**Document ID:** `02-spec/21-app/ui-and-ux-enhancements-for-antigravity-manager/03-progress-bar-spec.md`  
**Parent Task:** `ui-and-ux-enhancements-for-antigravity-manager`  
**Version:** 1.0.0  
**Updated:** 2026-10-15  
**Status:** APPROVED PROGRESS BAR SPECIFICATION  
**Target Release:** v4.200.0  
**Target Components:**
- `src/components/accounts/QuotaProgressBar.tsx`
- `src/components/common/WaterDrainProgressBar.tsx`
- `src/components/accounts/AccountCard/AccountCardQuota.tsx`
- `src/components/accounts/AccountRow.tsx`
- `src/components/instances/table/InstanceRowCells.tsx`
- `tools/ui-builder/index.html`

---

## 1. Executive Summary & Design Blueprint

Quota and resource tracking progress bars in Antigravity-Manager visualize real-time capacity across Google OAuth tokens, upstream proxy rate limits, and IDE sandboxes. Previous iterations suffered from:
1. **Excessive Track Bulk:** 16px to 18px track heights consumed too much vertical rhythm inside dense account and instance tables.
2. **Confined Miniature Nodes:** Milestone beads were constrained inside the track borders, rendering internal icons illegible and causing cramped visual density.
3. **Boundary Clutter:** Superfluous circular dots placed at the hard 0% and 100% boundary ends clashed with the track's rounded pill ends.
4. **Lack of Clear Milestone Indicators:** Milestone nodes lacked clear glyphs communicating milestone completion.
5. **Static Visual Weight:** Bars maintained constant brightness, lacking interactive depth when hovering over table rows.

### The Overhaul Solution:
- **Slimmer Track Height:** Reduced to exactly **10px** (`h-2.5` / `0.625rem` / `10px`).
- **Protruding Milestone Nodes:** Exactly **20px diameter** (`w-5 h-5` / `20px`), vertically centered on the 10px track so each node extends **5px above and 5px below** the track boundaries.
- **Checkmark (`v`) Glyphs:** Filled milestone nodes render a crisp checkmark icon (`v`) indicating verified completion of that capacity tier.
- **No Boundary End Nodes:** Strictly zero circular beads at 0% and 100% boundary ends. The track terminates with seamless rounded caps (`rounded-full`).
- **5 Internal Splits:** Exactly 5 internal milestone checkpoints distributed across the operational range (`20%`, `40%`, `60%`, `80%`, `90%`).
- **Full-Width Spatial Gradient:** Smooth gradient transition from Dark Red (`#7f1d1d` / `#dc2626`) at 0% on the left, blending through Warning Orange (`#f97316`), Amber Yellow (`#eab308`), Mint Emerald (`#10b981`), to Vibrant Neon Terminal Green (`#1af18d`) at 100% on the right.
- **Resting Opacity & Table Row Hover Glow:** Default resting opacity of 80–85%, smoothly transitioning to **100% brightness and ambient neon glow** when the parent table row or card is hovered.

---

## 2. Dimensional Geometry & Spatial Layout

### 2.1 Vertical Protrusion Architecture

```
                 +-- Node Top Boundary (-5px above track)
                 |
             +---+---+
           /   [ v ]   \  <- 20px Diameter Milestone Node (w-5 h-5)
     +----+-------------+----+-----------------------------------------------+
     |    |             |    |  10px Slim Track Height (h-2.5)               |
     +----+-------------+----+-----------------------------------------------+
           \           /
             +---+---+
                 |
                 +-- Node Bottom Boundary (+5px below track)
```

- **Track Height:** `10px` (`h-2.5` in Tailwind CSS, `height: 10px` in CSS).
- **Track Border Radius:** `rounded-full` (`border-radius: 9999px`).
- **Node Diameter:** `20px` (`w-5 h-5` in Tailwind CSS, `width: 20px; height: 20px` in CSS).
- **Node Vertical Alignment:** `top: 50%; transform: translate(-50%, -50%)`.
- **Top / Bottom Protrusion:**
  $$\text{Protrusion} = \frac{\text{Node Diameter} - \text{Track Height}}{2} = \frac{20\text{px} - 10\text{px}}{2} = 5\text{px}$$
  Each node protrudes exactly 5px above the track top edge and 5px below the track bottom edge, creating a distinctive modern gauge aesthetic.

### 2.2 Boundary Invariant: Elimination of 0% and 100% Dots

- **Strict Invariant:** No milestone nodes or circular beads are permitted at `0%` or `100%`.
- **Rationale:** The rounded ends of the 10px track already provide natural visual boundaries. Placing 20px nodes at the ends causes horizontal clipping outside table cell boundaries and creates visual distortion against adjacent label columns.
- **Internal Placement:** Milestone nodes exist **strictly within the interior** of the track ($0\% < x < 100\%$).

### 2.3 The 5 Internal Milestone Checkpoints

The 5 milestone checkpoints represent operational capacity tiers:

| Milestone ID | Target Element ID | Percentage Position | Capacity Meaning | Color Stop |
| :--- | :--- | :---: | :--- | :--- |
| **Node 1** | `agm-progress-node-1` | **20%** | Critical Threshold (Auto-Switch Armed) | `#ef4444` (Critical Red) |
| **Node 2** | `agm-progress-node-2` | **40%** | Cautionary Boundary | `#f97316` (Warning Orange) |
| **Node 3** | `agm-progress-node-3` | **60%** | Steady State Equilibrium | `#eab308` (Golden Amber) |
| **Node 4** | `agm-progress-node-4` | **80%** | Substantial Capacity Headroom | `#10b981` (Mint Emerald) |
| **Node 5** | `agm-progress-node-5` | **90%** | Optimal Operating Buffer | `#1af18d` (Neon Green) |

*Note: Alternatively, 5 equidistant quintile internal splits can be configured at `[16.6%, 33.3%, 50%, 66.6%, 83.3%]`. By default, the practical threshold standard `[20%, 40%, 60%, 80%, 90%]` is enforced.*

---

## 3. Color Gradient & Spatial Color Mapping

### 3.1 Canonical Color Palette

The progress bar maps from dark red on the left (0%) to vibrant terminal neon green on the right (100%):

```css
/* Quota Scale Color Stops */
--agm-gradient-stop-0:   #7f1d1d; /* 0%   - Deep Crimson Dark Red */
--agm-gradient-stop-15:  #dc2626; /* 15%  - Crimson Red */
--agm-gradient-stop-35:  #ea580c; /* 35%  - Warm Orange */
--agm-gradient-stop-55:  #eab308; /* 55%  - Warm Golden Yellow */
--agm-gradient-stop-75:  #10b981; /* 75%  - Mint Emerald */
--agm-gradient-stop-100: #1af18d; /* 100% - Vibrant Terminal Neon Green */
```

### 3.2 Full-Width Linear Gradient Formulation

The complete gradient declaration spans 100% of the horizontal track width:

```css
background: linear-gradient(
  90deg,
  #7f1d1d 0%,
  #dc2626 15%,
  #ea580c 35%,
  #eab308 55%,
  #10b981 75%,
  #1af18d 100%
);
```

### 3.3 Spatial Color Invariant
- The spatial color gradient is **never compressed or stretched** to fit the current progress percentage.
- The 6-stop gradient always maps across the **entire track width (0% to 100%)**.
- The active percentage is displayed either by:
  1. A masked overlay revealing the gradient up to `percentage%`, or
  2. A clipping container with `overflow: hidden; width: ${percentage}%` containing an inner div with `width: 100% of parent track; background: linear-gradient(...)`.
- This ensures that 20% capacity is **always** visually red, 50% capacity is **always** yellow, and 90% capacity is **always** neon green, regardless of how much quota remains.

---

## 4. Milestone Node Anatomy & Checkmark (`v`) Glyphs

Each milestone node is a 20px circular component structured with three visual layers:

```
+-------------------------------------------------------------+
|                     20px MILESTONE NODE                     |
|                                                             |
|   +-----------------------------------------------------+   |
|   | 1. Outer Border: 1.5px Solid (Color or Glass)       |   |
|   |   +---------------------------------------------+   |   |
|   |   | 2. Inner Fill: Solid Color Stop or Glass    |   |   |
|   |   |   +-------------------------------------+   |   |   |
|   |   |   | 3. Center: SVG Checkmark [ v ]      |   |   |   |
|   |   |   +-------------------------------------+   |   |   |
|   |   +---------------------------------------------+   |   |
|   +-----------------------------------------------------+   |
+-------------------------------------------------------------+
```

### 4.1 Node States

1. **Filled Node State (Current Quota $\ge$ Milestone %):**
   - **Background Fill:** Matches the spatial scale stop color (`#ef4444`, `#f97316`, `#eab308`, `#10b981`, `#1af18d`).
   - **Border:** 1.5px solid border in corresponding color stop with subtle highlight.
   - **Checkmark Icon (`v`):** Illuminated white checkmark SVG (`text-white drop-shadow-[0_1px_1px_rgba(0,0,0,0.5)]`).
   - **Box Shadow:** Luminous depth glow (`box-shadow: 0 0 8px rgba(26, 241, 141, 0.6), inset 0 1px 2px rgba(255, 255, 255, 0.4)`).

2. **Unfilled Node State (Current Quota $<$ Milestone %):**
   - **Background Fill:** Dark-glass surface (`bg-[#171025]/90` or `bg-slate-800/80`).
   - **Border:** Subtle dark border (`border-[#40285e]` or `border-slate-700/60`).
   - **Center Pip:** Faint hollow dot or dim checkmark outline with `opacity: 0.25`.
   - **Box Shadow:** Subtle inset shadow (`box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4)`).

### 4.2 SVG Checkmark (`v`) Specification

```svg
<svg 
    class="w-3 h-3 text-white pointer-events-none" 
    viewBox="0 0 16 16" 
    fill="none" 
    stroke="currentColor" 
    stroke-width="2.2" 
    stroke-linecap="round" 
    stroke-linejoin="round"
>
    <path d="M3.5 8.5L6.5 11.5L12.5 4.5" />
</svg>
```

---

## 5. Opacity & Table Row Hover Glow Architecture

### 5.1 Resting State (Default)
- **Container Opacity:** Default opacity is set between **80% and 85%** (`opacity: 0.82;` / `opacity-80` to `opacity-85`).
- **Visual Rationale:** Prevents vibrant fluorescent greens and fiery reds from causing visual fatigue or overwhelming tabular text when the user is reading account names, email addresses, or session tokens.

### 5.2 Table Row Hover State (`tr:hover` / `.group:hover`)
- **Interactive Trigger:** When the cursor enters any parent table row (`tr`), instance card (`AccountCard`), or the progress container itself.
- **Opacity Transition:** Transitions instantaneously to **100% opacity** (`opacity: 1.0;`).
- **Glow Activation (`agm-progress-glow`):**
  - The fill bar radiates an ambient neon aura:
    ```css
    filter: drop-shadow(0 0 8px rgba(26, 241, 141, 0.70));
    ```
  - The highest active milestone node emits an intensified focal glow:
    ```css
    box-shadow: 0 0 12px rgba(26, 241, 141, 0.85), 0 0 4px #ffffff;
    ```
- **Transition Duration:** Smooth CSS easing (`transition: opacity 180ms cubic-bezier(0.16, 1, 0.3, 1), filter 180ms ease, box-shadow 180ms ease`).

---

## 6. Standalone Reference Blueprint (HTML/CSS)

Below is the complete, self-contained, standalone reference implementation. It includes all mandatory element IDs (`agm-progress-container`, `agm-progress-track`, `agm-progress-fill`, `agm-progress-node-1` through `agm-progress-node-5`, `agm-progress-glow`) so any autonomous agent, blind AI, or frontend developer can replicate it with 100% fidelity.

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>AGM Progress Bar Blueprint Reference</title>
  <style>
    /* CSS Variables & Reset */
    :root {
      --agm-bg-dark: #090611;
      --agm-surface-dark: #171025;
      --agm-border-dark: #40285e;
      
      /* Quota Stops */
      --agm-stop-0:   #7f1d1d;
      --agm-stop-15:  #dc2626;
      --agm-stop-35:  #ea580c;
      --agm-stop-55:  #eab308;
      --agm-stop-75:  #10b981;
      --agm-stop-100: #1af18d;
      
      /* Spatial Gradient */
      --agm-full-gradient: linear-gradient(
        90deg,
        var(--agm-stop-0) 0%,
        var(--agm-stop-15) 15%,
        var(--agm-stop-35) 35%,
        var(--agm-stop-55) 55%,
        var(--agm-stop-75) 75%,
        var(--agm-stop-100) 100%
      );
    }

    body {
      background-color: var(--agm-bg-dark);
      color: #f8fafc;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      padding: 40px;
      display: flex;
      flex-direction: column;
      gap: 30px;
    }

    /* Container: 80-85% resting opacity, transitions to 100% on hover */
    #agm-progress-container {
      position: relative;
      width: 100%;
      max-width: 460px;
      padding: 12px 0;
      opacity: 0.82;
      transition: opacity 200ms ease, filter 200ms ease;
      cursor: pointer;
    }

    #agm-progress-container:hover {
      opacity: 1.0;
    }

    /* Track: exactly 10px height (h-2.5) */
    #agm-progress-track {
      position: relative;
      width: 100%;
      height: 10px;
      background-color: var(--agm-surface-dark);
      border: 1px solid var(--agm-border-dark);
      border-radius: 9999px;
      overflow: hidden;
      box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.5);
    }

    /* Fill Bar: spatial gradient matching 100% of parent width */
    #agm-progress-fill {
      position: absolute;
      top: 0;
      left: 0;
      bottom: 0;
      width: 78%; /* Example Active Progress: 78% */
      border-radius: 9999px;
      overflow: hidden;
      transition: width 300ms cubic-bezier(0.22, 1, 0.36, 1);
    }

    /* Inner gradient spanning 100% full track spatial width */
    #agm-progress-fill::before {
      content: "";
      position: absolute;
      top: 0;
      left: 0;
      width: 460px; /* Anchored to track width */
      height: 100%;
      background: var(--agm-full-gradient);
    }

    /* Table row hover ambient glow effect */
    #agm-progress-glow {
      position: absolute;
      inset: 0;
      border-radius: 9999px;
      pointer-events-none;
      opacity: 0;
      transition: opacity 200ms ease;
    }

    #agm-progress-container:hover #agm-progress-glow {
      opacity: 1;
      box-shadow: 0 0 16px rgba(26, 241, 141, 0.65), 0 0 6px rgba(26, 241, 141, 0.90);
    }

    /* 20px Diameter Protruding Milestone Nodes */
    .agm-progress-node {
      position: absolute;
      top: 50%;
      width: 20px;
      height: 20px;
      border-radius: 50%;
      transform: translate(-50%, -50%);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 10;
      pointer-events: none;
      transition: all 250ms ease;
      box-sizing: border-box;
    }

    /* Filled Node State */
    .agm-progress-node.filled {
      border: 1.5px solid #ffffff;
      box-shadow: 0 0 8px rgba(0, 0, 0, 0.4), inset 0 1px 2px rgba(255, 255, 255, 0.5);
    }

    #agm-progress-container:hover .agm-progress-node.filled {
      box-shadow: 0 0 10px rgba(26, 241, 141, 0.8), 0 0 4px #ffffff;
    }

    /* Unfilled Node State */
    .agm-progress-node.unfilled {
      background-color: var(--agm-surface-dark);
      border: 1.5px solid var(--agm-border-dark);
      box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    /* Checkmark SVG styling */
    .agm-progress-node svg {
      width: 11px;
      height: 11px;
      stroke: #ffffff;
      stroke-width: 2.2;
      stroke-linecap: round;
      stroke-linejoin: round;
      fill: none;
    }

    .agm-progress-node.unfilled svg {
      stroke: rgba(255, 255, 255, 0.2);
    }

    /* Explicit Milestone Node Positioning */
    #agm-progress-node-1 { left: 20%; background-color: #ef4444; }
    #agm-progress-node-2 { left: 40%; background-color: #f97316; }
    #agm-progress-node-3 { left: 60%; background-color: #eab308; }
    #agm-progress-node-4 { left: 80%; background-color: var(--agm-surface-dark); } /* Unfilled at 78% */
    #agm-progress-node-5 { left: 90%; background-color: var(--agm-surface-dark); } /* Unfilled at 78% */
  </style>
</head>
<body>

  <!-- Example 1: Active Progress Gauge at 78% -->
  <div id="agm-progress-container">
    <!-- 10px Track -->
    <div id="agm-progress-track">
      <!-- Active Fill -->
      <div id="agm-progress-fill"></div>
      <!-- Hover Ambient Glow -->
      <div id="agm-progress-glow"></div>
    </div>

    <!-- 5 Internal Milestone Nodes (20px Diameter, Protruding 5px Top & Bottom) -->
    <!-- Node 1: 20% (Filled) -->
    <div id="agm-progress-node-1" class="agm-progress-node filled" title="Milestone 20%">
      <svg viewBox="0 0 16 16"><path d="M3.5 8.5L6.5 11.5L12.5 4.5" /></svg>
    </div>

    <!-- Node 2: 40% (Filled) -->
    <div id="agm-progress-node-2" class="agm-progress-node filled" title="Milestone 40%">
      <svg viewBox="0 0 16 16"><path d="M3.5 8.5L6.5 11.5L12.5 4.5" /></svg>
    </div>

    <!-- Node 3: 60% (Filled) -->
    <div id="agm-progress-node-3" class="agm-progress-node filled" title="Milestone 60%">
      <svg viewBox="0 0 16 16"><path d="M3.5 8.5L6.5 11.5L12.5 4.5" /></svg>
    </div>

    <!-- Node 4: 80% (Unfilled at 78%) -->
    <div id="agm-progress-node-4" class="agm-progress-node unfilled" title="Milestone 80%">
      <svg viewBox="0 0 16 16"><path d="M3.5 8.5L6.5 11.5L12.5 4.5" /></svg>
    </div>

    <!-- Node 5: 90% (Unfilled at 78%) -->
    <div id="agm-progress-node-5" class="agm-progress-node unfilled" title="Milestone 90%">
      <svg viewBox="0 0 16 16"><path d="M3.5 8.5L6.5 11.5L12.5 4.5" /></svg>
    </div>
  </div>

</body>
</html>
```

---

## 7. React Component Integration Guidelines

When updating `src/components/accounts/QuotaProgressBar.tsx` and `src/components/common/WaterDrainProgressBar.tsx`:

1. **Height Prop Modernization:**
   - Update default `heightClassName` from `h-[18px]` to `h-2.5` (10px).
   - In compact table density mode, maintain `h-2` or `h-2.5`.
2. **Node Sizing:**
   - Set node dimensions to `w-5 h-5` (20px) with `top-1/2 -translate-y-1/2 -translate-x-1/2`.
   - In compact mode, adjust to `w-4 h-4` (16px) with an 8px track height.
3. **Internal Checkmark Rendering:**
   - Embed the inline SVG `<path d="M3.5 8.5L6.5 11.5L12.5 4.5" />` inside the milestone node mapping loop.
   - Conditionally render filled styling (`scaleColor`, white stroke) when `clamped >= checkpoint`.
4. **Boundary Filtering:**
   - Strictly filter out `0` and `100` from `activeCheckpoints`:
     ```typescript
     const internalCheckpoints = (checkpoints || [20, 40, 60, 80, 90])
       .filter((cp) => cp > 0 && cp < 100)
       .slice(0, 5);
     ```
5. **Table Row Hover Group Binding:**
   - Apply `group` class to parent `<tr className="... group">` in `AccountRow.tsx` and `InstanceRowCells.tsx`.
   - Apply `group-hover:opacity-100 group-hover:shadow-[0_0_12px_rgba(26,241,141,0.6)]` to the progress bar container.
