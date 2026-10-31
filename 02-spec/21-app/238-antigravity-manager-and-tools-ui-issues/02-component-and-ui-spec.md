# 238 - Antigravity Manager and Tools UI Issues: Component & UI Specification

## 1. UI Design Principles & System Conventions

### 1.1 Contiguous Segmented Pill Capsules
All action button groups in headers, toolbars, and cards must follow the segmented pill standard:
- **Geometry**: `rounded-full`, overflow hidden, contiguous border `border border-border/50`.
- **Background**: Dark-glass gradient `bg-surface-2/80 backdrop-blur-md hover:bg-surface-3/90 transition-all`.
- **Segment Dividers**: Subtle vertical dividers between primary action and dropdown chevron `border-l border-border/40`.
- **Typography & Icons**: `text-xs font-medium tracking-tight`, icon size `w-3.5 h-3.5`.
- **Zero Pill-in-Pill Defect**: Inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`).

---

## 2. Component Specifications

### 2.1 PromptTreeView DetailHeader Harmonization
- **File**: `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Current Defect**:
  - Capsule A (Copy): uses `rounded-lg` with `rounded-l-md` / `rounded-r-md` and inconsistent horizontal padding.
  - Capsule B (Send/Enqueue/Rerun): uses `rounded-full` with `rounded-l-full` / `rounded-r-full`.
  - Attached Images `[Copy]` button: uses un-styled border button.
- **Harmonized Spec**:
  - Both Capsule A and Capsule B adopt `rounded-full` enclosing borders.
  - Height fixed to `h-8` with balanced inner padding (`px-3 py-1`).
  - Active hover states use standard glow tokens (`hover:border-accent-cyan/40 hover:text-white`).
  - Attached Images copy button receives micro-pill capsule styling (`rounded-full px-2.5 py-0.5 text-2xs`).

### 2.2 Instance-Scoped Improvements Flywheel
- **Component**: `src/components/instances/InstanceImprovementsSection.tsx`
- **Location**: Mounted within `InstanceCard.tsx` under a dedicated collapsible tab/panel (`✨ Improvements`).
- **Controls**:
  - **Batch Size Dropdown**: Select `[5, 10, 20, 50]` improvements (Default: `20`).
  - **Action Buttons**:
    - `Plan Improvements`: Prepares the improvement sequence from coding guideline prompts.
    - `Auto-Enqueue (20)`: Submits the selected quantity of improvement prompts directly to the instance's active Antigravity IDE queue via CDP.
    - `Next 10`: Increments execution of next pending improvement batch.
  - **Status Overview**: Displays completed vs pending improvement count and active ledger file path.

### 2.3 Post-Switch Conversation Privacy Shield
- **Component**: `src/components/instances/PromptTreeView/SettlingMaskShield.tsx`
- **Behavior**:
  - When account switch settling timer is running (5–10s countdown), overlays or replaces raw prompt text with a privacy shield card.
  - Animated pulsing cyan badge: `⏳ Securing session tokens & restoring prompt in {secondsLeft}s...`
  - Prevents credential or prompt token leakage on screen while the new session is authenticating and recovering state.

### 2.4 Fleet Machines Live Status Banner
- **Component**: `src/components/instances/fleet/FleetSection.tsx` & `useFleetMachines.ts`
- **Behavior**:
  - Automatically loads cluster SSH nodes on mount without requiring manual refresh clicks.
  - Displays host machine IP, region, CPU/RAM utilization, active instance count, and SSH round-trip latency.
  - Visual pulse indicator: Green (`Online`), Amber (`Syncing`), Red (`Unreachable`).
