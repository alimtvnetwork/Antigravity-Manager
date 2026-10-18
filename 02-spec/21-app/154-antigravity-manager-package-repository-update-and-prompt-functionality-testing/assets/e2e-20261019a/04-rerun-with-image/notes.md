# E2E-154-04 — rerun same prompt WITH image re-injects image (run 20261019a)

- Prompt Ids: `#P002` (original, completed, 1 image) → `#P002-R1` (rerun; assigned at execution)
- Evidence Id: `E2E154-20261019a-<instance>-04`
- Verdict: **INCOMPLETE** — rerun dispatch needs a running app; side-channel inspection pending.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

- Side-channel contract (spec §2.1, 01-architecture-spec.md §5.2): images ride
  out-of-band via `.antigravity_resume_task.json` (built by `resume_task_document`);
  `agy -p` and CDP submit are text-only. The rerun MUST NOT silently drop the
  image: side-channel write lands BEFORE the dispatch call (repo Risky Path
  Replacement rule); on failure the dispatch aborts with `captureError`.
- `read_image_as_base64` native Rust cmd with 100MB cap exists
  (`src/components/instances/PromptTreeView/clipboard.ts` per spec; over-cap →
  error-store toast, flyout stays open).
- `dispatchRerunMessages` / rerun flyout live in `usePromptTreePromptActions.ts`
  and `DetailHeader.tsx` (rerunCount min/max, default 2) per spec.

## Machine-verification steps (exact commands, run on a real machine)

```sh
# pre: build agm, agm instances test-create --json -> <instance>; #P002 completed with 1 image
# 1. DetailHeader rerun flyout -> attach image -> rerun
# 2. inspect the dispatched payload and .antigravity_resume_task.json for #P002-R1
# 3. sha256(image_payload of rerun) == sha256(original image); fill payloads/sha256-manifest.md
# 4. screenshot: the new run displaying the image -> screenshots/04-rerun-with-image.png
```

Expected: `image_payload` present and sha256-identical to the original; the new
run displays the image (gap (c) closed on the rerun path).

## Bugs found

None (static review; dispatch execution not possible on this VM).
