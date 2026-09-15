# Subtask 02: Frontend Instance Store, Navbar Controls & Instances Tab

> **Subtask Path:** `.lovable/plans/subtasks/14-multi-instance/02-frontend-instance-store-and-tabs.md`
> **Parent Plan:** [.lovable/plans/pending/14-multi-instance-orchestration-and-ubuntu-parallelism.md](../../pending/14-multi-instance-orchestration-and-ubuntu-parallelism.md)
> **Status:** Pending Review

---

## 1. Objectives

1. Create `src/store/useInstanceStore.ts` using Zustand to manage instance state, active selection, and asynchronous IPC calls.
2. Update `src/components/navbar/Navbar.tsx`:
   - Add new navigation item `Instances` linking to the management tab.
   - Embed the instance dropdown selector (`Default`, `Instance 1`, `Instance 2`).
   - Add the quick `Copy Instance` button next to the dropdown.
3. Create `src/pages/Instances.tsx`:
   - Grid/list view displaying all instances with their status, bound account, and resource path.
   - "New Instance" dialog and "Clone Instance" action.
   - Direct launch/focus and terminate controls.

---

## 2. Technical Contracts

```typescript
// In src/store/useInstanceStore.ts
export interface AntigravityInstance {
    id: string;
    name: string;
    is_default: boolean;
    profile_path: string;
    assigned_account_id?: string;
    is_running: boolean;
    pid?: number;
    last_launched_at?: number;
}
```

---

## 3. Acceptance Criteria

- [ ] Instance selector accurately updates active instance state.
- [ ] Clicking `Copy Instance` prompts for a name and clones the active instance immediately.
- [ ] Instances page renders cleanly with responsive DaisyUI/Tailwind design tokens.
