---
name: agm-ui-modal-focus-accessibility
description: Specialized skill for managing UI modal exclusivity, explicit dismissal controls (X close buttons, Escape key), high-contrast focus/selection styling, and automated scroll-into-view behavior in Antigravity-Manager React frontend.
---

# AGM UI Modal Exclusivity, Focus & Accessibility Architecture

Governs modal lifecycle management, dismissal controls, contrast accessibility in dark themes, and automated element scrolling across Antigravity-Manager (`src/components/`).

---

## 1. Architectural Invariants

### 1.1 Invariant I1: Modal Exclusivity & Single-Active State
- No two floating modals may be open simultaneously unless explicitly designed as parent-child dialogs.
- Opening [`InstanceSelector`](src/components/navbar/InstanceSelector.tsx) must dismiss [`AgyCleanModal`](src/components/modals/agy-clean-modal.tsx) and [`UnifiedBackupModal`](src/components/modals/UnifiedBackupModal.tsx), and vice versa.
- All floating modals must implement:
  1. An explicit Close (`X`) button in the header bar.
  2. Outside click detection via document `mousedown` listener.
  3. `Escape` key capture listener for immediate dismissal.

### 1.2 Invariant I2: High-Contrast Selection & Dark-Theme Legibility
- Dark-on-dark active states (e.g. `bg-slate-800` on `text-slate-700`) are strictly forbidden.
- Selected or active items in dropdown lists must feature distinct visual indicators:
  - High-contrast background (e.g. bright blue/indigo accent or pure white in light mode).
  - Clear text contrast (minimum WCAG AA 4.5:1 ratio).
  - Prominent active badge (e.g. emerald/cyan pill indicator).
  - Active item border highlight (e.g. `border-l-4 border-l-amber-500` or `border-primary`).

### 1.3 Invariant I3: Automated Scroll-Into-View on Focus/Selection
- When an instance or profile is set active or focused via keyboard or click, the UI must automatically trigger:
  ```typescript
  elementRef.current?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  ```
- Ensures selected instances or accounts are immediately visible in long scrollable lists without requiring manual scrolling.

---

## 2. Key Implementation Files

- [`src/components/navbar/InstanceSelector.tsx`](src/components/navbar/InstanceSelector.tsx): Profile selector dropdown & modal with instance management controls.
- [`src/components/modals/agy-clean-modal.tsx`](src/components/modals/agy-clean-modal.tsx): Conversation cleaner & pruner modal.
- [`src/components/modals/UnifiedBackupModal.tsx`](src/components/modals/UnifiedBackupModal.tsx): Unified backup and restore modal.
- [`src/pages/Accounts.tsx`](src/pages/Accounts.tsx): Account grid/table with automated `handleFocusActiveAccount()` smooth scrolling and pulse highlights.
- [`src/components/common/ThemeManager.tsx`](src/components/common/ThemeManager.tsx): Master theme synchronization and native window color manager.

---

## 3. Dropdown & Modal Exclusivity Protocol

Dropdowns broadcast custom events to ensure only one navigation dropdown or floating panel is active:
```typescript
// On opening
window.dispatchEvent(
  new CustomEvent('agm:dropdown-open', { detail: { source: 'instance-selector' } })
);

// Listener in sibling dropdowns/modals
useEffect(() => {
  const handleDropdownOpen = (e: Event) => {
    const customEvent = e as CustomEvent<{ source?: string }>;
    if (customEvent.detail?.source !== 'instance-selector') {
      setIsOpen(false);
    }
  };
  window.addEventListener('agm:dropdown-open', handleDropdownOpen);
  return () => window.removeEventListener('agm:dropdown-open', handleDropdownOpen);
}, []);
```

---

## 4. Verification Checklist

When updating or creating UI components:
- [ ] Header bar contains an explicit `X` button with `cursor-pointer` and accessible aria-label.
- [ ] Pressing `Escape` closes the active modal cleanly.
- [ ] Clicking outside the modal container triggers clean dismissal.
- [ ] Selected profile/account is clearly distinguished with high contrast across both light and dark themes.
- [ ] Active profile automatically scrolls into view when opening the selector.
- [ ] Zero modal overlapping or stacked un-dismissible panels.
