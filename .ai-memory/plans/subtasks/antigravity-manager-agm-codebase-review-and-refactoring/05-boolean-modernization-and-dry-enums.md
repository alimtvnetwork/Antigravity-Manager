# Subtask 05: Boolean Modernization & DRY Enums Centralization

**Parent Plan:** `.ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md`  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md)  
**Owner:** Worker 02  
**Status:** Queued  

---

## 1. Owned Files
- `src/pages/instances/InstanceCard.tsx`
- `src/pages/instances/instanceCardData.ts`
- `src/pages/audit/audit-helpers.tsx`
- `src/types/audit.ts`
- `src/types/instance.ts` (or `src/pages/instances/instancePageTypes.ts`)

---

## 2. Implementation Steps

### 2.1 Centralizing Shared Enums
1. In `src/types/instance.ts`:
   - Declare `InstanceRunState` enum:
     ```typescript
     export enum InstanceRunState {
         Running = 'RUNNING',
         Idle = 'IDLE',
         Queued = 'QUEUED',
         Starting = 'STARTING',
         Stopping = 'STOPPING',
     }
     ```
   - Declare `RotationCondition` enum:
     ```typescript
     export enum RotationCondition {
         QuotaExhausted = 'QUOTA_EXHAUSTED',
         SiblingRunningConflict = 'SIBLING_RUNNING_CONFLICT',
         ManualTrigger = 'MANUAL_TRIGGER',
         ScheduledCron = 'SCHEDULED_CRON',
         FastForward = 'FAST_FORWARD',
     }
     ```
2. In `src/types/audit.ts`:
   - Declare numeric and string `AuditActionCode` enum:
     ```typescript
     export enum AuditActionCode {
         AddAccount = 1,
         UpdateAccount = 2,
         SwitchAccount = 3,
         SchedulePrompt = 4,
         RequeueConversation = 5,
         FastForwardInstance = 6,
         DeleteAccount = 7,
     }
     ```

### 2.2 Nested JSX Ternary Flattening in `InstanceCard.tsx`
1. Deconstruct compound nested ternary operators:
   ```tsx
   // Replace:
   {geminiModel || weeklyQuota ? (...) : boundAccount ? (...) : (...)}
   
   // With extracted helper function:
   function renderInstanceQuotaSection(data: InstanceCardData, refreshQuota: (id: string) => void) {
       if (data.geminiModel || data.weeklyQuota) {
           return <InstanceModelQuotaView geminiModel={data.geminiModel} weeklyQuota={data.weeklyQuota} />;
       }
       if (data.boundAccount) {
           return <InstanceUnsyncedQuotaView account={data.boundAccount} onSync={() => refreshQuota(data.boundAccount!.id)} />;
       }
       return <InstanceUnboundPlaceholderView />;
   }
   ```
2. Flatten nested default button state:
   - Extract `renderDefaultBadgeOrButton(isDefault: boolean, isBusy: boolean, onSetDefault: () => void)`.

### 2.3 Refactoring Multi-Condition Ifs in `audit-helpers.tsx`
1. Refactor `getActionLabel(item: TaskRecord)`:
   - Use `switch (item.action_code)` against `AuditActionCode` enum.
   - For legacy string actions, match against normalized uppercase or map lookup.
2. Refactor `isSchedulerAction` and `isSwitchAction`:
   - Replace long chained `||` clauses with `Set` lookup or enum matching:
     ```typescript
     const SCHEDULER_CODES = new Set([AuditActionCode.SchedulePrompt, AuditActionCode.RequeueConversation]);
     ```

### 2.4 Positive Boolean Invariant Enforcement
1. Audit and refactor boolean variables across `instanceCardData.ts` and `InstanceCard.tsx`:
   - Ensure all flags express positive presence (`isRunning`, `hasActiveTask`, `isEmailRevealed`, `isDefaultProfile`).
   - Eliminate negative flags and inverted double negatives.

---

## 3. Verification & Quality Gates
- `npm run build` compiles with zero TypeScript errors.
- Linters confirm zero nested JSX ternaries in `InstanceCard.tsx`.
- Magic string comparisons replaced by typed enum checks.
