---
plan: ui-and-functionality-enhancements
subtask: "01"
title: Window TitleBar Maximize Interaction Fix and RecordingIndicator Compact Status Capsule
domain: frontend-ui
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/ui-and-functionality-enhancements/01-architecture-and-ui-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/layout/TitleBar.tsx
  - src/components/navbar/Navbar.tsx
  - src/components/audit/RecordingIndicator.tsx
status: pending
---

# Subtask 01: Window TitleBar Maximize Interaction Fix and RecordingIndicator Compact Status Capsule

## 1. Context & Motivation
Users reported two persistent usability friction points in the desktop window chrome and audit indicator:
1. **Window Header Maximize Double-Click Failure**: Double-clicking on empty spaces in the TitleBar or Navbar frequently fails to toggle the window between maximized and unmaximized states. Investigation revealed that the `onMouseDown` handler calls `getCurrentWindow().startDragging()` unconditionally on `e.detail === 1`. On Windows (Win32), `startDragging()` activates a native modal drag loop that consumes mouse capture, swallowing the subsequent click of a double-click gesture and preventing `onDoubleClick` from firing in WebView2.
2. **Audit Indicator Horizontal & Vertical Bulk**: The audit `RecordingIndicator` in the navbar currently lacks height constraints, has excessive padding (`py-1`), and uses inconsistent text sizing. It must be compacted into a sleek 28px (`h-7`) dark-glass segmented pill capsule matching the project's house design conventions.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/layout/TitleBar.tsx` | `TitleBar`, `handleMouseDown`, `handleDoubleClick` | Add `if (e.detail > 1) return;` guard to `onMouseDown`, ensure `onDoubleClick` calls `handleToggleMaximize()` cleanly. |
| `src/components/navbar/Navbar.tsx` | `Navbar`, `handleMouseDown`, `handleDoubleClick` | Add `if (e.detail > 1) return;` guard to `handleMouseDown`, invoke `toggle_maximize_window` IPC on double-click with fallback. |
| `src/components/audit/RecordingIndicator.tsx` | `RecordingIndicator` | Constrain container to `h-7`, apply `bg-slate-800/80 dark:bg-slate-900/90 backdrop-blur-md border border-slate-700/60 divide-x divide-slate-700/60`, set typography to `text-[10px]`. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Update TitleBar Drag and Double-Click Handling in `src/components/layout/TitleBar.tsx`
1. Locate the `<header>` element with `data-tauri-drag-region`.
2. Extract the inline `onMouseDown` and `onDoubleClick` handlers into clean, resilient callbacks:
   ```tsx
   const handleHeaderMouseDown = (e: React.MouseEvent<HTMLElement>) => {
       if ((e.target as HTMLElement).closest('button, input, select, a, .no-drag')) return;
       // Skip dragging on multi-click gestures so that onDoubleClick reaches the DOM
       if (e.detail > 1) return;
       try {
           getCurrentWindow().startDragging();
       } catch (err) {
           useErrorStore.getState().trackWarning(err, {
               source: 'TitleBar.handleHeaderMouseDown',
               triggerAction: 'startDragging',
           });
       }
   };

   const handleHeaderDoubleClick = async (e: React.MouseEvent<HTMLElement>) => {
       if ((e.target as HTMLElement).closest('button, input, select, a, .no-drag')) return;
       await handleToggleMaximize();
   };
   ```
3. Bind `onMouseDown={handleHeaderMouseDown}` and `onDoubleClick={handleHeaderDoubleClick}` to `<header>`.

### 3.2 Step 2: Update Navbar Drag and Double-Click Handling in `src/components/navbar/Navbar.tsx`
1. In `Navbar.tsx`, update `handleMouseDown`:
   ```tsx
   const handleMouseDown = (e: React.MouseEvent<HTMLElement>) => {
       if (!isTauri()) return;
       const target = e.target as HTMLElement;
       if (target.closest('.no-drag, button, a, input, select, textarea')) return;
       // Prevent dragging when a double-click is in progress
       if (e.detail > 1) return;
       try {
           getCurrentWindow().startDragging();
       } catch (err) {
           useErrorStore.getState().trackWarning(err, {
               source: 'Navbar.handleMouseDown',
               triggerAction: 'startDragging',
           });
       }
   };
   ```
