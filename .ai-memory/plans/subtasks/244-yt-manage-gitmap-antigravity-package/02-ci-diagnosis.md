# Subtask 02 - CI diagnosis (244)

**Date**: 01-Nov-26
**Status**: DONE (remote logs only; no local build)

- `gitmap pe -t` BEFORE: tip `0733941` FAIL (`Release [FAIL]`, `CI [FAIL]`),
  `-1` `78a9078` FAIL, `-2` `9813722` PASS, `-3` `880be99` PASS,
  `-4` `5b2ab54` PASS.
- CI run for tip: `41221316712` (failure). Release run for tip:
  `41221319488` (failure at the release gate: CI concluded `failure`).
- CI run for `78a9078`: `41218704243` (failure).
- Single root cause in both runs: `error[E0061]` at
  `src-tauri/src/modules/repo_db/queue.rs:516:5`, zero-argument call to
  `invalidate_prompt_tree_cache`, defined at
  `src-tauri/src/modules/repo_db/tree.rs:109:8` with one
  `Option<&str>` parameter. `could not compile agm-ui (lib) due to
  1 previous error; 1234 warnings emitted` (warnings pre-existing, ignored).
- Introduced by commit `78a9078d0` (new function
  `clear_queued_prompts_for_instance`); inherited unchanged by `073394179`.
