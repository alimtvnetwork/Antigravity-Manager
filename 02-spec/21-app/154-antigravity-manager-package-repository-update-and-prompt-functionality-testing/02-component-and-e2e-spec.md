# 02 — Component & E2E Spec: "Now Running + Up Next" panel, rerun/resend with image, evidence contract

Task: `antigravity-manager-package-repository-update-and-prompt-functionality-testing` (spec 154).
Scope of THIS file: new UI component spec (§1), rerun/resend with-image flows (§2),
test evidence contract incl. A4 resolution (§3), E2E catalog E2E-154-01…07 (§4),
acceptance criteria (§5), test strategy (§6).

Companion: `01-…` (Author 1 — data contracts, IPC shapes, queue ordering source of truth).
This file CONSUMES Author 1's contracts; it does not redefine them.
Citations like `(153)`, `(145)`, `(125)` refer to prior specs; `§already-covered` = out of scope here.

Module: `src/components/instances/PromptTreeView/`
Verified present: `PromptTreeViewModal.tsx`, `TreePanel.tsx`, `usePromptTreeData.ts`
(IPC `get_project_conversation_tree`, running-priority auto-selection, 15s–2m auto-sync, SWR cache),
`DetailHeader.tsx` (rerun flyout `rerunCount` min/max default 2, FIFO banner),
`usePromptTreePromptActions.ts` (`dispatchRerunMessages`, `handleResendPrompt`, `handleEnqueuePrompt`),
`ResendAsModal.tsx` (`target_mode: 'new' | 'current'`), `clipboard.ts`
(`base64ToBlob`, `read_image_as_base64` native Rust cmd, 100MB cap),
`conversationGuards.ts` (`isConversationOrTurnQueued`), `tierClassification.ts` (4-tier).

---

## 1. "Now Running + Up Next" panel (NEW UI — gap (a))

No dedicated running-first + next-4 panel exists today (verified: the only
`slice(0, 4)` in the repo is `src/components/accounts/QuotaProgressBar.tsx`,
unrelated). Running priority, auto-selection and QUEUED badges exist in the tree;
this panel is a NEW summary strip. Spec it as new; do not retrofit `TreePanel.tsx`
rendering.

### 1.1 Placement
- Docked strip at the top of the tree view, inside `PromptTreeViewModal.tsx`,
  directly above the `TreePanel.tsx` renderer. Collapsible (chevron); collapsed
  state persisted per session only (no DB write).
- Visible only when the tree view is open AND at least one prompt is running or
  queued. Hidden otherwise (no empty chrome).

### 1.2 Props / data source
- `items: RunningQueueItem[]` — ordered array from Author 1's data contract
  (running item at index 0, queued items FIFO after it; ghost/TTL-filtered upstream).
- `isLoading: boolean`, `lastSyncAt: number | null`.
- Callbacks: `onSelectPrompt(promptId: string)`, `onRerun(promptId: string)`,
  `onResendWithImage(promptId: string)`, `onRefresh()`.
- This component MUST NOT reimplement ordering, TTL, or ghost filtration —
  it renders exactly what the data hook returns (single source of truth).

### 1.3 Ordering and truncation
- Position 0: the running prompt. If more than one running prompt exists
  (multi-instance), position 0 = earliest `started_at`; the rest join the
  queued section in FIFO order (they are "up next" for their instances).
- Positions 1–4: first four queued prompts in FIFO queue order, i.e.
  `queued.slice(0, 4)`. FIFO here is the repo's hard invariant (AGENTS.md):
  queue queries MUST use `ORDER BY created_at ASC, id ASC`; inverted LIFO
  (`ORDER BY updated_at DESC`) is strictly forbidden in dispatch logic. The
  panel inherits FIFO from the data hook — it never re-sorts.
- Overflow: if queued count > 4, render a `+N more` affordance that expands
  inline (accordion) or deep-links to the queue section of the tree.
  Truncation MUST be visible-count only — every queued prompt stays reachable;
  silent drops are a defect.

