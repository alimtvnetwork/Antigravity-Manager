# Subtask Plan 02: Profile Auto-Selection Urgency Scoring and UI Glitch Remediation

**Document ID:** `.ai-memory/plans/subtasks/226-antigravity-manager-clone-and-ui-enhancements/02-subtask-profile-selection-and-ui-glitches.md`  
**Parent Task:** `226-antigravity-manager-clone-and-ui-enhancements` (Tasks 05 & 06)  
**Parent Spec:** `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/02-component-spec.md`  
**Parent Plan:** `.ai-memory/plans/pending/226-antigravity-manager-clone-and-ui-enhancements.md`  
**Status:** READY FOR EXECUTION  
**Version:** 1.0.0  
**Updated:** 2026-10-25  

---

## 1. Context & Objectives

This subtask plan details the surgical implementation steps for:
1. **Task-05 (Account Auto-Selection & Quota Expiration Urgency):** Inverting the flawed headroom multiplication logic in `src-tauri/src/modules/auto_switcher/scoring.rs:168-170` that penalized expiring accounts by up to $\approx 81.65\times$. Implementing the formal Expiration Urgency Multiplier ($E_{\text{urgency}}$) and Two-Tier candidate partitioning so that accounts with $T_{\text{remaining}} \le 24\text{h}$ and usable weekly quota ($\ge 15\%$) are prioritized over fresh accounts with days of headroom.
2. **Task-06 (UI Glitch Remediation & GPU Compositing Optimization):** Eliminating the Windows DWM `(Not Responding)` ghost window and washed-out backdrop freeze seen in Screenshot 04 by introducing a bounded concurrency pool (concurrency: 3) for base64 image reads, removing nested `backdrop-blur-2xl` on inner modal dialogs, and hardening `loadTree` with monotonic request IDs and 30-second polling cooldowns.

---

## 2. File Ownership Map

This subtask has exclusive ownership of the following 9 files:

| File Path | Component / Role | Exact Line Ranges | Nature of Changes |
| :--- | :--- | :--- | :--- |
| `src-tauri/src/modules/auto_switcher/scoring.rs` | Rust Backend Scoring | `140–215`, `168–171` | Add `compute_expiration_urgency_multiplier`; replace defective `eff_pct * remaining_hours` with urgency-scaled scoring. |
| `src-tauri/src/modules/auto_switcher/selection.rs` | Rust Backend Candidate Selection | `280–310`, `355–375`, `480–525` | Add `is_urgent_expiration_candidate`; partition candidates into Tier 1 (Urgent, sorted ascending by $T_{\text{remaining}}$) and Tier 2 (Standard, sorted descending by score). |
| `src/services/instanceService/scoring.ts` | Frontend Scoring Service | `150–195`, `265–325`, `370–400` | Implement `computeExpirationUrgencyMultiplier`; add `extractWeeklyRemainingHours`; integrate $E_{\text{urgency}}$ into primary and fallback score calculations. |
| `src/utils/accountAlgorithmScore.ts` | Frontend Algorithm Score Helper | `20–45`, `140–210` | Extend `AccountAlgorithmScoreResult` with `urgencyMultiplier`, `isUrgentExpiration`, and `remainingHoursUntilReset`; update formula string and math breakdown terms. |
| `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` | Accounts Table Score Cell | `115–135`, `140–185` | Add urgent expiration pill badge (`⚡ Expiring in ${hours}h`); display urgency multiplier and expiration notice in tooltip breakdown. |
| `src/components/instances/PromptTreeView/imageLoaderPool.ts` | NEW Frontend Concurrency Pool | Entire File (New) | Bounded worker pool (max 3 concurrent IPC calls), in-memory LRU cache (200 items), and queue-clearing on unmount. |
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Prompt Turn Inspector | `215–260`, `270–295` | Replace direct unthrottled `invoke('read_image_as_base64')` calls with `imageLoaderPool.loadBase64(path)`. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | Main Modal Orchestrator | `895–915` | Remove nested `backdrop-blur-2xl` on modal container; replace with solid high-performance dark-glass styling `dark:bg-[#0c0817]`. |
| `src/components/instances/PromptTreeView/usePromptTreeData.ts` | Tree Data & Polling Hook | `538–605`, `730–750` | Add `latestRequestIdRef` to drop stale out-of-order IPC responses; enforce strict 30s auto-sync debounce guard. |