2. In `Navbar.tsx`, update `handleDoubleClick` to use the primary `toggle_maximize_window` Tauri command:
   ```tsx
   const handleDoubleClick = async (e: React.MouseEvent<HTMLElement>) => {
       if (!isTauri()) return;
       const target = e.target as HTMLElement;
       if (target.closest('.no-drag, button, a, input, select, textarea')) return;
       try {
           await invoke('toggle_maximize_window');
       } catch {
           try {
               await getCurrentWindow().toggleMaximize();
           } catch (err) {
               useErrorStore.getState().captureError(err, {
                   source: 'Navbar.tsx',
                   triggerAction: 'handleDoubleClick.toggleMaximize',
               });
           }
       }
   };
   ```

### 3.3 Step 3: Compact `RecordingIndicator` into 28px Capsule in `src/components/audit/RecordingIndicator.tsx`
1. Replace the existing container styles with:
   ```tsx
   <div
       className="inline-flex items-center h-7 rounded-full border border-slate-700/60 bg-slate-800/80 dark:bg-slate-900/90 backdrop-blur-md divide-x divide-slate-700/60 shadow-xs text-[10px] font-mono select-none"
       role="group"
       aria-label={t('audit.indicator_label', 'Audit recording controls')}
   >
       <button
           type="button"
           onClick={() => void toggleRecording()}
           title={isRecording ? t('audit.stop_recording', 'Stop audit recording') : t('audit.start_recording', 'Start audit recording')}
           className="px-2.5 h-full flex items-center gap-1.5 text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-l-full transition-colors cursor-pointer"
       >
           <span
               className={cn(
                   "w-1.5 h-1.5 rounded-full shrink-0 transition-all",
                   isRecording
                       ? "bg-red-500 animate-pulse shadow-[0_0_6px_rgba(239,68,68,0.7)]"
                       : "bg-slate-500"
               )}
           />
           <span className={isRecording ? "text-red-300 font-semibold" : "text-slate-400"}>
               {isRecording ? `REC ${elapsed}` : 'IDLE'}
           </span>
       </button>
       <button
           type="button"
           onClick={() => setPanelOpen(true)}
           title={t('audit.open_panel', 'Open audit log')}
           className="px-2 h-full flex items-center gap-1 text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-r-full transition-colors cursor-pointer"
       >
           <ScrollText className="w-3 h-3 text-slate-400" />
           <span>{t('audit.log', 'Log')}</span>
       </button>
   </div>
   ```

---

## 4. Visual & DOM Invariants
- The root `<header>` of `TitleBar.tsx` must maintain `data-tauri-drag-region`.
- The `RecordingIndicator` container MUST strictly have `h-7` (`28px` height) and `rounded-full` border radius.
- Buttons inside `RecordingIndicator` must have `h-full` so the hover state fills the capsule height uniformly.
- No bare `catch {}` blocks; all fallible Tauri calls must route errors to `useErrorStore.getState().trackWarning()`.

---

## 5. Verification Checklist
- [ ] Rapidly double-click TitleBar background: window transitions smoothly between maximized and unmaximized.
- [ ] Rapidly double-click Navbar background: window transitions smoothly between maximized and unmaximized.
- [ ] Single-click and drag on TitleBar: window drags normally without latency.
- [ ] Single-click and drag on Navbar: window drags normally without latency.
- [ ] Click minimize button: window minimizes.
- [ ] Click maximize button: window toggles maximize.
- [ ] Click close button: window closes.
- [ ] Audit indicator renders with height 28px in navbar.
- [ ] Click "IDLE": toggles recording, dot turns pulsing red, timer counts `REC MM:SS`.
- [ ] Click "Log": opens audit history drawer.
- [ ] Pre-flight checks: `npm run build` passes with zero TypeScript errors.