### 1.4 Row content
- 4-tier origin badge (from `tierClassification.ts`) — preserved on every row
  with the EXACT repo taxonomy (AGENTS.md): `USER_PROMPT` (Sky badge, User icon),
  `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon), `SYSTEM_MESSAGE` (Zinc badge,
  Terminal icon), `TOOL_OUTPUT` (Amber badge, Wrench icon). Subagent rows nest
  under their parent with branch glyphs (`↳`, `└──`).
- Prompt title, instance name, status chip (RUNNING amber-pulse / QUEUED amber),
  queue position number (#1..#4), elapsed time for the running prompt.
- Truncated titles/badges MUST NOT render raw `<truncated …>` text — use the
  interactive dark-glass omission banner
  (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`).
- Row actions (select-in-tree, rerun, resend-with-image) rendered as a
  contiguous segmented pill capsule (`rounded-full`, shared border, subtle
  dividers, dark-glass) per repo UI convention — never loose disjointed buttons.
  Rerun/resend delegate to the flows in §2 (no duplicated logic).

### 1.5 States
- Loading: skeleton rows (3) while `isLoading` and no cached items.
- Empty: panel hides itself (see §1.1); E2E-154-07 pins this.
- Error: never render stale data silently — on data-hook error, show the error
  via the error store toast (`captureError`) and keep last-good items with a
  "stale" marker + manual refresh button.
- Overflow: `+N more` affordance (§1.3).

### 1.6 Refresh behavior
- Follows `usePromptTreeData.ts` auto-sync (15s–2m) and SWR cache invalidation;
  no independent polling timer in this component.
- Manual refresh button calls `onRefresh()` → revalidates the SWR key.

### 1.7 Accessibility and labels
- `role="region"`, `aria-label="Now running and up next"`.
- Running row: `aria-live="polite"` announces running-prompt changes.
- All icon buttons have `aria-label`s; keyboard navigable (roving tabindex);
  4-tier badges expose text equivalents (not color-only).

### 1.8 File and naming decision
- New file: `src/components/instances/PromptTreeView/NowRunningUpNext.tsx`
  (PascalCase — matches the module's existing component convention:
  `TreePanel.tsx`, `DetailHeader.tsx`, `ResendAsModal.tsx`). All other new files
  (specs, plans, assets) stay lowercase per repo hygiene.

---

## 2. Rerun flyout + ResendAsModal with-image flows

Gap (c): unverified whether rerun/resend paths attach `image_payload`.
Terminal transports (`agy -p` text arg, CDP text-only submit) are text-only;
images travel the `.antigravity_resume_task.json` side-channel (verified:
backup/resumption skills confirm Base64 multimodal images ride this channel).

### 2.1 Rerun flyout (DetailHeader.tsx) — attach-image control
- Add an "attach image" control to the existing rerun flyout (which keeps its
  `rerunCount` min/max, default 2). Control opens the image picker; selected
  image is read via `read_image_as_base64` (100MB cap enforced; over-cap →
  error-store toast, flyout stays open).
- Dispatch: `dispatchRerunMessages` sends text verbatim (per image-resend spec)
  PLUS `image_payload` (base64) through the side-channel so the rerun re-injects
  the image. The rerun MUST NOT silently drop the image: if the side-channel
  write fails, abort the dispatch and toast via `captureError` (no silent errors).
  Side-effect ordering follows the repo's Risky Path Replacement rule (AGENTS.md):
  the side-channel write lands BEFORE the dispatch call, so failure happens
  before the point of no return — never dispatch-then-discover-the-image-lost.
- Post-dispatch: the new run appears in the panel (§1) at position 0 once running.

### 2.2 ResendAsModal — with-image resend
- Presets keep `target_mode: 'new' | 'current'` semantics (verified in
  `ResendAsModal.tsx`: `verify`/`fix`/`rerun` builtins).
- Add per-preset (or per-send) "include image" toggle. When on, the resent
  prompt carries `image_payload` (base64) alongside text; `target_mode='new'`
  creates a new conversation with the image, `target_mode='current'` appends
  to the current one.
- UI ↔ CLI parity: the CLI resend path and the UI modal path MUST produce
  byte-identical payloads for the same input (pinned by E2E-154-05).