---

## 3. Step-by-Step Implementation Tasks

### Phase 1: Rust Backend Quota Urgency Scoring (`scoring.rs`)

- [ ] **1.1 Implement `compute_expiration_urgency_multiplier`:**
  - File: `src-tauri/src/modules/auto_switcher/scoring.rs`
  - Insert public helper function:
    ```rust
    pub fn compute_expiration_urgency_multiplier(
        weekly_pct: f64,
        remaining_hours: f64,
        weekly_floor: f64,
    ) -> f64 {
        if weekly_pct < weekly_floor {
            return 1.0;
        }
        if remaining_hours <= 24.0 && remaining_hours >= 0.0 {
            let fraction = (24.0 - remaining_hours) / 24.0;
            5.0 + fraction * 5.0
        } else if remaining_hours <= 48.0 && remaining_hours > 24.0 {
            let fraction = (48.0 - remaining_hours) / 24.0;
            1.5 + fraction * 3.5
        } else {
            1.0
        }
    }
    ```
- [ ] **1.2 Remediate Weekly Score Calculation:**
  - File: `src-tauri/src/modules/auto_switcher/scoring.rs:168-171`
  - Replace legacy lines:
    ```rust
    // OLD:
    // let remaining_hours = compute_weekly_remaining_headroom_hours(b, now_sec);
    // gemini_weekly_scores.push(eff_pct * remaining_hours);

    // NEW:
    let remaining_hours = compute_weekly_remaining_headroom_hours(b, now_sec);
    let urgency = compute_expiration_urgency_multiplier(eff_pct, remaining_hours, weekly_zero_threshold);
    gemini_weekly_scores.push(eff_pct * urgency);
    ```
- [ ] **1.3 Add Comprehensive Unit Tests in `scoring.rs`:**
  - Verify that an account with 33% quota and 4.4h remaining scores $> 290.0$.
  - Verify that an account with 78% quota and 152h remaining scores $78.0$.
  - Verify that an account with 5% quota (below floor 15%) receives urgency multiplier $1.0$.

---

### Phase 2: Rust Backend Two-Tier Candidate Partitioning (`selection.rs`)

- [ ] **2.1 Implement `extract_urgent_expiration_hours`:**
  - File: `src-tauri/src/modules/auto_switcher/selection.rs`
  - Helper to extract minimum remaining weekly reset hours for accounts meeting the 24h threshold:
    ```rust
    pub(crate) fn extract_urgent_expiration_hours(acc: &Account, now_sec: i64) -> Option<f64> {
        let quota_data = acc.quota.as_ref()?;
        let groups = quota_data.quota_groups.as_ref()?;
        let mut min_hours: Option<f64> = None;
        for g in groups {
            for b in &g.buckets {
                if is_gemini_weekly_bucket(&b.window, &b.bucket_id) {
                    let pct = (b.remaining_fraction * 100.0).round();
                    if pct >= 15.0 {
                        let rem = compute_weekly_remaining_headroom_hours(b, now_sec);
                        if rem <= 24.0 {
                            min_hours = Some(min_hours.map_or(rem, |cur| cur.min(rem)));
                        }
                    }
                }
            }
        }
        min_hours
    }
    ```
