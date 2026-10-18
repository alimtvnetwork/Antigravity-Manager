# E2E-154-05 — resend-queued with image: CLI/UI parity (run 20261019a)

- Prompt Id: `#P003` (queued, 1 image; planned, assigned at execution)
- Evidence Id: `E2E154-20261019a-<instance>-05`
- Verdict: **INCOMPLETE** — UI resend needs a running app with display; CLI resend needs a built `agm` binary. Nothing executed here.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

- Parity requirement (spec §2.2): the CLI resend path and the UI modal path
  MUST produce byte-identical payloads for the same input.
- `ResendAsModal.tsx` keeps `target_mode: 'new' | 'current'` semantics
  (verified in spec: `verify`/`fix`/`rerun` builtins); `target_mode='new'`
  creates a new conversation with the image, `'current'` appends to current.
- UI resend entry: `handleResendPrompt` in `usePromptTreePromptActions.ts`;
  CLI resend entry: `agm prompts resend-queued --json` (per Task-01's
  machine-verification command list).

## Machine-verification steps (exact commands, run on a real machine)

```sh
# pre: build agm, agm instances test-create --json -> <instance>; #P003 queued with 1 image
# 1. UI: ResendAsModal -> include-image toggle ON -> target_mode='new' -> send
#    screenshot: screenshots/05-ui-resend.png ; capture dispatched payload P_ui
# 2. CLI: agm prompts resend-queued --json (same #P003, with image)
#    screenshot: screenshots/05-cli-resend.png ; capture dispatched payload P_cli
# 3. sha256(text of P_ui) == sha256(text of P_cli); sha256(image_payload of P_ui) == sha256(image_payload of P_cli)
#    fill payloads/sha256-manifest.md
```

Expected: text verbatim-identical and `image_payload` sha256-identical across
both paths.

## Bugs found

None (static review; execution not possible on this VM).