### 2.3 Failure handling (both flows)
- All failures go through the TS error store (`captureError`/`trackWarning`);
  user-facing via toast. Bare `catch {}` is banned in new/modified code.
- Failure cases to handle: image over 100MB cap, base64 decode failure,
  side-channel write failure, dispatch rejection, queue-full on resend.

---

## 3. Test evidence contract (A4 RESOLVED)

### 3.1 Decision
TWO id layers, each with one canonical format:

1. **Prompt Id** (what was tested): `#P<seq>` — e.g. `#P001`.
   Adopts scheme (i) from spec 125, the app's own prompt identity.
2. **Test evidence Id** (the test-run instance): `E2E154-<runid>-<instance>-<caseNN>`
   — e.g. `E2E154-20261019a-alim-main-03`.
   Adopts scheme (iii) (plan 140 style), extended with the spec-154 prefix.

**Justification:** (i) and (ii) identify *prompts*, not test runs — they cannot
distinguish rerun #1 from rerun #3 of the same evidence. (iii) is the only
run-scoped, sortable, collision-free format, and it matches the mandated
`E2E-154-NN` case numbering. Combining both satisfies the user's "every test
carries its Id": the prompt Id says WHAT, the evidence Id says WHICH RUN.
Scheme (ii)'s `[AGM:P001 | GM:#1]` telegram shorthand is rejected for evidence
(the pipe/colon characters break filenames and grep).

- `<runid>`: `yyyymmdd` + lowercase letter per run of the day (`20261019a`).
- `<instance>`: instance slug as shown in the app (lowercase, hyphens).
- `<caseNN>`: zero-padded catalog number (`03`).

### 3.2 Mandatory evidence bundle (every case, no exceptions)
1. Prompt Id (`#P<seq>`) of every prompt exercised.
2. Test evidence Id (`E2E154-<runid>-<instance>-<caseNN>`).
3. Screenshot/image file path(s) — PNG, relative path.
4. DB evidence-query output — the exact SQLite query text + its result
   (proves state, not just pixels).
5. Storage location — relative path under the assets dir (§3.3).

A case without all five is INCOMPLETE, not passed.

### 3.3 Storage (relative, lowercase)
`02-spec/21-app/154-antigravity-manager-package-repository-update-and-prompt-functionality-testing/assets/e2e-<runid>/<caseNN>-<short-name>/`
containing: `screenshots/*.png`, `db-queries.md` (query text + output),
`payloads/` (redacted excerpts / sha256 manifests for image bytes),
`notes.md`. An `index.md` at `assets/e2e-<runid>/` maps evidence Id →
folder. No absolute paths anywhere, including inside the evidence files.

---

## 4. E2E catalog (NEW cases only — nothing from §already-covered)

### E2E-154-01 — backup captures multi-image prompt
- Pre: instance with prompt `#P001` carrying ≥2 images.
- Steps: (1) send prompt with 2 images; (2) trigger backup (CLI backup path
  per spec 50, or account-switch backup); (3) query `backup_prompts.db`.
- Expected: one row for `#P001`; `has_images=true`; stored image blobs
  sha256-match the source files byte-for-byte.
- Evidence bundle: prompt Id, evidence Id `E2E154-<runid>-<instance>-01`,
  screenshot of prompt with images in tree, DB query + output, sha256 manifest.

### E2E-154-02 — restore rehydrates byte-identical images
- Pre: backup from E2E-154-01.
- Steps: (1) restore; (2) open `#P001` in tree view; (3) sha256-compare
  restored images vs originals.
- Expected: byte-identical; images render in the UI (no broken placeholders).
- Evidence bundle: prompt Id, evidence Id `…-02`, before/after sha256 table,
  screenshot of restored prompt.

### E2E-154-03 — panel shows running-first + next-4 in FIFO order
- Pre: seed 1 running + 6 queued prompts with known FIFO order.
- Steps: (1) open prompt tree view; (2) read the §1 panel.
- Expected: position 0 = the running prompt; positions 1–4 = first four queued
  in FIFO order; `+2 more` affordance; 4-tier badges on all rows.