- [ ] **2.2 Refactor Candidate Sorting in `select_candidate_profiles`:**
  - File: `src-tauri/src/modules/auto_switcher/selection.rs:480-525`
  - Partition `available_pool` into `urgent_pool` and `standard_pool`:
    - `urgent_pool`: Accounts where `extract_urgent_expiration_hours(acc, now_sec).is_some()`.
      Sorted **ascending by remaining hours** (least time remaining first). Tie-breakers: descending by score, then email.
    - `standard_pool`: All other available candidates.
      Sorted **descending by score**. Tie-breaker: email.
  - Return concatenated vector: `[urgent_pool, standard_pool]`.

---

### Phase 3: Frontend Quota Urgency Calculation (`scoring.ts`)

- [ ] **3.1 Add `computeExpirationUrgencyMultiplier`:**
  - File: `src/services/instanceService/scoring.ts`
  - Implement continuous piecewise multiplier matching the Rust backend ($5.0 \dots 10.0$ for $\le 24\text{h}$, $1.5 \dots 5.0$ for $\le 48\text{h}$).
- [ ] **3.2 Add `extractWeeklyRemainingHours`:**
  - File: `src/services/instanceService/scoring.ts`
  - Inspect `quota_groups` for Gemini weekly buckets and parse `reset_time` into hours remaining.
- [ ] **3.3 Update `calculateMultiplicativeScore` & `calculateFallbackMultiplicativeScore`:**
  - Factor in $E_{\text{urgency}}$ multiplier when computing raw score:
    ```typescript
    const remainingHours = extractWeeklyRemainingHours(acc, nowMs);
    const urgencyMultiplier = computeExpirationUrgencyMultiplier(weeklyQuotaPercent, remainingHours);
    const rawScore = activeFactor * tierMultiplier * (weeklyQuotaPercent * urgencyMultiplier);
    ```

---

### Phase 4: Frontend Score Breakdown & UI Tooltip (`accountAlgorithmScore.ts` & `AlgorithmScoreCell.tsx`)

- [ ] **4.1 Extend `AccountAlgorithmScoreResult` Interface:**
  - File: `src/utils/accountAlgorithmScore.ts`
  - Add fields:
    ```typescript
    urgencyMultiplier: number;
    isUrgentExpiration: boolean;
    remainingHoursUntilReset: number | null;
    ```
- [ ] **4.2 Update Math Breakdown Terms:**
  - File: `src/utils/accountAlgorithmScore.ts`
  - When `isUrgentExpiration` is true:
    - Formula string: `Score = (S_active × M_tier × (Q_weekly × E_urgency)) / 1000`
    - Add term: `E_urgency = ${urgencyMultiplier.toFixed(2)}x (Reset in ${hours}h)`
- [ ] **4.3 Enhance `AlgorithmScoreCell.tsx` UI:**
  - File: `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`
  - Render an urgent indicator pill when `result.isUrgentExpiration`:
    ```tsx
    <span className="px-1.5 py-0.5 rounded text-[9px] font-bold font-mono bg-amber-500/20 text-amber-300 border border-amber-500/30 flex items-center gap-1">
        <span>⚡</span>
        <span>{result.remainingHoursUntilReset?.toFixed(1)}h</span>
    </span>
    ```
  - Display the urgency multiplier breakdown inside the portal tooltip.

---

### Phase 5: Bounded Concurrency Image Reader (`imageLoaderPool.ts` & `PromptInstructionView.tsx`)

- [ ] **5.1 Create `src/components/instances/PromptTreeView/imageLoaderPool.ts`:**
  - Implement `ImageLoaderPool` class:
    - Bounded task queue with `concurrency = 3`.
    - In-memory `Map<string, string>` LRU cache (capacity: 200 items).
    - `clearQueue()` method to purge pending promises on unmount.
    - Error routing through `useErrorStore.getState().trackWarning`.
- [ ] **5.2 Integrate Pool into `PromptInstructionView.tsx`:**
  - File: `src/components/instances/PromptTreeView/PromptInstructionView.tsx:215-260`
  - Replace raw `invoke('read_image_as_base64', { path: localPath })` with `imageLoaderPool.loadBase64(localPath)`.
  - In `handleImgError`, route through `imageLoaderPool.loadBase64(localPath)`.
  - Add cleanup hook `useEffect(() => () => imageLoaderPool.clearQueue(), [selectedConversationId])`.

---

### Phase 6: UI Glitch Remediation: GPU Blur Elimination & 30s Polling Stabilization

- [ ] **6.1 Remove Nested `backdrop-blur-2xl` on Modal Frame:**
  - File: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx:906`
  - Remove `backdrop-blur-2xl` from the modal dialog frame. Retain `dark:bg-[#0c0817] shadow-[0_0_50px_rgba(0,0,0,0.8)] border border-purple-900/40`.
- [ ] **6.2 Remove Nested Blurs in Inner Panels:**
  - Files: `InspectorModal.tsx`, `DetailsModal.tsx`, `TreePanel.tsx`
  - Replace `backdrop-blur-md` on child headers, toolbars, and badges with solid dark-glass classes (`dark:bg-[#120a1f]`).
- [ ] **6.3 Harden `loadTree` with Monotonic Request ID:**
  - File: `src/components/instances/PromptTreeView/usePromptTreeData.ts`
  - Add `latestRequestIdRef = useRef(0)`.
  - Discard response if `requestId !== latestRequestIdRef.current` when promise resolves.
- [ ] **6.4 Enforce Strict 30s Polling Cooldown:**
  - File: `src/components/instances/PromptTreeView/usePromptTreeData.ts:545-551`
  - Ensure `now - lastLoadTimeRef.current < TREE_CACHE_TTL_MS` immediately returns for all non-force background loads.

---

## 4. Concrete Verification Criteria & Quality Gates

### 4.1 Scoring & Selection Mathematical Verification
- **Test Case 1 (Expiring Account Priority):**
  - Given Account A (`shohagbazar004@gmail.com`, 33% weekly, 4.4h to reset) and Account B (`riseup.team.now@gmail.com`, 78% weekly, 152h to reset).
  - Executing `select_candidate_profiles` MUST rank Account A at Index 0.
  - Verification: Target test in `scoring.rs` asserts `candidates[0].email == "shohagbazar004@gmail.com"`.
- **Test Case 2 (Low Quota Safety Check):**
  - Given Account C (5% weekly, 2h to reset). Because $5\% < 15\%$, Account C receives no urgency boost and is NOT selected.

### 4.2 UI Concurrency & DWM Ghosting Prevention
- **Test Case 3 (Batch Image Concurrency Limit):**
  - When rendering 33 images in a prompt, the number of active in-flight IPC calls to `read_image_as_base64` at any timestamp $t$ MUST be $\le 3$.
  - In-memory cache hit rate on re-render MUST be $100\%$ ($0$ IPC calls on re-render).
- **Test Case 4 (Backdrop Filter Audit):**
  - Running a regex check across `src/components/instances/PromptTreeView/*.tsx` for `backdrop-blur-2xl` MUST return 0 occurrences.

### 4.3 Git Hygiene & Relative Paths Guard
- Run diffstat and verify zero absolute filesystem paths exist in all touched files. All references must strictly use repository-relative Git paths.
- Verify strict adherence to repository-relative Git paths.

---

## 5. Risk Mitigation & Rollback Procedures

| Risk / Failure Mode | Detection Signal | Mitigation / Rollback Action |
| :--- | :--- | :--- |
| Candidate sorting starves standard accounts when an expiring account is stuck | Switcher repeatedly selects an account that fails proxy health check | The proxy rate-limit and validation block filters in `selection.rs` run BEFORE candidate ranking. If the expiring account fails, it is automatically excluded. |
| In-flight base64 image reads consume excessive RAM | Heap snapshot increases $> 100\text{MB}$ | `imageLoaderPool` caps LRU cache at 200 items. Excess entries are evicted FIFO. |
| Stale tree data displayed during rapid switching | Tree panel displays old project conversations | Monotonic `latestRequestIdRef` cancels stale response application. |