- Evidence bundle: prompt Ids of all 7, evidence Id `…-03`, panel screenshot,
  DB queue-order query + output.

### E2E-154-04 — rerun same prompt WITH image re-injects image
- Pre: prompt `#P002` with 1 image, completed.
- Steps: (1) DetailHeader rerun flyout → attach image → rerun (§2.1);
  (2) inspect dispatched payload and `.antigravity_resume_task.json`.
- Expected: `image_payload` present and sha256-identical to the original;
  new run displays the image (gap (c) closed on this path).
- Evidence bundle: original + rerun prompt Ids, evidence Id `…-04`,
  side-channel JSON excerpt + sha256, screenshot of rerun with image.

### E2E-154-05 — resend-queued with image: CLI/UI parity
- Pre: queued prompt `#P003` with 1 image.
- Steps: (1) resend with image via UI (`ResendAsModal`, `target_mode='new'`);
  (2) resend the same prompt with image via CLI; (3) compare payloads.
- Expected: text verbatim-identical and `image_payload` sha256-identical
  across both paths.
- Evidence bundle: prompt Id, evidence Id `…-05`, both payload sha256,
  screenshots of both flows.

### E2E-154-06 — queue overflow (>4) truncation
- Pre: 8 queued prompts, FIFO known.
- Steps: (1) open tree view; (2) read panel; (3) expand `+N more`.
- Expected: 4 rows + `+4 more`; expanding reveals all 8 in FIFO order;
  no prompt unreachable (truncation is display-only).
- Evidence bundle: prompt Ids, evidence Id `…-06`, collapsed + expanded
  screenshots, DB count query + output.

### E2E-154-07 — empty queue state
- Pre: zero queued prompts (running prompt may exist).
- Steps: open tree view.
- Expected: panel hides itself (§1.1); no stuck skeleton, no error toast,
  tree renders normally.
- Evidence bundle: evidence Id `…-07`, screenshot of tree view without panel.

---

## 5. Acceptance criteria

- AC-154-01: Panel renders the running prompt at position 0 within one
  auto-sync cycle of a run starting. (Builds on 153: running liveness.)
- AC-154-02: Positions 1–4 match FIFO queue order from the DB; no ghost or
  TTL-expired entries appear. (Builds on 153: ghost filtration.)
- AC-154-03: 4-tier origin badges render on every panel row, same visual
  language as tree badges. (Builds on 145: badges.)
- AC-154-04: Rerun-with-image delivers byte-identical `image_payload` via the
  side-channel; the new run displays the image. (Builds on image-resend spec:
  verbatim rerun; closes gap (c) for the rerun path.)
- AC-154-05: Resend-with-image produces byte-identical payloads via UI and CLI.
  (Builds on image-resend spec: templates; new parity guarantee.)
- AC-154-06: Backup/restore round-trip preserves multi-image prompts
  byte-identically (`has_images=true`). (Builds on 50: backup schema/retention.)
- AC-154-07: Every E2E case ships a complete 5-part evidence bundle (§3.2);
  incomplete bundles do not count as passed. (New: A4 contract.)
- AC-154-08: All failures surface via the error store; zero bare `catch {}`
  in new/modified code. (House rule: no silent errors.)
- AC-154-09: `npm run build` passes before any src/ change is committed.
  (House rule: pre-flight.)

---

## 6. Test strategy note

Tauri GUI cannot run headless in CI in this environment. E2E here is defined as:
CLI + SQLite evidence queries (CI-verifiable) + scripted UI verification steps
with screenshots captured on a real machine (machine-verified). Each catalog
case is marked:

- E2E-154-01, -02: CI-verifiable (CLI + DB) + machine-verified (screenshots).
- E2E-154-03, -06, -07: machine-verified (panel rendering).
- E2E-154-04, -05: CI-verifiable (payload/side-channel asserts) + machine-verified.

Machine-verified steps run on the user's machine per the scripted steps in §4;
their evidence bundles (§3.2) are committed under `assets/` so CI reviewers can
audit without re-running the GUI.
